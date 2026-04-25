//! Stream polling and control: poll_stream, stop_chat, is_chat_running.
//! Extracted from chat_stream.rs for file size.

use super::process_manager::{clear_activity, is_cancelled, kill_existing, request_cancel};
use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

/// Poll stream buffer — frontend calls this every 250ms during streaming
#[tauri::command]
pub fn poll_stream(state: State<Arc<AppState>>, project: Option<String>, offset: usize) -> Value {
    let chat_key = project.unwrap_or_else(|| "_orchestrator".to_string());
    let buf_path = state
        .root
        .join("tasks")
        .join(format!(".stream-{}.jsonl", chat_key));
    let content = std::fs::read_to_string(&buf_path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();

    if offset >= lines.len() {
        return json!({"events": [], "offset": offset, "done": false});
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

    json!({"events": events, "offset": lines.len(), "done": done})
}

/// Stop a running chat process — kills the child process
#[tauri::command]
pub fn stop_chat(state: State<Arc<AppState>>, project: Option<String>) -> Value {
    let chat_key = project.unwrap_or_else(|| "_orchestrator".to_string());
    request_cancel(&state, &chat_key);
    kill_existing(&state, &chat_key);
    clear_activity(&state, &chat_key);
    json!({"status": "stopped", "project": chat_key})
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
