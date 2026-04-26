//! Execution timeline: read-only normalized view over chat, Duo, and delegation events.

use crate::commands::event_contract::{
    event_contract_schema_value, normalize_chat_stream_event, normalize_delegation_state,
    normalize_delegation_stream_event, normalize_session_event, EventRow, EVENT_SCHEMA_VERSION,
};
use crate::state::{AppState, Delegation};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tauri::State;

fn read_recent_jsonl(path: &Path, limit: usize) -> Vec<Value> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut rows = Vec::new();
    for line in content.lines().rev() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            rows.push(value);
            if rows.len() >= limit {
                break;
            }
        }
    }
    rows.reverse();
    rows
}

fn chat_stream_rows(state: &AppState, project: &str, limit: usize) -> Vec<EventRow> {
    let chat_key = if project.trim().is_empty() {
        "_orchestrator"
    } else {
        project
    };
    let path = state
        .root
        .join("tasks")
        .join(format!(".stream-{}.jsonl", chat_key));
    read_recent_jsonl(&path, limit)
        .iter()
        .filter_map(|evt| normalize_chat_stream_event(evt, chat_key, ""))
        .collect()
}

fn session_rows(state: &AppState, session_id: &str, project: &str, limit: usize) -> Vec<EventRow> {
    state
        .get_session_events(session_id, limit)
        .iter()
        .map(|event| normalize_session_event(event, project))
        .collect()
}

fn delegation_stream_rows(
    state: &AppState,
    delegation: &Delegation,
    limit: usize,
) -> Vec<EventRow> {
    let path = state
        .root
        .join("tasks")
        .join(format!(".stream-deleg-{}.jsonl", delegation.id));
    read_recent_jsonl(&path, limit)
        .iter()
        .filter_map(|evt| normalize_delegation_stream_event(evt, delegation))
        .collect()
}

fn delegation_rows(state: &AppState, project: &str, limit: usize) -> Vec<EventRow> {
    let mut delegations: Vec<Delegation> = state
        .delegations
        .lock()
        .map(|items| {
            items
                .values()
                .filter(|item| project.is_empty() || item.project == project)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    delegations.sort_by(|a, b| b.ts.cmp(&a.ts));
    let mut rows = Vec::new();
    for delegation in delegations.into_iter().take(limit.min(8)) {
        rows.push(normalize_delegation_state(&delegation));
        rows.extend(delegation_stream_rows(state, &delegation, 4));
    }
    rows
}

pub fn build_execution_timeline(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: usize,
) -> Value {
    let project = project.unwrap_or_default();
    let limit = limit.clamp(10, 120);
    let mut rows = chat_stream_rows(state, &project, limit);

    if let Some(session_id) = room_session_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
    {
        rows.extend(session_rows(state, session_id, &project, limit.min(40)));
    }
    rows.extend(delegation_rows(state, &project, limit.min(24)));
    rows.sort_by(|a, b| a.ts.cmp(&b.ts));
    if rows.len() > limit {
        rows = rows[rows.len() - limit..].to_vec();
    }
    let warnings = rows.iter().filter(|row| row.warning_like()).count();
    json!({
        "status": "ok",
        "schema_version": EVENT_SCHEMA_VERSION,
        "project": if project.is_empty() { "_orchestrator" } else { project.as_str() },
        "big_plan": {
            "stage": "event_contract",
            "stage_index": 5,
            "stage_total": 6,
            "label": "Event contract + normalized source adapters"
        },
        "contract": event_contract_schema_value(),
        "counts": {
            "items": rows.len(),
            "warnings": warnings
        },
        "items": rows
    })
}

#[tauri::command]
pub fn get_execution_timeline(
    state: State<Arc<AppState>>,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: Option<usize>,
) -> Value {
    build_execution_timeline(&state, project, room_session_id, limit.unwrap_or(60))
}

#[cfg(test)]
mod tests {
    use super::build_execution_timeline;
    use crate::state::AppState;
    use serde_json::json;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-timeline-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("tasks")).expect("create temp tasks");
        path
    }

    #[test]
    fn stream_events_are_normalized_into_timeline() {
        let root = test_root("stream");
        let state = AppState::new(root.clone());
        let path = root.join("tasks").join(".stream-_orchestrator.jsonl");
        crate::commands::jsonl::append_jsonl_logged(
            &path,
            &json!({"type":"run_started","provider":"codex","model":"gpt-5.5","mode":"act","ts":"2026-04-26T10:00:00Z"}),
            "test stream start",
        );
        crate::commands::jsonl::append_jsonl_logged(
            &path,
            &json!({"type":"tool_use","tool":"Read","status":"started","ts":"2026-04-26T10:00:01Z"}),
            "test stream tool",
        );

        let result = build_execution_timeline(&state, None, None, 10);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["schema_version"], "agentos.event.v1");
        assert_eq!(result["big_plan"]["stage"], "event_contract");
        assert_eq!(result["items"][0]["kind"], "run");
        assert_eq!(result["items"][1]["kind"], "tool");

        let _ = std::fs::remove_dir_all(root);
    }
}
