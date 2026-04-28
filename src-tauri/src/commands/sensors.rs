//! Sensor Framework: pluggable reactive triggers for orchestration.
//! Sensors run in the background loop (30s tick) alongside auto_approve.
//! Each sensor checks state and returns actions: Trigger, Skip, Pause.

use crate::state::AppState;

/// Action a sensor can request.
pub enum SensorAction {
    /// Trigger a delegation approval
    Trigger { delegation_id: String },
    /// Pause auto-approve for a project (incident)
    Pause { project: String, reason: String },
    /// Skip — no action needed
    Skip,
}

/// Run all sensors once. Called from auto_approve_loop every 30s.
pub fn run_sensors(state: &AppState) -> Vec<SensorAction> {
    let mut actions = Vec::new();

    // Sensor 1: Strategy auto-queue — if a step completed, queue next ready steps
    actions.extend(sensor_strategy_next(state));

    // Sensor 2: Incident detector — 3+ critical signals in 10min → pause project
    actions.extend(sensor_incident_pause(state));

    // Sensor 3: Stale process detector — delegations running >45min with no progress
    actions.extend(sensor_stale_process(state));

    // Sensor 4: Cost guard — hourly spend check
    actions.extend(sensor_cost_guard(state));

    // Sensor 5: Verify conditions — auto-check Todo.verify
    sensor_verify_todos(state);

    actions
}

/// Sensor: auto-queue next strategy steps when previous completes.
fn sensor_strategy_next(state: &AppState) -> Vec<SensorAction> {
    // Strategy auto-queue is already handled in delegation.rs:283-301
    // (update_step_from_delegation → try_queue_next_steps)
    // This sensor catches edge cases where the loop was missed.
    let strategies = super::strategy_models::load_strategies(state);
    let mut actions = Vec::new();

    for strat in &strategies {
        if !strat.status.is_active() {
            continue;
        }
        for plan in &strat.plans {
            for step in &plan.steps {
                // Approved step with no delegation = needs queuing
                if step.status == crate::commands::status::StepStatus::Approved
                    && step.delegation_id.is_none()
                {
                    // Check deps met — search ALL plans in strategy, not just current plan (#14)
                    let all_steps: Vec<&super::strategy_models::Step> =
                        strat.plans.iter().flat_map(|p| p.steps.iter()).collect();
                    let deps_met = step.depends_on.iter().all(|dep_id| {
                        all_steps.iter().any(|s| {
                            s.id == *dep_id && s.status == crate::commands::status::StepStatus::Done
                        })
                    });
                    if deps_met {
                        crate::log_info!("[sensor:strategy_next] step {} ready, queuing", step.id);
                        let did = super::delegation::queue_delegation_internal(
                            state,
                            &plan.project,
                            &step.task,
                        );
                        // Link to strategy + persist (#4)
                        if let Ok(mut delegations) = state.delegations.lock() {
                            if let Some(del) = delegations.get_mut(&did) {
                                del.strategy_id = Some(strat.id.clone());
                                del.strategy_step_id = Some(step.id.clone());
                                del.room_session_id = strat.room_session_id.clone();
                            }
                        }
                        state.save_delegations();
                        if let Some(session_id) = strat.room_session_id.as_deref() {
                            super::multi_agent::link_delegation_to_session(
                                state,
                                session_id,
                                &did,
                                &plan.project,
                                &step.task,
                            );
                            super::multi_agent::emit_pipeline_event(
                                state,
                                session_id,
                                "strategy_step_queued",
                                "system",
                                &format!(
                                    "Sensor queued strategy step {} for {} as delegation {}",
                                    step.id, plan.project, did
                                ),
                                serde_json::json!({
                                    "strategy_id": strat.id,
                                    "step_id": step.id,
                                    "delegation_id": did,
                                    "project": plan.project,
                                }),
                            );
                        }
                        actions.push(SensorAction::Skip);
                    }
                }
            }
        }
    }
    actions
}

/// Sensor: detect incident pattern and pause auto-approve.
/// Uses unified count_recent_critical() with proper timestamp comparison (#9).
fn sensor_incident_pause(state: &AppState) -> Vec<SensorAction> {
    // Get all projects with delegations
    let projects: Vec<String> = match state.delegations.lock() {
        Ok(d) => d
            .values()
            .map(|del| del.project.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect(),
        Err(_) => return vec![],
    };
    let mut actions = Vec::new();
    for proj in &projects {
        let count = super::signals::count_recent_critical(state, Some(proj), 10);
        if count >= 3 {
            actions.push(SensorAction::Pause {
                project: proj.clone(),
                reason: format!("{} critical signals in 10min — auto-approve paused", count),
            });
        }
    }
    actions
}

/// Sensor: detect delegations stuck in Running for too long.
fn sensor_stale_process(state: &AppState) -> Vec<SensorAction> {
    let delegations = match state.delegations.lock() {
        Ok(d) => d,
        Err(e) => e.into_inner(),
    };
    let mut actions = Vec::new();
    let max_age_secs = 45 * 60; // 45 minutes

    for (id, d) in delegations.iter() {
        if d.status != crate::commands::status::DelegationStatus::Running
            && d.status != crate::commands::status::DelegationStatus::Verifying
        {
            continue;
        }

        let running_since = d.started_at.as_deref().unwrap_or(&d.ts);
        let age = chrono::DateTime::parse_from_rfc3339(running_since)
            .map(|dt| {
                chrono::Utc::now()
                    .signed_duration_since(dt)
                    .num_seconds()
                    .max(0) as u64
            })
            .unwrap_or(0);

        if age > max_age_secs {
            crate::log_warn!(
                "[sensor:stale] delegation {} in {} stuck for {}s",
                id,
                d.project,
                age
            );
            super::signals::emit_signal(
                state,
                super::signals::SignalSource::Timeout,
                super::signals::Severity::Warn,
                Some(&d.project),
                &format!(
                    "Delegation {} running for {}min — may be stuck",
                    &id[..8.min(id.len())],
                    age / 60
                ),
                Some(id),
            );
            actions.push(SensorAction::Skip);
        }
    }
    actions
}

/// Sensor: check hourly cost across all delegations.
fn sensor_cost_guard(state: &AppState) -> Vec<SensorAction> {
    let usage_path = state.root.join("tasks").join(".usage-log.jsonl");
    let content = match std::fs::read_to_string(&usage_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);
    let mut total_cost: f64 = 0.0;

    for line in content.lines().rev().take(200) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            let ts = v.get("ts").and_then(|t| t.as_str()).unwrap_or("");
            if chrono::DateTime::parse_from_rfc3339(ts)
                .map(|dt| dt < one_hour_ago)
                .unwrap_or(true)
            {
                break;
            }
            total_cost += v.get("cost_usd").and_then(|c| c.as_f64()).unwrap_or(0.0);
        }
    }

    let hourly_budget = 5.0; // $5/hour default
    if total_cost > hourly_budget {
        crate::log_warn!(
            "[sensor:cost] hourly spend ${:.2} exceeds budget ${:.2}",
            total_cost,
            hourly_budget
        );
        super::signals::emit_signal(
            state,
            super::signals::SignalSource::CostGuard,
            super::signals::Severity::Critical,
            None,
            &format!(
                "Hourly cost ${:.2} exceeds ${:.2} budget",
                total_cost, hourly_budget
            ),
            None,
        );
        return vec![SensorAction::Pause {
            project: "*".to_string(),
            reason: format!("Cost guard: ${:.2}/hr exceeds budget", total_cost),
        }];
    }
    vec![]
}

/// Sensor: check Todo.verify conditions and auto-mark verified.
fn sensor_verify_todos(state: &AppState) {
    use super::strategy_models::*;
    let mut strategies = load_strategies(state);
    let mut changed = false;

    for strat in &mut strategies {
        let tactics = if !strat.tactics.is_empty() {
            &mut strat.tactics
        } else {
            // Can't mutate plans through all_tactics() wrapper, skip legacy
            continue;
        };
        for tactic in tactics.iter_mut() {
            for plan in &mut tactic.plans {
                let project_dir = state.docs_dir.join(&plan.project);
                for step in &mut plan.steps {
                    // Only check done (unverified) or running steps with verify condition
                    if step.verify.is_none() {
                        continue;
                    }
                    if step.status != crate::commands::status::StepStatus::Done
                        && step.status != crate::commands::status::StepStatus::Running
                        && step.status != crate::commands::status::StepStatus::Queued
                    {
                        continue;
                    }

                    if let Some(ref vc) = step.verify {
                        if check_verify_condition(vc, &project_dir) {
                            if step.status != crate::commands::status::StepStatus::Done {
                                // Auto-verify even if delegation hasn't finished
                                crate::log_info!(
                                    "[sensor:verify] auto-verified: {} (condition met)",
                                    step.id
                                );
                                super::signals::emit_signal(
                                    state,
                                    super::signals::SignalSource::Scanner,
                                    super::signals::Severity::Info,
                                    Some(&plan.project),
                                    &format!("Auto-verified: {}", step.task),
                                    None,
                                );
                            }
                            step.status = crate::commands::status::StepStatus::Done;
                            if step.response.is_none() {
                                step.response = Some("auto-verified by sensor".to_string());
                            }
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    if changed {
        save_strategies(state, &strategies);
    }
}

/// Check a single verify condition against project file system.
fn check_verify_condition(
    vc: &super::strategy_models::VerifyCondition,
    project_dir: &std::path::Path,
) -> bool {
    use super::strategy_models::VerifyCondition;
    match vc {
        VerifyCondition::FileExists { path } => project_dir.join(path).exists(),
        VerifyCondition::GrepMatch { glob: _, pattern } => {
            // Use git grep for efficient content search (works in any git project)
            let re_check = regex::Regex::new(pattern).is_ok();
            if !re_check {
                return false;
            }
            super::claude_runner::silent_cmd("git")
                .args(["grep", "-q", "-E", pattern])
                .current_dir(project_dir)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        VerifyCondition::CommandExits { cmd, exit_code } => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return false;
            }
            super::claude_runner::silent_cmd(parts[0])
                .args(&parts[1..])
                .current_dir(project_dir)
                .output()
                .map(|o| o.status.code().unwrap_or(-1) == *exit_code)
                .unwrap_or(false)
        }
        VerifyCondition::GitChanged { path } => super::claude_runner::silent_cmd("git")
            .args(["diff", "--name-only", "--", path])
            .current_dir(project_dir)
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false),
    }
}
