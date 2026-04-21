use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::commands::claude_runner::silent_cmd;

static RE_BLOCKERS: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?m)^## [Bb]locker.*\n+\S").unwrap()
});

/// Scanned project info — mirrors Python scan-projects-fast.py output
#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: usize,
    pub name: String,
    pub status: String,
    pub branch: String,
    pub last_commit: String,
    pub uncommitted: u32,
    pub days: u64,
    pub template_version: String,
    pub task: String,
    pub blockers: bool,
    pub phase: String,
    pub lessons: u32,
    pub managed: bool,
    pub segment: String,
    // Extended context (Phase 2C)
    pub blocker_text: String,
    pub next_steps: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub agents: Vec<String>,
    pub has_brain: bool,
}


/// Scan a single git repository. Returns None if not a valid repo.
fn scan_repo(path: &Path, index: usize, project_segment: &std::collections::HashMap<String, String>) -> Option<ProjectInfo> {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return None;
    }

    let name = path.file_name()?.to_str()?.to_string();

    // git log -1 --format=%D|%cr|%ct
    let log_output = silent_cmd("git")
        .args(["-C", &path.to_string_lossy(), "log", "-1", "--format=%D|%cr|%ct"])
        .output()
        .ok()?;

    if !log_output.status.success() {
        return None;
    }

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    let parts: Vec<&str> = log_str.trim().split('|').collect();
    if parts.len() < 3 {
        return None;
    }

    // Extract branch
    let refs = parts[0];
    let mut branch = "unknown".to_string();
    for r in refs.split(',') {
        let r = r.trim();
        if let Some(b) = r.strip_prefix("HEAD -> ") {
            branch = b.to_string();
            break;
        }
    }

    let last_commit = parts[1].trim().to_string();
    let age_ts: u64 = parts[2].trim().parse().unwrap_or(0);

    // git status --porcelain
    let status_output = silent_cmd("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain", "-unormal"])
        .output()
        .ok();
    let uncommitted = status_output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count() as u32
        })
        .unwrap_or(0);

    // Template manifest
    let manifest_path = path.join(".template-manifest.json");
    let has_manifest = manifest_path.exists();
    let tpl_ver = if has_manifest {
        std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("template_version").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| "?".to_string())
    } else {
        "none".to_string()
    };

    // Current task from tasks/current.md
    let current_md = path.join("tasks").join("current.md");
    let (current_task, has_blockers, blocker_text, next_steps) = read_current_task(&current_md);
    let mcp_servers = read_mcp_servers(path);
    let agents = read_agents(path);
    let has_brain = path.join("brain").exists();

    // Phase from PROJECT_SPEC.md
    let phase = read_phase(&path.join("PROJECT_SPEC.md"));

    // Lessons count
    let lessons = count_lessons(&path.join("tasks").join("lessons.md"));

    // Calculate status
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = if age_ts > 0 {
        (now_ts.saturating_sub(age_ts)) / 86400
    } else {
        999
    };
    let status = if has_blockers {
        "blocked"
    } else if days <= 1 {
        "working"
    } else if days <= 7 {
        "idle"
    } else {
        "sleeping"
    };

    let segment = project_segment
        .get(&name)
        .cloned()
        .unwrap_or_else(|| {
            if has_manifest { "Other".to_string() } else { "Unmanaged".to_string() }
        });

    Some(ProjectInfo {
        id: index,
        name,
        status: status.to_string(),
        branch,
        last_commit,
        uncommitted,
        days,
        template_version: tpl_ver,
        task: current_task,
        blockers: has_blockers,
        phase,
        lessons,
        managed: has_manifest,
        blocker_text,
        next_steps,
        mcp_servers,
        agents,
        has_brain,
        segment,
    })
}

fn read_current_task(path: &Path) -> (String, bool, String, Vec<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (String::new(), false, String::new(), Vec::new()),
    };

    let mut task = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("---") {
            task = trimmed.chars().take(80).collect();
            break;
        }
    }

    let has_blockers = RE_BLOCKERS.is_match(&content);

    // Parse blocker text and next steps
    let mut blocker_text = String::new();
    let mut next_steps = Vec::new();
    let mut in_blockers = false;
    let mut in_next = false;
    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("blocker") && line.starts_with("## ") { in_blockers = true; in_next = false; continue; }
        if lower.contains("next") && line.starts_with("## ") { in_next = true; in_blockers = false; continue; }
        if line.starts_with("## ") { in_blockers = false; in_next = false; }
        let trimmed = line.trim();
        if in_blockers && (trimmed.starts_with('-') || trimmed.starts_with('*')) {
            let t = trimmed.trim_start_matches(|c: char| "-* ".contains(c));
            if !t.is_empty() { blocker_text = t.chars().take(200).collect(); }
        }
        if in_next && (trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with(|c: char| c.is_ascii_digit())) {
            let t = trimmed.trim_start_matches(|c: char| "-*0123456789. ".contains(c));
            if !t.is_empty() { next_steps.push(t.chars().take(100).collect()); }
        }
    }

    (task, has_blockers, blocker_text, next_steps)
}

fn read_phase(path: &Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("phase") && line.contains(':') {
            if let Some(val) = line.split(':').nth(1) {
                return val.trim().trim_matches(|c: char| "_[]*".contains(c)).trim().to_string();
            }
        }
    }
    String::new()
}

fn read_mcp_servers(path: &Path) -> Vec<String> {
    let mcp_path = path.join(".mcp.json");
    if let Ok(content) = std::fs::read_to_string(&mcp_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(servers) = data.get("mcpServers").and_then(|s| s.as_object()) {
                return servers.keys().cloned().collect();
            }
        }
    }
    Vec::new()
}

fn read_agents(path: &Path) -> Vec<String> {
    let agents_dir = path.join(".claude").join("agents");
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        return entries.flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") { Some(name[..name.len()-3].to_string()) } else { None }
            })
            .collect();
    }
    Vec::new()
}

fn count_lessons(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .map(|c| c.matches("### ").count() as u32)
        .unwrap_or(0)
}

/// Scan all projects in the documents directory.
pub fn scan_projects(docs_dir: &Path, project_segment: &std::collections::HashMap<String, String>) -> Vec<ProjectInfo> {
    let entries: Vec<PathBuf> = match std::fs::read_dir(docs_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir() && p.join(".git").exists())
            .collect(),
        Err(_) => return Vec::new(),
    };

    // Parallel scan using threads (matching Python's ThreadPoolExecutor)
    let segment_ref = project_segment;
    let results: Vec<Option<ProjectInfo>> = std::thread::scope(|s| {
        let handles: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(i, path)| {
                s.spawn(move || scan_repo(path, i, segment_ref))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or(None)).collect()
    });

    let mut projects: Vec<ProjectInfo> = results.into_iter().flatten().collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    // Re-index after sort
    for (i, p) in projects.iter_mut().enumerate() {
        p.id = i;
    }

    projects
}
