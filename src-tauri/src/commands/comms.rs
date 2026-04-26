//! Communication & reporting: daily report, dashboard, digest, alerts, partner updates.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// DAILY_REPORT: aggregate today's work
pub fn daily_report(state: &AppState) -> Option<String> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let delegations = state.delegations.lock().ok()?;

    let today_done: Vec<&crate::state::Delegation> = delegations
        .values()
        .filter(|d| {
            d.ts.starts_with(&today) && d.status == crate::commands::status::DelegationStatus::Done
        })
        .collect();
    let today_failed: Vec<&crate::state::Delegation> = delegations
        .values()
        .filter(|d| {
            d.ts.starts_with(&today)
                && d.status == crate::commands::status::DelegationStatus::Failed
        })
        .collect();

    let mut lines = vec![format!("**Daily Report — {}**", today)];
    lines.push(format!(
        "Delegations: {} done, {} failed",
        today_done.len(),
        today_failed.len()
    ));

    if !today_done.is_empty() {
        lines.push("Done:".to_string());
        for d in &today_done {
            lines.push(format!(
                "  ✓ {}: {}",
                d.project,
                d.task.chars().take(60).collect::<String>()
            ));
        }
    }
    if !today_failed.is_empty() {
        lines.push("Failed:".to_string());
        for d in &today_failed {
            lines.push(format!(
                "  ✗ {}: {}",
                d.project,
                d.task.chars().take(60).collect::<String>()
            ));
        }
    }

    // Strategy progress
    let strategies = super::strategy_models::load_strategies(state);
    let active: Vec<&super::strategy_models::Strategy> = strategies
        .iter()
        .filter(|s| s.status == crate::commands::status::StrategyStatus::Executing)
        .collect();
    if !active.is_empty() {
        lines.push("Active strategies:".to_string());
        for s in &active {
            let total: usize = s.plans.iter().map(|p| p.steps.len()).sum();
            let done = s
                .plans
                .iter()
                .flat_map(|p| &p.steps)
                .filter(|st| st.status == crate::commands::status::StepStatus::Done)
                .count();
            lines.push(format!("  {} ({}/{})", s.title, done, total));
        }
    }

    Some(lines.join("\n"))
}

/// DASHBOARD_FULL: comprehensive status
pub fn dashboard_full(state: &AppState) -> Option<String> {
    let mut sections = Vec::new();

    // Delegations summary
    if let Ok(d) = state.delegations.lock() {
        let pending = d
            .values()
            .filter(|d| d.status == crate::commands::status::DelegationStatus::Pending)
            .count();
        let running = d
            .values()
            .filter(|d| d.status == crate::commands::status::DelegationStatus::Running)
            .count();
        sections.push(format!(
            "**Delegations:** {} pending, {} running, {} total",
            pending,
            running,
            d.len()
        ));
    }

    // Strategies
    let strategies = super::strategy_models::load_strategies(state);
    let active_strats = strategies
        .iter()
        .filter(|s| s.status != crate::commands::status::StrategyStatus::Done)
        .count();
    sections.push(format!("**Strategies:** {} active", active_strats));

    // Plans
    let plans = super::plans::load_all_plans_internal(state);
    let active_plans = plans
        .iter()
        .filter(|p| p.status == crate::commands::status::PlanStatus::Active)
        .count();
    sections.push(format!("**Plans:** {} active", active_plans));

    // Cron
    let cron = super::cron::load_cron_jobs(state);
    let enabled = cron.iter().filter(|j| j.enabled).count();
    sections.push(format!(
        "**Cron:** {} scheduled ({} enabled)",
        cron.len(),
        enabled
    ));

    // Uptime
    sections.push(format!("**Uptime:** {}min", state.uptime_secs() / 60));

    Some(format!("**DASHBOARD**\n{}", sections.join("\n")))
}

/// ACTIVITY_DIGEST: summarize recent activity
pub fn activity_digest(state: &AppState, period: &str) -> Option<String> {
    let hours: u64 = match period {
        "today" => 24,
        "24h" => 24,
        "7d" => 168,
        "1h" => 1,
        _ => period.trim_end_matches('h').parse().unwrap_or(24),
    };

    let log_path = state.root.join("tasks").join(".delegation-log.jsonl");
    let content = std::fs::read_to_string(&log_path).unwrap_or_default();
    let threshold = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
    let mut by_project: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();

    for line in content.lines() {
        if let Ok(e) = serde_json::from_str::<Value>(line) {
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("");
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
                .or_else(|_| chrono::DateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%SZ"))
            {
                if dt >= threshold {
                    let project = e
                        .get("project")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string();
                    let status = e.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    let entry = by_project.entry(project).or_insert((0, 0));
                    if status == "complete" || status == "success" {
                        entry.0 += 1;
                    } else {
                        entry.1 += 1;
                    }
                }
            }
        }
    }

    if by_project.is_empty() {
        return Some(format!("No activity in last {}h.", hours));
    }
    let lines: Vec<String> = by_project
        .iter()
        .map(|(p, (ok, fail))| format!("  {} — {} ok, {} fail", p, ok, fail))
        .collect();
    Some(format!(
        "**Activity (last {}h):**\n{}",
        hours,
        lines.join("\n")
    ))
}

/// PARTNER_UPDATE: generate status for partner (send manually or via NOTIFY)
pub fn partner_update(state: &AppState, custom: &str) -> Option<String> {
    let msg = if custom.is_empty() {
        daily_report(state).unwrap_or_else(|| "No data".to_string())
    } else {
        custom.to_string()
    };
    // Save to orchestrator chat as system message for PA to see and send via TG
    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    super::jsonl::append_jsonl_logged(
        &orch_file,
        &json!({"ts": state.now_iso(), "role": "system", "msg": format!("[PARTNER_UPDATE]\n{}", msg)}),
        "partner update",
    );
    Some(format!(
        "**Partner Update prepared.** Use [NOTIFY:...] to send via Telegram.\n\n{}",
        msg
    ))
}

/// ALERT_CREATE: condition-based alert (stored in alerts.json)
#[derive(Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub check: String,
    pub condition: String,
    pub action: String,
    pub schedule: String,
    pub enabled: bool,
    pub created: String,
}

pub fn alert_create(state: &AppState, name: &str, body: &str) -> Option<String> {
    let alerts_path = state.root.join("tasks").join("alerts.json");
    let mut alerts: Vec<Alert> = std::fs::read_to_string(&alerts_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    // Parse body: "Check: X\nCondition: Y\nAction: Z\nSchedule: W"
    let mut check = String::new();
    let mut condition = String::new();
    let mut action = String::new();
    let mut schedule = "daily".to_string();
    for line in body.lines() {
        let l = line.trim();
        if l.starts_with("Check:") {
            check = l[6..].trim().to_string();
        }
        if l.starts_with("Condition:") {
            condition = l[10..].trim().to_string();
        }
        if l.starts_with("Action:") {
            action = l[7..].trim().to_string();
        }
        if l.starts_with("Schedule:") {
            schedule = l[9..].trim().to_string();
        }
    }

    alerts.push(Alert {
        name: name.to_string(),
        check,
        condition,
        action,
        schedule,
        enabled: true,
        created: state.now_iso(),
    });
    let _ = std::fs::write(
        &alerts_path,
        serde_json::to_string_pretty(&alerts).unwrap_or_default(),
    );
    Some(format!("**Alert created:** {}", name))
}
