//! Extended delegation command parsing: 12 DELEGATE_* commands.
//! Parsed here, executed in delegation_ext.rs.

use crate::state::AppState;
use super::pa_commands::ParsedCommand;
use super::delegation_models::DelegationPriority;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub enum DelegPaCommand {
    Batch { projects: Vec<String>, task: String },
    Chain { project: String, steps: Vec<String> },
    Retry { id: String, context: String },
    Cancel { id: String },
    Status { filter: String },
    Cleanup { hours: u64 },
    Priority { id: String, priority: DelegationPriority },
    Timeout { id: String, seconds: u64 },
    TemplateSave { name: String, task: String },
    TemplateUse { name: String, projects: Vec<String> },
    Log { filter: String },
    Diff { filter: String },
}

static RE_BATCH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DELEGATE_BATCH:([^\]]+)\]\s*\n?(.*?)\n?\[/DELEGATE_BATCH\]").unwrap()
});
static RE_CHAIN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DELEGATE_CHAIN:([^\]]+)\]\s*\n?(.*?)\n?\[/DELEGATE_CHAIN\]").unwrap()
});
static RE_RETRY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DELEGATE_RETRY:([^\]]+)\]\s*\n?(.*?)\n?\[/DELEGATE_RETRY\]").unwrap()
});
static RE_CANCEL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_CANCEL:([^\]]+)\]").unwrap()
});
static RE_STATUS: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_STATUS(?::([^\]]*))?\]").unwrap()
});
static RE_CLEANUP: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_CLEANUP(?::(\d+))?\]").unwrap()
});
static RE_PRIORITY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_PRIORITY:([^:]+):(HIGH|MED|LOW)\]").unwrap()
});
static RE_TIMEOUT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_TIMEOUT:([^:]+):(\d+)\]").unwrap()
});
static RE_TEMPLATE_SAVE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DELEGATE_TEMPLATE:save:([^\]]+)\]\s*\n?(.*?)\n?\[/DELEGATE_TEMPLATE\]").unwrap()
});
static RE_TEMPLATE_USE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_TEMPLATE:use:([^:]+):([^\]]+)\]").unwrap()
});
static RE_LOG: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_LOG(?::([^\]]*))?\]").unwrap()
});
static RE_DIFF: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[DELEGATE_DIFF(?::([^\]]*))?\]").unwrap()
});

/// Parse all DELEGATE_* extended commands from PA response
pub fn parse_delegation_commands(response: &str, state: &AppState) -> Vec<ParsedCommand> {
    let mut cmds = Vec::new();

    // BATCH
    if let Some(caps) = RE_BATCH.captures(response) {
        let projects: Vec<String> = caps.get(1).map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
        let task = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let (valid, error) = if projects.is_empty() || task.is_empty() {
            (false, Some("DELEGATE_BATCH requires projects and task".to_string()))
        } else {
            validate_projects(state, &projects)
        };
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Batch { projects, task }), valid, error });
    }

    // CHAIN
    if let Some(caps) = RE_CHAIN.captures(response) {
        let project = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let body = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        let steps: Vec<String> = body.lines().filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().trim_start_matches(|c: char| "-*0123456789. ".contains(c)).to_string())
            .filter(|s| !s.is_empty()).collect();
        let (valid, error) = if project.is_empty() || steps.is_empty() {
            (false, Some("DELEGATE_CHAIN requires project and steps".to_string()))
        } else if state.validate_project_name_from_llm(&project).is_none() {
            (false, Some(format!("Unknown project: {}", project)))
        } else { (true, None) };
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Chain { project, steps }), valid, error });
    }

    // RETRY
    if let Some(caps) = RE_RETRY.captures(response) {
        let id = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let context = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Retry { id, context }), valid: true, error: None });
    }

    // CANCEL
    if let Some(caps) = RE_CANCEL.captures(response) {
        let id = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let valid = !id.is_empty();
        let error = if valid { None } else { Some("Empty id".to_string()) };
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Cancel { id }), valid, error });
    }

    // STATUS
    if let Some(caps) = RE_STATUS.captures(response) {
        let filter = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Status { filter }), valid: true, error: None });
    }

    // CLEANUP
    if let Some(caps) = RE_CLEANUP.captures(response) {
        let hours = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(24);
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Cleanup { hours }), valid: true, error: None });
    }

    // PRIORITY
    if let Some(caps) = RE_PRIORITY.captures(response) {
        let id = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let pri = match caps.get(2).map(|m| m.as_str()) {
            Some("HIGH") => DelegationPriority::High,
            Some("LOW") => DelegationPriority::Low,
            _ => DelegationPriority::Med,
        };
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Priority { id, priority: pri }), valid: true, error: None });
    }

    // TIMEOUT
    if let Some(caps) = RE_TIMEOUT.captures(response) {
        let id = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let seconds = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(300);
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Timeout { id, seconds }), valid: true, error: None });
    }

    // TEMPLATE SAVE
    if let Some(caps) = RE_TEMPLATE_SAVE.captures(response) {
        let name = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let task = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let valid = !name.is_empty() && !task.is_empty();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::TemplateSave { name, task }), valid, error: if !valid { Some("Template requires name and task".to_string()) } else { None } });
    }

    // TEMPLATE USE
    if let Some(caps) = RE_TEMPLATE_USE.captures(response) {
        let name = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let projects: Vec<String> = caps.get(2).map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
        let valid = !name.is_empty() && !projects.is_empty();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::TemplateUse { name, projects }), valid, error: if !valid { Some("Template use requires name and projects".to_string()) } else { None } });
    }

    // LOG
    if let Some(caps) = RE_LOG.captures(response) {
        let filter = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Log { filter }), valid: true, error: None });
    }

    // DIFF
    if let Some(caps) = RE_DIFF.captures(response) {
        let filter = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        cmds.push(ParsedCommand { cmd: super::pa_commands::PaCommand::DelegExt(DelegPaCommand::Diff { filter }), valid: true, error: None });
    }

    for cmd in &cmds {
        if cmd.valid {
            crate::log_info!("[pa_cmd_deleg] parsed valid: {:?}", std::mem::discriminant(&cmd.cmd));
        } else {
            crate::log_warn!("[pa_cmd_deleg] invalid: {}", cmd.error.as_deref().unwrap_or("?"));
        }
    }

    cmds
}

fn validate_projects(state: &AppState, projects: &[String]) -> (bool, Option<String>) {
    for p in projects {
        if state.validate_project_name_from_llm(p).is_none() {
            return (false, Some(format!("Unknown project: {}", p)));
        }
    }
    (true, None)
}
