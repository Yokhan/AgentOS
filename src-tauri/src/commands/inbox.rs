//! Agent feedback inbox — accumulates results from delegations and agents.
//! Triage: if any item needs user attention → show to user. Otherwise → batch to PA.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: String,
    pub project: String,
    pub kind: String, // "delegation_result", "error", "question"
    pub message: String,
    pub needs_user: bool, // true if requires user decision
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_session_id: Option<String>,
    pub ts: String,
}

const MAX_INBOX: usize = 20;

/// Add item to inbox (called from delegation.rs after completion)
pub fn push_inbox(
    state: &AppState,
    project: &str,
    kind: &str,
    message: &str,
    needs_user: bool,
    delegation_id: Option<&str>,
    room_session_id: Option<&str>,
) {
    if let Ok(mut inbox) = state.inbox.lock() {
        if inbox.len() >= MAX_INBOX {
            inbox.remove(0); // drop oldest
        }
        let id = format!(
            "inbox-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        inbox.push(InboxItem {
            id,
            project: project.to_string(),
            kind: kind.to_string(),
            message: message.chars().take(500).collect(),
            needs_user,
            delegation_id: delegation_id.map(|s| s.to_string()),
            room_session_id: room_session_id.map(|s| s.to_string()),
            ts: state.now_iso(),
        });
    }
}

/// Get inbox items
#[tauri::command]
pub fn get_inbox(state: State<Arc<AppState>>) -> Value {
    let items = state
        .inbox
        .lock()
        .map(|inbox| {
            inbox
                .iter()
                .map(|i| serde_json::to_value(i).unwrap_or_default())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let needs_user = items.iter().any(|i| {
        i.get("needs_user")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });
    json!({"items": items, "count": items.len(), "needs_user": needs_user})
}

/// Clear inbox after processing
#[tauri::command]
pub fn clear_inbox(state: State<Arc<AppState>>) -> Value {
    if let Ok(mut inbox) = state.inbox.lock() {
        inbox.clear();
    }
    json!({"status": "ok"})
}

/// Process inbox: batch results to PA. Returns PA's summary response.
#[tauri::command]
pub async fn process_inbox(state: State<'_, Arc<AppState>>) -> Result<Value, String> {
    let items: Vec<InboxItem> = {
        let inbox = state.inbox.lock().map_err(|e| e.to_string())?;
        inbox.clone()
    };

    if items.is_empty() {
        return Ok(json!({"status": "empty", "message": "No items in inbox"}));
    }

    // Check if any need user attention
    let has_user_items = items.iter().any(|i| i.needs_user);
    if has_user_items {
        return Ok(json!({
            "status": "needs_user",
            "message": "Some items need your review before sending to PA",
            "items": items.iter().filter(|i| i.needs_user).map(|i| json!({"project": i.project, "kind": i.kind, "message": i.message})).collect::<Vec<_>>(),
        }));
    }

    // Build batch message for PA
    let mut batch_lines = Vec::new();
    for item in &items {
        let short: String = item.message.chars().take(150).collect();
        batch_lines.push(format!("- {} [{}]: {}", item.project, item.kind, short));
    }
    let batch_prompt = format!(
        "[AGENT FEEDBACK BATCH — {} results]\n{}\n[END BATCH]\n\
         Summarize these results briefly. If any failed, suggest next steps. Be concise.",
        items.len(),
        batch_lines.join("\n")
    );

    let state_arc = Arc::clone(&state);
    let item_count = items.len();
    let response = tokio::task::spawn_blocking(move || {
        let (orch_name, pa_dir) = state_arc.get_orch_dir();
        // Lock PA directory to prevent concurrent claude sessions
        state_arc.acquire_dir_lock(&orch_name);
        let perm_path = super::claude_runner::get_permission_path(&state_arc, "_orchestrator");
        let response = super::claude_runner::run_claude(&pa_dir, &batch_prompt, &perm_path);

        // Save to orchestrator chat
        let orch_file = state_arc.chats_dir.join("_orchestrator.jsonl");
        let ts = state_arc.now_iso();
        let sys_entry = json!({"ts": ts, "role": "system", "msg": format!("[Batch: {} results processed]", item_count)});
        let asst_entry = json!({"ts": ts, "role": "assistant", "msg": response});
        crate::commands::jsonl::append_jsonl_pair(&orch_file, &sys_entry, &asst_entry, "inbox batch response");

        if let Ok(mut inbox) = state_arc.inbox.lock() { inbox.clear(); }
        state_arc.release_dir_lock(&orch_name);
        crate::log_info!("[inbox] processed {} items, PA responded {} chars", item_count, response.len());
        response
    }).await.map_err(|e| e.to_string())?;

    Ok(json!({"status": "processed", "response": response, "count": items.len()}))
}
