//! Delegation streaming: real-time progress for delegation execution.
//! Reuses the stream buffer pattern from chat_stream.rs.

use crate::state::AppState;
use serde_json::{json, Value};
use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;
use tauri::State;

/// Run claude with streaming output for a delegation step.
/// Writes events to stream_buf as JSONL. Returns (response_text, is_permission_request).
pub fn run_delegation_streaming(
    state: &AppState,
    project_dir: &Path,
    task: &str,
    perm_path: &str,
    model: Option<&str>,
    effort: Option<&str>,
    stream_buf: &Path,
) -> (String, bool) {
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
    if use_continue { cmd.args(["--continue"]); }
    cmd.args(["-p", "--output-format", "stream-json", "--verbose", "--settings", perm_path]);

    if let Some(m) = model {
        if !m.is_empty() { cmd.args(["--model", m]); }
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

    // Heartbeat fix (#1): spawn watchdog thread that kills child if no events for 120s
    let child_pid = child.id();
    let heartbeat_flag = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
    ));
    let hb = heartbeat_flag.clone();
    let stream_buf_hb = stream_buf.to_path_buf();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(15));
            let last = hb.load(std::sync::atomic::Ordering::Relaxed);
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            if now - last > heartbeat_timeout_secs {
                crate::log_warn!("[deleg-stream] HEARTBEAT: no events for {}s, killing pid {}", now - last, child_pid);
                // Append safety event
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&stream_buf_hb) {
                    use std::io::Write;
                    let _ = writeln!(f, r#"{{"type":"safety","reason":"heartbeat_timeout"}}"#);
                }
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("taskkill").args(["/F", "/PID", &child_pid.to_string()]).output(); }
                #[cfg(not(target_os = "windows"))]
                { let _ = std::process::Command::new("kill").args(["-9", &child_pid.to_string()]).output(); }
                break;
            }
            if last == 0 { break; } // sentinel: reader finished normally
        }
    });

    let reader = std::io::BufReader::new(stdout);
    if let Ok(mut buf_file) = std::fs::OpenOptions::new().create(true).append(true).open(stream_buf) {
        use std::io::Write;

        let write_evt = |f: &mut std::fs::File, v: &Value| {
            let _ = writeln!(f, "{}", serde_json::to_string(v).unwrap_or_default());
            let _ = f.flush();
        };

        for line in reader.lines() {
            let Ok(raw) = line else { break };
            let trimmed = raw.trim_end_matches('\r');
            if trimmed.is_empty() { continue; }
            let Ok(evt) = serde_json::from_str::<Value>(trimmed) else { continue };
            let etype = evt.get("type").and_then(|t| t.as_str()).unwrap_or("");
            event_count += 1;
            heartbeat_flag.store(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(), std::sync::atomic::Ordering::Relaxed);

            match etype {
                "stream_event" => {
                    let empty_obj = json!({});
                    let inner = evt.get("event").unwrap_or(&empty_obj);
                    let inner_type = inner.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match inner_type {
                        "content_block_delta" => {
                            if let Some(delta) = inner.get("delta") {
                                let dtype = delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if dtype == "text_delta" {
                                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                        full_text.push_str(text);
                                        // Write every 10th text delta to reduce file I/O
                                        if event_count % 10 == 0 {
                                            write_evt(&mut buf_file, &json!({"type":"text_delta","text":text}));
                                        }
                                    }
                                } else if dtype == "thinking_delta" {
                                    if let Some(text) = delta.get("thinking").and_then(|t| t.as_str()) {
                                        write_evt(&mut buf_file, &json!({"type":"thinking","text":text}));
                                    }
                                }
                            }
                        }
                        "content_block_start" => {
                            if let Some(block) = inner.get("content_block") {
                                let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if btype == "tool_use" {
                                    let tool = block.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                                    write_evt(&mut buf_file, &json!({"type":"tool_start","tool":tool}));
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
                    if let Some(content) = evt.pointer("/message/content").and_then(|c| c.as_array()) {
                        for block in content {
                            if let Some("text") = block.get("type").and_then(|t| t.as_str()) {
                                let text = block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                                if !text.is_empty() { full_text = text.to_string(); }
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
                crate::log_warn!("[deleg-stream] TOKEN BUDGET exceeded: {} > {} — killing", total_tokens, token_budget);
                write_evt(&mut buf_file, &json!({"type":"safety","reason":"token_budget","tokens":total_tokens}));
                let _ = child.kill();
                full_text = format!("Error: token budget exceeded ({} tokens). Process killed.", total_tokens);
                break;
            }
            // Heartbeat handled by watchdog thread — no in-loop check needed
        }
    }

    // Signal watchdog to stop
    heartbeat_flag.store(0, std::sync::atomic::Ordering::Relaxed);

    // Cleanup
    let _ = child.wait();
    super::process_manager::untrack_pid(state, &chat_key);
    let _ = std::fs::remove_file(&tmp);

    let is_perm = super::claude_runner::is_permission_request(&full_text);
    crate::log_info!("[deleg-stream] finished: {} events, {} chars, perm_request={}", event_count, full_text.len(), is_perm);

    (full_text, is_perm)
}

/// Emit a stage transition event to delegation stream buffer
pub fn emit_stage(stream_buf: &Path, stage: &str, label: &str) {
    super::jsonl::append_jsonl_logged(stream_buf, &json!({"type":"stage","stage":stage,"label":label}), "deleg stage");
}

/// Emit done marker to delegation stream buffer
pub fn emit_done(stream_buf: &Path, status: &str, response: &str) {
    let preview: String = response.chars().take(200).collect();
    super::jsonl::append_jsonl_logged(stream_buf, &json!({"type":"done","status":status,"response":preview}), "deleg done");
}

/// Poll delegation stream buffer — frontend calls every 500ms
#[tauri::command]
pub fn poll_delegation_stream(state: State<Arc<AppState>>, id: String, offset: usize) -> Value {
    let buf_path = state.root.join("tasks").join(format!(".stream-deleg-{}.jsonl", id));
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

/// Context rotation: check if project has 3+ consecutive failed delegations.
/// If so, drop --continue to start a fresh Claude session (prevents gutter problem).
fn should_fresh_session(state: &AppState, project_dir: &Path) -> bool {
    let project_name = project_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let delegations = match state.delegations.lock() {
        Ok(d) => d, Err(e) => e.into_inner(),
    };
    let mut consecutive_fails = 0u32;
    // Check most recent delegations for this project
    let mut project_delgs: Vec<&crate::state::Delegation> = delegations.values()
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
        crate::log_warn!("[deleg-stream] context rotation: {} has {} consecutive fails — fresh session", project_name, consecutive_fails);
        true
    } else {
        false
    }
}
