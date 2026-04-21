//! Extended delegation commands execution: batch, chain, retry, cancel, etc.

use crate::state::AppState;
use super::pa_commands_deleg::DelegPaCommand;
use super::delegation_models::{DelegationPriority, DelegationTemplate};

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
    let batch_id = format!("batch-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    let mut ids = Vec::new();

    for project in projects {
        let valid = state.validate_project_name_from_llm(project);
        let name = valid.as_deref().unwrap_or(project);
        let did = super::delegation::queue_delegation_internal(state, name, task);
        if did.is_empty() { continue; }
        // Set batch_id
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&did) {
                del.batch_id = Some(batch_id.clone());
            }
        }
        ids.push(format!("{}: {}", name, did));
    }
    state.save_delegations();
    crate::log_info!("[deleg_ext] batch '{}' created: {} delegations", batch_id, ids.len());
    Some(format!("**Batch {}:** {} delegations queued\n{}", batch_id, ids.len(), ids.join("\n")))
}

fn exec_chain(state: &AppState, project: &str, steps: &[String]) -> Option<String> {
    let valid_name = state.validate_project_name_from_llm(project)?;
    let batch_id = format!("chain-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));

    // Create all delegations, only first one as Pending, rest as Scheduled (wait for previous)
    let mut ids = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let context = if i > 0 {
            format!("[CHAIN step {}/{}] Previous steps will execute first.\n{}", i + 1, steps.len(), step)
        } else {
            format!("[CHAIN step 1/{}] {}", steps.len(), step)
        };
        let did = super::delegation::queue_delegation_internal(state, &valid_name, &context);
        if did.is_empty() { continue; }
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
    crate::log_info!("[deleg_ext] chain '{}' created: {} steps for {}", batch_id, ids.len(), valid_name);
    Some(format!("**Chain {}:** {} steps for {}\nFirst step pending approval, rest scheduled.", batch_id, ids.len(), valid_name))
}

fn exec_retry(state: &AppState, id: &str, context: &str) -> Option<String> {
    let (project, original_task, error_info) = {
        let delegations = state.delegations.lock().ok()?;
        let del = delegations.get(id)?;
        if !del.status.is_terminal() {
            return Some(format!("Cannot retry: delegation {} is {}", id, del.status));
        }
        let err = del.response.as_deref().unwrap_or("no response");
        (del.project.clone(), del.task.clone(), err.chars().take(300).collect::<String>())
    };

    let retry_task = if context.is_empty() {
        format!("{}\n\n[PREVIOUS ATTEMPT FAILED]\n{}", original_task, error_info)
    } else {
        format!("{}\n\n[PREVIOUS ATTEMPT FAILED]\n{}\n\n[ADDITIONAL CONTEXT]\n{}", original_task, error_info, context)
    };

    let new_id = super::delegation::queue_delegation_internal(state, &project, &retry_task);
    // Copy priority from original delegation
    let orig_priority = state.delegations.lock().ok()
        .and_then(|d| d.get(id).map(|del| del.priority));
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
    let mut status_before = String::new();
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(id)?;
        status_before = del.status.to_string();
        if del.status == crate::commands::status::DelegationStatus::Running ||
           del.status == crate::commands::status::DelegationStatus::Escalated {
            // Try to kill the running process
            let chat_key = format!("deleg-{}", id);
            super::process_manager::kill_existing(state, &chat_key);
        }
        del.status = crate::commands::status::DelegationStatus::Cancelled;
    }
    state.save_delegations();
    crate::log_info!("[deleg_ext] cancelled {} (was {})", id, status_before);
    Some(format!("**Cancelled:** {} (was {})", id, status_before))
}

fn exec_status(state: &AppState, filter: &str) -> Option<String> {
    let delegations = state.delegations.lock().ok()?;
    let mut lines = Vec::new();

    for (id, d) in delegations.iter() {
        let matches = filter.is_empty()
            || filter.eq_ignore_ascii_case(&d.project)
            || (filter == "?failed" && d.status == crate::commands::status::DelegationStatus::Failed)
            || (filter == "?stale" && d.status == crate::commands::status::DelegationStatus::Pending)
            || d.batch_id.as_deref() == Some(filter)
            || id == filter;
        if !matches { continue; }
        let task_short: String = d.task.chars().take(50).collect();
        let pri = d.priority.map(|p| format!(" [{}]", p)).unwrap_or_default();
        lines.push(format!("  {} {} {}{}: {}", d.status, d.project, id.chars().take(12).collect::<String>(), pri, task_short));
    }

    if lines.is_empty() { return Some("No matching delegations.".to_string()); }
    Some(format!("**Delegations ({}):**\n{}", lines.len(), lines.join("\n")))
}

fn exec_cleanup(state: &AppState, hours: u64) -> Option<String> {
    super::delegation_analytics::cleanup_delegations(state, hours)
}

fn exec_priority(state: &AppState, id: &str, priority: DelegationPriority) -> Option<String> {
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(id)?;
        del.priority = Some(priority);
    }
    state.save_delegations();
    Some(format!("**Priority set:** {} → {}", id, priority))
}

fn exec_timeout(state: &AppState, id: &str, seconds: u64) -> Option<String> {
    if let Ok(mut delegations) = state.delegations.lock() {
        let del = delegations.get_mut(id)?;
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
    Some(format!("**Template saved:** {} ({} chars)", name, task.len()))
}

fn exec_template_use(state: &AppState, name: &str, projects: &[String]) -> Option<String> {
    let mut templates = super::delegation_models::load_templates(&state.root);
    let template = templates.iter_mut().find(|t| t.name == name)?;
    let task = template.task.clone();
    template.used_count += 1;
    super::delegation_models::save_templates(&state.root, &templates);

    // Create delegations for each project using template task
    let batch_id = format!("tmpl-{}-{}", name, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    let mut created = 0;
    for project in projects {
        let valid = state.validate_project_name_from_llm(project);
        let pname = valid.as_deref().unwrap_or(project);
        let did = super::delegation::queue_delegation_internal(state, pname, &task);
        if did.is_empty() { continue; }
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(&did) {
                del.batch_id = Some(batch_id.clone());
            }
        }
        created += 1;
    }
    state.save_delegations();
    Some(format!("**Template '{}' applied:** {} delegations queued (batch {})", name, created, batch_id))
}

fn exec_log(state: &AppState, filter: &str) -> Option<String> {
    super::delegation_analytics::get_delegation_log(state, filter)
}

fn exec_diff(state: &AppState, filter: &str) -> Option<String> {
    super::delegation_analytics::aggregate_diffs(state, filter)
}
