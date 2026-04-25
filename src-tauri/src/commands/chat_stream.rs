//! Streaming chat: stream_chat, poll_stream, stop_chat, is_chat_running.
use super::claude_runner::{get_permission_path, unique_tmp};
use super::process_manager::{
    clear_activity, clear_cancel, is_cancelled, kill_existing, set_activity, track_pid, untrack_pid,
};
use crate::state::AppState;
use serde_json::{json, Value};
use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;
use tauri::State;

fn append_stream_event(stream_buf: &Path, event: Value, label: &str) {
    crate::commands::jsonl::append_jsonl_logged(stream_buf, &event, label);
}

fn append_chat_system(
    state: &AppState,
    chat_file: &Path,
    event_type: &str,
    text: &str,
    command: Option<&str>,
    label: &str,
) {
    crate::commands::jsonl::append_jsonl_logged(
        chat_file,
        &json!({
            "ts": state.now_iso(),
            "role": "system",
            "kind": "pa_feedback",
            "pa_type": event_type,
            "pa_command": command,
            "msg": text
        }),
        label,
    );
}

fn append_pa_feedback(
    state: &AppState,
    chat_file: &Path,
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
    append_chat_system(state, chat_file, event_type, text, command, label);
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
    set_activity(&state, &chat_key, "streaming", &detail);

    // Kill any existing process for this chat (prevents zombie accumulation)
    kill_existing(&state, &chat_key);
    clear_cancel(&state, &chat_key);

    // Stream buffer file — per chat_key so multiple chats don't collide
    let stream_buf = state
        .root
        .join("tasks")
        .join(format!(".stream-{}.jsonl", chat_key));
    let _ = std::fs::write(&stream_buf, ""); // Clear buffer

    let (provider, resolved_model, resolved_effort) =
        super::provider_runner::resolve_single_chat_settings(
            &state,
            &project,
            provider.as_deref(),
            model.as_deref(),
            reasoning_effort.as_deref(),
        );

    let is_orchestrator = project.is_empty();

    if matches!(provider, super::provider_runner::ProviderKind::Codex) {
        let state_arc = Arc::clone(&state);
        let prompt_bg = prompt.clone();
        let cwd_bg = cwd.clone();
        let perm_path_bg = perm_path.clone();
        let chat_key_bg = chat_key.clone();
        let chat_file_bg = chat_file.clone();
        let stream_buf_bg = stream_buf.clone();
        let is_orchestrator_bg = is_orchestrator;
        std::thread::spawn(move || {
            let response = super::provider_runner::run_provider_with_opts(
                &state_arc,
                provider,
                &cwd_bg,
                &prompt_bg,
                Some(&perm_path_bg),
                resolved_model.as_deref(),
                resolved_effort.as_deref(),
            );
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
                &json!({"type":"text","text": response}),
                "stream codex text",
            );

            let final_response = if is_orchestrator_bg {
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
                )
            } else {
                response.clone()
            };
            crate::commands::jsonl::append_jsonl_logged(
                &stream_buf_bg,
                &json!({"type":"done","text": final_response,"tools":[]}),
                "stream codex done",
            );
            clear_activity(&state_arc, &chat_key_bg);
        });
        return Ok(json!({"status": "streaming", "project": chat_key}));
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
            is_orchestrator,
        );
    });

    Ok(json!({"status": "streaming", "project": chat_key}))
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
    is_orchestrator: bool,
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
                                            &json!({"type": "text_delta", "text": text, "full": full_text}),
                                        );
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
    untrack_pid(state, chat_key);
    let _ = std::fs::remove_file(tmp);
    clear_activity(state, chat_key);

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
        exit_status.ok().map(|s| s.code())
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

    let final_text = if is_orchestrator {
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
        )
    } else {
        full_text.clone()
    };

    // Always write "done" marker so frontend stops polling
    crate::commands::jsonl::append_jsonl_logged(
        stream_buf,
        &json!({"type":"done","text":final_text.trim(),"tools":tool_blocks}),
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
) -> PaLoopFeedback {
    let commands = super::pa_commands::parse_pa_commands(response, state);
    let warnings = super::pa_commands::detect_malformed_commands(response);
    let mut feedback = PaLoopFeedback::default();

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
        feedback.actionable += 1;
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
         AgentOS executed the PA commands from your previous response.\n\
         Results:\n{}\n\n\
         Continue autonomously from these results. Stop by returning a final status with no PA command tags when the task is complete or blocked. \
         Emit the next PA command tags only when another AgentOS action is actually required. \
         Do not ask the user to type continue.\n\
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
) -> String {
    let mut response = first_response.to_string();
    let mut final_response = response.clone();
    let mut last_signature = String::new();
    let mut repeat_count = 0usize;

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

        let feedback = execute_pa_commands_for_agent_loop(
            state,
            chat_file,
            stream_buf,
            &response,
            label_prefix,
        );
        if feedback.is_empty() {
            break;
        }

        let signature = feedback.signature();
        if signature == last_signature {
            repeat_count += 1;
        } else {
            last_signature = signature;
            repeat_count = 1;
        }
        if repeat_count >= AUTO_CONTINUE_REPEAT_LIMIT {
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
            json!({"type":"text","text": response}),
            &format!("{} auto text", label_prefix),
        );
    }

    if !is_cancelled(state, chat_key) {
        clear_cancel(state, chat_key);
    }

    final_response
}

// poll_stream, stop_chat, is_chat_running → moved to chat_stream_poll.rs
