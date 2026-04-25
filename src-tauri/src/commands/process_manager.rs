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
    let tasks_file = state.root.join("tasks").join(".running-tasks.json");
    if let Ok(acts) = state.activities.lock() {
        let map: serde_json::Map<String, Value> =
            acts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let _ = super::claude_runner::atomic_write(
            &tasks_file,
            &serde_json::to_string(&Value::Object(map)).unwrap_or_default(),
        );
    }
}

pub fn clear_activity(state: &AppState, project: &str) {
    if let Ok(mut acts) = state.activities.lock() {
        acts.remove(project);
    }
    let tasks_file = state.root.join("tasks").join(".running-tasks.json");
    if let Ok(acts) = state.activities.lock() {
        let map: serde_json::Map<String, Value> =
            acts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let _ = super::claude_runner::atomic_write(
            &tasks_file,
            &serde_json::to_string(&Value::Object(map)).unwrap_or_default(),
        );
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

pub fn kill_existing(state: &AppState, chat_key: &str) {
    if let Ok(mut pids) = state.running_pids.lock() {
        if let Some(pid) = pids.remove(chat_key) {
            #[cfg(target_os = "windows")]
            {
                let _ = super::claude_runner::silent_cmd("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = super::claude_runner::silent_cmd("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
        }
    }
}

pub fn kill_all_tracked(state: &AppState) {
    if let Ok(mut pids) = state.running_pids.lock() {
        for (_, pid) in pids.drain() {
            #[cfg(target_os = "windows")]
            {
                let _ = super::claude_runner::silent_cmd("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = super::claude_runner::silent_cmd("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
        }
    }
}
