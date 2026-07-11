//! Agent feedback inbox — accumulates results from delegations and agents.
//! Triage: if any item needs user attention → show to user. Otherwise → batch to PA.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    #[serde(default)]
    pub notified: bool,
    pub ts: String,
}

const MAX_INBOX: usize = 20;
static AUTO_INBOX_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static AUTO_INBOX_LAST_RUN: AtomicU64 = AtomicU64::new(0);

struct AutoInboxGuard;

impl Drop for AutoInboxGuard {
    fn drop(&mut self) {
        AUTO_INBOX_IN_FLIGHT.store(false, Ordering::Relaxed);
    }
}

fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

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
            notified: false,
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

fn build_batch_prompt(items: &[InboxItem]) -> String {
    let mut batch_lines = Vec::new();
    for item in items {
        let short: String = item.message.chars().take(150).collect();
        batch_lines.push(format!("- {} [{}]: {}", item.project, item.kind, short));
    }
    format!(
        "[AGENT FEEDBACK BATCH — {} results]\n{}\n[END BATCH]\n\
         Summarize these results briefly. If any failed, suggest next steps. \
         If follow-up AgentOS action is needed, emit the exact PA command tag. \
         Answer in the user's language.",
        items.len(),
        batch_lines.join("\n")
    )
}

fn append_pa_command_results(state: &AppState, response: &str, label: &str) -> usize {
    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    let commands = super::pa_commands::parse_pa_commands(response, state);
    let mut executed = 0usize;
    for parsed in &commands {
        let command_label = super::pa_commands::describe_pa_command(&parsed.cmd);
        if !parsed.valid {
            let msg = parsed.error.as_deref().unwrap_or("invalid AgentOS command");
            super::jsonl::append_jsonl_logged(
                &orch_file,
                &json!({
                    "ts": state.now_iso(),
                    "role": "system",
                    "kind": "pa_feedback",
                    "pa_type": "warning",
                    "pa_command": command_label,
                    "msg": msg
                }),
                &format!("{} invalid PA command", label),
            );
            continue;
        }

        if let Some(result_msg) = super::pa_commands::execute_pa_command(state, &parsed.cmd) {
            super::jsonl::append_jsonl_logged(
                &orch_file,
                &json!({
                    "ts": state.now_iso(),
                    "role": "system",
                    "kind": "pa_feedback",
                    "pa_type": "pa_result",
                    "pa_command": command_label,
                    "msg": result_msg
                }),
                &format!("{} PA command result", label),
            );
        }
        executed += 1;
    }
    executed
}

fn process_items_sync(state: &AppState, items: &[InboxItem], label: &str) -> String {
    let batch_prompt = build_batch_prompt(items);
    let (orch_name, pa_dir) = state.get_orch_dir();
    {
        let busy = state.dir_busy.lock().unwrap_or_else(|e| e.into_inner());
        if busy.contains(&orch_name) || busy.contains("_orchestrator") {
            return "Orchestrator is busy; inbox remains queued.".to_string();
        }
    }

    let _orchestrator_guard = match state.acquire_dir_guard(&orch_name) {
        Ok(guard) => guard,
        Err(error) => return format!("Orchestrator unavailable: {error}"),
    };
    let perm_path = super::claude_runner::get_permission_path(state, "_orchestrator");
    let response = super::provider_runner::run_orchestrator_once(
        state,
        &pa_dir,
        &batch_prompt,
        Some(&perm_path),
    );

    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    let ts = state.now_iso();
    let sys_entry = json!({"ts": ts, "role": "system", "msg": format!("[Batch: {} results processed]", items.len())});
    let asst_entry = json!({"ts": ts, "role": "assistant", "msg": response});
    crate::commands::jsonl::append_jsonl_pair(
        &orch_file,
        &sys_entry,
        &asst_entry,
        &format!("{} inbox batch response", label),
    );
    let executed = append_pa_command_results(state, &response, label);
    crate::log_info!(
        "[inbox] processed {} items, PA responded {} chars, {} commands executed",
        items.len(),
        response.len(),
        executed
    );
    response
}

fn remove_inbox_items(state: &AppState, processed_ids: &HashSet<String>) {
    if let Ok(mut inbox) = state.inbox.lock() {
        inbox.retain(|item| !processed_ids.contains(&item.id));
    }
}

fn inbox_response_processed(response: &str) -> bool {
    let trimmed = response.trim();
    !trimmed.is_empty()
        && trimmed != "Orchestrator is busy; inbox remains queued."
        && !trimmed.starts_with("Provider error:")
}

fn notify_user_items_once(state: &AppState) {
    let items = {
        let mut inbox = match state.inbox.lock() {
            Ok(inbox) => inbox,
            Err(_) => return,
        };
        let mut items = Vec::new();
        for item in inbox
            .iter_mut()
            .filter(|item| item.needs_user && !item.notified)
            .take(5)
        {
            item.notified = true;
            items.push(item.clone());
        }
        items
    };
    if items.is_empty() {
        return;
    }

    let lines = items
        .iter()
        .map(|item| {
            let id = item.delegation_id.as_deref().unwrap_or(&item.id);
            format!(
                "- {}: {} ({})",
                item.project,
                super::claude_runner::safe_truncate(&item.message, 120),
                id
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    super::jsonl::append_jsonl_logged(
        &orch_file,
        &json!({
            "ts": state.now_iso(),
            "role": "system",
            "kind": "inbox_attention",
            "msg": format!("Inbox ждёт решения пользователя:\n{}", lines)
        }),
        "inbox user attention",
    );
}

/// Called by the 30s background loop. Successful delegation results should not
/// sit silently in the inbox until the user manually pokes the orchestrator.
pub fn auto_process_inbox(state: &Arc<AppState>) {
    if AUTO_INBOX_IN_FLIGHT.swap(true, Ordering::Relaxed) {
        return;
    }
    let _guard = AutoInboxGuard;

    let now = unix_secs();
    let last = AUTO_INBOX_LAST_RUN.load(Ordering::Relaxed);
    if now.saturating_sub(last) < 15 {
        return;
    }

    notify_user_items_once(state);
    let items = match state.inbox.lock() {
        Ok(inbox) => inbox
            .iter()
            .filter(|item| !item.needs_user)
            .cloned()
            .collect::<Vec<_>>(),
        Err(_) => return,
    };
    if items.is_empty() {
        return;
    }

    AUTO_INBOX_LAST_RUN.store(now, Ordering::Relaxed);
    let processed_ids: HashSet<String> = items.iter().map(|item| item.id.clone()).collect();
    let response = process_items_sync(state, &items, "auto");
    if inbox_response_processed(&response) {
        remove_inbox_items(state, &processed_ids);
    }
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
    let _batch_prompt = format!(
        "[AGENT FEEDBACK BATCH — {} results]\n{}\n[END BATCH]\n\
         Summarize these results briefly. If any failed, suggest next steps. Be concise.",
        items.len(),
        batch_lines.join("\n")
    );

    let state_arc = Arc::clone(&state);
    let item_count = items.len();
    let processed_ids: HashSet<String> = items.iter().map(|item| item.id.clone()).collect();
    let response = tokio::task::spawn_blocking(move || {
        let response = process_items_sync(&state_arc, &items, "manual");
        if inbox_response_processed(&response) {
            remove_inbox_items(&state_arc, &processed_ids);
        }
        response
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({"status": "processed", "response": response, "count": item_count}))
}

#[cfg(test)]
mod tests {
    use super::{build_batch_prompt, inbox_response_processed, InboxItem};

    fn item(project: &str, needs_user: bool) -> InboxItem {
        InboxItem {
            id: format!("inbox-{}", project),
            project: project.to_string(),
            kind: "delegation_result".to_string(),
            message: "finished".to_string(),
            needs_user,
            delegation_id: Some("deleg-1".to_string()),
            room_session_id: None,
            notified: false,
            ts: "2026-05-11T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn batch_prompt_requires_exact_follow_up_commands() {
        let prompt = build_batch_prompt(&[item("AgentOS", false)]);

        assert!(prompt.contains("AGENT FEEDBACK BATCH"));
        assert!(prompt.contains("emit the exact PA command tag"));
    }

    #[test]
    fn failed_provider_response_keeps_inbox_queued() {
        assert!(!inbox_response_processed("Provider error: codex timed out"));
        assert!(!inbox_response_processed(""));
        assert!(!inbox_response_processed(
            "Orchestrator is busy; inbox remains queued."
        ));
        assert!(inbox_response_processed("Готово: результат обработан."));
    }
}
