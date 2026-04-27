//! Extended delegation commands execution: batch, chain, retry, cancel, etc.

use super::delegation_models::{DelegationPriority, DelegationTemplate};
use super::pa_commands_deleg::DelegPaCommand;
use crate::state::AppState;

/// Execute an extended delegation command. Returns optional text for PA response.
pub fn execute_deleg_command(state: &AppState, cmd: &DelegPaCommand) -> Option<String> {
    match cmd {
        DelegPaCommand::Batch { projects, task } => exec_batch(state, projects, task),
        DelegPaCommand::Chain { project, steps } => exec_chain(state, project, steps),
        DelegPaCommand::Retry { id, context } => exec_retry(state, id, context),
        DelegPaCommand::Cancel { id } => exec_cancel(state, id),
        DelegPaCommand::Status { filter } => exec_status(state, filter),
        DelegPaCommand::Cleanup { hours } => exec_cleanup(state, *hours),
        DelegPaCommand::Priority { id, priority } => exec_priority(state, id, *priority),
        DelegPaCommand::Timeout { id, seconds } => exec_timeout(state, id, *seconds),
        DelegPaCommand::TemplateSave { name, task } => exec_template_save(state, name, task),
        DelegPaCommand::TemplateUse { name, projects } => exec_template_use(state, name, projects),
        DelegPaCommand::Log { filter } => exec_log(state, filter),
        DelegPaCommand::Diff { filter } => exec_diff(state, filter),
    }
}

fn exec_batch(state: &AppState, projects: &[String], task: &str) -> Option<String> {
    let batch_id = format!(
        "batch-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let mut ids = Vec::new();

    for project in projects {
        let valid = state.validate_project_name_from_llm(project);
        let name = valid.as_deref().unwrap_or(project);
        let did = super::delegation::queue_delegation_internal(state, name, task);
        if did.is_empty() {
            continue;
        }
        // Set batch_id
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&did) {
                del.batch_id = Some(batch_id.clone());
            }
        }
        ids.push(format!("{}: {}", name, did));
    }
    state.save_delegations();
    crate::log_info!(
        "[deleg_ext] batch '{}' created: {} delegations",
        batch_id,
        ids.len()
    );
    Some(format!(
        "**Batch {}:** {} delegations queued\n{}",
        batch_id,
        ids.len(),
        ids.join("\n")
    ))
}

fn exec_chain(state: &AppState, project: &str, steps: &[String]) -> Option<String> {
    let valid_name = state.validate_project_name_from_llm(project)?;
    let batch_id = format!(
        "chain-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    // Create all delegations, only first one as Pending, rest as Scheduled (wait for previous)
    let mut ids = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let context = if i > 0 {
            format!(
                "[CHAIN step {}/{}] Previous steps will execute first.\n{}",
                i + 1,
                steps.len(),
                step
            )
        } else {
            format!("[CHAIN step 1/{}] {}", steps.len(), step)
        };
        let did = super::delegation::queue_delegation_internal(state, &valid_name, &context);
        if did.is_empty() {
            continue;
        }
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&did) {
                del.batch_id = Some(batch_id.clone());
                if i > 0 {
                    // Mark non-first steps as scheduled (will be unblocked by auto_approve loop or manual)
                    del.status = crate::commands::status::DelegationStatus::Scheduled;
                }
            }
        }
        ids.push(did);
    }
    state.save_delegations();
    crate::log_info!(
        "[deleg_ext] chain '{}' created: {} steps for {}",
        batch_id,
        ids.len(),
        valid_name
    );
    Some(format!(
        "**Chain {}:** {} steps for {}\nFirst step pending approval, rest scheduled.",
        batch_id,
        ids.len(),
        valid_name
    ))
}

fn exec_retry(state: &AppState, id: &str, context: &str) -> Option<String> {
    let resolved_id = resolve_delegation_id(state, id).ok()?;
    let (project, original_task, error_info) = {
        let delegations = state.delegations.lock().ok()?;
        let del = delegations.get(&resolved_id)?;
        if !del.status.is_terminal() {
            return Some(format!(
                "Cannot retry: delegation {} is {}",
                resolved_id, del.status
            ));
        }
        let err = del.response.as_deref().unwrap_or("no response");
        (
            del.project.clone(),
            del.task.clone(),
            err.chars().take(300).collect::<String>(),
        )
    };

    let retry_task = if context.is_empty() {
        format!(
            "{}\n\n[PREVIOUS ATTEMPT FAILED]\n{}",
            original_task, error_info
        )
    } else {
        format!(
            "{}\n\n[PREVIOUS ATTEMPT FAILED]\n{}\n\n[ADDITIONAL CONTEXT]\n{}",
            original_task, error_info, context
        )
    };

    let new_id = super::delegation::queue_delegation_internal(state, &project, &retry_task);
    // Copy priority from original delegation
    let orig_priority = state
        .delegations
        .lock()
        .ok()
        .and_then(|d| d.get(&resolved_id).map(|del| del.priority));
    if let Some(pri) = orig_priority {
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&new_id) {
                del.priority = pri;
            }
        }
    }
    state.save_delegations();
    crate::log_info!("[deleg_ext] retry {} → new {}", id, new_id);
    Some(format!("**Retry:** {} → new delegation {}", id, new_id))
}

fn exec_cancel(state: &AppState, id: &str) -> Option<String> {
    let resolved_id = resolve_delegation_id(state, id).ok()?;
    let mut status_before = String::new();
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(&resolved_id)?;
        status_before = del.status.to_string();
        if del.status == crate::commands::status::DelegationStatus::Running
            || del.status == crate::commands::status::DelegationStatus::Escalated
        {
            // Try to kill the running process
            let chat_key = format!("deleg-{}", resolved_id);
            super::process_manager::kill_existing(state, &chat_key);
        }
        del.status = crate::commands::status::DelegationStatus::Cancelled;
    }
    state.save_delegations();
    crate::log_info!(
        "[deleg_ext] cancelled {} (was {})",
        resolved_id,
        status_before
    );
    Some(format!(
        "**Cancelled:** {} (was {})",
        resolved_id, status_before
    ))
}

const STALE_PENDING_SECS: u64 = 15 * 60;

fn pending_age_secs(d: &crate::state::Delegation) -> u64 {
    chrono::DateTime::parse_from_rfc3339(&d.ts)
        .or_else(|_| chrono::DateTime::parse_from_str(&d.ts, "%Y-%m-%dT%H:%M:%SZ"))
        .map(|dt| {
            chrono::Utc::now()
                .signed_duration_since(dt)
                .num_seconds()
                .max(0) as u64
        })
        .unwrap_or(0)
}

fn is_stale_pending(d: &crate::state::Delegation) -> bool {
    d.status == crate::commands::status::DelegationStatus::Pending
        && pending_age_secs(d) >= STALE_PENDING_SECS
}

fn exec_status(state: &AppState, filter: &str) -> Option<String> {
    let delegations = state.delegations.lock().ok()?;
    let mut lines = Vec::new();

    for (id, d) in delegations.iter() {
        let matches = filter.is_empty()
            || filter.eq_ignore_ascii_case(&d.project)
            || (filter == "?failed"
                && d.status == crate::commands::status::DelegationStatus::Failed)
            || (filter == "?pending"
                && d.status == crate::commands::status::DelegationStatus::Pending)
            || (filter == "?running"
                && d.status == crate::commands::status::DelegationStatus::Running)
            || (filter == "?stale" && is_stale_pending(d))
            || d.batch_id.as_deref() == Some(filter)
            || id == filter
            || (!filter.is_empty() && id.starts_with(filter));
        if !matches {
            continue;
        }
        let task_short: String = d.task.chars().take(50).collect();
        let pri = d.priority.map(|p| format!(" [{}]", p)).unwrap_or_default();
        let pending_note = if d.status == crate::commands::status::DelegationStatus::Pending {
            let age_min = pending_age_secs(d) / 60;
            if age_min >= STALE_PENDING_SECS / 60 {
                format!(" (pending {}m, stale: needs approval or cancel)", age_min)
            } else {
                format!(" (pending {}m, waiting for user approval)", age_min)
            }
        } else {
            String::new()
        };
        lines.push(format!(
            "  {} {} {}{}{}: {}",
            d.status, d.project, id, pri, pending_note, task_short
        ));
    }

    if lines.is_empty() {
        return Some("No matching delegations.".to_string());
    }
    Some(format!(
        "**Delegations ({}):**\n{}",
        lines.len(),
        lines.join("\n")
    ))
}

fn exec_cleanup(state: &AppState, hours: u64) -> Option<String> {
    super::delegation_analytics::cleanup_delegations(state, hours)
}

fn exec_priority(state: &AppState, id: &str, priority: DelegationPriority) -> Option<String> {
    let resolved_id = resolve_delegation_id(state, id).ok()?;
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(&resolved_id)?;
        del.priority = Some(priority);
    }
    state.save_delegations();
    Some(format!("**Priority set:** {} → {}", id, priority))
}

fn exec_timeout(state: &AppState, id: &str, seconds: u64) -> Option<String> {
    let resolved_id = resolve_delegation_id(state, id).ok()?;
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(&resolved_id)?;
        del.timeout_secs = Some(seconds);
    }
    state.save_delegations();
    Some(format!("**Timeout set:** {} → {}s", id, seconds))
}

fn exec_template_save(state: &AppState, name: &str, task: &str) -> Option<String> {
    let mut templates = super::delegation_models::load_templates(&state.root);
    // Update existing or add new
    if let Some(t) = templates.iter_mut().find(|t| t.name == name) {
        t.task = task.to_string();
    } else {
        templates.push(DelegationTemplate {
            name: name.to_string(),
            task: task.to_string(),
            created: state.now_iso(),
            used_count: 0,
        });
    }
    super::delegation_models::save_templates(&state.root, &templates);
    crate::log_info!("[deleg_ext] template saved: {}", name);
    Some(format!(
        "**Template saved:** {} ({} chars)",
        name,
        task.len()
    ))
}

fn exec_template_use(state: &AppState, name: &str, projects: &[String]) -> Option<String> {
    let mut templates = super::delegation_models::load_templates(&state.root);
    let template = templates.iter_mut().find(|t| t.name == name)?;
    let task = template.task.clone();
    template.used_count += 1;
    super::delegation_models::save_templates(&state.root, &templates);

    // Create delegations for each project using template task
    let batch_id = format!(
        "tmpl-{}-{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let mut created = 0;
    for project in projects {
        let valid = state.validate_project_name_from_llm(project);
        let pname = valid.as_deref().unwrap_or(project);
        let did = super::delegation::queue_delegation_internal(state, pname, &task);
        if did.is_empty() {
            continue;
        }
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&did) {
                del.batch_id = Some(batch_id.clone());
            }
        }
        created += 1;
    }
    state.save_delegations();
    Some(format!(
        "**Template '{}' applied:** {} delegations queued (batch {})",
        name, created, batch_id
    ))
}

fn exec_log(state: &AppState, filter: &str) -> Option<String> {
    super::delegation_analytics::get_delegation_log(state, filter)
}

fn exec_diff(state: &AppState, filter: &str) -> Option<String> {
    super::delegation_analytics::aggregate_diffs(state, filter)
}

fn resolve_delegation_id(state: &AppState, id_or_prefix: &str) -> Result<String, String> {
    let query = id_or_prefix.trim();
    if query.is_empty() {
        return Err("empty delegation id".to_string());
    }

    let delegations = state
        .delegations
        .lock()
        .map_err(|_| "delegation lock poisoned".to_string())?;

    if delegations.contains_key(query) {
        return Ok(query.to_string());
    }

    let matches: Vec<String> = delegations
        .keys()
        .filter(|id| id.starts_with(query))
        .cloned()
        .collect();

    match matches.len() {
        1 => Ok(matches[0].clone()),
        0 => Err(format!("delegation not found: {}", query)),
        _ => Err(format!(
            "ambiguous delegation id prefix '{}': {} matches",
            query,
            matches.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::pa_commands_deleg::DelegPaCommand;
    use crate::commands::status::DelegationStatus;
    use crate::state::{AppState, Delegation};

    fn test_state(name: &str) -> AppState {
        let root = std::env::temp_dir().join(format!(
            "agentos-delegation-ext-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(root.join("tasks"));
        AppState::new(root)
    }

    fn insert_pending_at(state: &AppState, id: &str, ts: &str) {
        let delegation = Delegation {
            id: id.to_string(),
            project: "AgentOS".to_string(),
            task: "test task".to_string(),
            ts: ts.to_string(),
            status: DelegationStatus::Pending,
            response: None,
            retries: 0,
            plan_id: None,
            plan_step: None,
            escalation_info: None,
            strategy_id: None,
            strategy_step_id: None,
            room_session_id: None,
            project_session_id: None,
            work_item_id: None,
            executor_provider: None,
            reviewer_provider: None,
            git_diff: None,
            usage: None,
            scheduled_at: None,
            batch_id: None,
            priority: None,
            timeout_secs: None,
            gate_result: None,
            review_verdict: None,
        };
        state
            .delegations
            .lock()
            .unwrap()
            .insert(id.to_string(), delegation);
    }

    fn insert_pending(state: &AppState, id: &str) {
        insert_pending_at(state, id, "2026-04-25T00:00:00Z");
    }

    #[test]
    fn status_prints_full_id_and_cancel_accepts_unique_prefix() {
        let state = test_state("prefix-cancel");
        let full_id = "1775765745165879500-34220";
        insert_pending(&state, full_id);

        let status = execute_deleg_command(
            &state,
            &DelegPaCommand::Status {
                filter: "?pending".to_string(),
            },
        )
        .unwrap();
        assert!(status.contains(full_id));

        let cancel = execute_deleg_command(
            &state,
            &DelegPaCommand::Cancel {
                id: "177576574516".to_string(),
            },
        )
        .unwrap();
        assert!(cancel.contains("Cancelled"));

        let delegations = state.delegations.lock().unwrap();
        assert_eq!(
            delegations.get(full_id).unwrap().status,
            DelegationStatus::Cancelled
        );
    }

    #[test]
    fn stale_status_excludes_fresh_pending_approvals() {
        let state = test_state("stale-filter");
        let fresh_id = "fresh-approval";
        let stale_id = "stale-approval";
        let fresh_ts = chrono::Utc::now().to_rfc3339();
        let stale_ts = (chrono::Utc::now() - chrono::Duration::minutes(20)).to_rfc3339();

        insert_pending_at(&state, fresh_id, &fresh_ts);
        insert_pending_at(&state, stale_id, &stale_ts);

        let pending = execute_deleg_command(
            &state,
            &DelegPaCommand::Status {
                filter: "?pending".to_string(),
            },
        )
        .unwrap();
        assert!(pending.contains(fresh_id));
        assert!(pending.contains(stale_id));
        assert!(pending.contains("waiting for user approval"));

        let stale = execute_deleg_command(
            &state,
            &DelegPaCommand::Status {
                filter: "?stale".to_string(),
            },
        )
        .unwrap();
        assert!(!stale.contains(fresh_id));
        assert!(stale.contains(stale_id));
        assert!(stale.contains("needs approval or cancel"));
    }
}
