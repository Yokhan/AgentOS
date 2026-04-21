//! Shared Claude subprocess utilities.
//! Used by chat.rs, delegation.rs, strategy.rs.

use crate::state::AppState;
use serde_json::json;
use std::io::Read;
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

static CLAUDE_BIN: OnceLock<String> = OnceLock::new();

/// Create a Command that doesn't spawn a visible console window on Windows.
pub fn silent_cmd(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// Generate unique temp file path (nanos + pid, no collisions)
pub fn unique_tmp(prefix: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{}-{}-{}.txt", prefix, nanos, std::process::id()))
}

/// Run claude -p via subprocess — no shell wrapper, direct Command::new("claude")
/// Returns stdout text or error message.
/// Find claude binary — check PATH first, then common locations
pub fn find_claude() -> String {
    CLAUDE_BIN
        .get_or_init(|| {
            // Try PATH first
            if let Ok(output) = silent_cmd("claude").arg("--version").output() {
                if output.status.success() {
                    crate::log_info!("Found claude in PATH");
                    return "claude".to_string();
                }
            }
            // Common locations on Windows
            for path in &[
                dirs::data_dir().map(|d| d.join("npm").join("claude.cmd")),
                dirs::home_dir().map(|d| {
                    d.join("AppData")
                        .join("Roaming")
                        .join("npm")
                        .join("claude.cmd")
                }),
                dirs::home_dir().map(|d| d.join(".npm-global").join("bin").join("claude")),
            ] {
                if let Some(p) = path {
                    if p.exists() {
                        crate::log_info!("Found claude at {:?}", p);
                        return p.to_string_lossy().to_string();
                    }
                }
            }
            crate::log_warn!("claude not found, using fallback 'claude'");
            "claude".to_string()
        })
        .clone()
}

/// Run claude with optional model and reasoning effort
pub fn run_claude(cwd: &std::path::Path, prompt: &str, perm_path: &str) -> String {
    run_claude_with_opts(cwd, prompt, perm_path, None, None)
}

pub fn run_claude_with_opts(
    cwd: &std::path::Path,
    prompt: &str,
    perm_path: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
) -> String {
    run_claude_full(cwd, prompt, perm_path, model, reasoning_effort, true)
}

fn run_claude_full(
    cwd: &std::path::Path,
    prompt: &str,
    perm_path: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    continue_session: bool,
) -> String {
    let tmp = unique_tmp("chat");
    if std::fs::write(&tmp, prompt).is_err() {
        return "Error: could not write temp file".to_string();
    }

    let stdin_file = match std::fs::File::open(&tmp) {
        Ok(f) => f,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return format!("Error opening temp file: {}", e);
        }
    };

    let claude_bin = find_claude();
    let mut cmd = silent_cmd(&claude_bin);
    if continue_session {
        cmd.args(["--continue", "-p", "--settings", perm_path]);
    } else {
        cmd.args(["-p", "--settings", perm_path]);
    }

    if let Some(m) = model {
        if !m.is_empty() {
            cmd.args(["--model", m]);
        }
    }
    if let Some(re) = reasoning_effort {
        if !re.is_empty() && ["low", "medium", "high", "max"].contains(&re) {
            cmd.args(["--effort", re]);
        }
    }

    let mut child = match cmd
        .current_dir(cwd)
        .stdin(std::process::Stdio::from(stdin_file))
        .env("PYTHONIOENCODING", "utf-8")
        .env("LANG", "en_US.UTF-8")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return format!("Error running claude: {}", e);
        }
    };
    let stdout_handle = child.stdout.take().map(spawn_reader_thread);
    let stderr_handle = child.stderr.take().map(spawn_reader_thread);

    // Wait with 5-minute timeout
    // 45 min safety net — user controls abort via UI
    let timeout = std::time::Duration::from_secs(2700);
    let start = std::time::Instant::now();
    let mut last_log = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break, // process exited
            Ok(None) => {
                if start.elapsed() > timeout {
                    crate::log_warn!("claude subprocess timed out after 45 min, killing");
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = collect_reader(stdout_handle);
                    let _ = collect_reader(stderr_handle);
                    let _ = std::fs::remove_file(&tmp);
                    return "Error: claude timed out after 45 minutes".to_string();
                }
                // Log progress every 60 seconds
                if last_log.elapsed().as_secs() >= 60 {
                    crate::log_info!(
                        "claude still running... {}m elapsed",
                        start.elapsed().as_secs() / 60
                    );
                    last_log = std::time::Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                let _ = collect_reader(stdout_handle);
                let _ = collect_reader(stderr_handle);
                let _ = std::fs::remove_file(&tmp);
                return format!("Error waiting for claude: {}", e);
            }
        }
    }

    let _ = std::fs::remove_file(&tmp);

    let stdout = collect_reader(stdout_handle).unwrap_or_default();
    let stderr_bytes = collect_reader(stderr_handle).unwrap_or_default();

    let text = String::from_utf8(stdout)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());
    let stderr = String::from_utf8_lossy(&stderr_bytes);
    if !stderr.trim().is_empty() {
        crate::log_warn!("claude stderr: {}", safe_truncate(&stderr, 200));
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        if stderr.trim().is_empty() {
            "Agent returned empty response".to_string()
        } else {
            stderr.trim().to_string()
        }
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn spawn_reader_thread<R>(mut reader: R) -> std::thread::JoinHandle<Vec<u8>>
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf);
        buf
    })
}

pub(crate) fn collect_reader(
    handle: Option<std::thread::JoinHandle<Vec<u8>>>,
) -> Result<Vec<u8>, std::io::Error> {
    match handle {
        Some(h) => h
            .join()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "reader thread panicked")),
        None => Ok(Vec::new()),
    }
}

pub fn get_permission_path_for_profile(state: &AppState, profile: &str) -> String {
    let perms_dir = state.root.join("n8n").join("dashboard").join("permissions");
    let path = perms_dir.join(format!("{}.json", profile));
    if path.exists() {
        path.to_string_lossy().to_string()
    } else {
        perms_dir
            .join("balanced.json")
            .to_string_lossy()
            .to_string()
    }
}

/// Get permission settings path for a project
pub fn get_permission_path(state: &AppState, project: &str) -> String {
    let valid_profiles = ["restrictive", "balanced", "permissive"];
    let profile = {
        let cfg = state.config();
        let raw = cfg
            .get("project_permissions")
            .and_then(|pp| pp.get(project))
            .and_then(|v| v.as_str())
            .unwrap_or("balanced")
            .to_string();
        if valid_profiles.contains(&raw.as_str()) {
            raw
        } else {
            "balanced".to_string()
        }
    };

    get_permission_path_for_profile(state, &profile)
}

/// Get permission path for delegations — minimum "balanced" since user explicitly approved.
pub fn get_delegation_permission_path(state: &AppState, _project: &str, level: &str) -> String {
    get_permission_path_for_profile(state, level)
}

/// Check if a response looks like a permission request rather than a real result.
/// Returns false if response contains success markers (to avoid false positives).
pub fn is_permission_request(response: &str) -> bool {
    let lower = response.to_lowercase();
    // Success markers — if present, NOT a permission request
    let success = [
        "done",
        "committed",
        "created",
        "updated",
        "completed",
        "finished",
        "готово",
        "выполнено",
    ];
    if success.iter().any(|s| lower.contains(s)) && lower.len() > 100 {
        return false; // Long response with success markers = actual work done
    }
    let patterns = [
        "нужно разрешение",
        "нужно твоё одобрение",
        "нужно одобрение",
        "нужно удалить",
        "нужно вручную",
        "требуется разрешение",
        "мне нужно твоё",
        "need permission",
        "need your permission",
        "allow me to",
        "requires permission",
        "permission to run",
        "не могу выполнить",
        "cannot execute",
        "blocked by",
        "i need your approval",
        "approve this action",
        "need approval",
        "approve this",
        "grant permission",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

/// Safely truncate a string at a char boundary, never panics on multi-byte UTF-8.
pub fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Atomic write on Unix, replace-safe write on Windows.
/// Writes to a temp file first, fsyncs it, then replaces the destination.
/// On Windows, std::fs::rename cannot replace an existing file, so we fall back
/// to remove+rename and finally copy+remove if another process still races us.
pub fn atomic_write(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;

    let tmp = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);

    replace_file(&tmp, path)
}

fn replace_file(tmp: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        std::fs::rename(tmp, path)
    }

    #[cfg(target_os = "windows")]
    {
        match std::fs::rename(tmp, path) {
            Ok(()) => Ok(()),
            Err(rename_err) => {
                if path.exists() {
                    match std::fs::remove_file(path) {
                        Ok(()) => std::fs::rename(tmp, path),
                        Err(remove_err) => {
                            let copy_res = std::fs::copy(tmp, path)
                                .map(|_| ())
                                .and_then(|_| std::fs::remove_file(tmp));
                            copy_res.map_err(|copy_err| {
                                std::io::Error::new(
                                    copy_err.kind(),
                                    format!(
                                        "replace failed (rename: {}, remove: {}, copy: {})",
                                        rename_err, remove_err, copy_err
                                    ),
                                )
                            })
                        }
                    }
                } else {
                    Err(rename_err)
                }
            }
        }
    }
}

/// Log I/O errors instead of silencing them
#[allow(dead_code)]
pub fn log_io<T>(result: std::io::Result<T>, context: &str) {
    if let Err(e) = result {
        crate::log_warn!("I/O error [{}]: {}", context, e);
    }
}

/// Capture git changes after a delegation completes (recent commits + diff stat)
pub fn capture_git_changes(project_dir: &std::path::Path) -> Option<String> {
    let log_out = silent_cmd("git")
        .args(["log", "--oneline", "-3"])
        .current_dir(project_dir)
        .output()
        .ok()?;
    let stat_out = silent_cmd("git")
        .args(["diff", "--stat", "HEAD~1"])
        .current_dir(project_dir)
        .output()
        .ok()?;
    let log_t = String::from_utf8_lossy(&log_out.stdout);
    let stat_t = String::from_utf8_lossy(&stat_out.stdout);
    if log_t.trim().is_empty() {
        return None;
    }
    Some(format!(
        "Commits:\n{}\nChanges:\n{}",
        safe_truncate(log_t.trim(), 300),
        safe_truncate(stat_t.trim(), 500)
    ))
}

/// Append a chat event to .chat-history.jsonl (feeds the activity feed)
pub fn log_chat_event(root: &std::path::Path, project: &str, message: &str) {
    let path = root.join("tasks").join(".chat-history.jsonl");
    let entry = json!({
        "ts": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "project": project,
        "message": message.chars().take(80).collect::<String>(),
    });
    super::jsonl::append_jsonl_logged(&path, &entry, "chat event");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_existing_file() {
        let path = unique_tmp("atomic-write-test");
        std::fs::write(&path, "old").expect("seed file");
        atomic_write(&path, "new").expect("atomic write");
        let content = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(content, "new");
        let _ = std::fs::remove_file(path);
    }
}
