//! Auto-approve rules: config-driven automatic delegation approval.
//! Background loop checks pending delegations every 30s against rules.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct AutoApproveRule {
    pub project: String, // exact name or "*" for all
    #[serde(default = "default_pattern")]
    pub pattern: String, // regex for task content
    #[serde(default)]
    pub delay_seconds: u64, // 0 = immediate
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_pattern() -> String {
    ".*".to_string()
}
fn default_true() -> bool {
    true
}

pub fn load_rules(state: &AppState) -> Vec<AutoApproveRule> {
    let cfg = state.config();
    cfg.get("auto_approve_rules")
        .and_then(|v| serde_json::from_value::<Vec<AutoApproveRule>>(v.clone()).ok())
        .unwrap_or_default()
}

fn should_pause_all(
    actions: &[super::sensors::SensorAction],
    paused_projects: &mut std::collections::HashSet<String>,
) -> bool {
    let mut pause_all = false;
    for action in actions {
        if let super::sensors::SensorAction::Pause { project, .. } = action {
            if project == "*" {
                pause_all = true;
            } else {
                paused_projects.insert(project.clone());
            }
        }
    }
    pause_all
}

fn matches_rule<'a>(
    delegation: &crate::state::Delegation,
    rules: &'a [AutoApproveRule],
) -> Option<&'a AutoApproveRule> {
    for rule in rules {
        if !rule.enabled {
            continue;
        }
        // Project match: exact or wildcard
        let project_match =
            rule.project == "*" || rule.project.eq_ignore_ascii_case(&delegation.project);
        if !project_match {
            continue;
        }
        // Pattern match: regex on task
        if rule.pattern != ".*" {
            if let Ok(re) = regex::Regex::new(&rule.pattern) {
                if !re.is_match(&delegation.task) {
                    continue;
                }
            } else {
                crate::log_warn!("[auto-approve] invalid regex pattern: {}", rule.pattern);
                continue;
            }
        }
        return Some(rule);
    }
    None
}

fn delegation_age_secs(d: &crate::state::Delegation) -> u64 {
    // Parse ISO timestamp and compare to now
    chrono::DateTime::parse_from_rfc3339(&d.ts)
        .or_else(|_| chrono::DateTime::parse_from_str(&d.ts, "%Y-%m-%dT%H:%M:%SZ"))
        .map(|dt| {
            let now = chrono::Utc::now();
            now.signed_duration_since(dt).num_seconds().max(0) as u64
        })
        .unwrap_or(0)
}

/// Select recent unacknowledged critical signals for one auto-trigger cycle.
fn collect_unacked_critical_signals(content: &str, limit: usize) -> Vec<(String, String)> {
    let signal_entries: Vec<Value> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect();
    let acked: std::collections::HashSet<String> = signal_entries
        .iter()
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("ack"))
        .filter_map(|v| {
            v.get("signal_id")
                .and_then(|id| id.as_str())
                .map(String::from)
        })
        .collect();

    signal_entries
        .iter()
        .rev()
        .take(80)
        .filter(|v| {
            if v.get("type").and_then(|t| t.as_str()) == Some("ack") {
                return false;
            }
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
            v.get("severity").and_then(|s| s.as_str()) == Some("critical")
                && !id.is_empty()
                && !acked.contains(id)
                && !v
                    .get("acknowledged")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(false)
        })
        .take(limit)
        .filter_map(|v| {
            Some((
                v.get("id").and_then(|id| id.as_str())?.to_string(),
                v.get("message").and_then(|m| m.as_str())?.to_string(),
            ))
        })
        .collect()
}

/// Background loop — spawned in lib.rs, ticks every 30 seconds
pub async fn auto_approve_loop(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        interval.tick().await;

        // Housekeeping: rotate signals if too large
        super::signals::rotate_signals(&state);

        // Run sensor framework
        let sensor_actions = super::sensors::run_sensors(&state);

        // Auto-trigger PA on critical signals (max once per 5 min)
        auto_trigger_pa(&state);
        let mut paused_projects: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let pause_all = should_pause_all(&sensor_actions, &mut paused_projects);
        for action in &sensor_actions {
            match action {
                super::sensors::SensorAction::Pause { project, reason } => {
                    if project == "*" {
                        crate::log_warn!("[auto-approve] ALL paused: {}", reason);
                        continue;
                    }
                    crate::log_warn!("[auto-approve] paused {}: {}", project, reason);
                }
                super::sensors::SensorAction::Trigger { delegation_id } => {
                    let state_c = Arc::clone(&state);
                    let id = delegation_id.clone();
                    tokio::task::spawn_blocking(move || {
                        super::delegation::approve_delegation_core(&state_c, &id);
                    });
                }
                super::sensors::SensorAction::Skip => {}
            }
        }
        if pause_all {
            crate::log_info!(
                "[auto-approve] skipping approval cycle because global pause is active"
            );
            continue;
        }

        let rules = load_rules(&state);
        if rules.is_empty() || rules.iter().all(|r| !r.enabled) {
            continue;
        }

        // Collect pending delegations
        let pending: Vec<crate::state::Delegation> = {
            let delegations = match state.delegations.lock() {
                Ok(d) => d,
                Err(_) => continue,
            };
            delegations
                .values()
                .filter(|d| d.status == crate::commands::status::DelegationStatus::Pending)
                .cloned()
                .collect()
        };

        // Check scheduled delegations — execute if time has come
        let scheduled: Vec<crate::state::Delegation> = {
            let delegations = match state.delegations.lock() {
                Ok(d) => d,
                Err(_) => continue,
            };
            delegations
                .values()
                .filter(|d| d.status == crate::commands::status::DelegationStatus::Scheduled)
                .cloned()
                .collect()
        };
        let now = chrono::Utc::now();
        for d in &scheduled {
            if let Some(ref sched) = d.scheduled_at {
                let is_due = chrono::DateTime::parse_from_rfc3339(sched)
                    .map(|dt| now >= dt)
                    .unwrap_or(false);
                if is_due {
                    crate::log_info!(
                        "[auto-approve] scheduled delegation {} is due, executing",
                        d.id
                    );
                    // Change status to Pending first, then approve
                    if let Ok(mut delegations) = state.delegations.lock() {
                        if let Some(del) = delegations.get_mut(&d.id) {
                            del.status = crate::commands::status::DelegationStatus::Pending;
                            del.scheduled_at = None;
                        }
                    }
                    state.save_delegations();
                    let state_c = Arc::clone(&state);
                    let id = d.id.clone();
                    tokio::task::spawn_blocking(move || {
                        super::delegation::approve_delegation_core(&state_c, &id);
                    });
                }
            }
        }

        // Sort by priority (High first)
        let mut pending = pending;
        pending.sort_by(|a, b| {
            let pa = a.priority.map(|p| p.ord()).unwrap_or(1);
            let pb = b.priority.map(|p| p.ord()).unwrap_or(1);
            pa.cmp(&pb)
        });

        for d in &pending {
            // Skip paused projects (incident sensor)
            if paused_projects.contains(&d.project) || paused_projects.contains("*") {
                continue;
            }
            if let Some(rule) = matches_rule(d, &rules) {
                let age = delegation_age_secs(d);
                if age >= rule.delay_seconds {
                    crate::log_info!(
                        "[auto-approve] {} matched rule for '{}' (age={}s, delay={}s)",
                        d.project,
                        rule.project,
                        age,
                        rule.delay_seconds
                    );

                    // Log to orchestrator chat
                    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
                    let msg = format!(
                        "⚡ Auto-approved: {} (rule: {}, delay: {}s)",
                        d.project, rule.project, rule.delay_seconds
                    );
                    super::jsonl::append_jsonl_logged(
                        &orch_file,
                        &json!({"ts": state.now_iso(), "role": "system", "msg": msg}),
                        "auto-approve",
                    );

                    let state_c = Arc::clone(&state);
                    let id = d.id.clone();
                    tokio::task::spawn_blocking(move || {
                        super::delegation::approve_delegation_core(&state_c, &id);
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_pause_short_circuits_cycle() {
        let mut paused = std::collections::HashSet::new();
        let actions = vec![
            super::super::sensors::SensorAction::Pause {
                project: "*".to_string(),
                reason: "incident".to_string(),
            },
            super::super::sensors::SensorAction::Pause {
                project: "proj-a".to_string(),
                reason: "local".to_string(),
            },
        ];

        assert!(should_pause_all(&actions, &mut paused));
        assert!(paused.contains("proj-a"));
    }

    #[test]
    fn auto_trigger_ignores_acknowledged_critical_signals() {
        let content = [
            r#"{"id":"old-critical","severity":"critical","message":"old failure"}"#,
            r#"{"type":"ack","signal_id":"old-critical","ts":"2026-04-28T10:00:00Z"}"#,
            r#"{"id":"warn-only","severity":"warn","message":"warning"}"#,
            r#"{"id":"new-critical","severity":"critical","message":"new failure"}"#,
        ]
        .join("\n");

        let signals = collect_unacked_critical_signals(&content, 5);

        assert_eq!(
            signals,
            vec![("new-critical".to_string(), "new failure".to_string())]
        );
    }
}

#[tauri::command]
pub fn get_auto_approve_rules(state: State<Arc<AppState>>) -> Value {
    json!({"rules": load_rules(&state)})
}

#[tauri::command]
pub fn set_auto_approve_rules(state: State<Arc<AppState>>, rules: Vec<Value>) -> Value {
    let parsed: Vec<AutoApproveRule> = rules
        .iter()
        .filter_map(|v| serde_json::from_value::<AutoApproveRule>(v.clone()).ok())
        .collect();

    // Read config, update auto_approve_rules, save
    let mut cfg = state.config();
    if let Some(obj) = cfg.as_object_mut() {
        obj.insert(
            "auto_approve_rules".to_string(),
            serde_json::to_value(&parsed).unwrap_or_default(),
        );
        if let Ok(content) = serde_json::to_string_pretty(&cfg) {
            let _ = super::claude_runner::atomic_write(&state.config_path, &content);
            state.invalidate_config();
        }
    }
    crate::log_info!("[auto-approve] saved {} rules", parsed.len());
    json!({"status": "ok", "count": parsed.len()})
}

/// Auto-trigger PA conversation on unacknowledged critical signals.
/// Rate-limited: max once per 5 minutes. Uses sync chat (blocking).
fn auto_trigger_pa(state: &AppState) {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    static LAST_TRIGGER: AtomicU64 = AtomicU64::new(0);
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Init cooldown to now on first call (#17)
    if !INITIALIZED.swap(true, Ordering::Relaxed) {
        LAST_TRIGGER.store(now_secs, Ordering::Relaxed);
        return;
    }
    let last = LAST_TRIGGER.load(Ordering::Relaxed);
    if now_secs - last < 300 {
        return;
    } // 5 min cooldown

    // Check for unacknowledged critical signals
    let (crit, _, _) = super::signals::signal_counts(state);
    if crit == 0 {
        return;
    }

    // Build message from recent critical signals
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let critical_signals = collect_unacked_critical_signals(&content, 5);

    if critical_signals.is_empty() {
        return;
    }
    let critical_ids: Vec<String> = critical_signals.iter().map(|(id, _)| id.clone()).collect();
    let critical_msgs: Vec<String> = critical_signals
        .iter()
        .map(|(_, msg)| msg.clone())
        .collect();

    LAST_TRIGGER.store(now_secs, Ordering::Relaxed);
    crate::log_info!(
        "[auto-trigger] {} critical signals, triggering PA",
        critical_msgs.len()
    );

    // Build auto-trigger message for PA
    let trigger_msg = format!(
        "[AUTO-TRIGGER: {} critical signals detected]\n{}\n\nAnalyze these issues and take corrective action. Use [DELEGATE:...] to fix problems or [NOTIFY:...] to alert the user.",
        critical_msgs.len(),
        critical_msgs.iter().enumerate().map(|(i, m)| format!("{}. {}", i + 1, m)).collect::<Vec<_>>().join("\n")
    );

    // Write to orchestrator chat as system message
    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    let ts = state.now_iso();
    super::jsonl::append_jsonl_logged(
        &orch_file,
        &serde_json::json!({"ts": ts, "role": "system", "msg": format!("⚡ Auto-trigger: {} critical signals", critical_msgs.len())}),
        "auto-trigger system",
    );

    // Send to PA via sync chat (build prompt + run claude)
    let prompt = super::chat_parse::build_full_pa_prompt(state, &trigger_msg);
    let perm_path =
        super::claude_runner::get_delegation_permission_path(state, "_orchestrator", "balanced");
    let cfg = state.config();
    let model = cfg
        .get("orchestrator_model")
        .and_then(|v| v.as_str())
        .unwrap_or("sonnet");
    let effort = cfg
        .get("orchestrator_effort")
        .and_then(|v| v.as_str())
        .unwrap_or("high");

    // Non-blocking lock (#6): skip auto-trigger if PA is busy (user chatting)
    {
        let busy = state.dir_busy.lock().unwrap_or_else(|e| e.into_inner());
        if busy.contains("_orchestrator") {
            crate::log_info!("[auto-trigger] PA busy, skipping");
            return;
        }
    }
    state.acquire_dir_lock("_orchestrator");
    let response = super::claude_runner::run_claude_with_opts(
        &state.docs_dir.join(
            cfg.get("orchestrator_project")
                .and_then(|v| v.as_str())
                .unwrap_or("PersonalAssistant"),
        ),
        &prompt,
        &perm_path,
        Some(model),
        Some(effort),
    );
    state.release_dir_lock("_orchestrator");

    // Save response + process PA commands
    let ts2 = state.now_iso();
    super::jsonl::append_jsonl_logged(
        &orch_file,
        &serde_json::json!({"ts": ts2, "role": "assistant", "msg": response}),
        "auto-trigger response",
    );

    // Execute any PA commands from the response
    let commands = super::pa_commands::parse_pa_commands(&response, state);
    for parsed in &commands {
        if parsed.valid {
            if let Some(result_msg) = super::pa_commands::execute_pa_command(state, &parsed.cmd) {
                super::jsonl::append_jsonl_logged(
                    &orch_file,
                    &serde_json::json!({"ts": state.now_iso(), "role": "system", "msg": result_msg}),
                    "auto-trigger cmd result",
                );
            }
        }
    }
    for signal_id in &critical_ids {
        super::signals::acknowledge_signal(state, signal_id);
    }

    crate::log_info!(
        "[auto-trigger] PA responded ({} chars), {} commands, {} signals acked",
        response.len(),
        commands.len(),
        critical_ids.len()
    );
}
