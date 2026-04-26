//! Active orchestration scope resolver.
//!
//! This is intentionally read-only: UI should ask for "where am I now?" without
//! creating another source of truth beside sessions, plans, and work items.

use crate::commands::plans::load_all_plans_internal;
use crate::commands::status::PlanStatus;
use crate::commands::strategy_models::load_strategies;
use crate::state::{
    AppState, FileLeaseStatus, MultiAgentSession, ProjectSessionStatus, SessionStatus, WorkItem,
    WorkItemStatus,
};
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

    json!({
        "status": "ok",
        "big_plan": {
            "stage": "routing",
            "stage_index": 3,
            "stage_total": 6,
            "label": "Project routing + plan/work-item visibility",
            "done": ["foundation", "route_card", "live_transcript"],
            "current": ["orchestration_map", "project_agents", "code_context_status"],
            "next": ["execution_timeline", "event_unification", "provider_adapters"]
        },
        "scope": scope,
        "project": resolved_project,
        "plans": relevant_plans,
        "project_sessions": project_sessions,
        "work_items": work_items,
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

    #[test]
    fn orchestration_map_reports_big_plan_stage() {
        let root = test_root("map-stage");
        let state = AppState::new(root.clone());

        let result = resolve_orchestration_map(&state, Some("AgentOS".to_string()), None);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["big_plan"]["stage"], "routing");
        assert_eq!(result["big_plan"]["stage_index"], 3);
        assert_eq!(result["scope"]["kind"], "project");

        let _ = std::fs::remove_dir_all(root);
    }
}
