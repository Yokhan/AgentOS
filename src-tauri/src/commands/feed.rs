use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_feed(state: State<Arc<AppState>>) -> Value {
    let history_file = state.root.join("tasks").join(".chat-history.jsonl");
    let mut feed = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&history_file) {
        let lines: Vec<&str> = content.lines().collect();
        let recent = &lines[lines.len().saturating_sub(15)..];
        for line in recent {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                feed.push(json!({
                    "type": "chat",
                    "project": entry.get("project").and_then(|v| v.as_str()).unwrap_or(""),
                    "message": entry.get("message").and_then(|v| v.as_str()).unwrap_or(""),
                    "time": entry.get("ts").and_then(|v| v.as_str()).unwrap_or(""),
                }));
            }
        }
    }

    feed.reverse();
    json!({
        "feed": feed,
        "timestamp": state.now_iso(),
    })
}

#[tauri::command]
pub fn get_activity(state: State<Arc<AppState>>) -> Value {
    // Clean stale activities (>10 min old with no running PID)
    if let Ok(mut acts) = state.activities.lock() {
        let pids = state.running_pids.lock().ok();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        acts.retain(|k, v| {
            let started = v.get("started").and_then(|s| s.as_f64()).unwrap_or(0.0);
            let age = now - started;
            if age > 600.0 {
                // 10 min
                if let Some(ref p) = pids {
                    if !p.contains_key(k) {
                        return false;
                    }
                }
            }
            true
        });
    }
    let tasks = load_tasks(&state.root);
    let delegations = match state.delegations.lock() {
        Ok(d) => {
            let active: std::collections::HashMap<String, Value> = d
                .iter()
                .filter(|(_, v)| !v.status.is_terminal())
                .map(|(k, v)| (k.clone(), serde_json::to_value(v).unwrap_or_default()))
                .collect();
            active
        }
        Err(_) => std::collections::HashMap::new(),
    };

    json!({
        "activities": tasks,
        "delegations": delegations,
    })
}

#[tauri::command]
pub fn get_health(state: State<Arc<AppState>>) -> Value {
    health_snapshot(&state)
}

pub fn health_snapshot(state: &AppState) -> Value {
    let (orch_name, pa_dir) = state.get_orch_dir();
    let agent_count = match state.scan_cache.lock() {
        Ok(c) => c
            .data
            .as_ref()
            .and_then(|d| d.get("agents"))
            .and_then(|a| a.as_array())
            .map(|a| a.len())
            .unwrap_or(0),
        Err(_) => 0,
    };

    json!({
        "status": "ok",
        "uptime": state.uptime_secs(),
        "projects": agent_count,
        "orchestrator": if pa_dir.exists() && !orch_name.is_empty() {
            Some(orch_name)
        } else {
            None
        },
        "documents_dir": state.docs_dir.to_string_lossy(),
    })
}

#[tauri::command]
pub fn get_plan(state: State<Arc<AppState>>) -> Value {
    let agents = crate::commands::agents::get_agents(state.clone());
    let agents_arr = agents
        .get("agents")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut plan = Vec::new();
    for a in &agents_arr {
        let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let uncommitted = a.get("uncommitted").and_then(|v| v.as_u64()).unwrap_or(0);
        let days = a.get("days").and_then(|v| v.as_u64()).unwrap_or(999);
        let has_blockers = a.get("blockers").and_then(|v| v.as_bool()).unwrap_or(false);
        let task = a.get("task").and_then(|v| v.as_str()).unwrap_or("");
        let lessons = a.get("lessons").and_then(|v| v.as_u64()).unwrap_or(0);
        let status = a.get("status").and_then(|v| v.as_str()).unwrap_or("");

        let mut issues = Vec::new();
        let mut priority = "LOW";

        if uncommitted > 20 {
            issues.push(format!("{} uncommitted files — needs commit", uncommitted));
            priority = "HIGH";
        } else if uncommitted > 5 {
            issues.push(format!("{} uncommitted files", uncommitted));
            if priority == "LOW" {
                priority = "MED";
            }
        }

        if days > 14 {
            issues.push(format!("no activity for {} days", days));
            if priority == "LOW" {
                priority = "MED";
            }
        }

        if has_blockers {
            issues.push("has BLOCKERS in tasks/current.md".to_string());
            priority = "HIGH";
        }

        if task.is_empty() && status != "sleeping" {
            issues.push("no active task — needs direction".to_string());
        }

        if lessons > 50 {
            issues.push(format!("{} lessons — run /weekly", lessons));
            if priority == "LOW" {
                priority = "MED";
            }
        }

        if !issues.is_empty() {
            let suggested = if issues.iter().any(|i| i.contains("commit")) {
                "Review and commit changes"
            } else if issues.iter().any(|i| i.contains("BLOCKERS")) {
                "Investigate and resolve blocker"
            } else if issues.iter().any(|i| i.contains("no activity")) {
                "Check status, update tasks/current.md"
            } else {
                "Review project"
            };

            plan.push(json!({
                "project": name,
                "priority": priority,
                "issues": issues,
                "suggested_action": suggested,
                "status": status,
            }));
        }
    }

    // Sort by priority
    plan.sort_by_key(
        |p| match p.get("priority").and_then(|v| v.as_str()).unwrap_or("LOW") {
            "HIGH" => 0,
            "MED" => 1,
            _ => 2,
        },
    );

    let high_count = plan
        .iter()
        .filter(|p| p.get("priority").and_then(|v| v.as_str()) == Some("HIGH"))
        .count();
    let total_issues: usize = plan
        .iter()
        .map(|p| {
            p.get("issues")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0)
        })
        .sum();

    json!({
        "plan": plan,
        "total_issues": total_issues,
        "high_count": high_count,
        "generated_at": state.now_iso(),
    })
}

#[tauri::command]
pub fn get_digest(state: State<Arc<AppState>>) -> Value {
    let agents = crate::commands::agents::get_agents(state.clone());
    let agents_arr = agents
        .get("agents")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let plan = get_plan(state.clone());

    let working: Vec<&str> = agents_arr
        .iter()
        .filter_map(|a| {
            if a.get("status").and_then(|v| v.as_str()) == Some("working") {
                a.get("name").and_then(|v| v.as_str())
            } else {
                None
            }
        })
        .collect();
    let blocked: Vec<&str> = agents_arr
        .iter()
        .filter_map(|a| {
            if a.get("blockers").and_then(|v| v.as_bool()) == Some(true) {
                a.get("name").and_then(|v| v.as_str())
            } else {
                None
            }
        })
        .collect();
    let stale: Vec<&str> = agents_arr
        .iter()
        .filter_map(|a| {
            if a.get("days").and_then(|v| v.as_u64()).unwrap_or(0) > 7 {
                a.get("name").and_then(|v| v.as_str())
            } else {
                None
            }
        })
        .collect();
    let dirty: Vec<(&str, u64)> = agents_arr
        .iter()
        .filter_map(|a| {
            let u = a.get("uncommitted").and_then(|v| v.as_u64()).unwrap_or(0);
            if u > 10 {
                Some((a.get("name").and_then(|v| v.as_str()).unwrap_or(""), u))
            } else {
                None
            }
        })
        .collect();

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut lines = vec![
        format!("Daily Digest — {}\n", today),
        format!(
            "Projects: {} total, {} active, {} stale\n",
            agents_arr.len(),
            working.len(),
            stale.len()
        ),
    ];

    if !blocked.is_empty() {
        lines.push("BLOCKED:".to_string());
        for a in &agents_arr {
            if a.get("blockers").and_then(|v| v.as_bool()) == Some(true) {
                let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let task = a
                    .get("task")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no task info");
                lines.push(format!(
                    "  - {}: {}",
                    name,
                    task.chars().take(60).collect::<String>()
                ));
            }
        }
    }

    if !dirty.is_empty() {
        lines.push(format!("\nDIRTY ({}):", dirty.len()));
        for (name, count) in dirty.iter().take(20) {
            lines.push(format!("  - {}: {} files", name, count));
        }
    }

    let high_count = plan.get("high_count").and_then(|v| v.as_u64()).unwrap_or(0);
    if high_count > 0 {
        lines.push(format!("\nACTION ITEMS ({} high priority):", high_count));
        if let Some(plan_arr) = plan.get("plan").and_then(|v| v.as_array()) {
            for p in plan_arr.iter().take(20) {
                if p.get("priority").and_then(|v| v.as_str()) == Some("HIGH") {
                    let proj = p.get("project").and_then(|v| v.as_str()).unwrap_or("");
                    let issues = p
                        .get("issues")
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    lines.push(format!("  [HIGH] {}: {}", proj, issues));
                }
            }
        }
    }

    json!({
        "text": lines.join("\n"),
        "stats": {
            "total": agents_arr.len(),
            "working": working.len(),
            "blocked": blocked.len(),
            "stale": stale.len(),
            "dirty": dirty.len(),
            "high_priority": high_count,
        },
        "plan": plan.get("plan").and_then(|v| v.as_array()).map(|a| &a[..a.len().min(10)]).unwrap_or(&[]),
        "generated_at": state.now_iso(),
    })
}

#[tauri::command]
pub fn get_project_plan(state: State<Arc<AppState>>, project: String) -> Value {
    let project_dir = state.docs_dir.join(&project);
    if !project_dir.exists() {
        return json!({"error": format!("Project not found: {}", project)});
    }

    let mut plan = json!({
        "project": project,
        "next_steps": [],
        "issues": [],
        "blockers": [],
        "context": {},
        "has_plan": false,
    });

    // 1. Read tasks/current.md
    let current_md = project_dir.join("tasks").join("current.md");
    if let Ok(content) = std::fs::read_to_string(&current_md) {
        let mut next_steps = Vec::new();
        let mut blockers = Vec::new();
        let mut in_next = false;
        let mut in_blockers = false;

        for line in content.lines() {
            let lower = line.to_lowercase();
            if lower.contains("next step") || line.starts_with("## Next") {
                in_next = true;
                in_blockers = false;
                continue;
            }
            if lower.contains("blocker") || line.starts_with("## Blocker") {
                in_blockers = true;
                in_next = false;
                continue;
            }
            if line.starts_with("## ") {
                in_next = false;
                in_blockers = false;
            }

            let trimmed = line.trim();
            if in_next
                && (trimmed.starts_with('-')
                    || trimmed.starts_with('*')
                    || trimmed.starts_with(|c: char| c.is_ascii_digit()))
            {
                let step = trimmed.trim_start_matches(|c: char| "-*0123456789. ".contains(c));
                if !step.is_empty() {
                    next_steps.push(step.to_string());
                }
            }
            if in_blockers && (trimmed.starts_with('-') || trimmed.starts_with('*')) {
                let blocker = trimmed.trim_start_matches(|c: char| "-* ".contains(c));
                if !blocker.is_empty() {
                    blockers.push(blocker.to_string());
                }
            }
        }

        // Extract task title
        for line in content.lines().take(5) {
            if line.starts_with("# ") {
                plan["context"]["task_title"] = json!(line[2..].trim());
                break;
            }
        }

        plan["next_steps"] = json!(next_steps);
        plan["blockers"] = json!(blockers);
    }

    // 2. Git status
    if let Ok(output) = super::claude_runner::silent_cmd("git")
        .args(["status", "--porcelain"])
        .current_dir(&project_dir)
        .output()
    {
        let dirty = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();
        let mut issues = plan["issues"].as_array().cloned().unwrap_or_default();
        if dirty > 20 {
            issues.push(json!({"priority": "HIGH", "text": format!("{} uncommitted files — review and commit", dirty)}));
        } else if dirty > 5 {
            issues.push(json!({"priority": "MED", "text": format!("{} uncommitted files", dirty)}));
        }
        plan["issues"] = json!(issues);
    }

    // 3. Recent commits
    if let Ok(output) = super::claude_runner::silent_cmd("git")
        .args(["log", "--oneline", "-3"])
        .current_dir(&project_dir)
        .output()
    {
        let commits: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.trim().to_string())
            .collect();
        plan["context"]["recent_commits"] = json!(commits);
    }

    plan["has_plan"] = json!(
        !plan["next_steps"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true)
            || !plan["issues"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true)
            || !plan["blockers"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true)
    );

    plan
}

// --- Helpers ---

fn load_tasks(root: &std::path::Path) -> serde_json::Value {
    let path = root.join("tasks").join(".running-tasks.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or(json!({}))
}
