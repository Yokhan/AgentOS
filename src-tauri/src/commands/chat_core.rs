//! Shared chat logic — eliminates duplication between send_chat, stream_chat, and API handlers.

use crate::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Resolve project to working directory + chat key + chat file path.
/// Shared by send_chat, stream_chat, and API send_chat.
pub fn resolve_chat_context(
    state: &AppState,
    project: &str,
) -> Result<(PathBuf, String, PathBuf), String> {
    let (_, pa_dir) = state.get_orch_dir();
    let cwd = if !project.is_empty() {
        state.validate_project(project)?
    } else {
        pa_dir
    };
    let chat_key = if project.is_empty() {
        "_orchestrator".to_string()
    } else {
        project.to_string()
    };
    let chat_file = state.chats_dir.join(format!("{}.jsonl", chat_key));
    Ok((cwd, chat_key, chat_file))
}

/// Save user message to JSONL and build prompt. Returns the prompt to send to claude.
pub fn prepare_chat(
    state: &AppState,
    _chat_key: &str,
    chat_file: &std::path::Path,
    message: &str,
    is_orchestrator: bool,
) -> String {
    let ts = state.now_iso();
    let user_entry = json!({"ts": ts, "role": "user", "msg": message});
    super::jsonl::append_jsonl_logged(chat_file, &user_entry, "chat user msg");

    if is_orchestrator {
        super::chat_parse::build_full_pa_prompt(state, message)
    } else {
        message.to_string()
    }
}

/// Get list of all chats (shared by Tauri command and API handler).
pub fn get_chats_core(state: &AppState) -> Value {
    let mut chats = Vec::new();
    let entries = match std::fs::read_dir(&state.chats_dir) {
        Ok(e) => e,
        Err(_) => return json!({"chats": []}),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.ends_with(".jsonl") => n[..n.len() - 6].to_string(),
            _ => continue,
        };

        let (last_msg, last_ts, msg_count, role) = match std::fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let count = lines.len();
                if let Some(last_line) = lines.last() {
                    if let Ok(entry) = serde_json::from_str::<Value>(last_line) {
                        let msg = entry.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                        let ts = entry.get("ts").and_then(|v| v.as_str()).unwrap_or("");
                        let r = entry.get("role").and_then(|v| v.as_str()).unwrap_or("");
                        (
                            msg.chars().take(60).collect::<String>(),
                            ts.to_string(),
                            count,
                            r.to_string(),
                        )
                    } else {
                        (String::new(), String::new(), count, String::new())
                    }
                } else {
                    (String::new(), String::new(), 0, String::new())
                }
            }
            Err(_) => (String::new(), String::new(), 0, String::new()),
        };

        chats.push(json!({"project": name, "last_msg": last_msg, "last_ts": last_ts, "msg_count": msg_count, "role": role}));
    }

    chats.sort_by(|a, b| {
        let ts_a = a.get("last_ts").and_then(|v| v.as_str()).unwrap_or("");
        let ts_b = b.get("last_ts").and_then(|v| v.as_str()).unwrap_or("");
        ts_b.cmp(ts_a)
    });

    json!({"chats": chats})
}

/// Get a page of chat history. `before` is an exclusive line index in the JSONL file.
pub fn get_chat_history_page_core(
    state: &AppState,
    project: &str,
    before: Option<usize>,
    limit: Option<usize>,
) -> Value {
    if project.contains("..")
        || project.contains('/')
        || project.contains('\\')
        || project.contains(':')
    {
        return json!({"error": "Invalid project name"});
    }
    let path = state.chats_dir.join(format!("{}.jsonl", project));
    let mut messages = Vec::new();
    let mut total = 0usize;
    let mut start = 0usize;
    let mut end = 0usize;
    let limit = limit.unwrap_or(200).clamp(1, 500);

    if let Ok(content) = std::fs::read_to_string(&path) {
        let lines: Vec<&str> = content.lines().collect();
        total = lines.len();
        end = before.unwrap_or(total).min(total);
        start = end.saturating_sub(limit);
        for line in &lines[start..end] {
            if let Ok(msg) = serde_json::from_str::<Value>(line) {
                messages.push(msg);
            }
        }
    }

    json!({
        "project": project,
        "messages": messages,
        "total": total,
        "start": start,
        "end": end,
        "loaded": end.saturating_sub(start),
        "has_more": start > 0,
        "next_before": if start > 0 { json!(start) } else { Value::Null }
    })
}
