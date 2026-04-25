//! Chat CRUD: list chats, get history, send (sync) chat.

use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

use super::claude_runner::{get_permission_path, log_chat_event, run_claude_with_opts};
use super::process_manager::{clear_activity, set_activity};

#[tauri::command]
pub fn get_chats(state: State<Arc<AppState>>) -> Value {
    super::chat_core::get_chats_core(&state)
}

#[tauri::command]
pub fn get_chat_history(state: State<Arc<AppState>>, project: String) -> Value {
    super::chat_core::get_chat_history_core(&state, &project)
}

#[tauri::command]
pub async fn send_chat(
    state: State<'_, Arc<AppState>>,
    project: String,
    message: String,
    provider: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<String>,
) -> Result<Value, String> {
    if message.is_empty() {
        return Ok(json!({"status": "error", "error": "Empty message"}));
    }

    let (cwd, chat_key, chat_file) = match super::chat_core::resolve_chat_context(&state, &project)
    {
        Ok(ctx) => ctx,
        Err(e) => return Ok(json!({"status": "error", "error": e})),
    };
    let prompt =
        super::chat_core::prepare_chat(&state, &chat_key, &chat_file, &message, project.is_empty());

    let perm_path = get_permission_path(&state, &chat_key);
    let detail: String = message.chars().take(50).collect();
    set_activity(&state, &chat_key, "chatting", &detail);

    let (provider, resolved_model, resolved_effort) =
        super::provider_runner::resolve_single_chat_settings(
            &state,
            &project,
            provider.as_deref(),
            model.as_deref(),
            reasoning_effort.as_deref(),
        );

    let response = if matches!(provider, super::provider_runner::ProviderKind::Codex) {
        super::provider_runner::run_provider_with_opts(
            &state,
            provider,
            &cwd,
            &prompt,
            Some(&perm_path),
            resolved_model.as_deref(),
            resolved_effort.as_deref(),
        )
    } else {
        run_claude_with_opts(
            &cwd,
            &prompt,
            &perm_path,
            resolved_model.as_deref(),
            resolved_effort.as_deref(),
        )
    };

    clear_activity(&state, &chat_key);

    // Save assistant response
    let ts2 = state.now_iso();
    let asst_entry = json!({"ts": ts2, "role": "assistant", "msg": response});
    super::jsonl::append_jsonl_logged(&chat_file, &asst_entry, "chat asst response");

    // Log to activity feed
    log_chat_event(&state.root, &chat_key, &response);

    // Process PA commands from orchestrator response
    let mut final_response = response.clone();
    if project.is_empty() {
        let commands = super::pa_commands::parse_pa_commands(&response, &state);
        let warnings = super::pa_commands::detect_malformed_commands(&response);

        for parsed in &commands {
            if !parsed.valid {
                if let Some(err) = &parsed.error {
                    final_response += &format!("\n\n**⚠ {}**", err);
                }
                continue;
            }
            if let Some(text) = super::pa_commands::execute_pa_command(&state, &parsed.cmd) {
                final_response += &format!("\n\n---\n{}", text);
            }
        }

        for w in warnings {
            final_response += &format!("\n\n---\n**Note:** {}", w);
        }
    }

    Ok(json!({
        "status": "complete",
        "response": final_response,
        "project": chat_key,
        "ts": ts2,
    }))
}

#[tauri::command]
pub fn export_chat(state: State<Arc<AppState>>, project: String) -> Value {
    let path = state.chats_dir.join(format!("{}.jsonl", project));
    let mut md = format!("# Chat: {}\n\n", project);
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or("?");
                let msg = entry.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                let ts = entry.get("ts").and_then(|v| v.as_str()).unwrap_or("");
                md += &format!("**{}** ({})\n{}\n\n---\n\n", role, ts, msg);
            }
        }
    }
    json!({"markdown": md, "project": project})
}
