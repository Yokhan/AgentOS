use crate::scanner;
use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;

const CACHE_TTL_SECS: u64 = 30;

pub fn invalidate_scan_cache(state: &AppState) {
    if let Ok(mut cache) = state.scan_cache.lock() {
        cache.data = None;
        cache.updated = None;
    }
}

/// Shared agent-fetching logic — used by both Tauri command and HTTP API
pub fn get_agents_cached(state: &AppState) -> Value {
    let mut cache = match state.scan_cache.lock() {
        Ok(c) => c,
        Err(_) => return json!({"agents": [], "error": "lock error"}),
    };

    // Return cached if fresh
    if let (Some(data), Some(updated)) = (&cache.data, &cache.updated) {
        if updated.elapsed().as_secs() < CACHE_TTL_SECS {
            return data.clone();
        }
    }

    // Scan projects
    let ps = state
        .project_segment
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut projects = scanner::scan_projects(&state.docs_dir, &ps);
    drop(ps);

    // Check chat history for recent activity
    let history_file = state.root.join("tasks").join(".chat-history.jsonl");
    if let Ok(content) = std::fs::read_to_string(&history_file) {
        let lines: Vec<&str> = content.lines().collect();
        let recent_lines = &lines[lines.len().saturating_sub(10)..];
        let now = chrono::Utc::now().timestamp() as f64;
        let mut recent_projects = std::collections::HashSet::new();

        for line in recent_lines {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                if let Some(ts) = entry.get("ts").and_then(|v| v.as_str()) {
                    if let Some(ts_slice) = ts.get(..19) {
                        if let Ok(dt) =
                            chrono::NaiveDateTime::parse_from_str(ts_slice, "%Y-%m-%dT%H:%M:%S")
                        {
                            let entry_ts = dt.and_utc().timestamp() as f64;
                            if (now - entry_ts) / 60.0 < 30.0 {
                                if let Some(proj) = entry.get("project").and_then(|v| v.as_str()) {
                                    recent_projects.insert(proj.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        for p in &mut projects {
            if recent_projects.contains(&p.name) {
                p.status = "working".to_string();
            }
        }
    }

    // Project cards must reflect live project-agent work immediately, not only git age.
    if let Ok(delegations) = state.delegations.lock() {
        let now = chrono::Utc::now();
        for p in &mut projects {
            let latest = delegations
                .values()
                .filter(|d| d.project == p.name)
                .max_by(|a, b| a.ts.cmp(&b.ts));
            let Some(d) = latest else { continue };
            match d.status {
                crate::commands::status::DelegationStatus::Running
                | crate::commands::status::DelegationStatus::Escalated
                | crate::commands::status::DelegationStatus::Verifying
                | crate::commands::status::DelegationStatus::Deciding => {
                    p.status = "working".to_string();
                    p.task = format!(
                        "delegation: {}",
                        d.task.chars().take(72).collect::<String>()
                    );
                }
                crate::commands::status::DelegationStatus::Pending
                | crate::commands::status::DelegationStatus::NeedsPermission
                | crate::commands::status::DelegationStatus::Scheduled => {
                    if p.status != "working" {
                        p.status = "pending".to_string();
                        p.task = format!(
                            "queued delegation: {}",
                            d.task.chars().take(68).collect::<String>()
                        );
                    }
                }
                crate::commands::status::DelegationStatus::Failed => {
                    p.status = "blocked".to_string();
                    p.blockers = true;
                    p.blocker_text = d
                        .response
                        .clone()
                        .unwrap_or_else(|| "Delegation failed".to_string());
                }
                crate::commands::status::DelegationStatus::Done => {
                    let is_recent = chrono::DateTime::parse_from_rfc3339(&d.ts)
                        .map(|dt| {
                            now.signed_duration_since(dt.with_timezone(&chrono::Utc))
                                .num_hours()
                                < 24
                        })
                        .unwrap_or(false);
                    if is_recent && p.status != "blocked" {
                        p.status = "working".to_string();
                        p.task = "delegation completed recently".to_string();
                    }
                }
                _ => {}
            }
        }
    }

    let result = json!({
        "agents": projects,
        "timestamp": state.now_iso(),
    });

    cache.data = Some(result.clone());
    cache.updated = Some(Instant::now());

    result
}

#[tauri::command]
pub fn get_agents(state: State<Arc<AppState>>) -> Value {
    get_agents_cached(&state)
}

#[tauri::command]
pub fn get_segments(state: State<Arc<AppState>>) -> Value {
    json!({
        "segments": *state.segments.lock().unwrap_or_else(|e| e.into_inner()),
        "project_segment": *state.project_segment.lock().unwrap_or_else(|e| e.into_inner()),
    })
}
