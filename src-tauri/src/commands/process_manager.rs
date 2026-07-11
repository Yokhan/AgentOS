//! Process management: activity tracking, PID tracking, process killing.

use crate::state::AppState;
use serde_json::{json, Value};

/// Save a running task indicator (in-memory via AppState, no file races)
pub fn set_activity(state: &AppState, project: &str, action: &str, detail: &str) {
    if let Ok(mut acts) = state.activities.lock() {
        acts.insert(
            project.to_string(),
            json!({
                "action": action,
                "detail": detail,
                "started": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0),
            }),
        );
    }
    persist_activities(state);
}

pub fn clear_activity(state: &AppState, project: &str) {
    if let Ok(mut acts) = state.activities.lock() {
        acts.remove(project);
    }
    persist_activities(state);
}

pub(crate) fn persist_activities(state: &AppState) {
    let snapshot = state
        .activities
        .lock()
        .map(|activities| {
            Value::Object(
                activities
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )
        })
        .unwrap_or_else(|error| {
            crate::log_error!("[activity] lock poisoned: {}", error);
            json!({})
        });
    let tasks_file = state.tasks_dir.join(".running-tasks.json");
    let result = serde_json::to_string(&snapshot)
        .map_err(|error| error.to_string())
        .and_then(|content| {
            super::claude_runner::atomic_write(&tasks_file, &content)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        crate::log_error!("[activity] persistence failed: {}", error);
    }
}

pub fn track_pid(state: &AppState, chat_key: &str, pid: u32) {
    if let Ok(mut pids) = state.running_pids.lock() {
        pids.insert(chat_key.to_string(), pid);
    }
}

pub fn untrack_pid(state: &AppState, chat_key: &str) {
    if let Ok(mut pids) = state.running_pids.lock() {
        pids.remove(chat_key);
    }
}

pub fn untrack_pid_if_match(state: &AppState, chat_key: &str, pid: u32) {
    if let Ok(mut pids) = state.running_pids.lock() {
        if pids.get(chat_key).copied() == Some(pid) {
            pids.remove(chat_key);
        }
    }
}

pub fn clear_cancel(state: &AppState, chat_key: &str) {
    if let Ok(mut cancellations) = state.chat_cancellations.lock() {
        cancellations.remove(chat_key);
    }
}

pub fn request_cancel(state: &AppState, chat_key: &str) {
    if let Ok(mut cancellations) = state.chat_cancellations.lock() {
        cancellations.insert(chat_key.to_string());
    }
}

pub fn is_cancelled(state: &AppState, chat_key: &str) -> bool {
    state
        .chat_cancellations
        .lock()
        .map(|cancellations| cancellations.contains(chat_key))
        .unwrap_or(false)
}

pub fn kill_existing(state: &AppState, chat_key: &str) -> Option<u32> {
    let pid = state
        .running_pids
        .lock()
        .ok()
        .and_then(|mut pids| pids.remove(chat_key));
    if let Some(pid) = pid {
        kill_pid_tree(pid);
    }
    pid
}

pub fn kill_all_tracked(state: &AppState) {
    let pids: Vec<u32> = state
        .running_pids
        .lock()
        .map(|mut tracked| tracked.drain().map(|(_, pid)| pid).collect())
        .unwrap_or_default();
    for pid in pids {
        kill_pid_tree(pid);
    }
}

pub fn register_background_task(
    state: &AppState,
    key: String,
    handle: std::thread::JoinHandle<()>,
) {
    reap_background_tasks(state);
    let replaced = state
        .background_tasks
        .lock()
        .ok()
        .and_then(|mut tasks| tasks.insert(key.clone(), handle));
    if let Some(old) = replaced {
        crate::log_warn!("[runtime] replaced background task registry key={}", key);
        if old.is_finished() {
            let _ = old.join();
        }
    }
}

pub fn spawn_managed<F>(state: &std::sync::Arc<AppState>, label: &str, task: F)
where
    F: FnOnce() + Send + 'static,
{
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let handle = std::thread::spawn(task);
    register_background_task(state, format!("{label}:{suffix}"), handle);
}

pub fn reap_background_tasks(state: &AppState) {
    let completed = state
        .background_tasks
        .lock()
        .map(|mut tasks| {
            let keys: Vec<String> = tasks
                .iter()
                .filter(|(_, handle)| handle.is_finished())
                .map(|(key, _)| key.clone())
                .collect();
            keys.into_iter()
                .filter_map(|key| tasks.remove(&key).map(|handle| (key, handle)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (key, handle) in completed {
        if handle.join().is_err() {
            crate::log_error!("[runtime] background task panicked key={}", key);
        }
    }
}

pub fn shutdown_background_tasks(state: &AppState, timeout: std::time::Duration) {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        reap_background_tasks(state);
        let remaining = state
            .background_tasks
            .lock()
            .map(|tasks| tasks.len())
            .unwrap_or(0);
        if remaining == 0 || std::time::Instant::now() >= deadline {
            if remaining > 0 {
                crate::log_warn!(
                    "[runtime] {} background task(s) exceeded shutdown deadline",
                    remaining
                );
            }
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn kill_pid_tree(pid: u32) {
    #[cfg(target_os = "windows")]
    let result = super::claude_runner::silent_cmd("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .output();
    #[cfg(not(target_os = "windows"))]
    let result = super::claude_runner::silent_cmd("kill")
        .args(["-9", &pid.to_string()])
        .output();
    if let Err(error) = result {
        crate::log_warn!("[process] failed to kill pid={}: {}", pid, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_task_registry_reaps_completed_workers() {
        let root = std::env::temp_dir().join(format!(
            "agentos-runtime-registry-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("n8n")).expect("create runtime root");
        std::fs::write(root.join("n8n").join("config.json"), "{}").expect("seed config");
        let state = AppState::new(root.clone());
        let handle = std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(20));
        });
        register_background_task(&state, "test-worker".to_string(), handle);
        shutdown_background_tasks(&state, std::time::Duration::from_secs(1));
        assert!(state.background_tasks.lock().expect("registry").is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
