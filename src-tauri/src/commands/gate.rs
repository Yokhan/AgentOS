//! Gate Pipeline: post-delegation verification (verify script + diff analysis + cost check).
//! Runs automatically after each delegation completes, before marking Done.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateResult {
    pub status: GateStatus,
    pub verify_output: Option<String>,
    pub diff_stats: Option<DiffStats>,
    pub cost: Option<CostInfo>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus { Pass, Warn, Fail }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostInfo {
    pub tokens: u32,
    pub cost_usd: f64,
    pub duration_s: u32,
}

/// Run gate pipeline for a completed delegation.
/// Returns GateResult with pass/warn/fail + details.
pub fn run_gate(state: &AppState, project: &str, delegation_id: &str) -> GateResult {
    let project_dir = state.docs_dir.join(project);
    let mut errors = Vec::new();
    let mut gate_status = GateStatus::Pass;

    // Step 1: Run verify script (cargo check / npm test / etc.)
    let verify_output = run_verify_script(&project_dir);
    if let Some(ref output) = verify_output {
        // Primary: exit code (EXIT_FAIL prefix). Secondary: content heuristics (#7)
        if output.starts_with("EXIT_FAIL") {
            gate_status = GateStatus::Fail;
            errors.push(format!("Verify failed: {}", super::claude_runner::safe_truncate(output, 200)));
        } else if output.contains("warning:") || output.contains("Warning:") {
            // Only match "warning:" with colon — avoids false positives on comments
            if gate_status == GateStatus::Pass { gate_status = GateStatus::Warn; }
            errors.push(format!("Verify warnings: {}", super::claude_runner::safe_truncate(output, 200)));
        }
    }

    // Step 2: Git diff stats
    let diff_stats = get_diff_stats(&project_dir);

    // Step 3: Cost check from usage log
    let cost = get_delegation_cost(state, delegation_id);
    if let Some(ref c) = cost {
        if c.cost_usd > 1.0 {
            if gate_status == GateStatus::Pass { gate_status = GateStatus::Warn; }
            errors.push(format!("High cost: ${:.4} ({} tokens)", c.cost_usd, c.tokens));
        }
    }

    crate::log_info!("[gate] {} delegation {}: {:?} ({} errors)", project, delegation_id, gate_status, errors.len());

    GateResult { status: gate_status, verify_output, diff_stats, cost, errors }
}

/// Detect project type and run appropriate verify command.
fn run_verify_script(project_dir: &Path) -> Option<String> {
    if !project_dir.exists() { return None; }

    let cmd = if project_dir.join("Cargo.toml").exists() {
        vec!["cargo", "check", "--message-format=short"]
    } else if project_dir.join("package.json").exists() {
        // Try npm test, fallback to npm run build
        if let Ok(pkg) = std::fs::read_to_string(project_dir.join("package.json")) {
            if pkg.contains("\"test\"") { vec!["npm", "test", "--", "--passWithNoTests"] }
            else if pkg.contains("\"build\"") { vec!["npm", "run", "build"] }
            else { return None; }
        } else { return None; }
    } else if project_dir.join("requirements.txt").exists() || project_dir.join("pyproject.toml").exists() {
        vec!["python", "-m", "py_compile"]  // basic syntax check
    } else {
        return None;
    };

    let output = super::claude_runner::silent_cmd(cmd[0])
        .args(&cmd[1..])
        .current_dir(project_dir)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = format!("{}{}", stdout, stderr);
            // Use exit code as primary signal (#7), not string matching
            let prefix = if !o.status.success() { "EXIT_FAIL: " } else { "" };
            Some(format!("{}{}", prefix, super::claude_runner::safe_truncate(&combined, 1000)))
        }
        Err(e) => Some(format!("EXIT_FAIL: verify error: {}", e)),
    }
}

/// Get git diff --stat for uncommitted changes.
fn get_diff_stats(project_dir: &Path) -> Option<DiffStats> {
    let output = super::claude_runner::silent_cmd("git")
        .args(["diff", "--stat", "--shortstat"])
        .current_dir(project_dir)
        .output().ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse "3 files changed, 15 insertions(+), 7 deletions(-)"
    let mut stats = DiffStats { files_changed: 0, insertions: 0, deletions: 0 };
    for part in text.split(',') {
        let part = part.trim();
        if part.contains("file") {
            stats.files_changed = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
        } else if part.contains("insertion") {
            stats.insertions = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
        } else if part.contains("deletion") {
            stats.deletions = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
        }
    }
    if stats.files_changed > 0 { Some(stats) } else { None }
}

/// Lookup delegation cost from usage log.
fn get_delegation_cost(state: &AppState, delegation_id: &str) -> Option<CostInfo> {
    let usage_path = state.root.join("tasks").join(".usage-log.jsonl");
    let content = std::fs::read_to_string(&usage_path).ok()?;
    for line in content.lines().rev() {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            if entry.get("delegation_id").and_then(|d| d.as_str()) == Some(delegation_id) {
                return Some(CostInfo {
                    tokens: entry.get("total_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32,
                    cost_usd: entry.get("cost_usd").and_then(|c| c.as_f64()).unwrap_or(0.0),
                    duration_s: entry.get("duration_ms").and_then(|d| d.as_u64()).unwrap_or(0) as u32 / 1000,
                });
            }
        }
    }
    None
}

/// Build gate context for PA prompt (recent gate results).
pub fn build_gate_context(state: &AppState) -> String {
    // Read recent gate results from delegations
    let delegations = match state.delegations.lock() {
        Ok(d) => d,
        Err(e) => e.into_inner(),
    };

    let mut lines = Vec::new();
    for (id, d) in delegations.iter() {
        if let Some(ref gate) = d.gate_result {
            let status_icon = match gate.status {
                GateStatus::Pass => "✓",
                GateStatus::Warn => "⚠",
                GateStatus::Fail => "✗",
            };
            let cost_str = gate.cost.as_ref().map(|c| format!(" ${:.4}", c.cost_usd)).unwrap_or_default();
            let diff_str = gate.diff_stats.as_ref().map(|d| format!(" +{}/-{}", d.insertions, d.deletions)).unwrap_or_default();
            lines.push(format!("  {} {} [{}]{}{}", status_icon, d.project, id.chars().take(8).collect::<String>(), cost_str, diff_str));
            for err in &gate.errors {
                lines.push(format!("    → {}", err));
            }
        }
    }

    if lines.is_empty() { return String::new(); }
    format!("[GATE RESULTS]\n{}\n[END GATE]", lines.join("\n"))
}
