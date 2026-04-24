//! Unified PA command parsing with validation.
//! Replaces 7 inline regex parsers in chat.rs with a single validated pipeline.

use crate::state::AppState;
use std::sync::LazyLock;

static RE_DELEGATE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DELEGATE:([^\]]+)\]\s*\n?(.*?)\n?\[/DELEGATE\]").unwrap()
});
static RE_DEPLOY: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DEPLOY:([^\]]+)\]").unwrap());
static RE_HEALTH: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[HEALTH_CHECK:([^\]]+)\]").unwrap());
static RE_PLAN: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?s)\[PLAN:([^\]]+)\](.*?)\[/PLAN\]").unwrap());
static RE_QUEUE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[QUEUE:([^\]]+)\]").unwrap());
static RE_NOTIFY: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[NOTIFY:([^\]]+)\]").unwrap());
static RE_REMEMBER: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[REMEMBER:([^\]]+)\]").unwrap());
static RE_STRATEGY: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?s)\[STRATEGY:([^\]]+)\](.*?)\[/STRATEGY\]").unwrap());

#[derive(Debug, Clone)]
pub enum PaCommand {
    Delegate {
        project: String,
        task: String,
    },
    Deploy {
        project: String,
    },
    HealthCheck {
        target: String,
    },
    Plan {
        title: String,
        steps: Vec<(String, String)>,
    },
    Queue {
        task: String,
    },
    Notify {
        message: String,
    },
    Remember {
        note: String,
    },
    Strategy {
        goal: String,
        context: String,
    },
    DelegExt(super::pa_commands_deleg::DelegPaCommand),
    OpsExt(super::pa_commands_ops::OpsPaCommand),
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub cmd: PaCommand,
    pub valid: bool,
    pub error: Option<String>,
}

/// Parse and validate all PA commands from response text.
/// Returns commands in order of appearance with validation status.
pub fn parse_pa_commands(response: &str, state: &AppState) -> Vec<ParsedCommand> {
    let response = command_scan_text(response);
    if response.trim().is_empty() {
        return Vec::new();
    }
    let mut commands = Vec::new();

    // Delegations (multiple)
    for caps in RE_DELEGATE.captures_iter(&response) {
        let project = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let task = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        let (valid, error) = validate_delegation(state, &project, &task);
        commands.push(ParsedCommand {
            cmd: PaCommand::Delegate { project, task },
            valid,
            error,
        });
    }

    // Deploy
    if let Some(caps) = RE_DEPLOY.captures(&response) {
        let project = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let (valid, error) = validate_project_ref(state, &project);
        commands.push(ParsedCommand {
            cmd: PaCommand::Deploy { project },
            valid,
            error,
        });
    }

    // Health Check
    if let Some(caps) = RE_HEALTH.captures(&response) {
        let target = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let (valid, error) = if target == "all" {
            (true, None)
        } else {
            validate_project_ref(state, &target)
        };
        commands.push(ParsedCommand {
            cmd: PaCommand::HealthCheck { target },
            valid,
            error,
        });
    }

    // Plan
    if let Some(caps) = RE_PLAN.captures(&response) {
        let title = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let body = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        let steps: Vec<(String, String)> = body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let l = l
                    .trim()
                    .trim_start_matches(|c: char| "-*0123456789. ".contains(c));
                if l.contains(':') {
                    let (proj, task) = l.split_once(':')?;
                    Some((proj.trim().to_string(), task.trim().to_string()))
                } else {
                    Some(("_orchestrator".to_string(), l.to_string()))
                }
            })
            .collect();

        let (valid, error) = if title.is_empty() || steps.is_empty() {
            (
                false,
                Some("Plan requires title and at least one step".to_string()),
            )
        } else {
            (true, None)
        };
        commands.push(ParsedCommand {
            cmd: PaCommand::Plan { title, steps },
            valid,
            error,
        });
    }

    // Queue
    if let Some(caps) = RE_QUEUE.captures(&response) {
        let task = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let (valid, error) = if task.is_empty() {
            (false, Some("Empty queue task".to_string()))
        } else {
            (true, None)
        };
        commands.push(ParsedCommand {
            cmd: PaCommand::Queue { task },
            valid,
            error,
        });
    }

    // Notify
    if let Some(caps) = RE_NOTIFY.captures(&response) {
        let message = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        commands.push(ParsedCommand {
            cmd: PaCommand::Notify { message },
            valid: true,
            error: None,
        });
    }

    // Remember
    if let Some(caps) = RE_REMEMBER.captures(&response) {
        let note = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let (valid, error) = if note.is_empty() {
            (false, Some("Empty memory note".to_string()))
        } else {
            (true, None)
        };
        commands.push(ParsedCommand {
            cmd: PaCommand::Remember { note },
            valid,
            error,
        });
    }

    // Strategy
    if let Some(caps) = RE_STRATEGY.captures(&response) {
        let goal = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let context = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let (valid, error) = if goal.is_empty() {
            (false, Some("Empty strategy goal".to_string()))
        } else {
            (true, None)
        };
        commands.push(ParsedCommand {
            cmd: PaCommand::Strategy { goal, context },
            valid,
            error,
        });
    }

    // Extended delegation commands (DELEGATE_BATCH, CHAIN, RETRY, etc.)
    commands.extend(super::pa_commands_deleg::parse_delegation_commands(
        &response, state,
    ));
    // Ops commands (DEPLOY, GIT, MEMORY, CRON, etc.)
    commands.extend(super::pa_commands_ops::parse_ops_commands(&response, state));

    // Log all parsed commands
    for cmd in &commands {
        if cmd.valid {
            crate::log_info!(
                "[pa_cmd] parsed valid: {:?}",
                std::mem::discriminant(&cmd.cmd)
            );
        } else {
            crate::log_warn!(
                "[pa_cmd] invalid command: {:?} error={}",
                std::mem::discriminant(&cmd.cmd),
                cmd.error.as_deref().unwrap_or("?")
            );
        }
    }

    commands
}

fn command_scan_text(response: &str) -> String {
    let trimmed = response.trim_start();
    if trimmed.starts_with("Error:") || response.contains("\"type\":\"error\"") {
        return String::new();
    }

    let mut text = strip_fenced_code_blocks(response);
    for block in [
        "IDENTITY",
        "PROJECTS",
        "CATEGORIES",
        "DELEGATIONS",
        "STRATEGIES",
        "QUEUE",
        "YOUR MEMORY",
        "RECENT CONVERSATION",
        "USER MESSAGE",
    ] {
        text = strip_named_context_block(&text, block);
    }
    text
}

fn strip_named_context_block(input: &str, block: &str) -> String {
    let start_marker = format!("[{}", block);
    let end_marker = format!("[END {}]", block);
    let mut out = input.to_string();

    loop {
        let Some(start) = out.find(&start_marker) else {
            break;
        };
        let Some(relative_end) = out[start..].find(&end_marker) else {
            break;
        };
        let end = start + relative_end + end_marker.len();
        out.replace_range(start..end, "");
    }

    out
}

fn strip_fenced_code_blocks(input: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;

    for line in input.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

fn validate_delegation(state: &AppState, project: &str, task: &str) -> (bool, Option<String>) {
    if project.is_empty() {
        return (false, Some("Empty project name".to_string()));
    }
    if task.trim().is_empty() {
        return (false, Some("Empty delegation task".to_string()));
    }
    if task.len() > 5000 {
        return (false, Some("Task too long (>5000 chars)".to_string()));
    }
    match state.validate_project_name_from_llm(project) {
        Some(_) => (true, None),
        None => (false, Some(format!("Unknown project: {}", project))),
    }
}

fn validate_project_ref(state: &AppState, project: &str) -> (bool, Option<String>) {
    if project.is_empty() {
        return (false, Some("Empty project name".to_string()));
    }
    if state.validate_project(project).is_ok() {
        (true, None)
    } else {
        (false, Some(format!("Unknown project: {}", project)))
    }
}

/// Execute a single validated PA command. Returns optional text to append to response.
/// Used by both chat.rs (sync) and chat_stream.rs (stream) to avoid duplication.
pub fn execute_pa_command(state: &AppState, cmd: &PaCommand) -> Option<String> {
    match cmd {
        PaCommand::Delegate { project, task } => {
            if let Some(valid_name) = state.validate_project_name_from_llm(project) {
                let did = super::delegation::queue_delegation_internal(state, &valid_name, task);
                if !did.is_empty() {
                    return Some(format!(
                        "<delegation id=\"{}\" project=\"{}\"/>",
                        did, valid_name
                    ));
                }
            }
            None
        }
        PaCommand::Deploy { project } => {
            if state.validate_project(project).is_ok() {
                let result =
                    super::ops::execute_deploy_inline(&state.root, &state.docs_dir, project);
                return Some(format!("**Deploy {}:** {}", project, result));
            }
            None
        }
        PaCommand::HealthCheck { target } => {
            let result = super::ops::execute_health_inline(&state.docs_dir, target);
            Some(format!("**Health Check:**\n{}", result))
        }
        PaCommand::Plan { title, steps } => {
            // Create Strategy instead of standalone Plan — unified pipeline with auto-queue
            use super::strategy_models::*;
            use std::collections::HashMap;

            let id = format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            );
            // Goal ID: supports unicode/cyrillic (#10), fallback to timestamp if empty
            let goal_slug: String = title
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .take(50)
                .collect();
            let goal_id = if goal_slug.is_empty() {
                format!("plan-{}", &id)
            } else {
                goal_slug
            };

            // Group steps by project → Strategy Plans
            let mut by_project: Vec<(String, Vec<(usize, String)>)> = Vec::new();
            let mut proj_idx: HashMap<String, usize> = HashMap::new();
            for (idx, (proj, task)) in steps.iter().enumerate() {
                if let Some(&pi) = proj_idx.get(proj) {
                    by_project[pi].1.push((idx, task.clone()));
                } else {
                    proj_idx.insert(proj.clone(), by_project.len());
                    by_project.push((proj.clone(), vec![(idx, task.clone())]));
                }
            }

            let plans: Vec<Plan> = by_project
                .into_iter()
                .map(|(proj, tasks)| Plan {
                    project: proj.clone(),
                    steps: tasks
                        .iter()
                        .map(|(i, task)| Step {
                            id: format!("{}-{}", proj, i),
                            task: task.clone(),
                            status: super::status::StepStatus::Pending,
                            response: None,
                            depends_on: vec![],
                            delegation_id: None,
                            assignee: Assignee::default(),
                            verify: None,
                        })
                        .collect(),
                    priority: "MED".to_string(),
                    depends_on: vec![],
                    category: None,
                    context: String::new(),
                })
                .collect();

            let plan_count = plans.len();
            let strategy = Strategy {
                id: id.clone(),
                goal_id,
                title: title.clone(),
                tactics: vec![], // ad-hoc: plans stored directly (wrapped by all_tactics())
                plans,
                status: super::status::StrategyStatus::Draft,
                created: state.now_iso(),
                room_session_id: None,
                category: None,
                deadline: None,
                metrics: None,
            };

            let mut all = load_strategies(state);
            all.push(strategy);
            save_strategies(state, &all);
            crate::log_info!(
                "[pa_cmd] plan→strategy created: '{}' with {} plans",
                title,
                plan_count
            );

            Some(format!("Strategy '{}' created with {} project plans. Approve steps in Strategy view to execute.", title, plan_count))
        }
        PaCommand::Queue { task } => {
            let queue_path = state.root.join("tasks").join("queue.md");
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&queue_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "- [ ] {}", task)
                });
            Some(format!("Queued: {}", task))
        }
        PaCommand::Notify { message } => {
            crate::log_info!(
                "[pa_cmd] notification: {}",
                message.chars().take(80).collect::<String>()
            );
            None
        }
        PaCommand::Remember { note } => {
            let mem_path = state.root.join("tasks").join("pa-memory.jsonl");
            super::jsonl::append_jsonl_logged(
                &mem_path,
                &serde_json::json!({"ts": state.now_iso(), "note": note}),
                "PA memory",
            );
            None
        }
        PaCommand::DelegExt(cmd) => super::delegation_ext::execute_deleg_command(state, cmd),
        PaCommand::OpsExt(cmd) => super::pa_commands_ops::execute_ops_command(state, cmd),
        PaCommand::Strategy { goal, context } => {
            super::strategy::create_strategy_from_command(state, &goal, &context)
        }
    }
}

/// Check if response looks like it tried to use a command but failed parsing
// build_tactic_from_steps moved to strategy.rs

pub fn detect_malformed_commands(response: &str) -> Vec<String> {
    let response = command_scan_text(response);
    if response.trim().is_empty() {
        return Vec::new();
    }
    let mut warnings = Vec::new();
    if response.to_lowercase().contains("delegate")
        && response.contains('[')
        && !RE_DELEGATE.is_match(&response)
    {
        warnings.push(
            "Delegation not parsed. Use format: [DELEGATE:ProjectName]task[/DELEGATE]".to_string(),
        );
    }
    if response.contains("[PLAN:") && !RE_PLAN.is_match(&response) {
        warnings.push("Plan not parsed. Use format: [PLAN:title]steps[/PLAN]".to_string());
    }
    if response.contains("[STRATEGY:") && !RE_STRATEGY.is_match(&response) {
        warnings
            .push("Strategy not parsed. Use format: [STRATEGY:goal]context[/STRATEGY]".to_string());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    fn test_state(name: &str) -> AppState {
        let root = std::env::temp_dir().join(format!(
            "agentos-pa-commands-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(root.join("tasks"));
        AppState::new(root)
    }

    #[test]
    fn provider_error_echoing_identity_does_not_parse_commands() {
        let state = test_state("provider-error");
        let response = r#"Error: OpenAI Codex v0.121.0
user
[IDENTITY]
[DELEGATE:Project]task[/DELEGATE]
[CRON_CREATE:name:schedule]task[/CRON_CREATE]
[INCOME_RECORD:0:category]description[/INCOME_RECORD]
[END IDENTITY]
ERROR: {"type":"error","status":400}"#;

        assert!(parse_pa_commands(response, &state).is_empty());
        assert!(detect_malformed_commands(response).is_empty());
    }

    #[test]
    fn identity_context_is_ignored_by_command_parser() {
        let state = test_state("identity");
        let response = r#"[IDENTITY]
[DELEGATE:Project]task[/DELEGATE]
[QUEUE:task]
[END IDENTITY]"#;

        assert!(parse_pa_commands(response, &state).is_empty());
    }

    #[test]
    fn real_queue_command_still_parses() {
        let state = test_state("real-command");
        let commands = parse_pa_commands("[QUEUE:ship the fix]", &state);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].valid);
        assert!(matches!(commands[0].cmd, PaCommand::Queue { .. }));
    }

    #[test]
    fn fenced_command_examples_are_ignored() {
        let state = test_state("fenced");
        let response = "```text\n[QUEUE:example]\n```\nNo action.";

        assert!(parse_pa_commands(response, &state).is_empty());
    }
}
