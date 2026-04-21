//! Delegation Tauri commands: approve, reject, schedule, cancel.
//! Extracted from delegation.rs for file size.

use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn approve_delegation(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || super::delegation::approve_delegation_core(&state_arc, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_delegation(state: State<Arc<AppState>>, id: String) -> Value {
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(&id) {
            del.status = crate::commands::status::DelegationStatus::Rejected;
            let room_session_id = del.room_session_id.clone();
            let work_item_id = del.work_item_id.clone();
            drop(delegations);
            state.save_delegations();
            if let Some(work_item_id) = work_item_id.as_deref() {
                if let Ok(mut work_items) = state.work_items.lock() {
                    if let Some(item) = work_items.get_mut(work_item_id) {
                        item.status = crate::state::WorkItemStatus::Cancelled;
                        item.updated_at = state.now_iso();
                        item.result = Some("Delegation rejected".to_string());
                    }
                }
                state.save_work_items();
                if let Some(session_id) = room_session_id.as_deref() {
                    crate::commands::multi_agent::release_work_item_leases(
                        &state,
                        session_id,
                        work_item_id,
                        "delegation_rejected",
                    );
                }
            }
            return json!({"status": "rejected"});
        }
    }
    json!({"status": "error", "error": "Not found"})
}

#[tauri::command]
pub fn schedule_delegation(state: State<Arc<AppState>>, id: String, scheduled_at: String) -> Value {
    if chrono::DateTime::parse_from_rfc3339(&scheduled_at).is_err() {
        return json!({"status": "error", "error": "Invalid datetime format (expected ISO 8601)"});
    }
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(&id) {
            if del.status != crate::commands::status::DelegationStatus::Pending {
                return json!({"status": "error", "error": "Can only schedule pending delegations"});
            }
            del.status = crate::commands::status::DelegationStatus::Scheduled;
            del.scheduled_at = Some(scheduled_at.clone());
            drop(delegations);
            state.save_delegations();
            crate::log_info!("[delegation] scheduled {} for {}", id, scheduled_at);
            return json!({"status": "scheduled", "scheduled_at": scheduled_at});
        }
    }
    json!({"status": "error", "error": "Not found"})
}

#[tauri::command]
pub fn cancel_delegation(state: State<Arc<AppState>>, id: String) -> Value {
    match super::delegation_ext::execute_deleg_command(
        &state,
        &super::pa_commands_deleg::DelegPaCommand::Cancel { id },
    ) {
        Some(msg) => json!({"status": "ok", "message": msg}),
        None => json!({"status": "error", "error": "Not found"}),
    }
}
