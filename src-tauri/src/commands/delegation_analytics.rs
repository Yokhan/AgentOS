//! Delegation analytics and logging — extracted from delegation.rs for file size.

use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_analytics(state: State<Arc<AppState>>) -> Value {
    let log_path = state.root.join("tasks").join(".delegation-log.jsonl");
    if !log_path.exists() {
        return json!({"total": 0, "by_project": {}, "by_status": {}, "patterns": []});
    }

    let mut entries = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&log_path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                entries.push(entry);
            }
        }
    }

    let mut by_project: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut by_status: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    for e in &entries {
        let p = e
            .get("project")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let s = e
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        *by_status.entry(s.to_string()).or_insert(0) += 1;

        let proj = by_project
            .entry(p.to_string())
            .or_insert_with(|| json!({"total": 0, "complete": 0, "failed": 0}));
        if let Some(obj) = proj.as_object_mut() {
            let total = obj.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            obj.insert("total".to_string(), json!(total + 1));
            if s == "success" || s == "complete" {
                let count = obj.get("complete").and_then(|v| v.as_u64()).unwrap_or(0);
                obj.insert("complete".to_string(), json!(count + 1));
            }
            if s == "error" || s == "failed" {
                let count = obj.get("failed").and_then(|v| v.as_u64()).unwrap_or(0);
                obj.insert("failed".to_string(), json!(count + 1));
            }
        }
    }

    json!({
        "total": entries.len(),
        "by_project": by_project,
        "by_status": by_status,
        "patterns": [],
    })
}

/// Get delegation log filtered by project/status/time
pub fn get_delegation_log(state: &AppState, filter: &str) -> Option<String> {
    let log_path = state.root.join("tasks").join(".delegation-log.jsonl");
    let content = std::fs::read_to_string(&log_path).ok()?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut lines = Vec::new();
    for line in content.lines().rev().take(100) {
        if let Ok(e) = serde_json::from_str::<Value>(line) {
            let project = e.get("project").and_then(|v| v.as_str()).unwrap_or("");
            let status = e.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("");
            let task: String = e
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .chars()
                .take(60)
                .collect();

            let matches = filter.is_empty()
                || filter.eq_ignore_ascii_case(project)
                || (filter == "?today" && ts.starts_with(&today))
                || (filter == "?failed" && (status == "failed" || status == "error"));
            if matches {
                lines.push(format!(
                    "  {} {} [{}] {}",
                    ts.chars().take(16).collect::<String>(),
                    project,
                    status,
                    task
                ));
            }
        }
    }

    if lines.is_empty() {
        return Some("No matching log entries.".to_string());
    }
    Some(format!(
        "**Delegation Log ({} entries):**\n{}",
        lines.len(),
        lines.join("\n")
    ))
}

/// Archive old terminal delegations
pub fn cleanup_delegations(state: &AppState, hours: u64) -> Option<String> {
    let archive_path = state.root.join("tasks").join(".delegation-archive.jsonl");
    let now = chrono::Utc::now();
    let mut archived = 0;

    if let Ok(mut delegations) = state.delegations.lock() {
        let to_remove: Vec<String> = delegations
            .iter()
            .filter(|(_, d)| d.status.is_terminal())
            .filter(|(_, d)| {
                chrono::DateTime::parse_from_rfc3339(&d.ts)
                    .or_else(|_| chrono::DateTime::parse_from_str(&d.ts, "%Y-%m-%dT%H:%M:%SZ"))
                    .map(|dt| now.signed_duration_since(dt).num_hours() as u64 >= hours)
                    .unwrap_or(true)
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in &to_remove {
            if let Some(d) = delegations.remove(key) {
                let entry = serde_json::to_value(&d).unwrap_or_default();
                super::jsonl::append_jsonl_logged(&archive_path, &entry, "delegation archive");
                archived += 1;
            }
        }
    }
    state.save_delegations();
    crate::log_info!(
        "[deleg_ext] cleanup: archived {} delegations older than {}h",
        archived,
        hours
    );
    Some(format!(
        "**Cleanup:** archived {} delegations (older than {}h)",
        archived, hours
    ))
}

/// Aggregate git diffs from matching delegations
pub fn aggregate_diffs(state: &AppState, filter: &str) -> Option<String> {
    let delegations = state.delegations.lock().ok()?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut diffs = Vec::new();

    for (id, d) in delegations.iter() {
        if d.git_diff.is_none() {
            continue;
        }
        let matches = filter.is_empty()
            || d.batch_id.as_deref() == Some(filter)
            || (filter == "?today" && d.ts.starts_with(&today))
            || id == filter;
        if matches {
            diffs.push(format!(
                "**{}** ({}):\n{}",
                d.project,
                id.chars().take(12).collect::<String>(),
                d.git_diff.as_deref().unwrap_or("")
            ));
        }
    }

    if diffs.is_empty() {
        return Some("No diffs found.".to_string());
    }
    Some(format!(
        "**Diffs ({}):**\n\n{}",
        diffs.len(),
        diffs.join("\n\n---\n\n")
    ))
}

pub fn log_delegation(root: &std::path::Path, project: &str, task: &str, status: &str) {
    let log_path = root.join("tasks").join(".delegation-log.jsonl");
    let entry = json!({
        "ts": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "project": project,
        "task": task.chars().take(300).collect::<String>(),
        "status": status,
    });
    super::jsonl::append_jsonl_logged(&log_path, &entry, "delegation log");
}
