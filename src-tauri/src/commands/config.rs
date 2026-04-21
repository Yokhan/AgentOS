use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_permissions(state: State<Arc<AppState>>) -> Value {
    permissions_snapshot(&state)
}

pub fn permissions_snapshot(state: &AppState) -> Value {
    let perms_dir = state.root.join("n8n").join("dashboard").join("permissions");
    let mut profiles = serde_json::Map::new();

    for name in &["restrictive", "balanced", "permissive"] {
        let path = perms_dir.join(format!("{}.json", name));
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                profiles.insert(name.to_string(), data);
            }
        }
    }

    // Load project permissions from config
    let project_permissions = state
        .config()
        .get("project_permissions")
        .cloned()
        .unwrap_or(json!({}));

    json!({
        "profiles": profiles,
        "project_permissions": project_permissions,
    })
}

#[tauri::command]
pub fn set_permission(state: State<Arc<AppState>>, project: String, profile: String) -> Value {
    if project.is_empty() || !["restrictive", "balanced", "permissive"].contains(&profile.as_str())
    {
        return json!({"status": "error", "error": "Invalid project or profile"});
    }

    // Read current config
    let mut cfg = state.config();

    // Update project_permissions
    if cfg.get("project_permissions").is_none() {
        cfg["project_permissions"] = json!({});
    }
    let old = cfg
        .pointer("/project_permissions")
        .and_then(|pp| pp.get(&project))
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_string();
    cfg["project_permissions"][&project] = json!(profile);

    if let Ok(content) = serde_json::to_string_pretty(&cfg) {
        let _ = super::claude_runner::atomic_write(&state.config_path, &content);
        state.invalidate_config();
    }

    // Audit trail
    let audit_path = state.root.join("tasks").join(".audit-log.jsonl");
    let entry = json!({"ts": state.now_iso(), "action": "permission_change", "project": project, "old": old, "new": profile});
    super::jsonl::append_jsonl_logged(&audit_path, &entry, "permission audit");
    crate::log_info!("[config] permission: {} {} -> {}", project, old, profile);

    json!({"status": "ok", "project": project, "profile": profile})
}

/// Health history removed — was referencing dead state fields.
/// Use health_check command + delegation log for health tracking.
#[tauri::command]
pub fn get_health_history(_state: State<Arc<AppState>>, _project: Option<String>) -> Value {
    json!({"history": [], "monitoring_active": false})
}

#[tauri::command]
pub fn get_impact(state: State<Arc<AppState>>, project: String) -> Value {
    let mut impacts = json!({
        "project": project,
        "downstream": [],
        "upstream": [],
        "shared": [],
        "has_dependencies": false,
    });

    // Check ecosystem.md
    let ecosystem_path = state.docs_dir.join(&project).join("ecosystem.md");
    let fallback = state.root.join("ecosystem.md");
    let eco_path = if ecosystem_path.exists() {
        ecosystem_path
    } else {
        fallback
    };

    if let Ok(content) = std::fs::read_to_string(&eco_path) {
        let mut section = "";
        let mut downstream = Vec::new();
        let mut upstream = Vec::new();
        let mut shared = Vec::new();

        for line in content.lines() {
            if line.contains("Downstream") {
                section = "down";
            } else if line.contains("Upstream") {
                section = "up";
            } else if line.contains("Shared") {
                section = "shared";
            } else if line.starts_with("##") {
                section = "";
            } else if line.contains('|') && !section.is_empty() {
                let parts: Vec<&str> = line.split('|').filter(|p| !p.trim().is_empty()).collect();
                if parts.len() >= 2 && !["Project", "Resource", "---"].contains(&parts[0].trim()) {
                    let name = parts[0].trim().to_string();
                    match section {
                        "down" => downstream.push(name),
                        "up" => upstream.push(name),
                        "shared" => shared.push(name),
                        _ => {}
                    }
                }
            }
        }

        impacts["downstream"] = json!(downstream);
        impacts["upstream"] = json!(upstream);
        impacts["shared"] = json!(shared);
        impacts["has_dependencies"] =
            json!(!downstream.is_empty() || !upstream.is_empty() || !shared.is_empty());
    }

    impacts
}

#[tauri::command]
pub fn run_action(state: State<Arc<AppState>>, name: String) -> Value {
    match name.as_str() {
        "briefing" => {
            let agents = crate::commands::agents::get_agents(state.clone());
            let agents_arr = agents
                .get("agents")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let hot: Vec<&str> = agents_arr
                .iter()
                .filter_map(|a| {
                    if a.get("status").and_then(|v| v.as_str()) == Some("working") {
                        a.get("name").and_then(|v| v.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let idle_count = agents_arr
                .iter()
                .filter(|a| a.get("status").and_then(|v| v.as_str()) == Some("idle"))
                .count();
            let sleeping: Vec<&str> = agents_arr
                .iter()
                .filter_map(|a| {
                    if a.get("status").and_then(|v| v.as_str()) == Some("sleeping") {
                        a.get("name").and_then(|v| v.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let text = format!(
                "Briefing {}:\nHot: {}\nIdle: {} projects\nSleeping: {}",
                today,
                if hot.is_empty() {
                    "none".to_string()
                } else {
                    hot.join(", ")
                },
                idle_count,
                if sleeping.is_empty() {
                    "none".to_string()
                } else {
                    sleeping.join(", ")
                },
            );

            json!({"status": "complete", "text": text})
        }
        "weekly" => {
            let agents = crate::commands::agents::get_agents(state.clone());
            let agents_arr = agents
                .get("agents")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let active = agents_arr
                .iter()
                .filter(|a| a.get("status").and_then(|v| v.as_str()) != Some("sleeping"))
                .count();
            let dirty_total: u64 = agents_arr
                .iter()
                .map(|a| a.get("uncommitted").and_then(|v| v.as_u64()).unwrap_or(0))
                .sum();

            json!({
                "status": "complete",
                "total": agents_arr.len(),
                "active": active,
                "dirty_total": dirty_total,
                "text": format!("Weekly: {} projects, {} active", agents_arr.len(), active),
            })
        }
        _ => json!({"status": "error", "error": format!("Unknown action: {}", name)}),
    }
}

#[tauri::command]
pub async fn get_modules(
    state: State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    let project_path = state.docs_dir.join(&project);
    if !project_path.exists() {
        return Ok(json!({"status": "error", "error": "Project not found"}));
    }
    let script = state.root.join("scripts").join("module-status.sh");
    if !script.exists() {
        return Ok(json!({"status": "error", "error": "module-status.sh not found"}));
    }
    let root = state.root.clone();
    tokio::task::spawn_blocking(move || {
        match super::claude_runner::silent_cmd("bash").arg(&script).arg(&project_path).current_dir(&root).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let modules: Vec<Value> = stdout.lines().filter_map(|line| {
                    if !line.contains('|') { return None; }
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 5 {
                        Some(json!({"name": parts[0], "status": parts[1], "files": parts[2].parse::<u32>().unwrap_or(0), "lines": parts[3].parse::<u32>().unwrap_or(0), "issues": parts[4]}))
                    } else { None }
                }).collect();
                json!({"status": "complete", "project": project, "modules": modules})
            }
            Err(e) => json!({"status": "error", "error": e.to_string()}),
        }
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(state: State<Arc<AppState>>) -> Value {
    state.config()
}

#[tauri::command]
pub fn set_config(state: State<Arc<AppState>>, key: String, value: String) -> Value {
    let mut cfg = state.config();
    cfg[&key] = json!(value);

    match serde_json::to_string_pretty(&cfg) {
        Ok(content) => {
            if super::claude_runner::atomic_write(&state.config_path, &content).is_ok() {
                state.invalidate_config();
                json!({"status": "ok", "key": key, "value": value})
            } else {
                json!({"status": "error", "error": "Failed to write config"})
            }
        }
        Err(e) => json!({"status": "error", "error": e.to_string()}),
    }
}

#[tauri::command]
pub fn get_api_token(state: State<Arc<AppState>>) -> Value {
    json!({"token": state.api_token})
}
