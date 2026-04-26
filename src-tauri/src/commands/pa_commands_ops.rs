//! PA command parsing for Waves 2-4: Deploy, Git, Memory, Cron, Comms.
//! Parsed here, executed via deploy.rs, git_ops.rs, memory_ext.rs, cron.rs.

use super::pa_commands::ParsedCommand;
use crate::state::AppState;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub enum OpsPaCommand {
    // Wave 2: Deploy & Server
    DeployStatic {
        project: String,
        target: String,
        source: String,
    },
    DeployVerify {
        url: String,
        expected: String,
    },
    DeployRollback {
        target: String,
    },
    ServerExec {
        host: String,
        command: String,
    },
    ServerStatus {
        host: String,
    },
    NginxValidate {
        host: String,
    },
    SslMonitor,
    DnsVerify {
        domain: String,
    },
    // Wave 3: Git
    GitBulkPush {
        filter: String,
    },
    GitBulkPull {
        filter: String,
    },
    GitStatusAll,
    GitStaleBranches {
        days: u64,
    },
    GitSearch {
        mode: String,
        query: String,
    },
    TemplateAudit,
    // Wave 4: Memory
    MemoryList {
        filter: String,
    },
    MemorySearch {
        query: String,
    },
    MemoryDelete {
        filter: String,
    },
    // Wave 3 continued
    DependencyAudit {
        filter: String,
    },
    // Wave 4: Strategy
    StrategyProgress {
        filter: String,
    },
    StrategyMilestone {
        strategy: String,
        milestone: String,
        evidence: String,
    },
    // Wave 4: Cron
    CronCreate {
        name: String,
        schedule: String,
        task: String,
    },
    CronList,
    CronEdit {
        name: String,
        schedule: String,
        task: String,
    },
    CronDelete {
        name: String,
    },
    // Wave 4: Comms
    DailyReport,
    DashboardFull,
    ActivityDigest {
        period: String,
    },
    AlertCreate {
        name: String,
        body: String,
    },
    PartnerUpdate {
        message: String,
    },
    // Wave 4: Financial
    IncomeRecord {
        amount: f64,
        category: String,
        description: String,
    },
    FinancialDashboard,
    // Graph queries (Agent Protocol)
    GraphContext {
        project: String,
    },
    GraphDependents {
        project: String,
        file: String,
    },
    GraphImpact {
        project: String,
        file: String,
    },
    GraphVerify {
        project: String,
    },
    GraphRules {
        project: String,
    },
}

static RE_DEPLOY_STATIC: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DEPLOY_STATIC:([^:]+):([^\]]+)\]\s*\n?(.*?)\n?\[/DEPLOY_STATIC\]")
        .unwrap()
});
static RE_DEPLOY_VERIFY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[DEPLOY_VERIFY:([^\]]+)\]\s*\n?(.*?)\n?\[/DEPLOY_VERIFY\]").unwrap()
});
static RE_DEPLOY_ROLLBACK: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DEPLOY_ROLLBACK:([^\]]+)\]").unwrap());
static RE_SERVER_EXEC: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[SERVER_EXEC:([^\]]+)\]\s*\n?(.*?)\n?\[/SERVER_EXEC\]").unwrap()
});
static RE_SERVER_STATUS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[SERVER_STATUS:([^\]]+)\]").unwrap());
static RE_NGINX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[NGINX_VALIDATE:([^\]]+)\]").unwrap());
static RE_SSL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[SSL_MONITOR\]").unwrap());
static RE_DNS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DNS_VERIFY:([^\]]+)\]").unwrap());
static RE_GIT_PUSH: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GIT_BULK_PUSH(?::([^\]]*))?\]").unwrap());
static RE_GIT_PULL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GIT_BULK_PULL(?::([^\]]*))?\]").unwrap());
static RE_GIT_STATUS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GIT_STATUS_ALL\]").unwrap());
static RE_GIT_STALE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GIT_STALE_BRANCHES(?::(\d+))?\]").unwrap());
static RE_GIT_SEARCH: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GIT_SEARCH:(commit|code|file):([^\]]+)\]").unwrap());
static RE_TMPL_AUDIT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[TEMPLATE_AUDIT\]").unwrap());
static RE_MEM_LIST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[MEMORY_LIST(?::([^\]]*))?\]").unwrap());
static RE_MEM_SEARCH: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[MEMORY_SEARCH:([^\]]+)\]").unwrap());
static RE_MEM_DELETE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[MEMORY_DELETE:([^\]]+)\]").unwrap());
static RE_CRON_CREATE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[CRON_CREATE:([^:]+):([^\]]+)\]\s*\n?(.*?)\n?\[/CRON_CREATE\]")
        .unwrap()
});
static RE_CRON_LIST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[CRON_LIST\]").unwrap());
static RE_CRON_EDIT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[CRON_EDIT:([^:]+):([^\]]+)\]\s*\n?(.*?)\n?\[/CRON_EDIT\]").unwrap()
});
static RE_CRON_DELETE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[CRON_DELETE:([^\]]+)\]").unwrap());
static RE_DEP_AUDIT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DEPENDENCY_AUDIT(?::([^\]]*))?\]").unwrap());
static RE_STRAT_PROG: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[STRATEGY_PROGRESS(?::([^\]]*))?\]").unwrap());
static RE_STRAT_MILE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?s)\[STRATEGY_MILESTONE:([^:]+):([^\]]+)\]\s*\n?(.*?)\n?\[/STRATEGY_MILESTONE\]",
    )
    .unwrap()
});
static RE_DAILY: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DAILY_REPORT\]").unwrap());
static RE_DASHBOARD: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[DASHBOARD_FULL\]").unwrap());
static RE_DIGEST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[ACTIVITY_DIGEST(?::([^\]]*))?\]").unwrap());
static RE_ALERT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[ALERT_CREATE:([^\]]+)\]\s*\n?(.*?)\n?\[/ALERT_CREATE\]").unwrap()
});
static RE_PARTNER: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[PARTNER_UPDATE\]\s*\n?(.*?)\n?\[/PARTNER_UPDATE\]").unwrap()
});
static RE_INCOME: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)\[INCOME_RECORD:([^:]+):([^\]]+)\]\s*\n?(.*?)\n?\[/INCOME_RECORD\]")
        .unwrap()
});
static RE_FINANCIAL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[FINANCIAL_DASHBOARD\]").unwrap());
static RE_GRAPH_CTX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GRAPH_CONTEXT:([^\]]+)\]").unwrap());
static RE_GRAPH_DEPS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GRAPH_DEPENDENTS:([^:]+):([^\]]+)\]").unwrap());
static RE_GRAPH_IMPACT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GRAPH_IMPACT:([^:]+):([^\]]+)\]").unwrap());
static RE_GRAPH_VERIFY: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GRAPH_VERIFY:([^\]]+)\]").unwrap());
static RE_GRAPH_RULES: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[GRAPH_RULES:([^\]]+)\]").unwrap());

pub fn parse_ops_commands(response: &str, _state: &AppState) -> Vec<ParsedCommand> {
    let mut cmds = Vec::new();
    macro_rules! push_cmd {
        ($cmd:expr) => {
            cmds.push(ParsedCommand {
                cmd: super::pa_commands::PaCommand::OpsExt($cmd),
                valid: true,
                error: None,
            });
        };
    }

    if let Some(c) = RE_DEPLOY_STATIC.captures(response) {
        push_cmd!(OpsPaCommand::DeployStatic {
            project: cap(c.get(1)),
            target: cap(c.get(2)),
            source: cap(c.get(3))
        });
    }
    if let Some(c) = RE_DEPLOY_VERIFY.captures(response) {
        push_cmd!(OpsPaCommand::DeployVerify {
            url: cap(c.get(1)),
            expected: cap(c.get(2))
        });
    }
    if let Some(c) = RE_DEPLOY_ROLLBACK.captures(response) {
        push_cmd!(OpsPaCommand::DeployRollback {
            target: cap(c.get(1))
        });
    }
    if let Some(c) = RE_SERVER_EXEC.captures(response) {
        push_cmd!(OpsPaCommand::ServerExec {
            host: cap(c.get(1)),
            command: cap(c.get(2))
        });
    }
    if let Some(c) = RE_SERVER_STATUS.captures(response) {
        push_cmd!(OpsPaCommand::ServerStatus {
            host: cap(c.get(1))
        });
    }
    if let Some(c) = RE_NGINX.captures(response) {
        push_cmd!(OpsPaCommand::NginxValidate {
            host: cap(c.get(1))
        });
    }
    if RE_SSL.is_match(response) {
        push_cmd!(OpsPaCommand::SslMonitor);
    }
    if let Some(c) = RE_DNS.captures(response) {
        push_cmd!(OpsPaCommand::DnsVerify {
            domain: cap(c.get(1))
        });
    }
    if let Some(c) = RE_GIT_PUSH.captures(response) {
        push_cmd!(OpsPaCommand::GitBulkPush {
            filter: cap(c.get(1))
        });
    }
    if let Some(c) = RE_GIT_PULL.captures(response) {
        push_cmd!(OpsPaCommand::GitBulkPull {
            filter: cap(c.get(1))
        });
    }
    if RE_GIT_STATUS.is_match(response) {
        push_cmd!(OpsPaCommand::GitStatusAll);
    }
    if let Some(c) = RE_GIT_STALE.captures(response) {
        push_cmd!(OpsPaCommand::GitStaleBranches {
            days: c.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(30)
        });
    }
    if let Some(c) = RE_GIT_SEARCH.captures(response) {
        push_cmd!(OpsPaCommand::GitSearch {
            mode: cap(c.get(1)),
            query: cap(c.get(2))
        });
    }
    if RE_TMPL_AUDIT.is_match(response) {
        push_cmd!(OpsPaCommand::TemplateAudit);
    }
    if let Some(c) = RE_MEM_LIST.captures(response) {
        push_cmd!(OpsPaCommand::MemoryList {
            filter: cap(c.get(1))
        });
    }
    if let Some(c) = RE_MEM_SEARCH.captures(response) {
        push_cmd!(OpsPaCommand::MemorySearch {
            query: cap(c.get(1))
        });
    }
    if let Some(c) = RE_MEM_DELETE.captures(response) {
        push_cmd!(OpsPaCommand::MemoryDelete {
            filter: cap(c.get(1))
        });
    }
    if let Some(c) = RE_CRON_CREATE.captures(response) {
        push_cmd!(OpsPaCommand::CronCreate {
            name: cap(c.get(1)),
            schedule: cap(c.get(2)),
            task: cap(c.get(3))
        });
    }
    if RE_CRON_LIST.is_match(response) {
        push_cmd!(OpsPaCommand::CronList);
    }
    if let Some(c) = RE_CRON_EDIT.captures(response) {
        push_cmd!(OpsPaCommand::CronEdit {
            name: cap(c.get(1)),
            schedule: cap(c.get(2)),
            task: cap(c.get(3))
        });
    }
    if let Some(c) = RE_CRON_DELETE.captures(response) {
        push_cmd!(OpsPaCommand::CronDelete {
            name: cap(c.get(1))
        });
    }
    if let Some(c) = RE_DEP_AUDIT.captures(response) {
        push_cmd!(OpsPaCommand::DependencyAudit {
            filter: cap(c.get(1))
        });
    }
    if let Some(c) = RE_STRAT_PROG.captures(response) {
        push_cmd!(OpsPaCommand::StrategyProgress {
            filter: cap(c.get(1))
        });
    }
    if let Some(c) = RE_STRAT_MILE.captures(response) {
        push_cmd!(OpsPaCommand::StrategyMilestone {
            strategy: cap(c.get(1)),
            milestone: cap(c.get(2)),
            evidence: cap(c.get(3))
        });
    }
    if RE_DAILY.is_match(response) {
        push_cmd!(OpsPaCommand::DailyReport);
    }
    if RE_DASHBOARD.is_match(response) {
        push_cmd!(OpsPaCommand::DashboardFull);
    }
    if let Some(c) = RE_DIGEST.captures(response) {
        let p = cap(c.get(1));
        push_cmd!(OpsPaCommand::ActivityDigest {
            period: if p.is_empty() { "24h".to_string() } else { p }
        });
    }
    if let Some(c) = RE_ALERT.captures(response) {
        push_cmd!(OpsPaCommand::AlertCreate {
            name: cap(c.get(1)),
            body: cap(c.get(2))
        });
    }
    if let Some(c) = RE_PARTNER.captures(response) {
        push_cmd!(OpsPaCommand::PartnerUpdate {
            message: cap(c.get(1))
        });
    }
    if RE_PARTNER.is_match(response) && !RE_PARTNER.captures(response).is_some() { /* simple [PARTNER_UPDATE] */
    }
    if let Some(c) = RE_INCOME.captures(response) {
        push_cmd!(OpsPaCommand::IncomeRecord {
            amount: cap(c.get(1)).parse().unwrap_or(0.0),
            category: cap(c.get(2)),
            description: cap(c.get(3))
        });
    }
    if RE_FINANCIAL.is_match(response) {
        push_cmd!(OpsPaCommand::FinancialDashboard);
    }
    if let Some(c) = RE_GRAPH_CTX.captures(response) {
        push_cmd!(OpsPaCommand::GraphContext {
            project: cap(c.get(1))
        });
    }
    if let Some(c) = RE_GRAPH_DEPS.captures(response) {
        push_cmd!(OpsPaCommand::GraphDependents {
            project: cap(c.get(1)),
            file: cap(c.get(2))
        });
    }
    if let Some(c) = RE_GRAPH_IMPACT.captures(response) {
        push_cmd!(OpsPaCommand::GraphImpact {
            project: cap(c.get(1)),
            file: cap(c.get(2))
        });
    }
    if let Some(c) = RE_GRAPH_VERIFY.captures(response) {
        push_cmd!(OpsPaCommand::GraphVerify {
            project: cap(c.get(1))
        });
    }
    if let Some(c) = RE_GRAPH_RULES.captures(response) {
        push_cmd!(OpsPaCommand::GraphRules {
            project: cap(c.get(1))
        });
    }

    for cmd in &cmds {
        crate::log_info!(
            "[pa_cmd_ops] parsed: {:?}",
            std::mem::discriminant(&cmd.cmd)
        );
    }
    cmds
}

/// Execute an ops command
pub fn execute_ops_command(state: &AppState, cmd: &OpsPaCommand) -> Option<String> {
    match cmd {
        OpsPaCommand::DeployStatic {
            project,
            target,
            source,
        } => super::deploy::deploy_static(state, project, target, source),
        OpsPaCommand::DeployVerify { url, expected } => super::deploy::deploy_verify(url, expected),
        OpsPaCommand::DeployRollback { target } => super::deploy::deploy_rollback(target),
        OpsPaCommand::ServerExec { host, command } => super::deploy::server_exec(host, command),
        OpsPaCommand::ServerStatus { host } => super::deploy::server_status(host),
        OpsPaCommand::NginxValidate { host } => super::deploy::nginx_validate(host),
        OpsPaCommand::SslMonitor => {
            let cfg = state.config();
            let domains: Vec<String> = cfg
                .get("ssl_domains")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(|| {
                    vec![
                        "yokhanfitness.ru".to_string(),
                        "photo.yokhanfitness.ru".to_string(),
                        "app.yokhanfitness.ru".to_string(),
                    ]
                });
            let refs: Vec<&str> = domains.iter().map(|s| s.as_str()).collect();
            super::deploy::ssl_monitor(&refs)
        }
        OpsPaCommand::DnsVerify { domain } => super::deploy::dns_verify(domain),
        OpsPaCommand::GitBulkPush { filter } => super::git_ops::bulk_push(state, filter),
        OpsPaCommand::GitBulkPull { filter } => super::git_ops::bulk_pull(state, filter),
        OpsPaCommand::GitStatusAll => super::git_ops::status_all(state),
        OpsPaCommand::GitStaleBranches { days } => super::git_ops::stale_branches(state, *days),
        OpsPaCommand::GitSearch { mode, query } => super::git_ops::git_search(state, mode, query),
        OpsPaCommand::TemplateAudit => super::git_ops::template_audit(state),
        OpsPaCommand::MemoryList { filter } => super::memory_ext::memory_list(state, filter),
        OpsPaCommand::MemorySearch { query } => super::memory_ext::memory_search(state, query),
        OpsPaCommand::MemoryDelete { filter } => super::memory_ext::memory_delete(state, filter),
        OpsPaCommand::CronCreate {
            name,
            schedule,
            task,
        } => super::cron::cron_create(state, name, schedule, task),
        OpsPaCommand::CronList => super::cron::cron_list(state),
        OpsPaCommand::CronEdit {
            name,
            schedule,
            task,
        } => super::cron::cron_edit(state, name, schedule, task),
        OpsPaCommand::CronDelete { name } => super::cron::cron_delete(state, name),
        OpsPaCommand::DependencyAudit { filter } => super::git_ops::dependency_audit(state, filter),
        OpsPaCommand::StrategyProgress { filter } => {
            super::strategy_models::strategy_progress(state, filter)
        }
        OpsPaCommand::StrategyMilestone {
            strategy,
            milestone,
            evidence,
        } => super::strategy_models::strategy_milestone(state, strategy, milestone, evidence),
        OpsPaCommand::DailyReport => super::comms::daily_report(state),
        OpsPaCommand::DashboardFull => super::comms::dashboard_full(state),
        OpsPaCommand::ActivityDigest { period } => super::comms::activity_digest(state, period),
        OpsPaCommand::AlertCreate { name, body } => super::comms::alert_create(state, name, body),
        OpsPaCommand::PartnerUpdate { message } => super::comms::partner_update(state, message),
        OpsPaCommand::IncomeRecord {
            amount,
            category,
            description,
        } => super::financial::income_record(state, *amount, category, description),
        OpsPaCommand::FinancialDashboard => super::financial::financial_dashboard(state),
        OpsPaCommand::GraphContext { project } => {
            let ctx = super::graph_ops::build_graph_context(state, project);
            if ctx.is_empty() {
                Some("No graph data available for this project.".to_string())
            } else {
                Some(ctx)
            }
        }
        OpsPaCommand::GraphDependents { project, file } => {
            graph_query_dependents(state, project, file)
        }
        OpsPaCommand::GraphImpact { project, file } => graph_query_impact(state, project, file),
        OpsPaCommand::GraphRules { project } => {
            let graph = super::graph_scan::build_project_graph(state, project).ok()?;
            let mut rules = vec![
                "## Graph-Aware Development Rules".to_string(),
                String::new(),
            ];
            // Find hub modules (Ca > 3)
            let hubs: Vec<&super::graph_models::GraphNode> =
                graph.nodes.iter().filter(|n| n.metrics.ca >= 3).collect();
            if !hubs.is_empty() {
                rules.push(
                    "### Critical modules (3+ dependents — changes ripple widely):".to_string(),
                );
                for h in &hubs {
                    let deps: Vec<String> = graph
                        .edges
                        .iter()
                        .filter(|e| e.target == h.id && e.kind == "import")
                        .filter_map(|e| {
                            graph
                                .nodes
                                .iter()
                                .find(|n| n.id == e.source)
                                .map(|n| n.label.clone())
                        })
                        .collect();
                    rules.push(format!(
                        "- **{}** (Ca:{}) — depended on by: {}",
                        h.label,
                        h.metrics.ca,
                        deps.join(", ")
                    ));
                }
                rules.push(String::new());
            }
            if !graph.cycles.is_empty() {
                rules.push("### Circular dependencies (fix these):".to_string());
                for c in &graph.cycles {
                    rules.push(format!("- {}", c.join(" → ")));
                }
                rules.push(String::new());
            }
            rules.push("### Before modifying any exported type/function:".to_string());
            rules.push("1. Check dependents: which files import this module?".to_string());
            rules.push("2. Update all dependent files if signature changes.".to_string());
            rules.push("3. Run type checker (tsc/cargo check/mypy) after changes.".to_string());
            Some(rules.join("\n"))
        }
        OpsPaCommand::GraphVerify { project } => {
            let graph = super::graph_scan::build_project_graph(state, project).ok()?;
            let mut lines = vec![format!(
                "**Verify {}:** {} nodes, {} edges",
                project, graph.stats.total_nodes, graph.stats.total_edges
            )];
            if graph.cycles.is_empty() {
                lines.push("✓ No circular dependencies".to_string());
            } else {
                for cycle in &graph.cycles {
                    lines.push(format!("⚠ Cycle: {}", cycle.join(" → ")));
                }
            }
            let high_instab: Vec<&super::graph_models::GraphNode> = graph
                .nodes
                .iter()
                .filter(|n| n.metrics.instability > 0.8 && n.metrics.ca > 0)
                .collect();
            if !high_instab.is_empty() {
                lines.push(format!("ℹ {} high-instability modules", high_instab.len()));
            }
            Some(lines.join("\n"))
        }
    }
}

fn graph_query_dependents(state: &AppState, project: &str, file: &str) -> Option<String> {
    let graph = super::graph_scan::build_project_graph(state, project).ok()?;
    let file_lower = file.to_lowercase();
    let node = graph
        .nodes
        .iter()
        .find(|n| n.label.to_lowercase() == file_lower)
        .or_else(|| {
            graph
                .nodes
                .iter()
                .find(|n| n.id.to_lowercase().contains(&file_lower))
        })?;
    let dependents: Vec<String> = graph
        .edges
        .iter()
        .filter(|e| e.target == node.id && e.kind == "import")
        .filter_map(|e| {
            graph
                .nodes
                .iter()
                .find(|n| n.id == e.source)
                .map(|n| format!("  ← {} ({})", n.label, n.group.as_deref().unwrap_or("?")))
        })
        .collect();
    if dependents.is_empty() {
        return Some(format!("No dependents found for {} in {}", file, project));
    }
    Some(format!(
        "**Dependents of {} ({}):**\n{}\n\n⚠ If you modify {}, these {} files may need updating.",
        node.label,
        dependents.len(),
        dependents.join("\n"),
        node.label,
        dependents.len()
    ))
}

fn graph_query_impact(state: &AppState, project: &str, file: &str) -> Option<String> {
    let graph = super::graph_scan::build_project_graph(state, project).ok()?;
    let file_lower = file.to_lowercase();
    let node = graph
        .nodes
        .iter()
        .find(|n| n.label.to_lowercase() == file_lower)
        .or_else(|| {
            graph
                .nodes
                .iter()
                .find(|n| n.id.to_lowercase().contains(&file_lower))
        })?;

    // BFS for transitive dependents
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    let mut impact: Vec<(String, u32)> = Vec::new(); // (label, depth)
    queue.push_back((node.id.clone(), 0u32));
    visited.insert(node.id.clone());

    while let Some((current, depth)) = queue.pop_front() {
        if depth > 0 {
            if let Some(n) = graph.nodes.iter().find(|n| n.id == current) {
                impact.push((n.label.clone(), depth));
            }
        }
        if depth >= 3 {
            continue;
        } // max 3 hops
        for edge in graph
            .edges
            .iter()
            .filter(|e| e.target == current && e.kind == "import")
        {
            if !visited.contains(&edge.source) {
                visited.insert(edge.source.clone());
                queue.push_back((edge.source.clone(), depth + 1));
            }
        }
    }

    // Deduplicate by keeping first (shallowest) occurrence
    let mut seen = std::collections::HashSet::new();
    impact.retain(|(label, _)| seen.insert(label.clone()));

    if impact.is_empty() {
        return Some(format!("No transitive impact from {} in {}", file, project));
    }
    let mut lines = vec![format!("**Impact analysis for {} (depth 3):**", node.label)];
    for d in 1..=3 {
        let at_depth: Vec<&str> = impact
            .iter()
            .filter(|(_, dd)| *dd == d)
            .map(|(l, _)| l.as_str())
            .collect();
        if !at_depth.is_empty() {
            lines.push(format!("  Depth {}: {}", d, at_depth.join(", ")));
        }
    }
    lines.push(format!(
        "\nTotal: {} modules may be affected.",
        impact.len()
    ));
    Some(lines.join("\n"))
}

fn cap(m: Option<regex::Match>) -> String {
    m.map(|m| m.as_str().trim().to_string()).unwrap_or_default()
}
