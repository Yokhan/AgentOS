//! Cron scheduling: create, list, edit, delete recurring tasks.
//! Stored in tasks/cron.json. Checked every 60s by auto_approve loop.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub name: String,
    pub schedule: String,           // cron expression or "daily", "weekly", "every 4h"
    pub task: String,               // delegation task or PA command
    pub enabled: bool,
    pub created: String,
    #[serde(default)]
    pub last_run: Option<String>,
    #[serde(default)]
    pub last_status: Option<String>,
    #[serde(default)]
    pub run_count: u32,
}

fn cron_path(state: &AppState) -> std::path::PathBuf {
    state.root.join("tasks").join("cron.json")
}

pub fn load_cron_jobs(state: &AppState) -> Vec<CronJob> {
    std::fs::read_to_string(&cron_path(state))
        .ok().and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_cron_jobs(state: &AppState, jobs: &[CronJob]) {
    let _ = std::fs::write(&cron_path(state), serde_json::to_string_pretty(jobs).unwrap_or_default());
}

/// CRON_CREATE
pub fn cron_create(state: &AppState, name: &str, schedule: &str, task: &str) -> Option<String> {
    let mut jobs = load_cron_jobs(state);
    if jobs.iter().any(|j| j.name == name) {
        return Some(format!("**Error:** cron '{}' already exists. Use CRON_EDIT.", name));
    }
    jobs.push(CronJob {
        name: name.to_string(), schedule: schedule.to_string(), task: task.to_string(),
        enabled: true, created: state.now_iso(), last_run: None, last_status: None, run_count: 0,
    });
    save_cron_jobs(state, &jobs);
    crate::log_info!("[cron] created '{}' schedule='{}'", name, schedule);
    Some(format!("**Cron created:** {} ({})", name, schedule))
}

/// CRON_LIST
pub fn cron_list(state: &AppState) -> Option<String> {
    let jobs = load_cron_jobs(state);
    if jobs.is_empty() { return Some("No scheduled tasks.".to_string()); }
    let lines: Vec<String> = jobs.iter().map(|j| {
        format!("  {} [{}] {} — last: {} ({})",
            if j.enabled { "●" } else { "○" },
            j.schedule, j.name,
            j.last_run.as_deref().unwrap_or("never"),
            j.last_status.as_deref().unwrap_or("—"))
    }).collect();
    Some(format!("**Scheduled Tasks ({}):**\n{}", lines.len(), lines.join("\n")))
}

/// CRON_EDIT
pub fn cron_edit(state: &AppState, name: &str, schedule: &str, task: &str) -> Option<String> {
    let mut jobs = load_cron_jobs(state);
    let job = jobs.iter_mut().find(|j| j.name == name)?;
    if !schedule.is_empty() { job.schedule = schedule.to_string(); }
    if !task.is_empty() { job.task = task.to_string(); }
    save_cron_jobs(state, &jobs);
    Some(format!("**Cron updated:** {}", name))
}

/// CRON_DELETE
pub fn cron_delete(state: &AppState, name: &str) -> Option<String> {
    let mut jobs = load_cron_jobs(state);
    let archive = state.root.join("tasks").join("cron-archive.json");
    let idx = jobs.iter().position(|j| j.name == name)?;
    let removed = jobs.remove(idx);
    // Archive
    let mut archived: Vec<CronJob> = std::fs::read_to_string(&archive)
        .ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default();
    archived.push(removed);
    let _ = std::fs::write(&archive, serde_json::to_string_pretty(&archived).unwrap_or_default());
    save_cron_jobs(state, &jobs);
    Some(format!("**Cron deleted:** {} (archived)", name))
}

#[tauri::command]
pub fn get_cron_jobs(state: State<Arc<AppState>>) -> Value {
    json!({"jobs": load_cron_jobs(&state)})
}
