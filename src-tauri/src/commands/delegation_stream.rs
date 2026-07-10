//! Delegation streaming: real-time progress for delegation execution.
//! Reuses the stream buffer pattern from chat_stream.rs.

use crate::state::AppState;
use serde_json::{json, Value};
use std::io::{BufRead, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// Run the selected provider for a delegation step.
/// Writes events to stream_buf as JSONL. Returns (response_text, is_permission_request).
pub fn run_delegation_streaming(
    state: &AppState,
    provider: super::provider_runner::ProviderKind,
    project_dir: &Path,
    task: &str,
    perm_path: &str,
    model: Option<&str>,
    effort: Option<&str>,
    stream_buf: &Path,
    chat_key: Option<&str>,
) -> (String, bool) {
    if provider == super::provider_runner::ProviderKind::Codex {
        super::jsonl::append_jsonl_logged(
            stream_buf,
            &json!({
                "type": "provider",
                "provider": provider.as_str(),
                "model": model.unwrap_or(""),
                "effort": effort.unwrap_or("")
            }),
            "deleg provider",
        );
        let response = super::provider_runner::run_provider_with_chat_control(
            state,
            provider,
            project_dir,
            task,
            Some(perm_path),
            model,
            effort,
            chat_key,
        );
        let is_perm = super::claude_runner::is_permission_request(&response);
        if !response.trim().is_empty() {
            super::jsonl::append_jsonl_logged(
                stream_buf,
                &json!({
                    "type": "text_delta",
                    "text": response.chars().take(400).collect::<String>()
                }),
                "deleg provider response",
            );
        }
        crate::log_info!(
            "[deleg-stream] provider={} finished: {} chars, perm_request={}",
            provider,
            response.len(),
            is_perm
        );
        return (response, is_perm);
    }

    let tmp = super::claude_runner::unique_tmp("deleg-stream");
    if std::fs::write(&tmp, task).is_err() {
        return ("Error: could not write temp file".to_string(), false);
    }
    let stdin_file = match std::fs::File::open(&tmp) {
        Ok(f) => f,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return (format!("Error opening temp file: {}", e), false);
        }
    };

    let claude_bin = super::claude_runner::find_claude();
    let mut cmd = super::claude_runner::silent_cmd(&claude_bin);
    // Context rotation: drop --continue after 3 consecutive failures (prevents gutter problem)
    let use_continue = !should_fresh_session(state, project_dir);
    if use_continue {
        cmd.args(["--continue"]);
    }
    cmd.args([
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--settings",
        perm_path,
    ]);

    if let Some(m) = model {
        if !m.is_empty() {
            cmd.args(["--model", m]);
        }
    }
    if let Some(re) = effort {
        if !re.is_empty() && ["low", "medium", "high", "max"].contains(&re) {
            cmd.args(["--effort", re]);
        }
    }

    let mut child = match cmd
        .current_dir(project_dir)
        .stdin(std::process::Stdio::from(stdin_file))
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return (format!("Error running claude: {}", e), false);
        }
    };

    let pid = child.id();
    let chat_key = format!("deleg-{}", pid);
    super::process_manager::track_pid(state, &chat_key, pid);

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = std::fs::remove_file(&tmp);
            return ("Error: no stdout".to_string(), false);
        }
    };

    // Read stream events and write to buffer
    let mut full_text = String::new();
    let mut event_count: u32 = 0;
    let mut total_tokens: u64 = 0;
    let _start_time = std::time::Instant::now();
    // Safety: token budget (default 150K) and heartbeat (120s no events)
    let token_budget: u64 = 150_000;
    let heartbeat_timeout_secs: u64 = 120;

    // The watchdog is explicitly stopped and joined so it cannot outlive the delegation.
    let child_pid = child.id();
    let heartbeat = Arc::new(Mutex::new(Instant::now()));
    let watchdog_heartbeat = heartbeat.clone();
    let watchdog_timed_out = Arc::new(AtomicBool::new(false));
    let watchdog_timeout_flag = watchdog_timed_out.clone();
    let (watchdog_stop_tx, watchdog_stop_rx) = mpsc::channel();
    let watchdog = std::thread::spawn(move || loop {
        match watchdog_stop_rx.recv_timeout(Duration::from_secs(15)) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let elapsed = watchdog_heartbeat
            .lock()
            .map(|last| last.elapsed())
            .unwrap_or_default();
        if elapsed > Duration::from_secs(heartbeat_timeout_secs) {
            watchdog_timeout_flag.store(true, Ordering::Release);
            crate::log_warn!(
                "[deleg-stream] HEARTBEAT: no events for {}s, killing pid {}",
                elapsed.as_secs(),
                child_pid
            );
            #[cfg(target_os = "windows")]
            {
                let _ = super::claude_runner::silent_cmd("taskkill")
                    .args(["/F", "/T", "/PID", &child_pid.to_string()])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &child_pid.to_string()])
                    .output();
            }
            break;
        }
    });

    let reader = std::io::BufReader::new(stdout);
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
            if let Ok(mut last) = heartbeat.lock() {
                *last = Instant::now();
            }

            match etype {
                "stream_event" => {
                    let empty_obj = json!({});
                    let inner = evt.get("event").unwrap_or(&empty_obj);
                    let inner_type = inner.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match inner_type {
                        "content_block_delta" => {
                            if let Some(delta) = inner.get("delta") {
                                let dtype =
                                    delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if dtype == "text_delta" {
                                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                        full_text.push_str(text);
                                        // Write every 10th text delta to reduce file I/O
                                        if event_count % 10 == 0 {
                                            write_evt(
                                                &mut buf_file,
                                                &json!({"type":"text_delta","text":text}),
                                            );
                                        }
                                    }
                                } else if dtype == "thinking_delta" {
                                    if let Some(text) =
                                        delta.get("thinking").and_then(|t| t.as_str())
                                    {
                                        write_evt(
                                            &mut buf_file,
                                            &json!({"type":"thinking","text":text}),
                                        );
                                    }
                                }
                            }
                        }
                        "content_block_start" => {
                            if let Some(block) = inner.get("content_block") {
                                let btype =
                                    block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if btype == "tool_use" {
                                    let tool =
                                        block.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                                    write_evt(
                                        &mut buf_file,
                                        &json!({"type":"tool_start","tool":tool}),
                                    );
                                }
                            }
                        }
                        "content_block_stop" => {
                            write_evt(&mut buf_file, &json!({"type":"tool_stop"}));
                        }
                        _ => {}
                    }
                }
                "assistant" => {
                    if let Some(content) =
                        evt.pointer("/message/content").and_then(|c| c.as_array())
                    {
                        for block in content {
                            if let Some("text") = block.get("type").and_then(|t| t.as_str()) {
                                let text = block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                                if !text.is_empty() {
                                    full_text = text.to_string();
                                }
                            }
                        }
                    }
                }
                "result" => {
                    // Extract usage info for cost tracking + token budget
                    if let Some(usage) = evt.get("usage") {
                        write_evt(&mut buf_file, &json!({"type":"usage","usage":usage}));
                        if let Some(t) = usage.get("output_tokens").and_then(|t| t.as_u64()) {
                            total_tokens += t;
                        }
                        if let Some(t) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                            total_tokens += t;
                        }
                    }
                    if let Some(cost) = evt.get("cost_usd") {
                        write_evt(&mut buf_file, &json!({"type":"cost","cost_usd":cost}));
                    }
                }
                _ => {}
            }

            // Safety rails: token budget + heartbeat
            if total_tokens > token_budget {
                crate::log_warn!(
                    "[deleg-stream] TOKEN BUDGET exceeded: {} > {} — killing",
                    total_tokens,
                    token_budget
                );
                write_evt(
                    &mut buf_file,
                    &json!({"type":"safety","reason":"token_budget","tokens":total_tokens}),
                );
                super::process_manager::kill_existing(state, &chat_key);
                full_text = format!(
                    "Error: token budget exceeded ({} tokens). Process killed.",
                    total_tokens
                );
                break;
            }
            // Heartbeat handled by watchdog thread — no in-loop check needed
        }
    }

    let _ = watchdog_stop_tx.send(());
    if watchdog.join().is_err() {
        crate::log_warn!(
            "[deleg-stream] watchdog thread panicked for pid={}",
            child_pid
        );
    }
    if watchdog_timed_out.load(Ordering::Acquire) {
        super::jsonl::append_jsonl_logged(
            stream_buf,
            &json!({"type":"safety","reason":"heartbeat_timeout"}),
            "delegation heartbeat timeout",
        );
        if full_text.is_empty() {
            full_text = "Error: delegation provider stopped after heartbeat timeout.".to_string();
        }
    }

    // Cleanup
    let _ = child.wait();
    super::process_manager::untrack_pid(state, &chat_key);
    let _ = std::fs::remove_file(&tmp);

    let is_perm = super::claude_runner::is_permission_request(&full_text);
    crate::log_info!(
        "[deleg-stream] finished: {} events, {} chars, perm_request={}",
        event_count,
        full_text.len(),
        is_perm
    );

    (full_text, is_perm)
}

/// Emit a stage transition event to delegation stream buffer
pub fn emit_stage(stream_buf: &Path, stage: &str, label: &str) {
    super::jsonl::append_jsonl_logged(
        stream_buf,
        &json!({"type":"stage","stage":stage,"label":label}),
        "deleg stage",
    );
}

/// Emit done marker to delegation stream buffer
pub fn emit_done(stream_buf: &Path, status: &str, response: &str) {
    let preview: String = response.chars().take(200).collect();
    super::jsonl::append_jsonl_logged(
        stream_buf,
        &json!({"type":"done","status":status,"response":preview}),
        "deleg done",
    );
}

/// Poll delegation stream buffer — frontend calls every 500ms
pub fn parse_stream_usage(
    stream_buf: &Path,
    fallback_model: &str,
) -> Option<super::usage::UsageInfo> {
    let content = std::fs::read_to_string(stream_buf).ok()?;
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut cost_usd = 0.0f64;
    let mut model = fallback_model.to_string();

    for line in content.lines() {
        let Ok(evt) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match evt.get("type").and_then(|v| v.as_str()).unwrap_or("") {
            "provider" => {
                if model.is_empty() {
                    model = evt
                        .get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
            "usage" => {
                if let Some(usage) = evt.get("usage") {
                    input_tokens += usage
                        .get("input_tokens")
                        .or_else(|| usage.get("cache_creation_input_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    output_tokens += usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                }
            }
            "cost" => {
                cost_usd += evt.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
            }
            _ => {}
        }
    }

    if input_tokens == 0 && output_tokens == 0 && cost_usd <= 0.0 {
        return None;
    }

    Some(super::usage::UsageInfo {
        input_tokens,
        output_tokens,
        cost_usd,
        model,
    })
}

#[tauri::command]
pub fn poll_delegation_stream(state: State<Arc<AppState>>, id: String, offset: usize) -> Value {
    let buf_path = state.tasks_dir.join(format!(".stream-deleg-{}.jsonl", id));
    let file_len = std::fs::metadata(&buf_path)
        .map(|metadata| metadata.len() as usize)
        .unwrap_or(0);
    let safe_offset = if offset <= file_len { offset } else { 0 };
    if safe_offset >= file_len {
        return json!({"events": [], "offset": safe_offset, "byte_offset": safe_offset, "done": false});
    }
    let mut file = match std::fs::File::open(&buf_path) {
        Ok(file) => file,
        Err(_) => {
            return json!({"events": [], "offset": safe_offset, "byte_offset": safe_offset, "done": false})
        }
    };
    if file.seek(SeekFrom::Start(safe_offset as u64)).is_err() {
        return json!({"events": [], "offset": safe_offset, "byte_offset": safe_offset, "done": false});
    }
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return json!({"events": [], "offset": safe_offset, "byte_offset": safe_offset, "done": false});
    }
    let Some(last_newline) = content.rfind('\n') else {
        return json!({"events": [], "offset": safe_offset, "byte_offset": safe_offset, "done": false});
    };
    let next_offset = safe_offset + last_newline + 1;
    let mut events: Vec<Value> = Vec::new();
    let mut done = false;

    for line in content[..last_newline].lines() {
        if let Ok(evt) = serde_json::from_str::<Value>(line) {
            if evt.get("type").and_then(|t| t.as_str()) == Some("done") {
                done = true;
            }
            events.push(evt);
        }
    }

    json!({"events": events, "offset": next_offset, "byte_offset": next_offset, "done": done})
}

/// Context rotation: check if project has 3+ consecutive failed delegations.
/// If so, drop --continue to start a fresh Claude session (prevents gutter problem).
fn should_fresh_session(state: &AppState, project_dir: &Path) -> bool {
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let delegations = match state.delegations.lock() {
        Ok(d) => d,
        Err(e) => e.into_inner(),
    };
    let mut consecutive_fails = 0u32;
    // Check most recent delegations for this project
    let mut project_delgs: Vec<&crate::state::Delegation> = delegations
        .values()
        .filter(|d| d.project == project_name)
        .collect();
    project_delgs.sort_by(|a, b| b.ts.cmp(&a.ts)); // newest first
    for d in project_delgs.iter().take(5) {
        if d.status == crate::commands::status::DelegationStatus::Failed {
            consecutive_fails += 1;
        } else {
            break;
        }
    }
    if consecutive_fails >= 3 {
        crate::log_warn!(
            "[deleg-stream] context rotation: {} has {} consecutive fails — fresh session",
            project_name,
            consecutive_fails
        );
        true
    } else {
        false
    }
}
