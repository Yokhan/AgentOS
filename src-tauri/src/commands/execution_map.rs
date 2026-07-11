//! Branching execution map: read-only projection over normalized timeline events.
//!
//! The map is a UI projection, not a new source of truth. It uses persisted chat
//! stream events, Duo session events, and delegation state/streams.

use crate::commands::event_contract::EVENT_SCHEMA_VERSION;
use crate::commands::status::DelegationStatus;
use crate::commands::timeline::build_execution_timeline_for_map;
use crate::state::{AppState, Delegation};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::State;

const EXECUTION_MAP_SCHEMA_VERSION: &str = "agentos.execution_map.v2";

fn field(row: &Value, key: &str) -> String {
    let raw = row
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    crate::commands::event_contract::clean_display_text(&raw)
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
    let operation_id = field(row, "operation_id");
    if matches!(source.as_str(), "subagent" | "delegation") && !operation_id.is_empty() {
        format!("run:{}", safe_id(&operation_id))
    } else if source == "delegation" && project != "_orchestrator" {
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
    row.map(|v| {
        let role = field(v, "role");
        let project = clean_project(&field(v, "project"));
        if role.is_empty() || role == "project_agent" {
            project
        } else {
            format!("{} · {}", role, project)
        }
    })
    .filter(|v| !v.is_empty() && v != "_orchestrator")
    .unwrap_or_else(|| id.replace("project:", ""))
}

fn lane_kind(id: &str) -> &'static str {
    if id == "orchestrator" {
        "orchestrator"
    } else if id == "duo" {
        "room"
    } else if id.starts_with("run:") {
        "agent_run"
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

fn is_provider_state_sample(row: &Value) -> bool {
    let kind = field(row, "kind");
    if matches!(
        kind.as_str(),
        "run_started" | "provider_started" | "provider_heartbeat" | "model_output_delta"
    ) {
        return true;
    }
    let semantic = row.get("semantic").and_then(|v| v.as_bool());
    if semantic == Some(false) {
        return true;
    }
    if kind != "progress" && kind != "provider_heartbeat" {
        return false;
    }

    let status = field(row, "status").to_ascii_lowercase();
    let title = field(row, "title").to_ascii_lowercase();
    let detail = field(row, "detail").to_ascii_lowercase();
    let volatile_status = matches!(status.as_str(), "running" | "waiting" | "info" | "");
    let provider_phase = matches!(title.as_str(), "provider" | "heartbeat" | "stream");
    let provider_detail = detail.contains("provider")
        || detail.contains("subprocess")
        || detail.contains("waiting for")
        || detail.contains("still running");

    volatile_status && (provider_phase || provider_detail)
}

fn is_visual_map_row(row: &Value) -> bool {
    if is_provider_state_sample(row) {
        return false;
    }
    if row.get("semantic").and_then(|v| v.as_bool()) == Some(false) {
        return false;
    }

    let source = field(row, "source");
    let kind = field(row, "kind");

    match kind.as_str() {
        "queued"
        | "state"
        | "stage"
        | "done"
        | "safety"
        | "tool"
        | "tool_result"
        | "tool_started"
        | "tool_completed"
        | "delegation_queued"
        | "delegation_started"
        | "delegation_l1"
        | "delegation_l2"
        | "delegation_l3_decision"
        | "delegation_done"
        | "gate_started"
        | "review_verdict"
        | "subagent_observed"
        | "subagent_verified"
        | "run_cancelled" => true,
        "command" | "coordination" | "pa_command_started" | "pa_command_result"
        | "pa_command_warning" | "pa_command_missing" | "auto_continue" | "run_done" | "run"
        | "progress" | "thinking" | "usage" | "cost" | "system" | "model_output" => false,
        _ => source == "delegation",
    }
}

fn stable_state_detail(sample: &Value) -> String {
    let kind = field(sample, "kind");
    let waiting_for = field(sample, "waiting_for");
    if kind == "provider_heartbeat" {
        if waiting_for == "provider_output" {
            return "waiting for provider output".to_string();
        }
        return "provider process is alive".to_string();
    }
    if kind == "provider_started" {
        return "provider call is running".to_string();
    }
    if kind == "run_started" {
        return "run started".to_string();
    }
    if kind == "model_output_delta" {
        return "model is streaming output".to_string();
    }
    field(sample, "detail")
}

fn state_sample_weight(sample: &Value) -> u8 {
    match field(sample, "kind").as_str() {
        "provider_heartbeat" => 5,
        "provider_started" => 4,
        "model_output_delta" => 3,
        "run_started" => 1,
        _ => 2,
    }
}

fn delegation_id_for_operation(operation_id: &str) -> Option<&str> {
    operation_id
        .strip_prefix("delegation:")
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

fn is_terminal_operation_status(status: &str) -> bool {
    status == DelegationStatus::Done.to_string()
        || status == DelegationStatus::Failed.to_string()
        || status == DelegationStatus::Cancelled.to_string()
}

fn is_archived_terminal_delegation_operation(
    operation: &crate::commands::operation_state::OperationRecord,
    live_delegation_ids: &HashSet<String>,
) -> bool {
    let is_delegation_operation = operation.actor == "project_agent"
        || operation.actor == "gate"
        || delegation_id_for_operation(&operation.operation_id).is_some();
    if !is_delegation_operation || !is_terminal_operation_status(&operation.status) {
        return false;
    }
    delegation_id_for_operation(&operation.operation_id)
        .map(|id| !live_delegation_ids.contains(id))
        .unwrap_or(true)
}

fn operation_rows(state: &AppState, project_filter: &str, limit: usize) -> Vec<Value> {
    let operations = match state.operations.lock() {
        Ok(ops) => ops,
        Err(e) => e.into_inner(),
    };
    let live_delegation_ids: HashSet<String> = state
        .delegations
        .lock()
        .map(|delegations| delegations.keys().cloned().collect())
        .unwrap_or_default();
    let mut rows: Vec<Value> = operations
        .values()
        .filter(|operation| {
            !is_archived_terminal_delegation_operation(operation, &live_delegation_ids)
        })
        .flat_map(|operation| {
            operation.events.iter().filter_map(|event| {
                let project = clean_project(&event.project);
                if !project_filter.trim().is_empty() && project != project_filter {
                    return None;
                }
                let source = match event.actor.as_str() {
                    "project_agent" | "gate" => "delegation",
                    "orchestrator" | "agentos" => "chat",
                    actor if actor.starts_with("subagent:") => "subagent",
                    other => other,
                };
                Some(json!({
                    "source": source,
                    "kind": event.kind,
                    "status": event.status,
                    "title": event.phase,
                    "detail": event.title,
                    "project": project,
                    "provider": operation.provider.clone().unwrap_or_else(|| event.actor.clone()),
                    "model": operation.model.clone().unwrap_or_default(),
                    "ts": event.ts,
                    "semantic": event.semantic,
                    "operation_id": event.operation_id,
                    "parent_id": event.parent_id,
                    "root_id": event.root_id,
                    "event_id": event.id,
                    "waiting_for": operation.waiting_for,
                    "blocked_by": operation.blocked_by
                    ,"role": event.actor,
                    "access": operation.access,
                    "runtime_evidence": event.payload.get("runtime_evidence").cloned().unwrap_or(Value::Bool(false))
                }))
            })
        })
        .collect();
    for operation in operations.values() {
        if is_archived_terminal_delegation_operation(operation, &live_delegation_ids) {
            continue;
        }
        let Some(heartbeat_ts) = operation.heartbeat_ts.clone() else {
            continue;
        };
        let project = clean_project(&operation.project);
        if !project_filter.trim().is_empty() && project != project_filter {
            continue;
        }
        let stored_current_heartbeat = operation
            .events
            .iter()
            .any(|event| event.kind == "provider_heartbeat" && event.ts == heartbeat_ts);
        if stored_current_heartbeat {
            continue;
        }
        rows.push(json!({
            "source": if operation.actor == "project_agent" { "delegation" } else { "chat" },
            "kind": "provider_heartbeat",
            "status": operation.status.clone(),
            "title": operation.phase.clone(),
            "detail": operation.current_action.clone(),
            "project": project,
            "provider": operation.provider.clone().unwrap_or_else(|| operation.actor.clone()),
            "model": operation.model.clone().unwrap_or_default(),
            "ts": heartbeat_ts,
            "semantic": false,
            "operation_id": operation.operation_id.clone(),
            "parent_id": operation.parent_id.clone(),
            "root_id": operation.root_id.clone(),
            "event_id": format!("state-heartbeat-{}", safe_id(&operation.operation_id)),
            "waiting_for": operation.waiting_for.clone(),
            "blocked_by": operation.blocked_by.clone(),
            "heartbeat_beat": operation.heartbeat_beat
        }));
    }
    rows.sort_by(|a, b| {
        field(a, "ts")
            .cmp(&field(b, "ts"))
            .then_with(|| field(a, "event_id").cmp(&field(b, "event_id")))
    });
    if rows.len() > limit {
        rows.drain(0..rows.len() - limit);
    }
    rows
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
                            "pending" | "needs_permission" | "failed" | "rejected" | "cancelled"
                        )
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| b.ts.cmp(&a.ts));
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter_map(|delegation| {
            let key = if delegation.id.trim().is_empty() {
                format!("{}:{}", delegation.project, delegation.ts)
            } else {
                delegation.id.clone()
            };
            if !seen.insert(key) {
                return None;
            }
            let status = delegation.status.to_string();
            let action = match status.as_str() {
                "pending" | "needs_permission" => "approve",
                "failed" => "retry_or_archive",
                "rejected" | "cancelled" => "review_or_archive",
                _ => "review",
            };
            Some(json!({
                "id": delegation.id,
                "project": delegation.project,
                "status": status,
                "task": crate::commands::event_contract::short(&delegation.task, 220),
                "action": action,
                "ts": delegation.ts,
                "started_at": delegation.started_at
            }))
        })
        .collect()
}

fn parse_ts_ms(ts: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.timestamp_millis())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%SZ")
                .map(|dt| dt.and_utc().timestamp_millis())
        })
        .ok()
}

fn time_label(offset_ms: i64) -> String {
    let seconds = (offset_ms.max(0) + 999) / 1000;
    if seconds < 60 {
        format!("+{}s", seconds)
    } else {
        format!("+{}m{}s", seconds / 60, seconds % 60)
    }
}

fn enrich_event_times(events: &mut [Value]) {
    let parsed: Vec<Option<i64>> = events
        .iter()
        .map(|event| {
            event
                .get("ts")
                .and_then(|v| v.as_str())
                .and_then(parse_ts_ms)
        })
        .collect();
    let min_ts = parsed.iter().flatten().min().copied().unwrap_or(0);
    for (index, event) in events.iter_mut().enumerate() {
        let ts_ms = parsed
            .get(index)
            .and_then(|value| *value)
            .unwrap_or(min_ts + index as i64 * 1000);
        let offset_ms = ts_ms.saturating_sub(min_ts).max(0);
        if let Some(obj) = event.as_object_mut() {
            obj.insert("event_index".to_string(), json!(index));
            obj.insert("ts_ms".to_string(), json!(ts_ms));
            obj.insert("offset_ms".to_string(), json!(offset_ms));
            obj.insert("time_label".to_string(), json!(time_label(offset_ms)));
        }
    }
}

pub fn build_execution_map(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: usize,
) -> Value {
    let project_filter = match project.unwrap_or_default().trim() {
        "" | "_orchestrator" => String::new(),
        value => value.to_string(),
    };
    let limit = limit.clamp(20, 180);
    let timeline = build_execution_timeline_for_map(
        state,
        Some(project_filter.clone()),
        room_session_id.clone(),
        limit,
    );
    let timeline_rows = timeline
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let op_rows = operation_rows(state, &project_filter, limit);
    let rows_source = if op_rows.is_empty() {
        "timeline"
    } else {
        "operations"
    };
    let rows = if op_rows.is_empty() {
        timeline_rows
    } else {
        op_rows
    };
    let raw_fallback_ts = rows
        .first()
        .and_then(|row| row.get("ts"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let mut latest_state_samples: HashMap<String, Value> = HashMap::new();
    let mut state_sample_count = 0usize;
    let rows: Vec<Value> = rows
        .into_iter()
        .filter_map(|row| {
            if is_provider_state_sample(&row) {
                state_sample_count += 1;
                let lane_id = lane_id_for(&row);
                let replace = latest_state_samples
                    .get(&lane_id)
                    .map(|existing| state_sample_weight(&row) >= state_sample_weight(existing))
                    .unwrap_or(true);
                if replace {
                    latest_state_samples.insert(lane_id, row);
                }
                None
            } else if !is_visual_map_row(&row) {
                None
            } else {
                Some(row)
            }
        })
        .collect();
    let providers = delegation_provider_by_project(state);
    let has_activity = !rows.is_empty() || state_sample_count > 0;
    let fallback_ts = rows
        .first()
        .and_then(|row| row.get("ts"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .or(raw_fallback_ts)
        .unwrap_or_else(|| state.now_iso());

    let root_event = json!({
        "id": "evt-root",
        "lane_id": "orchestrator",
        "branch_id": "orchestrator",
        "source": "system",
        "kind": "root",
        "status": if has_activity { "running" } else { "idle" },
        "title": "Orchestrator context",
        "detail": if rows.is_empty() && state_sample_count > 0 {
            "Provider heartbeat is active; no semantic execution event yet."
        } else if rows.is_empty() {
            "No recent execution events."
        } else {
            "Execution map projection started."
        },
        "project": if project_filter.trim().is_empty() { "_orchestrator" } else { project_filter.as_str() },
        "provider": "orchestrator",
        "model": "",
        "ts": fallback_ts,
        "sequence": 0,
        "synthetic": true,
        "visible": !rows.is_empty()
    });

    let mut lanes: HashMap<String, Value> = HashMap::new();
    let mut lane_status: HashMap<String, String> = HashMap::new();
    let mut lane_order: HashMap<String, usize> = HashMap::new();
    let mut events = vec![root_event];
    let mut edges = Vec::new();
    let mut seen_lanes: HashSet<String> = HashSet::new();
    let mut latest_project_event: HashMap<String, String> = HashMap::new();
    let mut latest_operation_event: HashMap<String, String> = HashMap::new();
    let mut feedback_count = 0usize;

    lanes.insert(
        "orchestrator".to_string(),
        json!({
            "id": "orchestrator",
            "kind": "orchestrator",
            "label": "Orchestrator",
            "project": "_orchestrator",
            "status": if has_activity { "running" } else { "idle" },
            "provider": "orchestrator",
            "model": "",
            "order": 0
        }),
    );
    lane_status.insert(
        "orchestrator".to_string(),
        if has_activity { "running" } else { "idle" }.to_string(),
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
        } else if source == "subagent" {
            field(row, "provider")
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
                    "model": field(row, "model"),
                    "role": field(row, "role"),
                    "access": field(row, "access"),
                    "runtime_evidence": row.get("runtime_evidence").cloned().unwrap_or(Value::Bool(false)),
                    "order": order
                }),
            );
            lane_order.insert(lane_id.clone(), order);
        }
        let current_status = lane_status.get(&lane_id).cloned().unwrap_or_default();
        lane_status.insert(lane_id.clone(), merge_status(&current_status, &status));

        if lane_id != "orchestrator" && seen_lanes.insert(lane_id.clone()) {
            let parent_operation_id = field(row, "parent_id");
            let parent_event_id = latest_operation_event
                .get(&parent_operation_id)
                .cloned()
                .unwrap_or_else(|| "evt-root".to_string());
            edges.push(json!({
                "id": format!("edge-spawn-{}", safe_id(&lane_id)),
                "from": parent_event_id,
                "to": event_id,
                "type": "spawn",
                "label": if source == "subagent" { "spawn child" } else { "delegated" },
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
            "model": field(row, "model"),
            "role": field(row, "role"),
            "access": field(row, "access"),
            "runtime_evidence": row.get("runtime_evidence").cloned().unwrap_or(Value::Bool(false)),
            "operation_id": field(row, "operation_id"),
            "parent_id": field(row, "parent_id"),
            "root_id": field(row, "root_id"),
            "ts": field(row, "ts"),
            "sequence": idx + 1
        });
        events.push(event);
        let operation_id = field(row, "operation_id");
        if !operation_id.is_empty() {
            latest_operation_event.insert(operation_id, event_id.clone());
        }

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

    enrich_event_times(&mut events);
    let visual_event_count = events
        .iter()
        .filter(|event| {
            field(event, "kind") != "root"
                && event
                    .get("visible")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(true)
        })
        .count();

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
                if let Some(sample) = latest_state_samples.get(&id) {
                    obj.insert("last_state_kind".to_string(), json!(field(sample, "kind")));
                    obj.insert(
                        "last_state_title".to_string(),
                        json!(field(sample, "title")),
                    );
                    obj.insert(
                        "last_state_detail".to_string(),
                        json!(stable_state_detail(sample)),
                    );
                    obj.insert("last_state_ts".to_string(), json!(field(sample, "ts")));
                }
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
            "visual_events": visual_event_count,
            "edges": edges.len(),
            "state_samples": state_sample_count,
            "source": rows_source,
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

        let result = build_execution_map(&state, Some("_orchestrator".to_string()), None, 50);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["schema_version"], "agentos.execution_map.v2");
        assert!(result["lanes"].as_array().unwrap().len() >= 2);
        assert!(result["events"][0].get("event_index").is_some());
        assert!(result["events"][0].get("offset_ms").is_some());
        assert!(result["events"][0].get("time_label").is_some());
        assert!(result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| { edge.get("type").and_then(|v| v.as_str()) == Some("spawn") }));
        assert_eq!(result["waiting_for_user"].as_array().unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_delegation_is_visible_as_user_decision() {
        let root = test_root("failed-user-decision");
        let state = AppState::new(root.clone());
        state.delegations.lock().expect("delegations").insert(
            "d-failed".to_string(),
            delegation("d-failed", DelegationStatus::Failed),
        );

        let result = build_execution_map(&state, None, None, 50);
        let waiting = result["waiting_for_user"].as_array().unwrap();
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0]["id"], "d-failed");
        assert_eq!(waiting[0]["action"], "retry_or_archive");
        assert_eq!(result["counts"]["waiting"], 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn archived_terminal_delegation_operation_does_not_stay_live() {
        let root = test_root("archived-terminal-operation");
        let state = AppState::new(root.clone());
        let operation_id = "delegation:d-archived";

        crate::commands::operation_state::emit(
            &state,
            crate::commands::operation_state::OperationEventInput::new(
                operation_id,
                "project_agent",
                "HealthTracker",
                "delegation_started",
                "delegation",
                "running",
                "Delegation started",
            ),
        );
        crate::commands::operation_state::emit(
            &state,
            crate::commands::operation_state::OperationEventInput::new(
                operation_id,
                "project_agent",
                "HealthTracker",
                "delegation_done",
                "done",
                "failed",
                "Delegation failed",
            ),
        );

        let result = build_execution_map(&state, None, None, 50);
        let lanes = result["lanes"].as_array().expect("lanes");
        assert!(!lanes.iter().any(|lane| lane["project"] == "HealthTracker"));
        assert_eq!(result["counts"]["blocked"], 0);
        assert_eq!(result["counts"]["waiting"], 0);
        assert_eq!(result["counts"]["visual_events"], 0);

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

    #[test]
    fn includes_archived_delegation_as_project_lane_with_clean_text() {
        let root = test_root("archive-lane");
        let state = AppState::new(root.clone());
        let mut archived = delegation("d-archived", DelegationStatus::Failed);
        archived.project = "RABproject".to_string();
        archived.task =
            "\u{0420}\u{045f}\u{0421}\u{0402}\u{0420}\u{0451}\u{0420}\u{0406}\u{0420}\u{00b5}\u{0421}\u{201a} \u{0420}\u{0451}\u{0420}\u{00b7} \u{0420}\u{00b0}\u{0421}\u{0402}\u{0421}\u{2026}\u{0420}\u{0451}\u{0420}\u{0406}\u{0420}\u{00b0}"
                .to_string();
        archived.ts = "2026-04-28T08:02:00Z".to_string();
        append_jsonl_logged(
            &root.join("tasks").join(".delegation-archive.jsonl"),
            &serde_json::to_value(&archived).expect("archive json"),
            "test archive delegation",
        );

        let result = build_execution_map(&state, None, None, 50);
        let lanes = result["lanes"].as_array().expect("lanes");
        assert!(lanes.iter().any(|lane| lane["project"] == "RABproject"));
        let events = result["events"].as_array().expect("events");
        let archived_event = events
            .iter()
            .find(|event| event["project"] == "RABproject")
            .expect("archived project event");
        assert_eq!(archived_event["detail"], "Привет из архива");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn provider_heartbeats_are_lane_state_not_event_nodes() {
        let root = test_root("provider-heartbeats");
        let state = AppState::new(root.clone());
        let chat_path = root.join("tasks").join(".stream-_orchestrator.jsonl");
        append_jsonl_logged(
            &chat_path,
            &json!({"type":"run_started","provider":"codex","model":"gpt-5.5","mode":"act","ts":"2026-04-28T08:00:00Z"}),
            "test run started",
        );
        for idx in 1..=3 {
            append_jsonl_logged(
                &chat_path,
                &json!({
                    "type":"run_heartbeat",
                    "status":"running",
                    "phase":"provider",
                    "detail": format!("Codex subprocess is still running; waiting for provider output ({}s).", idx * 10),
                    "ts": format!("2026-04-28T08:00:0{}Z", idx)
                }),
                "test provider heartbeat",
            );
        }
        append_jsonl_logged(
            &chat_path,
            &json!({"type":"tool_use","tool":"PA command","status":"started","ts":"2026-04-28T08:00:04Z"}),
            "test semantic tool event",
        );

        let result = build_execution_map(&state, Some("_orchestrator".to_string()), None, 50);
        let events = result["events"].as_array().expect("events");
        assert!(!events.iter().any(|event| {
            event.get("kind").and_then(|v| v.as_str()) == Some("progress")
                && event.get("title").and_then(|v| v.as_str()) == Some("provider")
        }));
        assert!(events
            .iter()
            .any(|event| { event.get("kind").and_then(|v| v.as_str()) == Some("tool") }));
        assert_eq!(result["counts"]["state_samples"], 3);
        let orchestrator = result["lanes"]
            .as_array()
            .expect("lanes")
            .iter()
            .find(|lane| lane["id"] == "orchestrator")
            .expect("orchestrator lane");
        assert_eq!(orchestrator["last_state_title"], "provider");
        assert!(orchestrator["last_state_detail"]
            .as_str()
            .unwrap_or_default()
            .contains("waiting for provider output"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn operation_provider_wait_is_state_only_until_semantic_output() {
        let root = test_root("operation-provider-wait-state");
        let state = AppState::new(root.clone());
        let operation_id = "chat:test-run".to_string();

        crate::commands::operation_state::emit(
            &state,
            crate::commands::operation_state::OperationEventInput::new(
                operation_id.clone(),
                "orchestrator",
                "_orchestrator",
                "run_started",
                "queued",
                "running",
                "Run accepted",
            ),
        );
        crate::commands::operation_state::emit(
            &state,
            crate::commands::operation_state::OperationEventInput::new(
                operation_id.clone(),
                "orchestrator",
                "_orchestrator",
                "provider_started",
                "provider",
                "running",
                "Provider call started",
            )
            .waiting_for("provider_output"),
        );
        crate::commands::operation_state::emit(
            &state,
            crate::commands::operation_state::OperationEventInput::new(
                operation_id,
                "orchestrator",
                "_orchestrator",
                "provider_heartbeat",
                "provider",
                "running",
                "Provider process is alive",
            )
            .heartbeat()
            .detail("Codex subprocess pid=42 is still running; waiting for provider output (322s).")
            .waiting_for("provider_output")
            .payload(json!({"beat": 12})),
        );

        let result = build_execution_map(&state, None, None, 50);
        assert_eq!(result["counts"]["visual_events"], 0);
        assert_eq!(result["counts"]["state_samples"], 3);
        assert_eq!(result["events"][0]["kind"], "root");
        assert_eq!(result["events"][0]["visible"], false);
        let orchestrator = result["lanes"]
            .as_array()
            .expect("lanes")
            .iter()
            .find(|lane| lane["id"] == "orchestrator")
            .expect("orchestrator lane");
        assert_eq!(
            orchestrator["last_state_detail"],
            "waiting for provider output"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn pa_command_and_auto_continue_noise_do_not_render_as_map_events() {
        let root = test_root("pa-command-noise");
        let state = AppState::new(root.clone());
        let operation_id = "chat:test-run".to_string();

        for (kind, phase, status, title) in [
            (
                "pa_command_started",
                "command",
                "running",
                "Running [DASHBOARD_FULL]",
            ),
            (
                "pa_command_result",
                "command",
                "running",
                "Completed [DASHBOARD_FULL]",
            ),
            (
                "pa_command_missing",
                "command",
                "needs_user",
                "Claimed action without executable AgentOS command",
            ),
            (
                "auto_continue",
                "agent_loop",
                "running",
                "Auto-continue turn 3",
            ),
            ("run_done", "done", "done", "done"),
        ] {
            crate::commands::operation_state::emit(
                &state,
                crate::commands::operation_state::OperationEventInput::new(
                    operation_id.clone(),
                    "agentos",
                    "_orchestrator",
                    kind,
                    phase,
                    status,
                    title,
                ),
            );
        }

        let result = build_execution_map(&state, None, None, 50);
        assert_eq!(result["counts"]["visual_events"], 0);
        let events = result["events"].as_array().expect("events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["kind"], "root");
        assert_eq!(events[0]["visible"], false);

        let _ = std::fs::remove_dir_all(root);
    }
}
