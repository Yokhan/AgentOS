//! Git & cross-project operations: bulk push/pull, status, stale branches, search, audits.

use crate::state::AppState;
use super::claude_runner::{silent_cmd, safe_truncate};
use std::path::Path;

/// GIT_BULK_PUSH: push all non-client projects
pub fn bulk_push(state: &AppState, filter: &str) -> Option<String> {
    let projects = get_filtered_projects(state, filter);
    let mut results = Vec::new();
    let mut pushed = 0;
    let mut skipped = 0;

    for (name, path) in &projects {
        // Check if ahead of remote
        let status = git_short(path, &["status", "--porcelain", "-b"]);
        if !status.contains("ahead") {
            skipped += 1;
            continue;
        }
        let output = silent_cmd("git").args(["-C", &path.to_string_lossy(), "push"]).output();
        match output {
            Ok(o) if o.status.success() => { pushed += 1; results.push(format!("  ✓ {} pushed", name)); }
            Ok(o) => { results.push(format!("  ✗ {} failed: {}", name, String::from_utf8_lossy(&o.stderr).chars().take(80).collect::<String>())); }
            Err(e) => { results.push(format!("  ✗ {} error: {}", name, e)); }
        }
    }
    Some(format!("**Git Push:** {} pushed, {} skipped (up-to-date)\n{}", pushed, skipped, results.join("\n")))
}

/// GIT_BULK_PULL: pull latest across all projects
pub fn bulk_pull(state: &AppState, filter: &str) -> Option<String> {
    let projects = get_filtered_projects(state, filter);
    let mut results = Vec::new();
    let mut updated = 0;
    let mut current = 0;

    for (name, path) in &projects {
        let output = silent_cmd("git").args(["-C", &path.to_string_lossy(), "pull", "--ff-only"]).output();
        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                if text.contains("Already up to date") { current += 1; }
                else { updated += 1; results.push(format!("  ✓ {} updated", name)); }
            }
            Ok(o) => { results.push(format!("  ⚠ {} non-ff: {}", name, String::from_utf8_lossy(&o.stderr).chars().take(60).collect::<String>())); }
            Err(e) => { results.push(format!("  ✗ {} error: {}", name, e)); }
        }
    }
    Some(format!("**Git Pull:** {} updated, {} current\n{}", updated, current, results.join("\n")))
}

/// GIT_STATUS_ALL: branch + dirty status across all repos
pub fn status_all(state: &AppState) -> Option<String> {
    let projects = get_all_projects(state);
    let mut lines = Vec::new();

    for (name, path) in &projects {
        let branch = git_short(path, &["branch", "--show-current"]);
        let status = git_short(path, &["status", "--porcelain", "-unormal"]);
        let dirty = status.lines().count();
        let ahead = if git_short(path, &["status", "-b", "--porcelain"]).contains("ahead") { " ↑" } else { "" };
        let behind = if git_short(path, &["status", "-b", "--porcelain"]).contains("behind") { " ↓" } else { "" };
        lines.push(format!("  {} [{}{}{}] {}",
            name, branch, ahead, behind,
            if dirty > 0 { format!("{} dirty", dirty) } else { "clean".to_string() }
        ));
    }
    Some(format!("**Git Status ({}):**\n{}", lines.len(), lines.join("\n")))
}

/// GIT_STALE_BRANCHES: find old branches
pub fn stale_branches(state: &AppState, days: u64) -> Option<String> {
    let projects = get_all_projects(state);
    let mut lines = Vec::new();
    let threshold = chrono::Utc::now() - chrono::Duration::days(days as i64);

    for (name, path) in &projects {
        let output = silent_cmd("git")
            .args(["-C", &path.to_string_lossy(), "for-each-ref", "--format=%(refname:short) %(committerdate:iso)", "refs/heads/"])
            .output().ok();
        let text = output.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        for line in text.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() < 2 { continue; }
            let branch = parts[0];
            if branch == "main" || branch == "master" { continue; }
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(parts[1].trim())
                .or_else(|_| chrono::DateTime::parse_from_str(parts[1].trim(), "%Y-%m-%d %H:%M:%S %z")) {
                if dt < threshold {
                    lines.push(format!("  {} / {} ({})", name, branch, parts[1].trim().chars().take(10).collect::<String>()));
                }
            }
        }
    }
    if lines.is_empty() { return Some(format!("No stale branches (>{} days).", days)); }
    Some(format!("**Stale Branches ({}, >{} days):**\n{}", lines.len(), days, lines.join("\n")))
}

/// GIT_SEARCH: search commits or code across all repos
pub fn git_search(state: &AppState, mode: &str, query: &str) -> Option<String> {
    let projects = get_all_projects(state);
    let mut results = Vec::new();

    for (name, path) in &projects {
        let output = match mode {
            "commit" => silent_cmd("git").args(["-C", &path.to_string_lossy(), "log", "--oneline", "--all", "-20", "--grep", query]).output(),
            "code" => silent_cmd("git").args(["-C", &path.to_string_lossy(), "grep", "-n", "--max-count=5", query]).output(),
            "file" => silent_cmd("git").args(["-C", &path.to_string_lossy(), "ls-files", query]).output(),
            _ => continue,
        };
        if let Ok(o) = output {
            let text = String::from_utf8_lossy(&o.stdout);
            let text_trimmed = text.trim();
            if !text_trimmed.is_empty() {
                let lines: String = text_trimmed.lines().take(5).map(|l| format!("    {}", safe_truncate(l, 100))).collect::<Vec<_>>().join("\n");
                results.push(format!("  **{}:**\n{}", name, lines));
            }
        }
        if results.len() >= 50 { break; }
    }
    if results.is_empty() { return Some(format!("No results for '{}' ({}).", query, mode)); }
    Some(format!("**Search '{}' ({}):**\n{}", query, mode, results.join("\n")))
}

/// TEMPLATE_AUDIT: show template version per project
pub fn template_audit(state: &AppState) -> Option<String> {
    let projects = get_all_projects(state);
    let mut lines = Vec::new();

    for (name, path) in &projects {
        let manifest = path.join(".template-manifest.json");
        let ver = if manifest.exists() {
            std::fs::read_to_string(&manifest).ok()
                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                .and_then(|v| v.get("template_version").and_then(|v| v.as_str()).map(String::from))
                .unwrap_or_else(|| "?".to_string())
        } else { "none".to_string() };
        lines.push(format!("  {} → {}", name, ver));
    }
    Some(format!("**Template Versions ({}):**\n{}", lines.len(), lines.join("\n")))
}

/// DEPENDENCY_AUDIT: check for outdated/vulnerable deps
pub fn dependency_audit(state: &AppState, filter: &str) -> Option<String> {
    let projects = if filter.is_empty() { get_all_projects(state) }
    else { vec![(filter.to_string(), state.docs_dir.join(filter))] };
    let mut results = Vec::new();

    for (name, path) in &projects {
        // Detect package manager and run audit
        let (cmd, args_vec) = if path.join("package.json").exists() {
            ("npm", vec!["audit", "--json"])
        } else if path.join("Cargo.toml").exists() {
            ("cargo", vec!["audit", "--json"])
        } else if path.join("requirements.txt").exists() {
            ("pip", vec!["audit"])
        } else { continue };

        let output = silent_cmd(cmd).args(&args_vec).current_dir(path).output().ok();
        let text = output.map(|o| String::from_utf8_lossy(&o.stdout).chars().take(200).collect::<String>()).unwrap_or_else(|| "audit unavailable".to_string());
        results.push(format!("  **{}** ({}): {}", name, cmd, text.lines().next().unwrap_or("ok")));
    }

    if results.is_empty() { return Some("No projects with supported package managers.".to_string()); }
    Some(format!("**Dependency Audit ({}):**\n{}", results.len(), results.join("\n")))
}

fn git_short(path: &Path, args: &[&str]) -> String {
    // Build git command with project path
    let path_str = path.to_string_lossy().to_string();
    let output = silent_cmd("git").args(["-C", &path_str]).args(args).output();
    output.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn get_all_projects(state: &AppState) -> Vec<(String, std::path::PathBuf)> {
    std::fs::read_dir(&state.docs_dir).ok()
        .map(|entries| entries.flatten()
            .filter(|e| e.path().join(".git").exists())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                Some((name, e.path()))
            })
            .collect())
        .unwrap_or_default()
}

fn get_filtered_projects(state: &AppState, filter: &str) -> Vec<(String, std::path::PathBuf)> {
    let all = get_all_projects(state);
    if filter.is_empty() { return all; }

    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(projects_in_cat) = segments.get(filter) {
        return all.into_iter().filter(|(name, _)| projects_in_cat.contains(name)).collect();
    }
    // Filter by comma-separated names
    let names: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
    all.into_iter().filter(|(name, _)| names.iter().any(|n| n.eq_ignore_ascii_case(name))).collect()
}
