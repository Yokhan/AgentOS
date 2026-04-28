//! Branching execution map: read-only projection over normalized timeline events.
//!
//! The map is a UI projection, not a new source of truth. It uses persisted chat
//! stream events, Duo session events, and delegation state/streams.

use crate::commands::event_contract::EVENT_SCHEMA_VERSION;
use crate::commands::timeline::build_execution_timeline;
use crate::state::{AppState, Delegation};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::State;

const EXECUTION_MAP_SCHEMA_VERSION: &str = "agentos.execution_map.v1";

fn field(row: &Value, key: &str) -> String {
    row.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn clean_project(project: &str) -> String {
    let value = project.trim();
    if value.is_empty() {
        "_orchestrator".to_string()
    } else {
        value.to_string()
    }
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn lane_id_for(row: &Value) -> String {
    let source = field(row, "source");
    let project = clean_project(&field(row, "project"));
    if source == "delegation" && project != "_orchestrator" {
        format!("project:{}", safe_id(&project))
    } else if source == "duo" {
        "duo".to_string()
    } else {
        "orchestrator".to_string()
    }
}

fn lane_label_for(id: &str, row: Option<&Value>) -> String {
    if id == "orchestrator" {
        return "Orchestrator".to_string();
    }
    if id == "duo" {
        return "Duo room".to_string();
    }
    row.map(|v| clean_project(&field(v, "project")))
        .filter(|v| !v.is_empty() && v != "_orchestrator")
        .unwrap_or_else(|| id.replace("project:", ""))
}

fn lane_kind(id: &str) -> &'static str {
    if id == "orchestrator" {
        "orchestrator"
    } else if id == "duo" {
        "room"
    } else {
        "project"
    }
}

fn status_weight(status: &str) -> u8 {
    match status {
        "needs_permission" | "pending" | "waiting" => 5,
        "failed" | "error" | "cancelled" | "warning" => 4,
        "running" | "escalated" | "verifying" | "deciding" => 3,
        "scheduled" | "queued" => 2,
        "done" | "complete" | "completed" => 1,
        _ => 0,
    }
}

fn merge_status(current: &str, next: &str) -> String {
    if status_weight(next) >= status_weight(current) {
        next.to_string()
    } else {
        current.to_string()
    }
}

fn is_waiting(status: &str) -> bool {
    matches!(status, "pending" | "needs_permission" | "waiting")
}

fn is_terminal_feedback(status: &str, kind: &str) -> bool {
    matches!(
        status,
        "done" | "complete" | "completed" | "failed" | "error" | "cancelled" | "warning"
    ) || matches!(kind, "done" | "tool_result" | "review_verdict")
}

fn delegation_provider_by_project(state: &AppState) -> HashMap<String, String> {
    state
        .delegations
        .lock()
        .map(|items| {
            items
                .values()
                .filter_map(|delegation| {
                    let provider = delegation
                        .executor_provider
                        .as_ref()
                        .or(delegation.reviewer_provider.as_ref())
                        .map(|provider| provider.to_string())
                        .unwrap_or_else(|| "project-agent".to_string());
                    Some((delegation.project.clone(), provider))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn provider_for_delegation(delegation: &Delegation) -> String {
    delegation
        .executor_provider
        .as_ref()
        .or(delegation.reviewer_provider.as_ref())
        .map(|provider| provider.to_string())
        .unwrap_or_else(|| "project-agent".to_string())
}

fn live_delegations(state: &AppState, project: &str) -> Vec<Delegation> {
    let project = project.trim();
    let mut items: Vec<Delegation> = state
        .delegations
        .lock()
        .map(|delegations| {
            delegations
                .values()
                .filter(|delegation| {
                    (project.is_empty() || delegation.project == project)
                        && !delegation.status.is_terminal()
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| a.ts.cmp(&b.ts));
    items
}

fn waiting_delegations(state: &AppState, project: &str) -> Vec<Value> {
    let project = project.trim();
    let mut items: Vec<Delegation> = state
        .delegations
        .lock()
        .map(|delegations| {
            delegations
                .values()
                .filter(|delegation| {
                    (project.is_empty() || delegation.project == project)
                        && matches!(
                            delegation.status.to_string().as_str(),
                            "pending" | "needs_permission"
                        )
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| b.ts.cmp(&a.ts));
    items
        .into_iter()
        .map(|delegation| {
            json!({
                "id": delegation.id,
                "project": delegation.project,
                "status": delegation.status.to_string(),
                "task": crate::commands::event_contract::short(&delegation.task, 220),
                "action": "approve",
                "ts": delegation.ts,
                "started_at": delegation.started_at
            })
        })
        .collect()
}

pub fn build_execution_map(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: usize,
) -> Value {
    let project_filter = project.unwrap_or_default();
    let limit = limit.clamp(20, 180);
    let timeline = build_execution_timeline(
        state,
        Some(project_filter.clone()),
        room_session_id.clone(),
        limit,
    );
    let rows = timeline
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let providers = delegation_provider_by_project(state);
    let fallback_ts = rows
        .first()
        .and_then(|row| row.get("ts"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .unwrap_or_else(|| state.now_iso());

    let root_event = json!({
        "id": "evt-root",
        "lane_id": "orchestrator",
        "branch_id": "orchestrator",
        "source": "system",
        "kind": "root",
        "status": if rows.is_empty() { "idle" } else { "running" },
        "title": "Orchestrator context",
        "detail": if rows.is_empty() { "No recent execution events." } else { "Execution map projection started." },
        "project": if project_filter.trim().is_empty() { "_orchestrator" } else { project_filter.as_str() },
        "provider": "orchestrator",
        "model": "",
        "ts": fallback_ts,
        "sequence": 0
    });

    let mut lanes: HashMap<String, Value> = HashMap::new();
    let mut lane_status: HashMap<String, String> = HashMap::new();
    let mut lane_order: HashMap<String, usize> = HashMap::new();
    let mut events = vec![root_event];
    let mut edges = Vec::new();
    let mut seen_lanes: HashSet<String> = HashSet::new();
    let mut latest_project_event: HashMap<String, String> = HashMap::new();
    let mut feedback_count = 0usize;

    lanes.insert(
        "orchestrator".to_string(),
        json!({
            "id": "orchestrator",
            "kind": "orchestrator",
            "label": "Orchestrator",
            "project": "_orchestrator",
            "status": if rows.is_empty() { "idle" } else { "running" },
            "provider": "orchestrator",
            "model": "",
            "order": 0
        }),
    );
    lane_status.insert(
        "orchestrator".to_string(),
        if rows.is_empty() { "idle" } else { "running" }.to_string(),
    );
    lane_order.insert("orchestrator".to_string(), 0);

    for (idx, row) in rows.iter().enumerate() {
        let source = field(row, "source");
        let kind = field(row, "kind");
        let status = field(row, "status");
        let project = clean_project(&field(row, "project"));
        let lane_id = lane_id_for(row);
        let event_id = format!("evt-{}-{}-{}", idx + 1, safe_id(&lane_id), safe_id(&kind));
        let provider = if source == "delegation" {
            providers
                .get(&project)
                .cloned()
                .unwrap_or_else(|| "project-agent".to_string())
        } else if source == "duo" {
            "duo".to_string()
        } else {
            source.clone()
        };

        if !lanes.contains_key(&lane_id) {
            let order = lanes.len();
            lanes.insert(
                lane_id.clone(),
                json!({
                    "id": lane_id,
                    "kind": lane_kind(&lane_id),
                    "label": lane_label_for(&lane_id, Some(row)),
                    "project": project,
                    "status": status,
                    "provider": provider,
                    "model": "",
                    "order": order
                }),
            );
            lane_order.insert(lane_id.clone(), order);
        }
        let current_status = lane_status.get(&lane_id).cloned().unwrap_or_default();
        lane_status.insert(lane_id.clone(), merge_status(&current_status, &status));

        if lane_id != "orchestrator" && seen_lanes.insert(lane_id.clone()) {
            edges.push(json!({
                "id": format!("edge-spawn-{}", safe_id(&lane_id)),
                "from": "evt-root",
                "to": event_id,
                "type": "spawn",
                "label": "delegated",
                "status": status
            }));
        }

        let event = json!({
            "id": event_id,
            "lane_id": lane_id,
            "branch_id": lane_id,
            "source": source,
            "kind": kind,
            "status": status,
            "title": field(row, "title"),
            "detail": field(row, "detail"),
            "project": project,
            "provider": provider,
            "model": "",
            "ts": field(row, "ts"),
            "sequence": idx + 1
        });
        events.push(event);

        if lane_id != "orchestrator" {
            latest_project_event.insert(lane_id.clone(), event_id.clone());
        }

        if lane_id != "orchestrator" && is_terminal_feedback(&status, &kind) {
            feedback_count += 1;
            let feedback_id = format!("evt-feedback-{}-{}", feedback_count, safe_id(&project));
            events.push(json!({
                "id": feedback_id,
                "lane_id": "orchestrator",
                "branch_id": "orchestrator",
                "source": "feedback",
                "kind": "feedback_received",
                "status": status,
                "title": format!("Feedback from {}", lane_label_for(&lane_id, Some(row))),
                "detail": field(row, "detail"),
                "project": project,
                "provider": provider,
                "model": "",
                "ts": field(row, "ts"),
                "sequence": idx + 1
            }));
            edges.push(json!({
                "id": format!("edge-merge-{}-{}", feedback_count, safe_id(&lane_id)),
                "from": event_id,
                "to": feedback_id,
                "type": "merge",
                "label": "feedback",
                "status": status
            }));
        }
    }

    for (live_idx, delegation) in live_delegations(state, &project_filter)
        .into_iter()
        .enumerate()
    {
        let project = clean_project(&delegation.project);
        let lane_id = format!("project:{}", safe_id(&project));
        let provider = provider_for_delegation(&delegation);
        let status = delegation.status.to_string();
        let event_id = format!("evt-live-delegation-{}", safe_id(&delegation.id));
        if !lanes.contains_key(&lane_id) {
            let order = lanes.len();
            lanes.insert(
                lane_id.clone(),
                json!({
                    "id": lane_id,
                    "kind": lane_kind(&lane_id),
                    "label": project,
                    "project": project,
                    "status": status,
                    "provider": provider,
                    "model": "",
                    "order": order
                }),
            );
            lane_order.insert(lane_id.clone(), order);
        }
        let current_status = lane_status.get(&lane_id).cloned().unwrap_or_default();
        lane_status.insert(lane_id.clone(), merge_status(&current_status, &status));
        if seen_lanes.insert(lane_id.clone()) {
            edges.push(json!({
                "id": format!("edge-live-spawn-{}", safe_id(&lane_id)),
                "from": "evt-root",
                "to": event_id,
                "type": "spawn",
                "label": "live delegation",
                "status": status
            }));
        }
        latest_project_event.insert(lane_id.clone(), event_id.clone());
        events.push(json!({
            "id": event_id,
            "lane_id": lane_id,
            "branch_id": lane_id,
            "source": "delegation",
            "kind": "delegation_state",
            "status": status,
            "title": format!("{} delegation", delegation.status),
            "detail": crate::commands::event_contract::short(&delegation.task, 220),
            "project": project,
            "provider": provider,
            "model": "",
            "ts": delegation.started_at.clone().unwrap_or_else(|| delegation.ts.clone()),
            "queued_at": delegation.ts,
            "started_at": delegation.started_at,
            "delegation_id": delegation.id,
            "sequence": rows.len() + live_idx + 1
        }));
    }

    let mut lane_values: Vec<Value> = lanes
        .into_iter()
        .map(|(id, mut lane)| {
            if let Some(obj) = lane.as_object_mut() {
                if let Some(status) = lane_status.get(&id) {
                    obj.insert("status".to_string(), json!(status));
                }
                obj.insert(
                    "last_event_id".to_string(),
                    json!(latest_project_event
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| "evt-root".to_string())),
                );
            }
            lane
        })
        .collect();
    lane_values
        .sort_by_key(|lane| lane.get("order").and_then(|v| v.as_u64()).unwrap_or(999) as usize);

    let waiting = waiting_delegations(state, &project_filter);
    let blocked = lane_values
        .iter()
        .filter(|lane| {
            lane.get("status")
                .and_then(|v| v.as_str())
                .map(|status| matches!(status, "failed" | "error" | "cancelled" | "warning"))
                .unwrap_or(false)
        })
        .count();
    let waiting_lanes = lane_values
        .iter()
        .filter(|lane| {
            lane.get("status")
                .and_then(|v| v.as_str())
                .map(is_waiting)
                .unwrap_or(false)
        })
        .count();

    json!({
        "status": "ok",
        "schema_version": EXECUTION_MAP_SCHEMA_VERSION,
        "event_schema_version": EVENT_SCHEMA_VERSION,
        "project": if project_filter.trim().is_empty() { "_orchestrator" } else { project_filter.as_str() },
        "room_session_id": room_session_id.unwrap_or_default(),
        "big_plan": {
            "stage": "branching_execution_map",
            "stage_index": 10,
            "stage_total": 10,
            "label": "Branching execution map + live orchestration visibility"
        },
        "counts": {
            "lanes": lane_values.len(),
            "events": events.len(),
            "edges": edges.len(),
            "waiting": waiting.len() + waiting_lanes,
            "blocked": blocked
        },
        "lanes": lane_values,
        "events": events,
        "edges": edges,
        "waiting_for_user": waiting
    })
}

#[tauri::command]
pub fn get_execution_map(
    state: State<Arc<AppState>>,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: Option<usize>,
) -> Value {
    build_execution_map(&state, project, room_session_id, limit.unwrap_or(100))
}

#[cfg(test)]
mod tests {
    use super::build_execution_map;
    use crate::commands::jsonl::append_jsonl_logged;
    use crate::commands::provider_runner::ProviderKind;
    use crate::commands::status::DelegationStatus;
    use crate::state::{AppState, Delegation};
    use serde_json::json;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-execution-map-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("tasks")).expect("create temp tasks");
        path
    }

    fn delegation(id: &str, status: DelegationStatus) -> Delegation {
        Delegation {
            id: id.to_string(),
            project: "HealthTracker".to_string(),
            task: "Run project health check".to_string(),
            ts: "2026-04-28T08:00:03Z".to_string(),
            started_at: if status == DelegationStatus::Running {
                Some("2026-04-28T08:01:00Z".to_string())
            } else {
                None
            },
            status,
            response: None,
            retries: 0,
            plan_id: None,
            plan_step: None,
            escalation_info: None,
            strategy_id: None,
            strategy_step_id: None,
            room_session_id: None,
            project_session_id: None,
            work_item_id: None,
            executor_provider: Some(ProviderKind::Codex),
            reviewer_provider: None,
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
    fn builds_branching_map_from_stream_and_pending_delegation() {
        let root = test_root("basic");
        let state = AppState::new(root.clone());
        let chat_path = root.join("tasks").join(".stream-_orchestrator.jsonl");
        append_jsonl_logged(
            &chat_path,
            &json!({"type":"run_started","provider":"codex","model":"gpt-5.5","mode":"act","ts":"2026-04-28T08:00:00Z"}),
            "test run started",
        );
        append_jsonl_logged(
            &chat_path,
            &json!({"type":"delegation","project":"HealthTracker","task":"Run health check","ts":"2026-04-28T08:00:02Z"}),
            "test delegation queued",
        );
        state.delegations.lock().expect("delegations").insert(
            "d-1".to_string(),
            delegation("d-1", DelegationStatus::Pending),
        );

        let result = build_execution_map(&state, None, None, 50);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["schema_version"], "agentos.execution_map.v1");
        assert!(result["lanes"].as_array().unwrap().len() >= 2);
        assert!(result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| { edge.get("type").and_then(|v| v.as_str()) == Some("spawn") }));
        assert_eq!(result["waiting_for_user"].as_array().unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn includes_live_running_delegation_without_stream_output() {
        let root = test_root("live-running");
        let state = AppState::new(root.clone());
        state.delegations.lock().expect("delegations").insert(
            "d-live".to_string(),
            delegation("d-live", DelegationStatus::Running),
        );

        let result = build_execution_map(&state, None, None, 50);
        let events = result["events"].as_array().expect("events");
        let live = events
            .iter()
            .find(|event| event.get("kind").and_then(|v| v.as_str()) == Some("delegation_state"))
            .expect("live delegation event");

        assert_eq!(live["status"], "running");
        assert_eq!(live["started_at"], "2026-04-28T08:01:00Z");
        assert_eq!(live["provider"], "codex");

        let _ = std::fs::remove_dir_all(root);
    }
}
