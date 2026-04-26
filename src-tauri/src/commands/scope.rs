//! Active orchestration scope resolver.
//!
//! This is intentionally read-only: UI should ask for "where am I now?" without
//! creating another source of truth beside sessions, plans, and work items.

use crate::commands::plans::{load_all_plans_internal, Plan as SavedPlan};
use crate::commands::status::{DelegationStatus, PlanStatus};
use crate::commands::strategy_models::{load_strategies, Assignee};
use crate::state::{
    AppState, Delegation, FileLeaseStatus, MultiAgentSession, ProjectSession, ProjectSessionStatus,
    ReviewVerdictStatus, SessionStatus, WorkItem, WorkItemAssignee, WorkItemStatus,
    WorkItemWriteIntent,
};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::State;

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn label_for_kind(kind: &str) -> &'static str {
    match kind {
        "strategy" => "Strategy",
        "plan" => "Plan",
        "work_item" => "Task",
        "project" => "Project",
        _ => "Global",
    }
}

fn action(id: &str, label: &str, tone: &str) -> Value {
    json!({ "id": id, "label": label, "tone": tone })
}

fn actions_for(kind: &str) -> Vec<Value> {
    match kind {
        "work_item" => vec![
            action("ask_both", "Ask both", "neutral"),
            action("execute_with_lead", "Execute with lead", "primary"),
            action("review_result", "Review result", "neutral"),
        ],
        "plan" => vec![
            action("ask_both", "Ask both", "neutral"),
            action("execute_next_step", "Execute next step", "primary"),
            action("create_work_item", "Create task", "neutral"),
            action("replan", "Replan", "neutral"),
        ],
        "strategy" => vec![
            action("ask_both", "Ask both", "neutral"),
            action("create_plan", "Turn into plan", "primary"),
            action("execute_next", "Execute next", "neutral"),
        ],
        "project" => vec![
            action("ask_both", "Ask both", "neutral"),
            action("create_plan", "Create plan", "primary"),
            action("queue_task", "Queue task", "neutral"),
        ],
        _ => vec![
            action("ask_both", "Ask both", "neutral"),
            action("create_strategy", "Create strategy", "primary"),
            action("pick_project", "Pick project", "neutral"),
        ],
    }
}

fn summary_for(kind: &str, title: &str) -> String {
    match kind {
        "work_item" => format!("Duo actions apply to task: {}", title),
        "plan" => format!("Duo actions apply to plan: {}", title),
        "strategy" => format!("Duo actions apply to strategy: {}", title),
        "project" => format!("Duo actions apply to project: {}", title),
        _ => "Duo is operating at global orchestration level.".to_string(),
    }
}

fn active_session_for(
    state: &AppState,
    room_session_id: Option<String>,
    project: Option<String>,
) -> Option<MultiAgentSession> {
    let sessions = state.sessions.lock().ok()?;
    if let Some(session_id) = room_session_id {
        if let Some(session) = sessions.get(&session_id) {
            return Some(session.clone());
        }
    }

    let project = project.unwrap_or_default();
    let mut matching: Vec<MultiAgentSession> = sessions
        .values()
        .filter(|session| {
            let same_project = session.project == project;
            same_project && !matches!(session.status, SessionStatus::Closed)
        })
        .cloned()
        .collect();
    matching.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    matching.into_iter().next()
}

fn latest_linked_work_item(
    state: &AppState,
    session: Option<&MultiAgentSession>,
    project: &str,
) -> Option<WorkItem> {
    let work_items = state.work_items.lock().ok()?;
    if let Some(session) = session {
        for id in session.linked_work_item_ids.iter().rev() {
            if let Some(item) = work_items.get(id) {
                return Some(item.clone());
            }
        }
    }

    let mut matching: Vec<WorkItem> = work_items
        .values()
        .filter(|item| {
            (project.is_empty() || item.project == project)
                && !matches!(
                    item.status,
                    crate::state::WorkItemStatus::Completed
                        | crate::state::WorkItemStatus::Failed
                        | crate::state::WorkItemStatus::Cancelled
                )
        })
        .cloned()
        .collect();
    matching.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    matching.into_iter().next()
}

fn breadcrumbs(kind: &str, project: &str, title: &str) -> Vec<Value> {
    let mut crumbs = vec![json!({ "kind": "global", "label": "Global" })];
    if !project.is_empty() {
        crumbs.push(json!({ "kind": "project", "label": project }));
    }
    if kind != "global" && kind != "project" {
        crumbs.push(json!({ "kind": kind, "label": title }));
    }
    crumbs
}

#[tauri::command]
pub fn get_active_scope(
    state: State<Arc<AppState>>,
    project: Option<String>,
    room_session_id: Option<String>,
) -> Value {
    resolve_active_scope(&state, project, room_session_id)
}

#[tauri::command]
pub fn get_orchestration_map(
    state: State<Arc<AppState>>,
    project: Option<String>,
    room_session_id: Option<String>,
) -> Value {
    resolve_orchestration_map(&state, project, room_session_id)
}

fn active_work_item_status(status: &WorkItemStatus) -> bool {
    !matches!(
        status,
        WorkItemStatus::Completed | WorkItemStatus::Failed | WorkItemStatus::Cancelled
    )
}

fn work_item_status_label(status: &WorkItemStatus) -> &'static str {
    match status {
        WorkItemStatus::Draft => "draft",
        WorkItemStatus::Ready => "ready",
        WorkItemStatus::Queued => "queued",
        WorkItemStatus::Running => "running",
        WorkItemStatus::Reviewing => "reviewing",
        WorkItemStatus::Completed => "completed",
        WorkItemStatus::Failed => "failed",
        WorkItemStatus::Cancelled => "cancelled",
    }
}

fn review_verdict_status_label(status: &ReviewVerdictStatus) -> &'static str {
    match status {
        ReviewVerdictStatus::Approve => "approve",
        ReviewVerdictStatus::Warn => "warn",
        ReviewVerdictStatus::Fail => "fail",
    }
}

fn route_phase_label(phase: &str) -> &'static str {
    match phase {
        "ready" => "ready to queue",
        "queued" => "waiting in delegation queue",
        "running" => "agent is working",
        "verifying" => "verification is running",
        "reviewing" => "reviewer is checking",
        "needs_user" => "needs user decision",
        "blocked" => "blocked",
        "done" => "completed",
        _ => "idle",
    }
}

fn project_session_status_label(status: &ProjectSessionStatus) -> &'static str {
    match status {
        ProjectSessionStatus::Active => "active",
        ProjectSessionStatus::Paused => "paused",
        ProjectSessionStatus::Closed => "closed",
    }
}

fn write_intent_label(intent: &WorkItemWriteIntent) -> &'static str {
    match intent {
        WorkItemWriteIntent::ReadOnly => "read_only",
        WorkItemWriteIntent::ProposeWrite => "propose_write",
        WorkItemWriteIntent::ExclusiveWrite => "exclusive_write",
    }
}

fn work_item_route_priority(item: &WorkItem) -> u8 {
    match item.status {
        WorkItemStatus::Running => 0,
        WorkItemStatus::Ready => 1,
        WorkItemStatus::Queued => 2,
        WorkItemStatus::Reviewing => 3,
        WorkItemStatus::Draft => 4,
        WorkItemStatus::Failed => 5,
        WorkItemStatus::Cancelled => 6,
        WorkItemStatus::Completed => 7,
    }
}

fn can_queue_work_item(item: &WorkItem) -> bool {
    matches!(item.status, WorkItemStatus::Draft | WorkItemStatus::Ready)
        && matches!(item.assignee, WorkItemAssignee::Agent)
        && item.delegation_id.is_none()
}

#[derive(Clone)]
struct RouteLane {
    id: String,
    project: String,
    project_session_id: Option<String>,
    title: String,
    status: String,
    executor_provider: String,
    reviewer_provider: Option<String>,
    work_item_ids: HashSet<String>,
    ready: usize,
    queued: usize,
    running: usize,
    reviewing: usize,
    draft: usize,
    completed: usize,
    blocked: usize,
    active_leases: usize,
    pending_delegations: usize,
    active_delegations: usize,
    verifying_delegations: usize,
    done_delegations: usize,
    needs_user_delegations: usize,
    blocker_delegation_ids: Vec<String>,
    active_delegation_id: Option<String>,
    active_work_item_id: Option<String>,
    review_verdict_status: Option<String>,
    review_verdict_summary: Option<String>,
    synthetic: bool,
    next_work_item: Option<WorkItem>,
    updated_at: String,
}

impl RouteLane {
    fn from_session(session: &ProjectSession) -> Self {
        Self {
            id: format!("session:{}", session.id),
            project: session.project.clone(),
            project_session_id: Some(session.id.clone()),
            title: session.title.clone(),
            status: project_session_status_label(&session.status).to_string(),
            executor_provider: session.executor_provider.as_str().to_string(),
            reviewer_provider: session.reviewer_provider.map(|p| p.as_str().to_string()),
            work_item_ids: HashSet::new(),
            ready: 0,
            queued: 0,
            running: 0,
            reviewing: 0,
            draft: 0,
            completed: 0,
            blocked: 0,
            active_leases: 0,
            pending_delegations: 0,
            active_delegations: 0,
            verifying_delegations: 0,
            done_delegations: 0,
            needs_user_delegations: 0,
            blocker_delegation_ids: Vec::new(),
            active_delegation_id: None,
            active_work_item_id: None,
            review_verdict_status: None,
            review_verdict_summary: None,
            synthetic: false,
            next_work_item: None,
            updated_at: session.updated_at.clone(),
        }
    }

    fn from_work_item(item: &WorkItem) -> Self {
        let project_session_id = item.project_session_id.clone();
        Self {
            id: project_session_id
                .as_ref()
                .map(|id| format!("session:{}", id))
                .unwrap_or_else(|| {
                    format!(
                        "project:{}:{}",
                        item.project,
                        item.executor_provider.as_str()
                    )
                }),
            project: item.project.clone(),
            project_session_id,
            title: format!("{} agent lane", item.project),
            status: "active".to_string(),
            executor_provider: item.executor_provider.as_str().to_string(),
            reviewer_provider: item.reviewer_provider.map(|p| p.as_str().to_string()),
            work_item_ids: HashSet::new(),
            ready: 0,
            queued: 0,
            running: 0,
            reviewing: 0,
            draft: 0,
            completed: 0,
            blocked: 0,
            active_leases: 0,
            pending_delegations: 0,
            active_delegations: 0,
            verifying_delegations: 0,
            done_delegations: 0,
            needs_user_delegations: 0,
            blocker_delegation_ids: Vec::new(),
            active_delegation_id: None,
            active_work_item_id: None,
            review_verdict_status: None,
            review_verdict_summary: None,
            synthetic: false,
            next_work_item: None,
            updated_at: item.updated_at.clone(),
        }
    }

    fn from_delegation(delegation: &Delegation) -> Self {
        let mut work_item_ids = HashSet::new();
        if let Some(work_item_id) = delegation.work_item_id.as_ref() {
            work_item_ids.insert(work_item_id.clone());
        }
        Self {
            id: format!("delegation-route:{}", delegation.id),
            project: delegation.project.clone(),
            project_session_id: delegation.project_session_id.clone(),
            title: if is_blocking_delegation_status(delegation.status) {
                "Blocked delegation".to_string()
            } else {
                "Delegation route".to_string()
            },
            status: delegation.status.to_string(),
            executor_provider: delegation
                .executor_provider
                .map(|p| p.as_str().to_string())
                .unwrap_or_else(|| "agent".to_string()),
            reviewer_provider: delegation.reviewer_provider.map(|p| p.as_str().to_string()),
            work_item_ids,
            ready: 0,
            queued: 0,
            running: 0,
            reviewing: 0,
            draft: 0,
            completed: 0,
            blocked: 0,
            active_leases: 0,
            pending_delegations: 0,
            active_delegations: 0,
            verifying_delegations: 0,
            done_delegations: 0,
            needs_user_delegations: 0,
            blocker_delegation_ids: Vec::new(),
            active_delegation_id: None,
            active_work_item_id: delegation.work_item_id.clone(),
            review_verdict_status: delegation
                .review_verdict
                .as_ref()
                .map(|v| review_verdict_status_label(&v.status).to_string()),
            review_verdict_summary: delegation
                .review_verdict
                .as_ref()
                .map(|v| v.summary.clone()),
            synthetic: true,
            next_work_item: None,
            updated_at: delegation.ts.clone(),
        }
    }

    fn add_work_item(&mut self, item: &WorkItem) {
        self.work_item_ids.insert(item.id.clone());
        match item.status {
            WorkItemStatus::Ready => self.ready += 1,
            WorkItemStatus::Queued => self.queued += 1,
            WorkItemStatus::Running => self.running += 1,
            WorkItemStatus::Reviewing => self.reviewing += 1,
            WorkItemStatus::Draft => self.draft += 1,
            WorkItemStatus::Failed | WorkItemStatus::Cancelled => self.blocked += 1,
            WorkItemStatus::Completed => self.completed += 1,
        }
        if active_work_item_status(&item.status) || item.review_verdict.is_some() {
            self.active_work_item_id = Some(item.id.clone());
        }
        if let Some(delegation_id) = item.delegation_id.as_ref() {
            self.active_delegation_id = Some(delegation_id.clone());
        }
        if let Some(verdict) = item.review_verdict.as_ref() {
            self.review_verdict_status =
                Some(review_verdict_status_label(&verdict.status).to_string());
            self.review_verdict_summary = Some(verdict.summary.clone());
        }
        let replace_next = self
            .next_work_item
            .as_ref()
            .map(|current| work_item_route_priority(item) < work_item_route_priority(current))
            .unwrap_or(true);
        if replace_next {
            self.next_work_item = Some(item.clone());
        }
        if item.updated_at > self.updated_at {
            self.updated_at = item.updated_at.clone();
        }
    }

    fn mark_delegation_blocker(&mut self, delegation: &Delegation) {
        if !self.blocker_delegation_ids.contains(&delegation.id) {
            self.blocked += 1;
            self.blocker_delegation_ids.push(delegation.id.clone());
        }
        if let Some(work_item_id) = delegation.work_item_id.as_ref() {
            self.work_item_ids.insert(work_item_id.clone());
        }
        if self.executor_provider == "agent" {
            if let Some(provider) = delegation.executor_provider {
                self.executor_provider = provider.as_str().to_string();
            }
        }
        if self.reviewer_provider.is_none() {
            self.reviewer_provider = delegation.reviewer_provider.map(|p| p.as_str().to_string());
        }
        if delegation.ts > self.updated_at {
            self.updated_at = delegation.ts.clone();
        }
    }

    fn add_delegation(&mut self, delegation: &Delegation) {
        match delegation.status {
            DelegationStatus::Pending | DelegationStatus::Scheduled => {
                self.pending_delegations += 1;
                self.active_delegation_id = Some(delegation.id.clone());
            }
            DelegationStatus::Running
            | DelegationStatus::Escalated
            | DelegationStatus::Deciding => {
                self.active_delegations += 1;
                self.active_delegation_id = Some(delegation.id.clone());
            }
            DelegationStatus::Verifying => {
                self.verifying_delegations += 1;
                self.active_delegation_id = Some(delegation.id.clone());
            }
            DelegationStatus::Done => {
                self.done_delegations += 1;
            }
            DelegationStatus::Failed
            | DelegationStatus::Rejected
            | DelegationStatus::Cancelled
            | DelegationStatus::NeedsPermission => {
                self.needs_user_delegations += 1;
                self.active_delegation_id = Some(delegation.id.clone());
                self.mark_delegation_blocker(delegation);
            }
        }
        if let Some(work_item_id) = delegation.work_item_id.as_ref() {
            self.active_work_item_id = Some(work_item_id.clone());
            self.work_item_ids.insert(work_item_id.clone());
        }
        if let Some(verdict) = delegation.review_verdict.as_ref() {
            self.review_verdict_status =
                Some(review_verdict_status_label(&verdict.status).to_string());
            self.review_verdict_summary = Some(verdict.summary.clone());
        }
        if delegation.ts > self.updated_at {
            self.updated_at = delegation.ts.clone();
        }
    }

    fn progress_phase(&self) -> &'static str {
        if self.needs_user_delegations > 0 {
            "needs_user"
        } else if self.blocked > 0 {
            "blocked"
        } else if self.verifying_delegations > 0 {
            "verifying"
        } else if self.reviewing > 0 {
            "reviewing"
        } else if self.running > 0 || self.active_delegations > 0 || self.active_leases > 0 {
            "running"
        } else if self.queued > 0 || self.pending_delegations > 0 {
            "queued"
        } else if self.ready > 0 || self.draft > 0 {
            "ready"
        } else if self.completed > 0 || self.done_delegations > 0 {
            "done"
        } else {
            "idle"
        }
    }

    fn route_state(&self) -> &'static str {
        self.progress_phase()
    }

    fn lifecycle_steps(&self, phase: &str) -> Vec<Value> {
        let phases = [
            "ready",
            "queued",
            "running",
            "verifying",
            "reviewing",
            "done",
        ];
        let current_index = phases.iter().position(|item| *item == phase);
        phases
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let status = if phase == "needs_user" || phase == "blocked" {
                    if index <= 2 {
                        "blocked"
                    } else {
                        "pending"
                    }
                } else if Some(index) == current_index {
                    "current"
                } else if current_index
                    .map(|current| index < current)
                    .unwrap_or(false)
                {
                    "done"
                } else {
                    "pending"
                };
                json!({ "id": item, "status": status })
            })
            .collect()
    }

    fn into_value(self) -> Value {
        let route_state = self.route_state();
        let can_queue_next = self
            .next_work_item
            .as_ref()
            .map(can_queue_work_item)
            .unwrap_or(false);
        let action = if can_queue_next {
            "queue_next"
        } else if matches!(
            route_state,
            "running" | "verifying" | "reviewing" | "queued"
        ) {
            "monitor"
        } else if route_state == "needs_user" {
            "resolve_user_blocker"
        } else if route_state == "blocked" {
            "resolve_blocker"
        } else if route_state == "done" {
            "review_result"
        } else {
            "prompt"
        };
        let lifecycle_steps = self.lifecycle_steps(route_state);
        let blocker_delegation_ids = self.blocker_delegation_ids.clone();
        let active_delegation_id = self.active_delegation_id.clone();
        let active_work_item_id = self.active_work_item_id.clone();
        let review_verdict_status = self.review_verdict_status.clone();
        let review_verdict_summary = self.review_verdict_summary.clone();
        let next_work_item = self.next_work_item.clone().map(|item| {
            json!({
                "id": item.id,
                "title": item.title,
                "task": item.task,
                "status": work_item_status_label(&item.status),
                "write_intent": write_intent_label(&item.write_intent),
                "declared_paths": item.declared_paths,
                "delegation_id": item.delegation_id
            })
        });
        let progress = json!({
            "phase": route_state,
            "label": route_phase_label(route_state),
            "needs_user": route_state == "needs_user",
            "active_delegation_id": active_delegation_id,
            "active_work_item_id": active_work_item_id,
            "review_verdict_status": review_verdict_status,
            "review_verdict_summary": review_verdict_summary,
            "lease_count": self.active_leases,
            "blocker_delegation_ids": blocker_delegation_ids,
            "steps": lifecycle_steps,
            "suggested_action": action
        });
        json!({
            "id": self.id,
            "project": self.project,
            "project_session_id": self.project_session_id,
            "title": self.title,
            "status": self.status,
            "route_state": route_state,
            "action": action,
            "can_queue_next": can_queue_next,
            "executor_provider": self.executor_provider,
            "reviewer_provider": self.reviewer_provider,
            "has_blockers": self.blocked > 0,
            "blocker_delegation_ids": progress["blocker_delegation_ids"].clone(),
            "synthetic": self.synthetic,
            "progress": progress,
            "counts": {
                "work_items": self.work_item_ids.len(),
                "ready": self.ready,
                "queued": self.queued,
                "running": self.running,
                "reviewing": self.reviewing,
                "draft": self.draft,
                "completed": self.completed,
                "blocked": self.blocked,
                "active_leases": self.active_leases,
                "pending_delegations": self.pending_delegations,
                "active_delegations": self.active_delegations,
                "verifying_delegations": self.verifying_delegations,
                "done_delegations": self.done_delegations,
                "needs_user_delegations": self.needs_user_delegations
            },
            "next_work_item": next_work_item,
            "updated_at": self.updated_at
        })
    }
}

fn is_blocking_delegation_status(status: DelegationStatus) -> bool {
    matches!(
        status,
        DelegationStatus::Failed
            | DelegationStatus::Rejected
            | DelegationStatus::Cancelled
            | DelegationStatus::NeedsPermission
    )
}

fn collect_project_agent_routes(
    state: &AppState,
    resolved_project: &str,
    room_session_id: Option<&str>,
) -> Vec<Value> {
    let sessions: Vec<ProjectSession> = state
        .project_sessions
        .lock()
        .map(|items| items.values().cloned().collect())
        .unwrap_or_default();
    let work_items: Vec<WorkItem> = state
        .work_items
        .lock()
        .map(|items| items.values().cloned().collect())
        .unwrap_or_default();

    let mut lanes: HashMap<String, RouteLane> = HashMap::new();
    for session in sessions.iter().filter(|session| {
        (resolved_project.is_empty() || session.project == resolved_project)
            && room_session_id
                .map(|id| session.parent_room_session_id == id)
                .unwrap_or(true)
            && !matches!(session.status, ProjectSessionStatus::Closed)
    }) {
        lanes.insert(
            format!("session:{}", session.id),
            RouteLane::from_session(session),
        );
    }

    for item in work_items.iter().filter(|item| {
        (resolved_project.is_empty() || item.project == resolved_project)
            && room_session_id
                .map(|id| item.parent_room_session_id == id)
                .unwrap_or(true)
    }) {
        let key = item
            .project_session_id
            .as_ref()
            .map(|id| format!("session:{}", id))
            .unwrap_or_else(|| {
                format!(
                    "project:{}:{}",
                    item.project,
                    item.executor_provider.as_str()
                )
            });
        if !active_work_item_status(&item.status) && !lanes.contains_key(&key) {
            continue;
        }
        lanes
            .entry(key)
            .or_insert_with(|| RouteLane::from_work_item(item))
            .add_work_item(item);
    }

    if let Ok(delegations) = state.delegations.lock() {
        for d in delegations
            .values()
            .filter(|d| resolved_project.is_empty() || d.project == resolved_project)
        {
            let provider_key = d
                .executor_provider
                .map(|provider| format!("project:{}:{}", d.project, provider.as_str()));
            let keys = [
                d.project_session_id
                    .as_ref()
                    .map(|id| format!("session:{}", id)),
                d.work_item_id.as_ref().and_then(|work_id| {
                    lanes.iter().find_map(|(key, lane)| {
                        if lane.work_item_ids.contains(work_id) {
                            Some(key.clone())
                        } else {
                            None
                        }
                    })
                }),
                provider_key,
                Some(format!("project:{}:claude", d.project)),
                Some(format!("project:{}:codex", d.project)),
            ];
            let mut attached = false;
            for key in keys.into_iter().flatten() {
                if let Some(lane) = lanes.get_mut(&key) {
                    lane.add_delegation(d);
                    attached = true;
                    break;
                }
            }
            if !attached {
                let mut lane = RouteLane::from_delegation(d);
                lane.add_delegation(d);
                lanes.insert(format!("delegation-route:{}", d.id), lane);
            }
        }
    }

    if let Ok(leases) = state.file_leases.lock() {
        for lease in leases.values().filter(|lease| {
            (resolved_project.is_empty() || lease.project == resolved_project)
                && matches!(lease.status, FileLeaseStatus::Active)
        }) {
            if let Some((_key, lane)) = lanes
                .iter_mut()
                .find(|(_, lane)| lane.work_item_ids.contains(&lease.work_item_id))
            {
                lane.active_leases += 1;
            }
        }
    }

    let mut routes: Vec<RouteLane> = lanes.into_values().collect();
    routes.sort_by(|a, b| {
        let state_rank = |state: &str| match state {
            "needs_user" => 0,
            "blocked" => 1,
            "running" | "verifying" | "reviewing" => 2,
            "queued" => 3,
            "ready" => 4,
            _ => 5,
        };
        state_rank(a.route_state())
            .cmp(&state_rank(b.route_state()))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    routes.into_iter().map(RouteLane::into_value).collect()
}

fn collect_route_progress_aggregate(routes: &[Value]) -> Value {
    let phases = [
        "ready",
        "queued",
        "running",
        "verifying",
        "reviewing",
        "needs_user",
        "blocked",
        "done",
        "idle",
    ];
    let mut counts: HashMap<&str, usize> = phases.iter().map(|phase| (*phase, 0usize)).collect();
    for route in routes {
        let phase = route
            .get("progress")
            .and_then(|progress| progress.get("phase"))
            .and_then(Value::as_str)
            .or_else(|| route.get("route_state").and_then(Value::as_str))
            .unwrap_or("idle");
        let entry = counts.entry(phase).or_insert(0);
        *entry += 1;
    }
    let active = counts.get("running").copied().unwrap_or(0)
        + counts.get("verifying").copied().unwrap_or(0)
        + counts.get("reviewing").copied().unwrap_or(0);
    let queueable = routes
        .iter()
        .filter(|route| route.get("can_queue_next").and_then(Value::as_bool) == Some(true))
        .count();
    let needs_user = counts.get("needs_user").copied().unwrap_or(0);
    let blocked = counts.get("blocked").copied().unwrap_or(0);
    let done = counts.get("done").copied().unwrap_or(0);
    let headline = if needs_user > 0 {
        format!("{} route ждут твоего решения", needs_user)
    } else if active > 0 {
        format!("{} route активны сейчас", active)
    } else if queueable > 0 {
        format!("{} route готовы к запуску", queueable)
    } else if blocked > 0 {
        format!("{} route заблокированы", blocked)
    } else if done > 0 {
        format!("{} route завершены", done)
    } else {
        "Нет live-прогресса маршрутов".to_string()
    };
    json!({
        "total": routes.len(),
        "active": active,
        "queueable": queueable,
        "needs_user": needs_user,
        "blocked": blocked,
        "done": done,
        "headline": headline,
        "counts": {
            "ready": counts.get("ready").copied().unwrap_or(0),
            "queued": counts.get("queued").copied().unwrap_or(0),
            "running": counts.get("running").copied().unwrap_or(0),
            "verifying": counts.get("verifying").copied().unwrap_or(0),
            "reviewing": counts.get("reviewing").copied().unwrap_or(0),
            "needs_user": needs_user,
            "blocked": blocked,
            "done": done,
            "idle": counts.get("idle").copied().unwrap_or(0)
        }
    })
}

fn percent(part: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        ((part.min(total) * 100) + (total / 2)) / total
    }
}

fn score_grade(score: usize) -> &'static str {
    match score {
        85..=100 => "A",
        70..=84 => "B",
        50..=69 => "C",
        _ => "D",
    }
}

fn collect_managerial_leverage(
    state: &AppState,
    resolved_project: &str,
    room_session_id: Option<&str>,
    routes: &[Value],
    route_progress: &Value,
    graph_context: &Value,
    plans: &[SavedPlan],
) -> Value {
    let scope_matches_project =
        |project: &str| resolved_project.is_empty() || project == resolved_project;
    let scope_matches_room =
        |room_id: &str| room_session_id.map(|id| room_id == id).unwrap_or(true);

    let work_items: Vec<WorkItem> = state
        .work_items
        .lock()
        .map(|items| {
            items
                .values()
                .filter(|item| {
                    scope_matches_project(&item.project)
                        && scope_matches_room(&item.parent_room_session_id)
                        && active_work_item_status(&item.status)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let project_sessions: Vec<ProjectSession> = state
        .project_sessions
        .lock()
        .map(|sessions| {
            sessions
                .values()
                .filter(|session| {
                    scope_matches_project(&session.project)
                        && scope_matches_room(&session.parent_room_session_id)
                        && !matches!(session.status, ProjectSessionStatus::Closed)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let delegations: Vec<Delegation> = state
        .delegations
        .lock()
        .map(|delegations| {
            delegations
                .values()
                .filter(|d| scope_matches_project(&d.project))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let active_leases = state
        .file_leases
        .lock()
        .map(|leases| {
            leases
                .values()
                .filter(|lease| {
                    scope_matches_project(&lease.project)
                        && matches!(lease.status, FileLeaseStatus::Active)
                })
                .count()
        })
        .unwrap_or(0);

    let mut projects_in_motion: HashSet<String> = HashSet::new();
    for item in &work_items {
        projects_in_motion.insert(item.project.clone());
    }
    for session in &project_sessions {
        projects_in_motion.insert(session.project.clone());
    }
    for delegation in &delegations {
        if matches!(
            delegation.status,
            DelegationStatus::Pending
                | DelegationStatus::Scheduled
                | DelegationStatus::Running
                | DelegationStatus::Escalated
                | DelegationStatus::Deciding
                | DelegationStatus::Verifying
        ) {
            projects_in_motion.insert(delegation.project.clone());
        }
    }

    let running_work = work_items
        .iter()
        .filter(|item| matches!(item.status, WorkItemStatus::Running))
        .count();
    let reviewing_work = work_items
        .iter()
        .filter(|item| matches!(item.status, WorkItemStatus::Reviewing))
        .count();
    let manual_work_items = work_items
        .iter()
        .filter(|item| matches!(&item.assignee, WorkItemAssignee::User))
        .count();
    let unaligned_work_items = work_items
        .iter()
        .filter(|item| item.source_kind.is_none() && item.source_id.is_none())
        .count();
    let work_items_with_reviewer = work_items
        .iter()
        .filter(|item| item.reviewer_provider.is_some())
        .count();

    let active_routes = routes.len();
    let queueable_routes = routes
        .iter()
        .filter(|route| route.get("can_queue_next").and_then(Value::as_bool) == Some(true))
        .count();
    let blocked_routes = routes
        .iter()
        .filter(|route| route.get("has_blockers").and_then(Value::as_bool) == Some(true))
        .count();
    let routes_with_reviewer = routes
        .iter()
        .filter(|route| {
            route
                .get("reviewer_provider")
                .and_then(Value::as_str)
                .is_some()
        })
        .count();
    let cross_provider_routes = routes
        .iter()
        .filter(|route| {
            let executor = route.get("executor_provider").and_then(Value::as_str);
            let reviewer = route.get("reviewer_provider").and_then(Value::as_str);
            matches!((executor, reviewer), (Some(a), Some(b)) if a != b)
        })
        .count();
    let route_needs_user = route_progress
        .get("needs_user")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let live_active_routes = route_progress
        .get("active")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;

    let relevant_plans: Vec<&SavedPlan> = plans
        .iter()
        .filter(|plan| {
            plan.status == PlanStatus::Active
                && (resolved_project.is_empty()
                    || plan
                        .steps
                        .iter()
                        .any(|step| step.project == resolved_project))
        })
        .collect();
    let active_plan_count = relevant_plans.len();
    let open_plan_steps = relevant_plans
        .iter()
        .flat_map(|plan| plan.steps.iter())
        .filter(|step| {
            (resolved_project.is_empty() || step.project == resolved_project)
                && !step.status.is_terminal()
        })
        .count();

    let strategies = load_strategies(state);
    let mut active_strategy_count = 0usize;
    let mut strategy_steps_total = 0usize;
    let mut strategy_steps_done = 0usize;
    let mut strategy_steps_open = 0usize;
    let mut manual_strategy_steps = 0usize;
    for strategy in strategies
        .iter()
        .filter(|strategy| strategy.status.is_active())
    {
        let mut strategy_relevant = false;
        for tactic in strategy.all_tactics() {
            for plan in tactic.plans {
                if !resolved_project.is_empty() && plan.project != resolved_project {
                    continue;
                }
                strategy_relevant = true;
                for step in plan.steps {
                    strategy_steps_total += 1;
                    if step.status.is_terminal() {
                        strategy_steps_done += 1;
                    } else {
                        strategy_steps_open += 1;
                    }
                    if matches!(step.assignee, Assignee::User) {
                        manual_strategy_steps += 1;
                    }
                }
            }
        }
        if strategy_relevant || resolved_project.is_empty() {
            active_strategy_count += 1;
        }
    }

    let mut pending_delegations = 0usize;
    let mut active_delegations = 0usize;
    let mut blocked_delegations = 0usize;
    for delegation in &delegations {
        match delegation.status {
            DelegationStatus::Pending | DelegationStatus::Scheduled => pending_delegations += 1,
            DelegationStatus::Running
            | DelegationStatus::Escalated
            | DelegationStatus::Deciding
            | DelegationStatus::Verifying => active_delegations += 1,
            DelegationStatus::Failed
            | DelegationStatus::Rejected
            | DelegationStatus::Cancelled
            | DelegationStatus::NeedsPermission => blocked_delegations += 1,
            DelegationStatus::Done => {}
        }
    }

    let reviewable_units = active_routes.max(work_items.len());
    let reviewer_units = routes_with_reviewer.max(work_items_with_reviewer);
    let reviewer_coverage = percent(reviewer_units, reviewable_units);
    let quality_status = if reviewer_coverage >= 70 && cross_provider_routes > 0 {
        "strong"
    } else if reviewer_coverage >= 35 {
        "partial"
    } else {
        "weak"
    };

    let alignment_status = if active_strategy_count == 0 {
        "missing_strategy"
    } else if active_plan_count == 0 && open_plan_steps == 0 {
        "missing_plan"
    } else if unaligned_work_items > 0 {
        "weak"
    } else {
        "aligned"
    };
    let alignment_score = match alignment_status {
        "aligned" => 100,
        "weak" => 65,
        "missing_plan" => 45,
        _ => 25,
    };

    let user_attention = pending_delegations
        + blocked_delegations
        + manual_work_items
        + manual_strategy_steps
        + blocked_routes
        + route_needs_user;
    let control_load = if user_attention >= 6 || blocked_delegations >= 3 || route_needs_user >= 2 {
        "high"
    } else if user_attention >= 2 || blocked_delegations > 0 {
        "medium"
    } else {
        "low"
    };

    let parallel_score = (projects_in_motion.len() * 20
        + running_work * 15
        + reviewing_work * 10
        + live_active_routes * 12
        + queueable_routes * 10
        + active_routes * 5)
        .min(100);
    let graph_ready = graph_context
        .get("available")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let quality_score = (reviewer_coverage + if graph_ready { 10 } else { 0 }).min(100);
    let control_score = 100usize.saturating_sub((user_attention * 12).min(85));
    let score = ((parallel_score * 30)
        + (quality_score * 30)
        + (alignment_score * 25)
        + (control_score * 15))
        / 100;

    let primary_bottleneck = if route_needs_user > 0 {
        "needs_user_routes"
    } else if blocked_delegations > 0 {
        "blocked_delegations"
    } else if pending_delegations > 0 {
        "pending_approval"
    } else if unaligned_work_items > 0 {
        "strategy_drift"
    } else if reviewer_coverage < 35 && reviewable_units > 0 {
        "missing_cross_check"
    } else if queueable_routes > 0 {
        "ready_to_execute"
    } else {
        "none"
    };

    let recommendation = match primary_bottleneck {
        "needs_user_routes" => {
            "Есть route lanes, которые ждут твоего решения: сначала разбери permission/fail блокеры, потом запускай новую волну."
        }
        "blocked_delegations" => {
            "Разбери blockers перед новой волной: есть упавшие или permission-blocked делегации."
        }
        "pending_approval" => {
            "Подтверди или отклони pending delegations, чтобы освободить очередь."
        }
        "strategy_drift" => {
            "Привяжи текущие work items к plan/strategy или явно заархивируй лишнюю работу."
        }
        "missing_cross_check" => {
            "Добавь reviewer/cross-check для активных route lanes перед масштабированием."
        }
        "ready_to_execute" => "Можно запускать следующую queueable route lane.",
        _ if active_strategy_count == 0 => {
            "Сначала создай или выбери стратегию, иначе execution рискует уйти в стол."
        }
        _ => "Система управляема: продолжай execution по route lanes и контролируй timeline.",
    };

    let summary = format!(
        "{} / parallel {} / quality {} / control {} / alignment {}",
        score_grade(score),
        projects_in_motion.len(),
        quality_status,
        control_load,
        alignment_status
    );

    let management_prompt = format!(
        "[MANAGEMENT_REVIEW]\nОцени текущую управляемость Agent OS: parallelism={}, quality={}, control_load={}, alignment={}. Рекомендация системы: {} Сформулируй следующий управленческий шаг без микроменеджмента.",
        projects_in_motion.len(),
        quality_status,
        control_load,
        alignment_status,
        recommendation
    );

    json!({
        "score": score,
        "grade": score_grade(score),
        "summary": summary,
        "recommendation": recommendation,
        "management_prompt": management_prompt,
        "parallelism": {
            "score": parallel_score,
            "projects_in_motion": projects_in_motion.len(),
            "active_project_sessions": project_sessions.len(),
            "active_routes": active_routes,
            "live_active_routes": live_active_routes,
            "queueable_routes": queueable_routes,
            "running_work_items": running_work,
            "reviewing_work_items": reviewing_work,
            "active_leases": active_leases
        },
        "quality": {
            "score": quality_score,
            "status": quality_status,
            "reviewer_coverage_percent": reviewer_coverage,
            "routes_with_reviewer": routes_with_reviewer,
            "cross_provider_routes": cross_provider_routes,
            "work_items_with_reviewer": work_items_with_reviewer,
            "graph_context_ready": graph_ready
        },
        "control": {
            "score": control_score,
            "load": control_load,
            "user_attention": user_attention,
            "pending_delegations": pending_delegations,
            "active_delegations": active_delegations,
            "blocked_delegations": blocked_delegations,
            "manual_work_items": manual_work_items,
            "manual_strategy_steps": manual_strategy_steps,
            "blocked_routes": blocked_routes,
            "needs_user_routes": route_needs_user,
            "primary_bottleneck": primary_bottleneck
        },
        "alignment": {
            "score": alignment_score,
            "status": alignment_status,
            "active_strategies": active_strategy_count,
            "active_plans": active_plan_count,
            "open_plan_steps": open_plan_steps,
            "strategy_steps_total": strategy_steps_total,
            "strategy_steps_done": strategy_steps_done,
            "strategy_steps_open": strategy_steps_open,
            "unaligned_work_items": unaligned_work_items
        }
    })
}

pub fn resolve_orchestration_map(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
) -> Value {
    let requested_project = clean(project);
    let requested_session = clean(room_session_id);
    let scope_result = resolve_active_scope(state, requested_project.clone(), requested_session);
    let scope = scope_result
        .get("scope")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let resolved_project = scope
        .get("project")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .filter(|v| !v.is_empty())
        .or(requested_project)
        .unwrap_or_default();
    let room_session_id = scope
        .get("room_session_id")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let plans = load_all_plans_internal(state);
    let relevant_plans: Vec<Value> = plans
        .iter()
        .filter(|plan| {
            plan.status == PlanStatus::Active
                && (resolved_project.is_empty()
                    || plan
                        .steps
                        .iter()
                        .any(|step| step.project == resolved_project))
        })
        .take(5)
        .map(|plan| {
            let done = plan
                .steps
                .iter()
                .filter(|step| step.status.is_terminal())
                .count();
            let running = plan
                .steps
                .iter()
                .filter(|step| step.status.to_string() == "running")
                .count();
            let next_step = plan
                .steps
                .iter()
                .find(|step| !step.status.is_terminal())
                .map(|step| {
                    json!({
                        "id": step.id,
                        "project": step.project,
                        "task": step.task,
                        "status": step.status,
                        "work_item_id": step.work_item_id,
                        "delegation_id": step.delegation_id
                    })
                });
            json!({
                "id": plan.id,
                "title": plan.title,
                "status": plan.status,
                "updated": plan.updated,
                "counts": {
                    "steps": plan.steps.len(),
                    "done": done,
                    "running": running,
                    "open": plan.steps.len().saturating_sub(done)
                },
                "next_step": next_step
            })
        })
        .collect();

    let project_sessions: Vec<Value> = state
        .project_sessions
        .lock()
        .map(|sessions| {
            let mut items: Vec<_> = sessions
                .values()
                .filter(|session| {
                    (resolved_project.is_empty() || session.project == resolved_project)
                        && (room_session_id
                            .as_ref()
                            .map(|id| session.parent_room_session_id == *id)
                            .unwrap_or(true))
                        && !matches!(session.status, ProjectSessionStatus::Closed)
                })
                .cloned()
                .collect();
            items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            items
                .into_iter()
                .take(5)
                .map(|session| {
                    json!({
                        "id": session.id,
                        "project": session.project,
                        "title": session.title,
                        "status": session.status,
                        "executor_provider": session.executor_provider,
                        "reviewer_provider": session.reviewer_provider,
                        "linked_work_items": session.linked_work_item_ids.len(),
                        "updated_at": session.updated_at
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let work_items: Vec<Value> = state
        .work_items
        .lock()
        .map(|items| {
            let mut relevant: Vec<_> = items
                .values()
                .filter(|item| {
                    (resolved_project.is_empty() || item.project == resolved_project)
                        && (room_session_id
                            .as_ref()
                            .map(|id| item.parent_room_session_id == *id)
                            .unwrap_or(true))
                        && active_work_item_status(&item.status)
                })
                .cloned()
                .collect();
            relevant.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            relevant
                .into_iter()
                .take(8)
                .map(|item| {
                    json!({
                        "id": item.id,
                        "project": item.project,
                        "title": item.title,
                        "task": item.task,
                        "status": item.status,
                        "executor_provider": item.executor_provider,
                        "reviewer_provider": item.reviewer_provider,
                        "write_intent": item.write_intent,
                        "declared_paths": item.declared_paths,
                        "delegation_id": item.delegation_id,
                        "source_kind": item.source_kind,
                        "source_id": item.source_id,
                        "updated_at": item.updated_at
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let delegation_values: Vec<Value> = state
        .delegations
        .lock()
        .map(|delegations| {
            let mut items: Vec<_> = delegations
                .values()
                .filter(|d| resolved_project.is_empty() || d.project == resolved_project)
                .cloned()
                .collect();
            items.sort_by(|a, b| b.ts.cmp(&a.ts));
            items
                .into_iter()
                .take(8)
                .map(|d| {
                    json!({
                        "id": d.id,
                        "project": d.project,
                        "status": d.status,
                        "task": d.task,
                        "room_session_id": d.room_session_id,
                        "project_session_id": d.project_session_id,
                        "work_item_id": d.work_item_id,
                        "created_at": d.ts
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let delegation_counts = state
        .delegations
        .lock()
        .map(|delegations| {
            let mut pending = 0usize;
            let mut running = 0usize;
            let mut failed = 0usize;
            let mut done = 0usize;
            for d in delegations
                .values()
                .filter(|d| resolved_project.is_empty() || d.project == resolved_project)
            {
                let s = d.status.to_string();
                if matches!(
                    s.as_str(),
                    "running" | "escalated" | "deciding" | "verifying"
                ) {
                    running += 1;
                } else if matches!(s.as_str(), "failed" | "rejected" | "cancelled") {
                    failed += 1;
                } else if s == "done" {
                    done += 1;
                } else {
                    pending += 1;
                }
            }
            json!({ "pending": pending, "running": running, "failed": failed, "done": done })
        })
        .unwrap_or_else(|_| json!({ "pending": 0, "running": 0, "failed": 0, "done": 0 }));

    let lease_counts = state
        .file_leases
        .lock()
        .map(|leases| {
            let active: Vec<Value> = leases
                .values()
                .filter(|lease| {
                    (resolved_project.is_empty() || lease.project == resolved_project)
                        && matches!(lease.status, FileLeaseStatus::Active)
                })
                .map(|lease| {
                    json!({
                        "id": lease.id,
                        "project": lease.project,
                        "work_item_id": lease.work_item_id,
                        "provider": lease.provider,
                        "paths": lease.paths,
                        "write_intent": lease.write_intent,
                        "updated_at": lease.updated_at
                    })
                })
                .collect();
            json!({ "active": active.len(), "items": active.into_iter().take(5).collect::<Vec<_>>() })
        })
        .unwrap_or_else(|_| json!({ "active": 0, "items": [] }));

    let graph_context = if resolved_project.is_empty() {
        json!({
            "available": false,
            "project": "",
            "reason": "global scope",
            "label": "overview graph"
        })
    } else {
        let project_for_graph = resolved_project.clone();
        match super::graph_scan::build_project_graph(state, &project_for_graph) {
            Ok(graph) => json!({
                "available": !graph.nodes.is_empty(),
                "project": project_for_graph.clone(),
                "nodes": graph.stats.total_nodes,
                "edges": graph.stats.total_edges,
                "cycles": graph.stats.cycle_count,
                "context_chars": super::graph_ops::build_graph_context(state, &project_for_graph).len(),
                "label": "[GRAPH_CONTEXT]"
            }),
            Err(e) => json!({
                "available": false,
                "project": project_for_graph.clone(),
                "reason": e,
                "label": "[GRAPH_CONTEXT]"
            }),
        }
    };

    let project_agent_routes =
        collect_project_agent_routes(state, &resolved_project, room_session_id.as_deref());
    let route_progress = collect_route_progress_aggregate(&project_agent_routes);
    let managerial_leverage = collect_managerial_leverage(
        state,
        &resolved_project,
        room_session_id.as_deref(),
        &project_agent_routes,
        &route_progress,
        &graph_context,
        &plans,
    );

    json!({
        "status": "ok",
        "big_plan": {
            "stage": "live_route_progress",
            "stage_index": 9,
            "stage_total": 9,
            "label": "Live route progress + operational control",
            "done": ["foundation", "route_card", "live_transcript", "orchestration_map", "execution_timeline", "event_contract", "project_agent_routing", "route_lane_stabilization"],
            "current": ["live_route_progress", "needs_user_visibility", "review_lifecycle", "operational_control"],
            "next": ["provider_expansion", "project_rollout", "agent_board"]
        },
        "scope": scope,
        "project": resolved_project,
        "plans": relevant_plans,
        "project_sessions": project_sessions,
        "work_items": work_items,
        "project_agent_routes": project_agent_routes,
        "route_progress": route_progress,
        "managerial_leverage": managerial_leverage,
        "delegations": {
            "counts": delegation_counts,
            "items": delegation_values
        },
        "leases": lease_counts,
        "graph_context": graph_context
    })
}

pub fn resolve_active_scope(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
) -> Value {
    let requested_project = clean(project);
    let session = active_session_for(state, clean(room_session_id), requested_project.clone());
    let resolved_project = session
        .as_ref()
        .map(|s| s.project.trim().to_string())
        .filter(|p| !p.is_empty())
        .or(requested_project)
        .unwrap_or_default();
    let session_ref = session.as_ref();

    if let Some(item) = latest_linked_work_item(state, session_ref, &resolved_project) {
        let kind = "work_item";
        let title = if item.title.trim().is_empty() {
            item.task.clone()
        } else {
            item.title.clone()
        };
        return json!({
            "status": "ok",
            "scope": {
                "kind": kind,
                "label": label_for_kind(kind),
                "title": title,
                "project": item.project,
                "room_session_id": item.parent_room_session_id,
                "work_item_id": item.id,
                "source_kind": item.source_kind,
                "source_id": item.source_id,
                "breadcrumbs": breadcrumbs(kind, &item.project, &title),
                "available_actions": actions_for(kind),
                "summary": summary_for(kind, &title),
            }
        });
    }

    let plans = load_all_plans_internal(state);
    if let Some(session) = session_ref {
        for plan_id in session.linked_plan_ids.iter().rev() {
            if let Some(plan) = plans.iter().find(|plan| &plan.id == plan_id) {
                let kind = "plan";
                return json!({
                    "status": "ok",
                    "scope": {
                        "kind": kind,
                        "label": label_for_kind(kind),
                        "title": plan.title,
                        "project": resolved_project,
                        "room_session_id": session.id,
                        "plan_id": plan.id,
                        "counts": {
                            "steps": plan.steps.len(),
                            "active_steps": plan.steps.iter().filter(|step| !step.status.is_terminal()).count()
                        },
                        "breadcrumbs": breadcrumbs(kind, &resolved_project, &plan.title),
                        "available_actions": actions_for(kind),
                        "summary": summary_for(kind, &plan.title),
                    }
                });
            }
        }
    }
    if let Some(plan) = plans.iter().find(|plan| {
        plan.status == PlanStatus::Active
            && plan
                .steps
                .iter()
                .any(|step| !resolved_project.is_empty() && step.project == resolved_project)
    }) {
        let kind = "plan";
        return json!({
            "status": "ok",
            "scope": {
                "kind": kind,
                "label": label_for_kind(kind),
                "title": plan.title,
                "project": resolved_project,
                "room_session_id": session_ref.map(|s| s.id.clone()),
                "plan_id": plan.id,
                "counts": {
                    "steps": plan.steps.len(),
                    "active_steps": plan.steps.iter().filter(|step| !step.status.is_terminal()).count()
                },
                "breadcrumbs": breadcrumbs(kind, &resolved_project, &plan.title),
                "available_actions": actions_for(kind),
                "summary": summary_for(kind, &plan.title),
            }
        });
    }

    let strategies = load_strategies(state);
    if let Some(session) = session_ref {
        for strategy_id in session.linked_strategy_ids.iter().rev() {
            if let Some(strategy) = strategies
                .iter()
                .find(|strategy| &strategy.id == strategy_id)
            {
                let kind = "strategy";
                return json!({
                    "status": "ok",
                    "scope": {
                        "kind": kind,
                        "label": label_for_kind(kind),
                        "title": strategy.title,
                        "project": resolved_project,
                        "room_session_id": session.id,
                        "strategy_id": strategy.id,
                        "counts": {
                            "tactics": strategy.all_tactics().len(),
                            "steps": strategy.all_steps_flat().len()
                        },
                        "breadcrumbs": breadcrumbs(kind, &resolved_project, &strategy.title),
                        "available_actions": actions_for(kind),
                        "summary": summary_for(kind, &strategy.title),
                    }
                });
            }
        }
    }

    let kind = if resolved_project.is_empty() {
        "global"
    } else {
        "project"
    };
    let title = if resolved_project.is_empty() {
        "_orchestrator".to_string()
    } else {
        resolved_project.clone()
    };
    json!({
        "status": "ok",
        "scope": {
            "kind": kind,
            "label": label_for_kind(kind),
            "title": title,
            "project": resolved_project,
            "room_session_id": session_ref.map(|s| s.id.clone()),
            "breadcrumbs": breadcrumbs(kind, &resolved_project, &title),
            "available_actions": actions_for(kind),
            "summary": summary_for(kind, &title),
            "counts": session_ref.map(|session| json!({
                "linked_strategies": session.linked_strategy_ids.len(),
                "linked_plans": session.linked_plan_ids.len(),
                "linked_work_items": session.linked_work_item_ids.len()
            }))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{resolve_active_scope, resolve_orchestration_map};
    use crate::commands::plans::{save_plan_internal, Plan, PlanStep};
    use crate::commands::provider_runner::ProviderKind;
    use crate::commands::status::{
        DelegationStatus, PlanStatus, PlanStepStatus, StepStatus, StrategyStatus,
    };
    use crate::commands::strategy_models::{
        save_strategies, Assignee, Plan as StrategyPlan, Step, Strategy, Tactic, TacticStatus,
    };
    use crate::state::{
        AppState, Delegation, MultiAgentSession, ProjectSession, ProjectSessionStatus,
        ReviewVerdict, ReviewVerdictStatus, SessionMode, SessionStatus, WorkItem, WorkItemAssignee,
        WorkItemStatus, WorkItemWriteIntent,
    };
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-scope-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create temp root");
        path
    }

    fn test_delegation(id: &str, project: &str, status: DelegationStatus, ts: &str) -> Delegation {
        Delegation {
            id: id.to_string(),
            project: project.to_string(),
            task: "Blocked project-agent task".to_string(),
            ts: ts.to_string(),
            status,
            response: None,
            retries: 0,
            plan_id: None,
            plan_step: None,
            escalation_info: None,
            strategy_id: None,
            strategy_step_id: None,
            room_session_id: Some("room-1".to_string()),
            project_session_id: None,
            work_item_id: None,
            executor_provider: Some(ProviderKind::Codex),
            reviewer_provider: Some(ProviderKind::Claude),
            git_diff: None,
            usage: None,
            scheduled_at: None,
            batch_id: None,
            priority: None,
            timeout_secs: None,
            gate_result: None,
            review_verdict: None,
        }
    }

    #[test]
    fn linked_plan_beats_project_fallback() {
        let root = test_root("linked-plan");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        let plan = Plan {
            id: "plan-1".to_string(),
            title: "Release scope plan".to_string(),
            status: PlanStatus::Active,
            steps: Vec::new(),
            room_session_id: Some("room-1".to_string()),
            created: ts.clone(),
            updated: ts.clone(),
        };
        save_plan_internal(&state, &plan);
        state.sessions.lock().unwrap().insert(
            "room-1".to_string(),
            MultiAgentSession {
                id: "room-1".to_string(),
                title: "Project room".to_string(),
                project: "AgentOS".to_string(),
                status: SessionStatus::Active,
                mode: SessionMode::Review,
                participants: Vec::new(),
                orchestrator_participant_id: None,
                current_working_set: Vec::new(),
                active_round_id: None,
                active_speaker: None,
                presence: Default::default(),
                pending_challenge: None,
                pending_rebuttal: None,
                linked_strategy_ids: Vec::new(),
                linked_project_session_ids: Vec::new(),
                linked_work_item_ids: Vec::new(),
                linked_tactic_ids: Vec::new(),
                linked_plan_ids: vec!["plan-1".to_string()],
                linked_delegation_ids: Vec::new(),
                created_at: ts.clone(),
                updated_at: ts,
            },
        );

        let result = resolve_active_scope(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        assert_eq!(result["status"], "ok");
        assert_eq!(result["scope"]["kind"], "plan");
        assert_eq!(result["scope"]["plan_id"], "plan-1");
        assert_eq!(result["scope"]["project"], "AgentOS");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_big_plan_stage() {
        let root = test_root("map-stage");
        let state = AppState::new(root.clone());

        let result = resolve_orchestration_map(&state, Some("AgentOS".to_string()), None);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["big_plan"]["stage"], "live_route_progress");
        assert_eq!(result["big_plan"]["stage_index"], 9);
        assert_eq!(result["scope"]["kind"], "project");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_project_agent_routes() {
        let root = test_root("route-lanes");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.project_sessions.lock().unwrap().insert(
            "ps-1".to_string(),
            ProjectSession {
                id: "ps-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project: "AgentOS".to_string(),
                title: "AgentOS release lane".to_string(),
                status: ProjectSessionStatus::Active,
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                linked_work_item_ids: vec!["wi-1".to_string()],
                created_at: ts.clone(),
                updated_at: ts.clone(),
            },
        );
        state.work_items.lock().unwrap().insert(
            "wi-1".to_string(),
            WorkItem {
                id: "wi-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project_session_id: Some("ps-1".to_string()),
                project: "AgentOS".to_string(),
                title: "Ship routing lane".to_string(),
                task: "Expose project-agent route lanes".to_string(),
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                assignee: WorkItemAssignee::Agent,
                write_intent: WorkItemWriteIntent::ProposeWrite,
                declared_paths: vec!["src-ui/chat.js".to_string()],
                verify: None,
                status: WorkItemStatus::Ready,
                delegation_id: None,
                result: None,
                review_verdict: None,
                source_kind: Some("plan_step".to_string()),
                source_id: Some("step-1".to_string()),
                created_at: ts.clone(),
                updated_at: ts,
            },
        );

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        assert_eq!(result["status"], "ok");
        assert_eq!(result["project_agent_routes"][0]["route_state"], "ready");
        assert_eq!(result["project_agent_routes"][0]["action"], "queue_next");
        assert_eq!(result["project_agent_routes"][0]["can_queue_next"], true);
        assert_eq!(
            result["project_agent_routes"][0]["progress"]["phase"],
            "ready"
        );
        assert_eq!(result["route_progress"]["queueable"], 1);
        assert_eq!(
            result["project_agent_routes"][0]["next_work_item"]["id"],
            "wi-1"
        );
        assert_eq!(
            result["project_agent_routes"][0]["executor_provider"],
            "codex"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_managerial_leverage() {
        let root = test_root("managerial-leverage");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        save_plan_internal(
            &state,
            &Plan {
                id: "plan-1".to_string(),
                title: "Leverage plan".to_string(),
                status: PlanStatus::Active,
                steps: vec![PlanStep {
                    id: "step-1".to_string(),
                    project: "AgentOS".to_string(),
                    task: "Route work through cross-check".to_string(),
                    assignee: Assignee::Agent,
                    verify: None,
                    work_item_id: Some("wi-1".to_string()),
                    status: PlanStepStatus::Pending,
                    delegation_id: None,
                    result: None,
                }],
                room_session_id: Some("room-1".to_string()),
                created: ts.clone(),
                updated: ts.clone(),
            },
        );
        save_strategies(
            &state,
            &[Strategy {
                id: "strat-1".to_string(),
                goal_id: "goal-1".to_string(),
                title: "Improve orchestration leverage".to_string(),
                tactics: vec![Tactic {
                    id: "tactic-1".to_string(),
                    title: "Execution quality".to_string(),
                    category: None,
                    plans: vec![StrategyPlan {
                        project: "AgentOS".to_string(),
                        steps: vec![Step {
                            id: "s1".to_string(),
                            task: "Cross-check execution lane".to_string(),
                            status: StepStatus::Approved,
                            response: None,
                            depends_on: Vec::new(),
                            delegation_id: None,
                            assignee: Assignee::Agent,
                            verify: None,
                        }],
                        priority: "HIGH".to_string(),
                        depends_on: Vec::new(),
                        category: None,
                        context: "Quality through reviewer coverage".to_string(),
                    }],
                    status: TacticStatus::Active,
                }],
                plans: Vec::new(),
                status: StrategyStatus::Approved,
                created: ts.clone(),
                room_session_id: Some("room-1".to_string()),
                category: None,
                deadline: None,
                metrics: None,
            }],
        );
        state.project_sessions.lock().unwrap().insert(
            "ps-1".to_string(),
            ProjectSession {
                id: "ps-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project: "AgentOS".to_string(),
                title: "AgentOS leverage lane".to_string(),
                status: ProjectSessionStatus::Active,
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                linked_work_item_ids: vec!["wi-1".to_string()],
                created_at: ts.clone(),
                updated_at: ts.clone(),
            },
        );
        state.work_items.lock().unwrap().insert(
            "wi-1".to_string(),
            WorkItem {
                id: "wi-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project_session_id: Some("ps-1".to_string()),
                project: "AgentOS".to_string(),
                title: "Queue leverage work".to_string(),
                task: "Start a cross-checked route lane".to_string(),
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                assignee: WorkItemAssignee::Agent,
                write_intent: WorkItemWriteIntent::ProposeWrite,
                declared_paths: vec!["src-ui/chat.js".to_string()],
                verify: None,
                status: WorkItemStatus::Ready,
                delegation_id: None,
                result: None,
                review_verdict: None,
                source_kind: Some("plan_step".to_string()),
                source_id: Some("step-1".to_string()),
                created_at: ts.clone(),
                updated_at: ts,
            },
        );

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        let leverage = &result["managerial_leverage"];
        assert_eq!(leverage["alignment"]["status"], "aligned");
        assert_eq!(leverage["quality"]["status"], "strong");
        assert_eq!(leverage["control"]["load"], "low");
        assert_eq!(leverage["parallelism"]["queueable_routes"], 1);
        assert_eq!(leverage["quality"]["cross_provider_routes"], 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_blocker_only_route_lanes() {
        let root = test_root("route-blocker-only");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.delegations.lock().unwrap().insert(
            "del-1".to_string(),
            test_delegation("del-1", "AgentOS", DelegationStatus::NeedsPermission, &ts),
        );

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        assert_eq!(result["status"], "ok");
        let route = &result["project_agent_routes"][0];
        assert_eq!(route["route_state"], "needs_user");
        assert_eq!(route["action"], "resolve_user_blocker");
        assert_eq!(route["synthetic"], true);
        assert_eq!(route["has_blockers"], true);
        assert_eq!(route["progress"]["needs_user"], true);
        assert_eq!(route["blocker_delegation_ids"][0], "del-1");
        assert_eq!(result["route_progress"]["needs_user"], 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_keeps_running_state_with_blocker_metadata() {
        let root = test_root("route-running-blocked");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.work_items.lock().unwrap().insert(
            "wi-1".to_string(),
            WorkItem {
                id: "wi-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project_session_id: None,
                project: "AgentOS".to_string(),
                title: "Run active work".to_string(),
                task: "Keep active work visible despite blockers".to_string(),
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                assignee: WorkItemAssignee::Agent,
                write_intent: WorkItemWriteIntent::ProposeWrite,
                declared_paths: vec!["src-ui/chat.js".to_string()],
                verify: None,
                status: WorkItemStatus::Running,
                delegation_id: None,
                result: None,
                review_verdict: None,
                source_kind: None,
                source_id: None,
                created_at: ts.clone(),
                updated_at: ts.clone(),
            },
        );
        let mut delegation = test_delegation("del-1", "AgentOS", DelegationStatus::Failed, &ts);
        delegation.work_item_id = Some("wi-1".to_string());
        state
            .delegations
            .lock()
            .unwrap()
            .insert("del-1".to_string(), delegation);

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        assert_eq!(result["status"], "ok");
        let route = &result["project_agent_routes"][0];
        assert_eq!(route["route_state"], "needs_user");
        assert_eq!(route["has_blockers"], true);
        assert_eq!(route["counts"]["blocked"], 1);
        assert_eq!(route["progress"]["active_delegation_id"], "del-1");
        assert_eq!(route["blocker_delegation_ids"][0], "del-1");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_verifying_route_progress() {
        let root = test_root("route-verifying");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.delegations.lock().unwrap().insert(
            "del-verify".to_string(),
            test_delegation("del-verify", "AgentOS", DelegationStatus::Verifying, &ts),
        );

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        let route = &result["project_agent_routes"][0];
        assert_eq!(route["route_state"], "verifying");
        assert_eq!(route["action"], "monitor");
        assert_eq!(route["progress"]["active_delegation_id"], "del-verify");
        assert_eq!(result["route_progress"]["active"], 1);
        assert_eq!(result["route_progress"]["counts"]["verifying"], 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn orchestration_map_reports_review_verdict_progress() {
        let root = test_root("route-review-verdict");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.project_sessions.lock().unwrap().insert(
            "ps-1".to_string(),
            ProjectSession {
                id: "ps-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project: "AgentOS".to_string(),
                title: "Review verdict lane".to_string(),
                status: ProjectSessionStatus::Active,
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                linked_work_item_ids: vec!["wi-1".to_string()],
                created_at: ts.clone(),
                updated_at: ts.clone(),
            },
        );
        state.work_items.lock().unwrap().insert(
            "wi-1".to_string(),
            WorkItem {
                id: "wi-1".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project_session_id: Some("ps-1".to_string()),
                project: "AgentOS".to_string(),
                title: "Review completed route".to_string(),
                task: "Keep reviewer verdict visible after completion".to_string(),
                executor_provider: ProviderKind::Codex,
                reviewer_provider: Some(ProviderKind::Claude),
                assignee: WorkItemAssignee::Agent,
                write_intent: WorkItemWriteIntent::ProposeWrite,
                declared_paths: vec!["src-ui/chat.js".to_string()],
                verify: None,
                status: WorkItemStatus::Completed,
                delegation_id: Some("del-1".to_string()),
                result: Some("Implemented".to_string()),
                review_verdict: Some(ReviewVerdict {
                    status: ReviewVerdictStatus::Warn,
                    summary: "Works, but needs follow-up smoke test".to_string(),
                    next_action: Some("Run smoke test".to_string()),
                    source_response: "WARN\nWorks, but needs follow-up smoke test".to_string(),
                }),
                source_kind: Some("plan_step".to_string()),
                source_id: Some("step-1".to_string()),
                created_at: ts.clone(),
                updated_at: ts,
            },
        );

        let result = resolve_orchestration_map(
            &state,
            Some("AgentOS".to_string()),
            Some("room-1".to_string()),
        );
        let route = &result["project_agent_routes"][0];
        assert_eq!(route["route_state"], "done");
        assert_eq!(route["action"], "review_result");
        assert_eq!(route["progress"]["review_verdict_status"], "warn");
        assert_eq!(route["progress"]["active_work_item_id"], "wi-1");
        assert_eq!(result["route_progress"]["done"], 1);

        let _ = std::fs::remove_dir_all(root);
    }
}
