//! Streaming chat: stream_chat, poll_stream, stop_chat, is_chat_running.
use super::claude_runner::{get_permission_path, get_permission_path_for_profile, unique_tmp};
use super::process_manager::{
    clear_activity, clear_cancel, is_cancelled, kill_existing, set_activity, track_pid,
    untrack_pid_if_match,
};
use crate::state::AppState;
use serde_json::{json, Value};
use std::io::BufRead;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::State;

fn unix_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn response_outcome(text: &str, cancelled: bool, exit_code: Option<i32>) -> &'static str {
    if cancelled {
        return "cancelled";
    }
    if let Some(code) = exit_code {
        if code != 0 {
            return "failed";
        }
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "no_output";
    }
    if trimmed.starts_with("Provider error:") || trimmed.starts_with("ERROR:") {
        return "failed";
    }
    "done"
}

fn should_allow_plan_readonly_pa_loop(message: &str) -> bool {
    let lower = message.to_lowercase();
    [
        "подключ",
        "подхват",
        "онборд",
        "проверь",
        "аудит",
        "статус",
        "diagnostic",
        "diagnostics",
        "audit",
        "status",
        "onboard",
        "connect project",
        "connect projects",
        "project onboarding",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn append_stream_event(stream_buf: &Path, event: Value, label: &str) {
    crate::commands::jsonl::append_jsonl_logged(stream_buf, &event, label);
}

fn spawn_provider_heartbeat(
    state: Arc<AppState>,
    chat_key: String,
    stream_buf: std::path::PathBuf,
    run_id: String,
    operation_id: String,
    active: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let mut beat = 0u64;
        while active.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if !active.load(Ordering::Relaxed) {
                break;
            }
            beat += 1;
            let elapsed = started.elapsed().as_secs();
            let pid = state
                .running_pids
                .lock()
                .ok()
                .and_then(|pids| pids.get(&chat_key).copied());
            let cancelled = is_cancelled(&state, &chat_key);
            let detail = if cancelled {
                "Stop requested; waiting for provider process to exit.".to_string()
            } else if let Some(pid) = pid {
                if super::provider_runner::process_exists(pid) {
                    format!(
                        "Codex subprocess pid={} is still running; waiting for provider output ({}s).",
                        pid, elapsed
                    )
                } else {
                    format!(
                        "Codex subprocess pid={} disappeared; waiting for provider cleanup ({}s).",
                        pid, elapsed
                    )
                }
            } else {
                format!(
                    "Provider call is active; waiting for process registration/output ({}s).",
                    elapsed
                )
            };
            append_stream_event(
                &stream_buf,
                json!({
                    "type": "run_heartbeat",
                    "run_id": run_id.as_str(),
                    "status": "running",
                    "phase": "provider",
                    "detail": detail,
                    "elapsed_ms": elapsed * 1000,
                    "pid": pid,
                    "beat": beat,
                    "ts": state.now_iso()
                }),
                "stream provider heartbeat",
            );
            super::operation_state::emit(
                &state,
                super::operation_state::OperationEventInput::new(
                    operation_id.clone(),
                    super::operation_state::chat_actor(&chat_key),
                    chat_key.clone(),
                    "provider_heartbeat",
                    "provider",
                    "running",
                    "Provider process is alive",
                )
                .heartbeat()
                .detail(detail)
                .waiting_for("provider_output")
                .payload(json!({
                    "run_id": run_id.as_str(),
                    "pid": pid,
                    "beat": beat,
                    "elapsed_ms": elapsed * 1000
                })),
            );
        }
    })
}

fn pa_feedback_severity(event_type: &str, text: &str) -> &'static str {
    if event_type == "warning" {
        return "warning";
    }
    let lower = text.to_ascii_lowercase();
    if lower.contains("failed") || lower.contains("error") || lower.contains("blocked") {
        "warning"
    } else if event_type == "pa_result" {
        "success"
    } else {
        "info"
    }
}

fn pa_feedback_kind(event_type: &str) -> &'static str {
    match event_type {
        "warning" => "command_warning",
        "pa_result" => "command_result",
        "pa_status" => "command_status",
        _ => "agentos_event",
    }
}

fn append_pa_notification(state: &AppState, event_type: &str, text: &str, command: Option<&str>) {
    super::notifications::append_notification(
        state,
        super::notifications::NotificationInput {
            severity: pa_feedback_severity(event_type, text).to_string(),
            source: "agentos".to_string(),
            kind: pa_feedback_kind(event_type).to_string(),
            title: command.unwrap_or("AgentOS").to_string(),
            message: text.to_string(),
            project: None,
            command: command.map(|value| value.to_string()),
            operation_id: None,
        },
    );
}

fn append_pa_feedback(
    state: &AppState,
    _chat_file: &Path,
    stream_buf: &Path,
    event_type: &str,
    text: &str,
    command: Option<&str>,
    label: &str,
) {
    append_stream_event(
        stream_buf,
        json!({"type": event_type, "text": text, "command": command}),
        label,
    );
    append_pa_notification(state, event_type, text, command);
}

fn permission_path_for_chat(
    state: &AppState,
    chat_key: &str,
    permission_profile: Option<&str>,
    plan_mode: bool,
) -> String {
    if plan_mode {
        return get_permission_path_for_profile(state, "restrictive");
    }
    match permission_profile
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "read" | "readonly" | "read-only" | "restrictive" => {
            get_permission_path_for_profile(state, "restrictive")
        }
        "full" | "permissive" | "danger" | "danger-full-access" => {
            get_permission_path_for_profile(state, "permissive")
        }
        "write" | "workspace" | "workspace-write" | "balanced" => {
            get_permission_path_for_profile(state, "balanced")
        }
        _ => get_permission_path(state, chat_key),
    }
}

#[tauri::command]
pub async fn stream_chat(
    _app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    project: String,
    message: String,
    provider: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<String>,
    run_mode: Option<String>,
    permission_profile: Option<String>,
) -> Result<Value, String> {
    if message.is_empty() {
        return Ok(json!({"status": "error", "error": "Empty message"}));
    }

    let normalized_project = super::chat_core::normalize_chat_project(&project);
    let is_orchestrator = normalized_project.is_empty();
    let (cwd, chat_key, chat_file) =
        match super::chat_core::resolve_chat_context(&state, &normalized_project) {
            Ok(ctx) => ctx,
            Err(e) => return Ok(json!({"status": "error", "error": e})),
        };
    let prompt =
        super::chat_core::prepare_chat(&state, &chat_key, &chat_file, &message, is_orchestrator);
    let plan_mode = matches!(
        run_mode
            .as_deref()
            .unwrap_or("act")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "plan" | "planning"
    );
    let prompt = if plan_mode {
        format!(
            "{}\n\n[AGENTOS RUN MODE]\nPlan mode is ON. Read and reason only. Do not edit files and do not run write operations. You may emit read-only AgentOS PA command tags for diagnostics, audits, and dry-runs only, for example [PROJECT_ONBOARD_PLAN:Other:balanced:5], [PROJECT_ONBOARD_AUDIT] or [PROJECT_CONNECT_MISSING:Other:balanced:dry]. Do not emit write commands. Return a plan, questions, a read-only diagnostic command, or a concrete blocker. Preserve [RESPONSE POLICY]: match the user's language for prose.",
            prompt
        )
    } else {
        prompt
    };

    let perm_path =
        permission_path_for_chat(&state, &chat_key, permission_profile.as_deref(), plan_mode);
    let detail: String = message.chars().take(50).collect();
    set_activity(&state, &chat_key, "streaming", &detail);

    // Kill any existing process for this chat (prevents zombie accumulation)
    kill_existing(&state, &chat_key);
    clear_cancel(&state, &chat_key);

    // Stream buffer file — per chat_key so multiple chats don't collide
    let (provider, resolved_model, resolved_effort) =
        super::provider_runner::resolve_single_chat_settings(
            &state,
            &normalized_project,
            provider.as_deref(),
            model.as_deref(),
            reasoning_effort.as_deref(),
        );
    let run_id = format!("{}-{}", chat_key, unix_millis());
    let stream_buf = super::chat_stream_poll::stream_buffer_path(&state, &chat_key, Some(&run_id));
    let _ = std::fs::write(&stream_buf, "");
    let operation_id = super::operation_state::chat_operation_id(&run_id);
    let run_mode_label = if plan_mode { "plan" } else { "act" };
    let access_label = if plan_mode {
        "read".to_string()
    } else {
        permission_profile
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or("project")
            .to_string()
    };
    append_stream_event(
        &stream_buf,
        json!({
            "type": "run_started",
            "run_id": run_id.as_str(),
            "project": chat_key.as_str(),
            "provider": provider.as_str(),
            "model": resolved_model.as_deref().unwrap_or("auto"),
            "effort": resolved_effort.as_deref().unwrap_or(""),
            "mode": run_mode_label,
            "access": access_label.as_str(),
            "status": "running",
            "phase": "queued",
            "detail": "starting provider",
            "ts": state.now_iso()
        }),
        "stream run started",
    );
    super::operation_state::emit(
        &state,
        super::operation_state::OperationEventInput::new(
            operation_id.clone(),
            super::operation_state::chat_actor(&chat_key),
            chat_key.clone(),
            "run_started",
            "queued",
            "running",
            "Starting provider",
        )
        .provider(
            Some(provider.as_str()),
            resolved_model.as_deref().or(Some("auto")),
            resolved_effort.as_deref(),
        )
        .mode(Some(run_mode_label), Some(access_label.as_str()))
        .waiting_for("provider_start")
        .payload(json!({"run_id": run_id.as_str()})),
    );

    let allow_pa_loop =
        is_orchestrator && (!plan_mode || should_allow_plan_readonly_pa_loop(&message));
    let read_only_pa_loop = plan_mode;

    if matches!(provider, super::provider_runner::ProviderKind::Codex) {
        let state_arc = Arc::clone(&state);
        let prompt_bg = prompt.clone();
        let cwd_bg = cwd.clone();
        let perm_path_bg = perm_path.clone();
        let chat_key_bg = chat_key.clone();
        let chat_file_bg = chat_file.clone();
        let stream_buf_bg = stream_buf.clone();
        let allow_pa_loop_bg = allow_pa_loop;
        let read_only_pa_loop_bg = read_only_pa_loop;
        let run_id_bg = run_id.clone();
        let operation_id_bg = operation_id.clone();
        let run_mode_label_bg = run_mode_label.to_string();
        let access_label_bg = access_label.clone();
        std::thread::spawn(move || {
            append_stream_event(
                &stream_buf_bg,
                json!({
                    "type": "run_progress",
                    "run_id": run_id_bg.as_str(),
                    "status": "running",
                    "phase": "provider",
                    "detail": "waiting for codex",
                    "ts": state_arc.now_iso()
                }),
                "stream codex provider running",
            );
            super::operation_state::emit(
                &state_arc,
                super::operation_state::OperationEventInput::new(
                    operation_id_bg.clone(),
                    super::operation_state::chat_actor(&chat_key_bg),
                    chat_key_bg.clone(),
                    "provider_started",
                    "provider",
                    "running",
                    "Waiting for Codex output",
                )
                .provider(
                    Some(provider.as_str()),
                    resolved_model.as_deref(),
                    resolved_effort.as_deref(),
                )
                .mode(
                    Some(run_mode_label_bg.as_str()),
                    Some(access_label_bg.as_str()),
                )
                .waiting_for("provider_output")
                .payload(json!({"run_id": run_id_bg.as_str()})),
            );
            let heartbeat_active = Arc::new(AtomicBool::new(true));
            let heartbeat_handle = spawn_provider_heartbeat(
                Arc::clone(&state_arc),
                chat_key_bg.clone(),
                stream_buf_bg.clone(),
                run_id_bg.clone(),
                operation_id_bg.clone(),
                Arc::clone(&heartbeat_active),
            );
            let response = super::provider_runner::run_provider_with_chat_control(
                &state_arc,
                provider,
                &cwd_bg,
                &prompt_bg,
                Some(&perm_path_bg),
                resolved_model.as_deref(),
                resolved_effort.as_deref(),
                Some(&chat_key_bg),
            );
            heartbeat_active.store(false, Ordering::Relaxed);
            let _ = heartbeat_handle.join();
            if is_cancelled(&state_arc, &chat_key_bg) {
                append_stream_event(
                    &stream_buf_bg,
                    json!({
                        "type": "run_done",
                        "run_id": run_id_bg.as_str(),
                        "status": "cancelled",
                        "phase": "cancelled",
                        "outcome": "cancelled",
                        "detail": "stopped by user",
                        "text_len": 0,
                        "ts": state_arc.now_iso()
                    }),
                    "stream codex cancelled after provider",
                );
                super::operation_state::emit(
                    &state_arc,
                    super::operation_state::OperationEventInput::new(
                        operation_id_bg.clone(),
                        super::operation_state::chat_actor(&chat_key_bg),
                        chat_key_bg.clone(),
                        "run_cancelled",
                        "cancelled",
                        "cancelled",
                        "Stopped by user",
                    )
                    .payload(json!({"run_id": run_id_bg.as_str()})),
                );
                crate::commands::jsonl::append_jsonl_logged(
                    &stream_buf_bg,
                    &json!({"type":"done","run_id":run_id_bg.as_str(),"text":"","tools":[],"outcome":"cancelled"}),
                    "stream codex cancelled done",
                );
                clear_activity(&state_arc, &chat_key_bg);
                return;
            }
            let ts = state_arc.now_iso();
            let asst_entry = json!({"ts": ts, "role": "assistant", "msg": response});
            crate::commands::jsonl::append_jsonl_logged(
                &chat_file_bg,
                &asst_entry,
                "stream codex response",
            );
            super::claude_runner::log_chat_event(&state_arc.root, &chat_key_bg, &response);
            crate::commands::jsonl::append_jsonl_logged(
                &stream_buf_bg,
                &json!({"type":"text","run_id":run_id_bg.as_str(),"text": response}),
                "stream codex text",
            );
            super::operation_state::emit(
                &state_arc,
                super::operation_state::OperationEventInput::new(
                    operation_id_bg.clone(),
                    super::operation_state::chat_actor(&chat_key_bg),
                    chat_key_bg.clone(),
                    "model_output",
                    "model_output",
                    "running",
                    "Model output received",
                )
                .provider(
                    Some(provider.as_str()),
                    resolved_model.as_deref(),
                    resolved_effort.as_deref(),
                )
                .payload(json!({
                    "run_id": run_id_bg.as_str(),
                    "text_len": response.len()
                })),
            );

            let final_response = if allow_pa_loop_bg {
                run_pa_agent_loop(
                    &state_arc,
                    provider,
                    &cwd_bg,
                    &perm_path_bg,
                    &chat_key_bg,
                    &chat_file_bg,
                    &stream_buf_bg,
                    &response,
                    resolved_model.as_deref(),
                    resolved_effort.as_deref(),
                    "stream codex",
                    Some(operation_id_bg.as_str()),
                    Some(run_id_bg.as_str()),
                    read_only_pa_loop_bg,
                )
            } else {
                response.clone()
            };
            let outcome = response_outcome(
                &final_response,
                is_cancelled(&state_arc, &chat_key_bg),
                None,
            );
            append_stream_event(
                &stream_buf_bg,
                json!({
                    "type": "run_done",
                    "run_id": run_id_bg.as_str(),
                    "status": if outcome == "done" { "done" } else if outcome == "cancelled" { "cancelled" } else { "failed" },
                    "phase": "done",
                    "outcome": outcome,
                    "detail": outcome,
                    "text_len": final_response.len(),
                    "ts": state_arc.now_iso()
                }),
                "stream codex run done",
            );
            super::operation_state::emit(
                &state_arc,
                super::operation_state::OperationEventInput::new(
                    operation_id_bg.clone(),
                    super::operation_state::chat_actor(&chat_key_bg),
                    chat_key_bg.clone(),
                    "run_done",
                    "done",
                    outcome,
                    outcome,
                )
                .provider(
                    Some(provider.as_str()),
                    resolved_model.as_deref(),
                    resolved_effort.as_deref(),
                )
                .payload(json!({
                    "run_id": run_id_bg.as_str(),
                    "text_len": final_response.len()
                })),
            );
            crate::commands::jsonl::append_jsonl_logged(
                &stream_buf_bg,
                &json!({"type":"done","run_id":run_id_bg.as_str(),"text": final_response,"tools":[],"outcome":outcome}),
                "stream codex done",
            );
            clear_activity(&state_arc, &chat_key_bg);
        });
        return Ok(json!({"status": "streaming", "project": chat_key, "run_id": run_id}));
    }

    let tmp = unique_tmp("stream");
    std::fs::write(&tmp, &prompt).map_err(|e| e.to_string())?;
    let stdin_file = std::fs::File::open(&tmp).map_err(|e| e.to_string())?;

    let claude_bin = super::claude_runner::find_claude();
    let mut cmd = super::claude_runner::silent_cmd(&claude_bin);
    cmd.args([
        "--continue",
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--settings",
        &perm_path,
    ]);

    if let Some(m) = &resolved_model {
        if !m.is_empty() {
            cmd.args(["--model", m]);
        }
    }
    if let Some(re) = &resolved_effort {
        if ["low", "medium", "high", "max"].contains(&re.as_str()) {
            cmd.args(["--effort", re]);
        }
    }

    let mut child = cmd
        .current_dir(&cwd)
        .stdin(std::process::Stdio::from(stdin_file))
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // Track PID for zombie prevention
    let pid = child.id();
    track_pid(&state, &chat_key, pid);
    crate::log_info!("[stream:{}] spawned claude pid={}", chat_key, pid);

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

    // Clone values needed by the background thread
    let state_arc = Arc::clone(&state);
    let chat_key_bg = chat_key.clone();
    let stream_buf_bg = stream_buf.clone();
    let chat_file_bg = chat_file.clone();
    let tmp_bg = tmp.clone();
    let cwd_bg = cwd.clone();
    let perm_path_bg = perm_path.clone();
    let resolved_model_bg = resolved_model.clone();
    let resolved_effort_bg = resolved_effort.clone();
    let operation_id_bg = operation_id.clone();
    let run_id_bg = run_id.clone();
    let read_only_pa_loop_bg = read_only_pa_loop;

    // Spawn background thread for blocking I/O — returns immediately
    std::thread::spawn(move || {
        stream_reader_loop(
            child,
            stdout,
            &state_arc,
            &chat_key_bg,
            &stream_buf_bg,
            &chat_file_bg,
            &tmp_bg,
            &cwd_bg,
            &perm_path_bg,
            resolved_model_bg.as_deref(),
            resolved_effort_bg.as_deref(),
            &run_id_bg,
            &operation_id_bg,
            allow_pa_loop,
            read_only_pa_loop_bg,
        );
    });

    Ok(json!({"status": "streaming", "project": chat_key, "run_id": run_id}))
}

/// Blocking reader loop — runs on a background thread
fn stream_reader_loop(
    mut child: std::process::Child,
    stdout: std::process::ChildStdout,
    state: &AppState,
    chat_key: &str,
    stream_buf: &std::path::Path,
    chat_file: &std::path::Path,
    tmp: &std::path::Path,
    cwd: &std::path::Path,
    perm_path: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    run_id: &str,
    operation_id: &str,
    allow_pa_loop: bool,
    read_only_pa_loop: bool,
) {
    crate::log_info!(
        "[stream:{}] reader loop started, pid={}",
        chat_key,
        child.id()
    );
    let stderr_handle = child
        .stderr
        .take()
        .map(super::claude_runner::spawn_reader_thread);
    let reader = std::io::BufReader::new(stdout);
    let mut full_text = String::new();
    let mut tool_blocks: Vec<Value> = Vec::new();
    let mut cur_tool_name = String::new();
    let mut cur_tool_input_json = String::new();
    let mut event_count: u32 = 0;
    let mut block_index: u32 = 0;
    let mut is_thinking = false; // track thinking content blocks
    let mut saved_chain: Vec<Value> = Vec::new(); // persist chain for reload

    if let Ok(mut buf_file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stream_buf)
    {
        use std::io::Write;

        let write_evt = |f: &mut std::fs::File, v: &Value| {
            let _ = writeln!(f, "{}", serde_json::to_string(v).unwrap_or_default());
            let _ = f.flush();
        };
        write_evt(
            &mut buf_file,
            &json!({
                "type": "run_progress",
                "run_id": run_id,
                "status": "running",
                "phase": "provider",
                "detail": "claude stream opened",
                "ts": state.now_iso()
            }),
        );
        super::operation_state::emit(
            state,
            super::operation_state::OperationEventInput::new(
                operation_id.to_string(),
                super::operation_state::chat_actor(chat_key),
                chat_key.to_string(),
                "provider_started",
                "provider",
                "running",
                "Claude stream opened",
            )
            .provider(Some("claude"), model, reasoning_effort)
            .waiting_for("provider_output")
            .payload(json!({"run_id": run_id})),
        );

        for line in reader.lines() {
            let Ok(raw) = line else { break };
            let trimmed = raw.trim_end_matches('\r');
            if trimmed.is_empty() {
                continue;
            }

            let Ok(evt) = serde_json::from_str::<Value>(trimmed) else {
                continue;
            };
            let etype = evt.get("type").and_then(|t| t.as_str()).unwrap_or("");
            event_count += 1;

            match etype {
                "stream_event" => {
                    let empty_obj = json!({});
                    let inner = evt.get("event").unwrap_or(&empty_obj);
                    let inner_type = inner.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match inner_type {
                        "content_block_start" => {
                            block_index += 1;
                            if let Some(block) = inner.get("content_block") {
                                let btype =
                                    block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if btype == "thinking" {
                                    is_thinking = true;
                                    write_evt(&mut buf_file, &json!({"type": "thinking_start"}));
                                } else if btype == "tool_use" {
                                    is_thinking = false;
                                    cur_tool_name = block
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("?")
                                        .to_string();
                                    cur_tool_input_json.clear();
                                    crate::log_info!(
                                        "[stream:{}] tool started: {}",
                                        chat_key,
                                        cur_tool_name
                                    );
                                    write_evt(
                                        &mut buf_file,
                                        &json!({"type": "tool_use", "tool": cur_tool_name, "input": {}, "status": "started"}),
                                    );
                                    super::operation_state::emit(
                                        state,
                                        super::operation_state::OperationEventInput::new(
                                            operation_id.to_string(),
                                            super::operation_state::chat_actor(chat_key),
                                            chat_key.to_string(),
                                            "tool_started",
                                            "tool",
                                            "running",
                                            format!("Tool started: {}", cur_tool_name),
                                        )
                                        .provider(Some("claude"), model, reasoning_effort)
                                        .current_tool(cur_tool_name.clone())
                                        .payload(json!({"run_id": run_id, "tool": cur_tool_name})),
                                    );
                                    write_evt(
                                        &mut buf_file,
                                        &json!({
                                            "type": "run_progress",
                                            "run_id": run_id,
                                            "status": "running",
                                            "phase": "tool",
                                            "detail": cur_tool_name,
                                            "ts": state.now_iso()
                                        }),
                                    );
                                } else {
                                    is_thinking = false;
                                }
                            }
                        }
                        "content_block_delta" => {
                            if let Some(delta) = inner.get("delta") {
                                let dtype =
                                    delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if dtype == "text_delta" {
                                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                        full_text.push_str(text);
                                        write_evt(
                                            &mut buf_file,
                                            &json!({
                                                "type": "text_delta",
                                                "run_id": run_id,
                                                "text": text,
                                                "text_len": full_text.len()
                                            }),
                                        );
                                        if event_count % 20 == 0 {
                                            super::operation_state::emit(
                                                state,
                                                super::operation_state::OperationEventInput::new(
                                                    operation_id.to_string(),
                                                    super::operation_state::chat_actor(chat_key),
                                                    chat_key.to_string(),
                                                    "model_output_delta",
                                                    "model_output",
                                                    "running",
                                                    "Streaming model output",
                                                )
                                                .provider(Some("claude"), model, reasoning_effort)
                                                .payload(json!({
                                                    "run_id": run_id,
                                                    "text_len": full_text.len()
                                                })),
                                            );
                                        }
                                    }
                                } else if dtype == "thinking_delta" {
                                    if let Some(text) =
                                        delta.get("thinking").and_then(|t| t.as_str())
                                    {
                                        write_evt(
                                            &mut buf_file,
                                            &json!({"type": "thinking_delta", "text": text}),
                                        );
                                    }
                                } else if dtype == "input_json_delta" {
                                    if let Some(partial) =
                                        delta.get("partial_json").and_then(|j| j.as_str())
                                    {
                                        cur_tool_input_json.push_str(partial);
                                    }
                                }
                            }
                        }
                        "content_block_stop" => {
                            // Thinking block finished
                            if is_thinking {
                                write_evt(&mut buf_file, &json!({"type": "thinking_stop"}));
                                is_thinking = false;
                            }
                            // Tool block finished — save accumulated tool
                            if !cur_tool_name.is_empty() {
                                let input: Value =
                                    serde_json::from_str(&cur_tool_input_json).unwrap_or(json!({}));
                                tool_blocks.push(json!({"tool": cur_tool_name, "input": input, "_bi": block_index}));
                                write_evt(
                                    &mut buf_file,
                                    &json!({"type": "tool_use", "tool": cur_tool_name, "input": input, "status": "complete"}),
                                );
                                super::operation_state::emit(
                                    state,
                                    super::operation_state::OperationEventInput::new(
                                        operation_id.to_string(),
                                        super::operation_state::chat_actor(chat_key),
                                        chat_key.to_string(),
                                        "tool_completed",
                                        "tool_complete",
                                        "running",
                                        format!("Tool completed: {}", cur_tool_name),
                                    )
                                    .provider(Some("claude"), model, reasoning_effort)
                                    .current_tool(cur_tool_name.clone())
                                    .payload(json!({"run_id": run_id, "tool": cur_tool_name})),
                                );
                                write_evt(
                                    &mut buf_file,
                                    &json!({
                                        "type": "run_progress",
                                        "run_id": run_id,
                                        "status": "running",
                                        "phase": "tool_complete",
                                        "detail": cur_tool_name,
                                        "ts": state.now_iso()
                                    }),
                                );
                                saved_chain.push(json!({"type":"tool","tool":cur_tool_name,"input":input,"status":"complete"}));
                                crate::log_info!(
                                    "[stream:{}] tool complete: {}",
                                    chat_key,
                                    cur_tool_name
                                );
                                cur_tool_name.clear();
                                cur_tool_input_json.clear();
                            }
                        }
                        _ => {}
                    }
                }
                "assistant" => {
                    if let Some(content) =
                        evt.pointer("/message/content").and_then(|c| c.as_array())
                    {
                        for block in content {
                            match block.get("type").and_then(|t| t.as_str()) {
                                Some("text") => {
                                    let text =
                                        block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                                    if !text.is_empty() {
                                        full_text = text.to_string();
                                        write_evt(
                                            &mut buf_file,
                                            &json!({"type": "text", "text": text}),
                                        );
                                    }
                                }
                                Some("tool_use") => {
                                    let tool =
                                        block.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                                    let input = block.get("input").cloned().unwrap_or(json!({}));
                                    // Dedup by block_index — stream_event blocks are tracked by index
                                    let already = tool_blocks.iter().any(|tb| {
                                        tb.get("_bi").and_then(|v| v.as_u64())
                                            == Some(block_index as u64)
                                    });
                                    if !already {
                                        tool_blocks.push(json!({"tool": tool, "input": input, "_bi": block_index}));
                                    }
                                    write_evt(
                                        &mut buf_file,
                                        &json!({"type": "tool_use", "tool": tool, "input": input, "status": "complete"}),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "user" => {
                    if let Some(content) =
                        evt.pointer("/message/content").and_then(|c| c.as_array())
                    {
                        for block in content {
                            if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                                let rc =
                                    block.get("content").and_then(|c| c.as_str()).unwrap_or("");
                                let truncated_rc = super::claude_runner::safe_truncate(rc, 500);
                                write_evt(
                                    &mut buf_file,
                                    &json!({"type": "tool_result", "content": truncated_rc}),
                                );
                                // Attach result to last tool in chain
                                if let Some(last_tool) = saved_chain.iter_mut().rev().find(|c| {
                                    c.get("type").and_then(|t| t.as_str()) == Some("tool")
                                        && c.get("result").is_none()
                                }) {
                                    last_tool.as_object_mut().map(|o| {
                                        o.insert("result".to_string(), json!(truncated_rc))
                                    });
                                }
                            }
                        }
                    }
                    if let Some(tur) = evt.get("tool_use_result") {
                        if tur.get("type").and_then(|t| t.as_str()) == Some("text") {
                            let fc = tur
                                .pointer("/file/content")
                                .and_then(|c| c.as_str())
                                .unwrap_or("");
                            if !fc.is_empty() {
                                write_evt(
                                    &mut buf_file,
                                    &json!({"type": "tool_result", "content": super::claude_runner::safe_truncate(fc, 500)}),
                                );
                            }
                        }
                    }
                }
                "system" => {
                    let subtype = evt.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
                    write_evt(&mut buf_file, &json!({"type": "system", "system": subtype}));
                }
                "result" => {
                    let rt = evt.get("result").and_then(|r| r.as_str()).unwrap_or("");
                    if !rt.is_empty() && full_text.is_empty() {
                        full_text = rt.to_string();
                    }
                    write_evt(
                        &mut buf_file,
                        &json!({
                            "type": "run_progress",
                            "run_id": run_id,
                            "status": "running",
                            "phase": "result",
                            "detail": "model result received",
                            "ts": state.now_iso()
                        }),
                    );
                    write_evt(
                        &mut buf_file,
                        &json!({"type": "result", "cost": evt.get("total_cost_usd"), "duration_ms": evt.get("duration_ms"), "tokens": evt.pointer("/usage/output_tokens")}),
                    );
                }
                _ => {}
            }
        }
    }

    // Cleanup — always runs regardless of how the loop exited
    // Drain stderr before wait to avoid deadlock
    let exit_status = child.wait();
    let stderr_bytes = super::claude_runner::collect_reader(stderr_handle).unwrap_or_default();
    untrack_pid_if_match(state, chat_key, child.id());
    let _ = std::fs::remove_file(tmp);
    clear_activity(state, chat_key);
    let exit_code = exit_status.as_ref().ok().and_then(|s| s.code());

    let stderr = String::from_utf8_lossy(&stderr_bytes);
    if !stderr.trim().is_empty() {
        crate::log_warn!(
            "[stream:{}] stderr: {}",
            chat_key,
            super::claude_runner::safe_truncate(&stderr, 500)
        );
    }

    crate::log_info!(
        "[stream:{}] finished: {} events, {} tools, {} chars text, exit={:?}",
        chat_key,
        event_count,
        tool_blocks.len(),
        full_text.len(),
        exit_code
    );

    // Save full response
    let ts2 = state.now_iso();
    // Add final text block to chain
    if !full_text.trim().is_empty() {
        saved_chain.push(json!({"type":"text","text":full_text.trim()}));
    }
    let asst_entry = json!({"ts": ts2, "role": "assistant", "msg": full_text.trim(), "tools": tool_blocks, "chain": saved_chain});
    crate::commands::jsonl::append_jsonl_logged(chat_file, &asst_entry, "stream asst response");

    // Log to activity feed
    super::claude_runner::log_chat_event(&state.root, chat_key, &full_text);

    // Validate stream result
    if full_text.is_empty() && event_count > 30 {
        crate::log_error!(
            "[stream:{}] empty response after {} events — possible stream failure",
            chat_key,
            event_count
        );
    }

    let final_text = if allow_pa_loop {
        run_pa_agent_loop(
            state,
            super::provider_runner::ProviderKind::Claude,
            cwd,
            perm_path,
            chat_key,
            chat_file,
            stream_buf,
            &full_text,
            model,
            reasoning_effort,
            "stream",
            Some(operation_id),
            Some(run_id),
            read_only_pa_loop,
        )
    } else {
        full_text.clone()
    };
    let outcome = response_outcome(&final_text, is_cancelled(state, chat_key), exit_code);
    append_stream_event(
        stream_buf,
        json!({
            "type": "run_done",
            "run_id": run_id,
            "status": if outcome == "done" { "done" } else if outcome == "cancelled" { "cancelled" } else { "failed" },
            "phase": "done",
            "outcome": outcome,
            "detail": outcome,
            "text_len": final_text.len(),
            "exit_code": exit_code,
            "ts": state.now_iso()
        }),
        "stream run done",
    );
    super::operation_state::emit(
        state,
        super::operation_state::OperationEventInput::new(
            operation_id.to_string(),
            super::operation_state::chat_actor(chat_key),
            chat_key.to_string(),
            "run_done",
            "done",
            outcome,
            outcome,
        )
        .provider(Some("claude"), model, reasoning_effort)
        .payload(json!({
            "run_id": run_id,
            "text_len": final_text.len(),
            "exit_code": exit_code
        })),
    );

    // Always write "done" marker so frontend stops polling
    crate::commands::jsonl::append_jsonl_logged(
        stream_buf,
        &json!({"type":"done","run_id":run_id,"text":final_text.trim(),"tools":tool_blocks,"outcome":outcome}),
        "stream done marker",
    );
}

const MAX_AUTO_CONTINUE_TURNS: usize = 20;
const AUTO_CONTINUE_REPEAT_LIMIT: usize = 2;

#[derive(Default)]
struct PaLoopFeedback {
    items: Vec<String>,
    actionable: usize,
    warnings: usize,
}

impl PaLoopFeedback {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn signature(&self) -> String {
        self.items.join("\n")
    }
}

impl From<super::coordinator_wait::WaitCoordinatorSnapshot> for PaLoopFeedback {
    fn from(snapshot: super::coordinator_wait::WaitCoordinatorSnapshot) -> Self {
        Self {
            items: snapshot.items,
            actionable: snapshot.actionable,
            warnings: snapshot.warnings,
        }
    }
}

fn feedback_preview(text: &str) -> String {
    let clean = text.replace('\r', "").trim().to_string();
    let preview: String = clean.chars().take(900).collect();
    if clean.chars().count() > 900 {
        format!("{}...", preview)
    } else {
        preview
    }
}

fn execute_pa_commands_for_agent_loop(
    state: &AppState,
    chat_file: &Path,
    stream_buf: &Path,
    response: &str,
    label_prefix: &str,
    chat_key: &str,
    operation_id: Option<&str>,
    read_only_only: bool,
) -> PaLoopFeedback {
    let command_text = super::pa_commands::recover_bare_readonly_commands(response);
    let recovered_readonly = super::pa_commands::recoverable_bare_readonly_commands(response);
    let commands = super::pa_commands::parse_pa_commands(&command_text, state);
    let warnings = super::pa_commands::detect_malformed_commands(&command_text);
    let mut feedback = PaLoopFeedback::default();
    if !recovered_readonly.is_empty() && !commands.is_empty() {
        append_pa_feedback(
            state,
            chat_file,
            stream_buf,
            "pa_status",
            &format!(
                "Recovered read-only AgentOS commands from inline code: {}. Write commands are not inferred from prose.",
                recovered_readonly.join(", ")
            ),
            None,
            &format!("{} recovered readonly cmds", label_prefix),
        );
    }

    for parsed in &commands {
        if !parsed.valid {
            if let Some(err) = &parsed.error {
                let command_label = super::pa_commands::describe_pa_command(&parsed.cmd);
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "warning",
                    err,
                    Some(command_label.as_str()),
                    &format!("{} warning", label_prefix),
                );
                feedback.warnings += 1;
                feedback
                    .items
                    .push(format!("{} -> warning: {}", command_label, err));
            }
            continue;
        }

        let command_label = super::pa_commands::describe_pa_command(&parsed.cmd);
        if read_only_only && !super::pa_commands::is_read_only_pa_command(&parsed.cmd) {
            let text = format!(
                "{} -> skipped: current run mode is Plan/read-only; switch to act/full or use a dry-run/read-only command.",
                command_label
            );
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "warning",
                &text,
                None,
                &format!("{} readonly skip", label_prefix),
            );
            feedback.warnings += 1;
            feedback.items.push(text);
            continue;
        }
        feedback.actionable += 1;
        if let Some(operation_id) = operation_id {
            super::operation_state::emit(
                state,
                super::operation_state::OperationEventInput::new(
                    operation_id.to_string(),
                    "agentos",
                    chat_key.to_string(),
                    "pa_command_started",
                    "command",
                    "running",
                    format!("Running {}", command_label),
                )
                .current_tool(command_label.clone())
                .waiting_for("agentos_command")
                .payload(json!({"command": command_label.as_str()})),
            );
        }
        append_pa_feedback(
            state,
            chat_file,
            stream_buf,
            "pa_status",
            &format!("Running {}", command_label),
            Some(command_label.as_str()),
            &format!("{} pa start", label_prefix),
        );

        if let Some(text) = super::pa_commands::execute_pa_command(state, &parsed.cmd) {
            if let Some(operation_id) = operation_id {
                super::operation_state::emit(
                    state,
                    super::operation_state::OperationEventInput::new(
                        operation_id.to_string(),
                        "agentos",
                        chat_key.to_string(),
                        "pa_command_result",
                        "command",
                        "running",
                        format!("Completed {}", command_label),
                    )
                    .current_tool(command_label.clone())
                    .payload(json!({
                        "command": command_label.as_str(),
                        "text_len": text.len()
                    })),
                );
            }
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "pa_result",
                &text,
                Some(command_label.as_str()),
                &format!("{} pa result", label_prefix),
            );
            feedback
                .items
                .push(format!("{} -> {}", command_label, feedback_preview(&text)));
        } else {
            let done = format!("Completed {} (no output)", command_label);
            if let Some(operation_id) = operation_id {
                super::operation_state::emit(
                    state,
                    super::operation_state::OperationEventInput::new(
                        operation_id.to_string(),
                        "agentos",
                        chat_key.to_string(),
                        "pa_command_result",
                        "command",
                        "running",
                        done.clone(),
                    )
                    .current_tool(command_label.clone())
                    .payload(json!({"command": command_label.as_str()})),
                );
            }
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "pa_status",
                &done,
                Some(command_label.as_str()),
                &format!("{} pa complete", label_prefix),
            );
            feedback.items.push(done);
        }
    }

    for warning in warnings {
        if let Some(operation_id) = operation_id {
            super::operation_state::emit(
                state,
                super::operation_state::OperationEventInput::new(
                    operation_id.to_string(),
                    "agentos",
                    chat_key.to_string(),
                    "pa_command_warning",
                    "command",
                    "needs_user",
                    "Malformed AgentOS command",
                )
                .blocked_by("malformed_command")
                .payload(json!({"warning": warning.as_str()})),
            );
        }
        append_pa_feedback(
            state,
            chat_file,
            stream_buf,
            "warning",
            &warning,
            None,
            &format!("{} malformed cmd", label_prefix),
        );
        feedback.warnings += 1;
        feedback.items.push(format!("warning -> {}", warning));
    }

    feedback
}

fn build_auto_continue_prompt(turn: usize, feedback: &PaLoopFeedback) -> String {
    format!(
        "[AUTO-CONTINUE AFTER AGENTOS COMMANDS]\n\
         AgentOS executed the PA commands from your previous response or refreshed the coordination context.\n\
         Results / context:\n{}\n\n\
         Continue autonomously from these results. Stop by returning a final status with no PA command tags when the task is complete or blocked. \
         If a ready route offers [WORK_ITEM_QUEUE:id] and it is still priority, emit that command instead of idling on running projects. \
         If the context says you claimed an action but AgentOS executed no command, correct it now: emit the exact command tag or explicitly say that no action was launched. \
         If the context says you repeated the same diagnostic command, do not run it again; pick a different next action or give a final blocked status. \
         Emit the next PA command tags only when another AgentOS action is actually required. \
         Do not ask the user to type continue. Continue in the same user-facing language as the conversation; if recent user messages are Russian/Cyrillic, reply in Russian.\n\
         Auto-continue turn: {}/{} safety ceiling. Actionable commands: {}. Warnings: {}.",
        feedback.items
            .iter()
            .enumerate()
            .map(|(idx, item)| format!("{}. {}", idx + 1, item))
            .collect::<Vec<_>>()
            .join("\n"),
        turn,
        MAX_AUTO_CONTINUE_TURNS,
        feedback.actionable,
        feedback.warnings
    )
}

fn claims_agentos_action_without_command(response: &str) -> bool {
    let text = response.to_lowercase();
    if text.trim().is_empty() {
        return false;
    }

    let negated = [
        "не отправил",
        "не отправлял",
        "не запустил",
        "не запускал",
        "не делегировал",
        "ничего не отправил",
        "ничего не запускал",
        "не подключил",
        "не подключаю",
        "не проверил",
        "ничего не подключил",
        "did not delegate",
        "didn't delegate",
        "did not send",
        "didn't send",
        "no action was launched",
    ];
    if negated.iter().any(|needle| text.contains(needle)) {
        return false;
    }

    let action_verbs = [
        "отправил",
        "отправляю",
        "запустил",
        "запускаю",
        "сделегировал",
        "делегировал",
        "поставил задачу",
        "создал делегацию",
        "проверяю",
        "проверил",
        "подключаю",
        "подключил",
        "добавляю",
        "завожу",
        "sent delegation",
        "queued delegation",
        "started delegation",
        "delegated",
        "launched",
    ];
    let action_targets = [
        "делегац",
        "проект",
        "проекты",
        "подключение",
        "онборд",
        "agentos",
        "delegate",
        "delegation",
        "route",
        "work item",
        "project onboarding",
        "project_connect",
        "задачу на",
    ];

    action_verbs.iter().any(|needle| text.contains(needle))
        && action_targets.iter().any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{
        build_auto_continue_prompt, claims_agentos_action_without_command,
        should_allow_plan_readonly_pa_loop, PaLoopFeedback,
    };

    #[test]
    fn detects_claimed_delegation_without_command() {
        assert!(claims_agentos_action_without_command(
            "Отправил точечную делегацию на Avilex и жду результата."
        ));
        assert!(claims_agentos_action_without_command(
            "Sent delegation to AgentOS and queued the next route."
        ));
        assert!(claims_agentos_action_without_command(
            "Проверяю подключение проектов к AgentOS."
        ));
        assert!(claims_agentos_action_without_command(
            "Подключил проекты к onboarding metadata."
        ));
    }

    #[test]
    fn ignores_negated_delegation_claim() {
        assert!(!claims_agentos_action_without_command(
            "Не отправил делегацию, потому что нет валидной команды."
        ));
        assert!(!claims_agentos_action_without_command(
            "I did not delegate anything; no action was launched."
        ));
    }

    #[test]
    fn auto_continue_prompt_tells_agent_to_correct_missing_commands() {
        let mut feedback = PaLoopFeedback::default();
        feedback
            .items
            .push("Agent claimed an action but no command was executed.".to_string());
        feedback.warnings = 1;

        let prompt = build_auto_continue_prompt(1, &feedback);

        assert!(prompt.contains("claimed an action"));
        assert!(prompt.contains("emit the exact command tag"));
        assert!(prompt.contains("repeated the same diagnostic command"));
    }

    #[test]
    fn plan_mode_allows_readonly_loop_for_onboarding_intents() {
        assert!(should_allow_plan_readonly_pa_loop(
            "Проверь подключение проектов к AgentOS"
        ));
        assert!(should_allow_plan_readonly_pa_loop(
            "Run project onboarding audit"
        ));
        assert!(!should_allow_plan_readonly_pa_loop(
            "Напиши архитектурный план без действий"
        ));
    }
}

fn run_pa_agent_loop(
    state: &AppState,
    provider: super::provider_runner::ProviderKind,
    cwd: &Path,
    perm_path: &str,
    chat_key: &str,
    chat_file: &Path,
    stream_buf: &Path,
    first_response: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    label_prefix: &str,
    operation_id: Option<&str>,
    run_id: Option<&str>,
    read_only_only: bool,
) -> String {
    let mut response = first_response.to_string();
    let mut final_response = response.clone();
    let mut last_signature = String::new();
    let mut repeat_count = 0usize;
    let mut wait_context_sent = false;
    let mut missing_command_recovery_sent = false;
    let mut repeat_recovery_sent = false;

    for turn in 1..=MAX_AUTO_CONTINUE_TURNS {
        if is_cancelled(state, chat_key) {
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "warning",
                "Auto-run stopped by user.",
                None,
                &format!("{} auto cancelled", label_prefix),
            );
            break;
        }

        let mut feedback = execute_pa_commands_for_agent_loop(
            state,
            chat_file,
            stream_buf,
            &response,
            label_prefix,
            chat_key,
            operation_id,
            read_only_only,
        );
        if feedback.is_empty() {
            if !missing_command_recovery_sent && claims_agentos_action_without_command(&response) {
                let text = "Агент заявил, что запустил действие, но AgentOS не получил ни одной исполняемой команды. Исправь это следующим ответом: выдай точный тег команды, например [PROJECT_ONBOARD_AUDIT], [PROJECT_CONNECT:Project:Other:balanced:dry] или [DELEGATE:Project]...[/DELEGATE], либо явно напиши, что действие не было запущено.";
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "warning",
                    text,
                    None,
                    &format!("{} missing command recovery", label_prefix),
                );
                if let Some(operation_id) = operation_id {
                    super::operation_state::emit(
                        state,
                        super::operation_state::OperationEventInput::new(
                            operation_id.to_string(),
                            "agentos",
                            chat_key.to_string(),
                            "pa_command_missing",
                            "command",
                            "needs_user",
                            "Claimed action without executable AgentOS command",
                        )
                        .blocked_by("missing_agentos_command")
                        .payload(json!({"response_preview": feedback_preview(&response)})),
                    );
                }
                feedback.warnings += 1;
                feedback.items.push(text.to_string());
                missing_command_recovery_sent = true;
                wait_context_sent = false;
            } else if wait_context_sent {
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "pa_status",
                    "Автоцикл остановлен: новых AgentOS-команд нет, wait-context уже был отправлен.",
                    None,
                    &format!("{} auto idle stop", label_prefix),
                );
                break;
            }
            if feedback.is_empty() {
                let Some(wait_snapshot) =
                    super::coordinator_wait::build_wait_coordinator_snapshot(state, None, None, 4)
                else {
                    append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "pa_status",
                    "Автоцикл остановлен: агент не выдал AgentOS-команд, готовых маршрутов для продолжения нет.",
                    None,
                    &format!("{} auto no commands", label_prefix),
                );
                    break;
                };
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "pa_status",
                    &format!("Waiting coordinator: {}", wait_snapshot.summary),
                    None,
                    &format!("{} wait coordinator", label_prefix),
                );
                feedback = wait_snapshot.into();
                wait_context_sent = true;
            }
        } else {
            wait_context_sent = false;
            missing_command_recovery_sent = false;
        }

        let signature = feedback.signature();
        if signature == last_signature {
            repeat_count += 1;
        } else {
            last_signature = signature;
            repeat_count = 1;
        }
        if repeat_count >= AUTO_CONTINUE_REPEAT_LIMIT {
            if !repeat_recovery_sent {
                let text = "Агент повторил тот же диагностический результат. Не запускай эту же команду снова: выбери другой следующий шаг, поставь точную делегацию, проверь другой блокер или дай финальный статус, если дальнейшего действия нет.";
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "warning",
                    text,
                    None,
                    &format!("{} auto repeat recovery", label_prefix),
                );
                feedback.warnings += 1;
                feedback.items.push(text.to_string());
                repeat_recovery_sent = true;
                last_signature.clear();
                repeat_count = 0;
            } else {
                append_pa_feedback(
                    state,
                    chat_file,
                    stream_buf,
                    "warning",
                    "Auto-run stopped because the agent repeated the same command result loop.",
                    None,
                    &format!("{} auto repeat stop", label_prefix),
                );
                break;
            }
        } else {
            repeat_recovery_sent = false;
        }

        if is_cancelled(state, chat_key) {
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "warning",
                "Auto-run stopped by user after the current command batch.",
                None,
                &format!("{} auto cancelled after commands", label_prefix),
            );
            break;
        }

        append_pa_feedback(
            state,
            chat_file,
            stream_buf,
            "pa_status",
            &format!(
                "Auto-continuing after {} AgentOS action{}",
                feedback.items.len(),
                if feedback.items.len() == 1 { "" } else { "s" }
            ),
            None,
            &format!("{} auto continue", label_prefix),
        );
        if let Some(operation_id) = operation_id {
            super::operation_state::emit(
                state,
                super::operation_state::OperationEventInput::new(
                    operation_id.to_string(),
                    "orchestrator",
                    chat_key.to_string(),
                    "auto_continue",
                    "agent_loop",
                    "running",
                    format!("Auto-continue turn {}", turn),
                )
                .waiting_for("provider_output")
                .payload(json!({
                    "turn": turn,
                    "actions": feedback.items.len(),
                    "warnings": feedback.warnings
                })),
            );
        }

        let user_message = build_auto_continue_prompt(turn, &feedback);
        let prompt = super::chat_parse::build_full_pa_prompt(state, &user_message);
        response = super::provider_runner::run_provider_with_opts(
            state,
            provider,
            cwd,
            &prompt,
            Some(perm_path),
            model,
            reasoning_effort,
        );
        final_response = response.clone();

        if is_cancelled(state, chat_key) {
            append_pa_feedback(
                state,
                chat_file,
                stream_buf,
                "warning",
                "Auto-run stopped by user.",
                None,
                &format!("{} auto cancelled after agent", label_prefix),
            );
            break;
        }

        let ts = state.now_iso();
        let asst_entry = json!({"ts": ts, "role": "assistant", "msg": response});
        crate::commands::jsonl::append_jsonl_logged(
            chat_file,
            &asst_entry,
            &format!("{} auto response", label_prefix),
        );
        super::claude_runner::log_chat_event(state.root.as_path(), chat_key, &response);
        append_stream_event(
            stream_buf,
            json!({"type":"text","run_id":run_id,"text": response}),
            &format!("{} auto text", label_prefix),
        );
    }

    if !is_cancelled(state, chat_key) {
        clear_cancel(state, chat_key);
    }

    final_response
}

// poll_stream, stop_chat, is_chat_running → moved to chat_stream_poll.rs
