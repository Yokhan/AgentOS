//! Stream polling and control: poll_stream, stop_chat, is_chat_running.
//! Extracted from chat_stream.rs for file size.

use super::process_manager::{clear_activity, is_cancelled, kill_existing, request_cancel};
use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

fn safe_stream_segment(value: &str) -> String {
    let safe = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if safe.is_empty() {
        "_orchestrator".to_string()
    } else {
        safe
    }
}

pub fn stream_buffer_path(
    state: &AppState,
    chat_key: &str,
    run_id: Option<&str>,
) -> std::path::PathBuf {
    let name = match run_id.map(str::trim).filter(|value| !value.is_empty()) {
        Some(run_id) => format!(
            ".stream-{}-{}.jsonl",
            safe_stream_segment(chat_key),
            safe_stream_segment(run_id)
        ),
        None => format!(".stream-{}.jsonl", safe_stream_segment(chat_key)),
    };
    state.root.join("tasks").join(name)
}

/// Poll stream buffer — frontend calls this every 250ms during streaming
#[tauri::command]
pub fn poll_stream(
    state: State<Arc<AppState>>,
    project: Option<String>,
    offset: usize,
    run_id: Option<String>,
) -> Value {
    let chat_key = project.unwrap_or_else(|| "_orchestrator".to_string());
    let run_id = run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let buf_path = stream_buffer_path(&state, &chat_key, run_id);
    let activity = state
        .activities
        .lock()
        .ok()
        .and_then(|activities| activities.get(&chat_key).cloned());
    let pid_running = state
        .running_pids
        .lock()
        .map(|pids| pids.contains_key(&chat_key))
        .unwrap_or(false);
    let running = pid_running || activity.is_some();
    let content = std::fs::read_to_string(&buf_path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();

    if offset >= lines.len() {
        return json!({
            "events": [],
            "offset": offset,
            "done": false,
            "running": running,
            "cancelled": is_cancelled(&state, &chat_key),
            "project": chat_key,
            "run_id": run_id,
            "activity": activity
        });
    }

    let new_lines = &lines[offset..];
    let mut events: Vec<Value> = Vec::new();
    let mut done = false;

    for line in new_lines {
        if let Ok(evt) = serde_json::from_str::<Value>(line) {
            if evt.get("type").and_then(|t| t.as_str()) == Some("done") {
                done = true;
            }
            events.push(evt);
        }
    }

    json!({
        "events": events,
        "offset": lines.len(),
        "done": done,
        "running": running,
        "cancelled": is_cancelled(&state, &chat_key),
        "project": chat_key,
        "run_id": run_id,
        "activity": activity
    })
}

/// Stop a running chat process — kills the child process
#[tauri::command]
pub fn stop_chat(
    state: State<Arc<AppState>>,
    project: Option<String>,
    run_id: Option<String>,
) -> Value {
    let chat_key = project.unwrap_or_else(|| "_orchestrator".to_string());
    request_cancel(&state, &chat_key);
    let run_id = run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let buf_path = stream_buffer_path(&state, &chat_key, run_id);
    let _ = super::jsonl::append_jsonl_logged(
        &buf_path,
        &json!({
            "type": "run_done",
            "run_id": run_id,
            "status": "cancelled",
            "phase": "cancelled",
            "outcome": "cancelled",
            "detail": "stopped by user"
        }),
        "stream cancelled by user",
    );
    let _ = super::jsonl::append_jsonl_logged(
        &buf_path,
        &json!({"type":"done","run_id":run_id,"text":"","tools":[],"outcome":"cancelled"}),
        "stream cancelled done marker",
    );
    let killed_pid = kill_existing(&state, &chat_key);
    clear_activity(&state, &chat_key);
    json!({
        "status": "stopped",
        "project": chat_key,
        "killed": killed_pid.is_some(),
        "pid": killed_pid
    })
}

/// Check if a chat is currently streaming
#[tauri::command]
pub fn is_chat_running(state: State<Arc<AppState>>, project: Option<String>) -> Value {
    let chat_key = project.unwrap_or_else(|| "_orchestrator".to_string());
    let running = state
        .running_pids
        .lock()
        .map(|pids| pids.contains_key(&chat_key))
        .unwrap_or(false);
    json!({"running": running, "cancelled": is_cancelled(&state, &chat_key), "project": chat_key})
}
