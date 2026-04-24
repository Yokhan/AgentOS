//! Active orchestration scope resolver.
//!
//! This is intentionally read-only: UI should ask for "where am I now?" without
//! creating another source of truth beside sessions, plans, and work items.

use crate::commands::plans::load_all_plans_internal;
use crate::commands::status::PlanStatus;
use crate::commands::strategy_models::load_strategies;
use crate::state::{AppState, MultiAgentSession, SessionStatus, WorkItem};
use serde_json::{json, Value};
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
    use super::resolve_active_scope;
    use crate::commands::plans::{save_plan_internal, Plan};
    use crate::commands::status::PlanStatus;
    use crate::state::{AppState, MultiAgentSession, SessionMode, SessionStatus};
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
}
