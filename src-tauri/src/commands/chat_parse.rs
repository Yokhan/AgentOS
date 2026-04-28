//! PA prompt builder: identity, project context, categories, delegations, memory.

use crate::state::AppState;

/// Build full PA prompt: system context + chat history + delegation status + user message.
/// PA sees everything it needs to make decisions.
pub fn build_full_pa_prompt(state: &AppState, user_message: &str) -> String {
    let policy = build_response_policy_context(user_message);
    let template_policy = build_agent_template_context(state);
    let identity = build_identity_context(state);
    let context = build_project_context(state);
    let categories = super::category::build_category_context(state);
    let history = build_chat_history(state);
    let deleg_status = build_delegation_status(state);
    let strategies = super::strategy_models::build_strategy_context(state);
    let queue = build_queue_context(state);
    let gates = super::gate::build_gate_context(state);
    let signals = super::signals::build_signals_context(state);
    let memory = build_pa_memory(state);

    format!(
        "{policy}\n{template_policy}\n{identity}\n{context}\n{categories}\n{deleg_status}\n{strategies}\n{gates}\n{signals}\n{queue}\n{memory}\n{history}\n[USER MESSAGE]\n{user_message}",
    )
}

pub fn build_response_policy_context(user_message: &str) -> String {
    let target_language = if user_message
        .chars()
        .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
    {
        "Russian"
    } else {
        "the user's language"
    };

    format!(
        "[RESPONSE POLICY]\n\
         - Answer in {target_language}. Match the user's language for all user-facing prose.\n\
         - If the user writes in Cyrillic/Russian, reply in Russian even when AgentOS context, command output, model names, or PA command tags are English.\n\
         - Keep PA command tags, file paths, model names, and literal UI labels exact; do not translate command syntax.\n\
         - Lead with the practical result. Be concise, concrete, and avoid build-log narration unless it changes a decision.\n\
         - Treat this policy as the user-facing chat contract for orchestrator, duo, and auto-continue turns.\n\
         [END RESPONSE POLICY]\n"
    )
}

fn build_agent_template_context(state: &AppState) -> String {
    let candidates = vec![
        state.root.join("AGENTS.md"),
        state.root.join("CLAUDE.md"),
        state
            .docs_dir
            .join("agent-project-template")
            .join("AGENTS.md"),
        state
            .docs_dir
            .join("agent-project-template")
            .join("CLAUDE.md"),
    ];
    let mut chunks = Vec::new();
    for path in candidates {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut selected = Vec::new();
        for heading in ["## Philosophy", "## Work Report Style", "## DON'T"] {
            if let Some(section) = extract_template_section(&content, heading) {
                selected.push(section);
            }
        }
        if selected.is_empty() {
            continue;
        }
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent instructions");
        let body: String = selected.join("\n\n").chars().take(3500).collect();
        chunks.push(format!("Source: {}\n{}", label, body));
        if chunks.len() >= 2 {
            break;
        }
    }
    if chunks.is_empty() {
        return String::new();
    }
    format!(
        "[AGENT TEMPLATE POLICY]\n{}\n[END AGENT TEMPLATE POLICY]\n",
        chunks.join("\n\n---\n")
    )
}

fn extract_template_section(content: &str, heading: &str) -> Option<String> {
    let start = content.find(heading)?;
    let rest = &content[start..];
    let end = rest[heading.len()..]
        .find("\n## ")
        .map(|idx| heading.len() + idx)
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn build_identity_context(state: &AppState) -> String {
    let project_count = {
        let cache = state.scan_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .data
            .as_ref()
            .and_then(|d| {
                d.get("agents")
                    .and_then(|a| a.as_array())
                    .or_else(|| d.as_array())
            })
            .map(|a| a.len())
            .unwrap_or(0)
    };
    let seg_count = state.segments.lock().map(|s| s.len()).unwrap_or(0);
    let active_deleg = state
        .delegations
        .lock()
        .map(|d| d.values().filter(|v| !v.status.is_terminal()).count())
        .unwrap_or(0);
    let _active_plans = super::plans::load_all_plans_internal(state)
        .iter()
        .filter(|p| p.status == crate::commands::status::PlanStatus::Active)
        .count();
    let uptime_min = state.uptime_secs() / 60;

    format!(
        "[IDENTITY]\n\
         You are PA Orchestrator inside AgentOS (Tauri desktop app).\n\
         You manage {} projects across {} categories.\n\n\
         YOUR CAPABILITIES (51 commands):\n\n\
         Delegation:\n\
         Pending delegations are approval requests, not running work. Report the approval need once, then continue independent routes if available; only [DELEGATE_STATUS:?stale] means the queue is actually stuck.\n\
         [DELEGATE:Project]task[/DELEGATE] — send task to agent (user approves → L1→L2→L3)\n\
         [DELEGATE_BATCH:p1,p2]task[/DELEGATE_BATCH] — batch to multiple projects\n\
         [DELEGATE_CHAIN:Project]step1\\nstep2[/DELEGATE_CHAIN] — sequential chain\n\
         [DELEGATE_RETRY:id]context[/DELEGATE_RETRY] — retry failed with error context\n\
         [DELEGATE_CANCEL:id] — cancel running/pending\n\
         [DELEGATE_STATUS:filter] — query status (?failed/?stale/project/id)\n\
         [DELEGATE_CLEANUP:hours] — archive old terminal delegations\n\
         [DELEGATE_PRIORITY:id:HIGH|MED|LOW] — set priority\n\
         [DELEGATE_TIMEOUT:id:seconds] — per-delegation timeout\n\
         [DELEGATE_TEMPLATE:save:name]task[/DELEGATE_TEMPLATE] — save template\n\
         [DELEGATE_TEMPLATE:use:name:p1,p2] — apply template\n\
         [DELEGATE_LOG:filter] — history (?today/?failed/project)\n\
         [DELEGATE_DIFF:filter] — aggregate git diffs\n\n\
         Route-aware work items:\n\
         [WORK_ITEM_QUEUE:id] - queue an existing ready work item without losing plan/route linkage\n\n\
         Planning:\n\
         [PLAN:title]Project: task\\n...[/PLAN] — create tracked plan\n\
         [STRATEGY:goal]context[/STRATEGY] — multi-project strategy\n\
         [STRATEGY_PROGRESS:filter] — check completion %\n\
         [STRATEGY_MILESTONE:strategy:name]evidence[/STRATEGY_MILESTONE]\n\n\
         Deploy & Server:\n\
         [DEPLOY:Project] — sync template\n\
         [DEPLOY_STATIC:project:target]source[/DEPLOY_STATIC] — SCP to VPS\n\
         [DEPLOY_VERIFY:url]expected[/DEPLOY_VERIFY] — HTTP check\n\
         [DEPLOY_ROLLBACK:target] — restore backup\n\
         [SERVER_EXEC:host]command[/SERVER_EXEC] — SSH command\n\
         [SERVER_STATUS:host] — disk/memory/uptime\n\
         [NGINX_VALIDATE:host] — test nginx config\n\
         [SSL_MONITOR] — cert expiry all domains\n\
         [DNS_VERIFY:domain] — A/AAAA/CNAME/MX records\n\n\
         Git:\n\
         [GIT_BULK_PUSH:filter] — push all\n\
         [GIT_BULK_PULL:filter] — pull --ff-only all\n\
         [GIT_STATUS_ALL] — branch+dirty across repos\n\
         [GIT_STALE_BRANCHES:days] — old branches\n\
         [GIT_SEARCH:commit|code|file:query] — cross-repo search\n\
         [TEMPLATE_AUDIT] — template versions\n\
         [DEPENDENCY_AUDIT:project] — outdated deps\n\n\
         Scheduling:\n\
         [CRON_CREATE:name:schedule]task[/CRON_CREATE]\n\
         [CRON_LIST] [CRON_EDIT:name:schedule]task[/CRON_EDIT] [CRON_DELETE:name]\n\n\
         Memory:\n\
         [REMEMBER:note] [MEMORY_LIST:filter] [MEMORY_SEARCH:query] [MEMORY_DELETE:filter]\n\n\
         Communication:\n\
         [DAILY_REPORT] [DASHBOARD_FULL] [ACTIVITY_DIGEST:period]\n\
         [ALERT_CREATE:name]Check:\\nCondition:\\nAction:[/ALERT_CREATE]\n\
         [PARTNER_UPDATE]message[/PARTNER_UPDATE]\n\
         [NOTIFY:message] [QUEUE:task] [HEALTH_CHECK:Project|all]\n\n\
         Graph (Agent Protocol):\n\
         [GRAPH_CONTEXT:project] — get module dependency graph for project\n\
         [GRAPH_DEPENDENTS:project:file] — who depends on this file\n\
         [GRAPH_IMPACT:project:file] — transitive impact analysis (3-hop)\n\
         [GRAPH_VERIFY:project] — verify project: check cycles, instability\n\
         [GRAPH_RULES:project] — generate CLAUDE.md-compatible dependency rules\n\n\
         Financial:\n\
         [INCOME_RECORD:amount:category]description[/INCOME_RECORD]\n\
         [FINANCIAL_DASHBOARD]\n\n\
         STRATEGY PIPELINE (4 levels):\n\
         Strategy = user's real-world goal (increase revenue, ship release)\n\
         → Tactic = direction for a category of projects (launch premium tier)\n\
         → Plan = concrete plan for ONE project (integrate Stripe in FitnessApp)\n\
         → Todo = atomic task (agent-executed via delegation, or user-manual)\n\n\
         [PLAN:title]Project: task\\n...[/PLAN] creates Strategy with auto-grouped plans.\n\
         [STRATEGY:goal]context[/STRATEGY] for multi-tactic strategies.\n\
         User approves steps → delegation queue → L1/L2/L3 → Gate → Signal.\n\n\
         GATE PIPELINE (auto, after each delegation):\n\
         1. Verify: project build/test check (exit code)\n\
         2. Diff: git changes (files, insertions, deletions)\n\
         3. Cost: tokens and $ spent\n\
         Gate FAIL → delegation marked failed → Signal(critical) → you may be auto-triggered.\n\n\
         SIGNALS: You see [SIGNALS] section with recent events.\n\
         🔴 critical = action required. 🟡 warn = attention. 🔵 info = FYI.\n\
         3+ critical in 10min = INCIDENT: auto-approve paused for that project.\n\n\
         SAFETY RAILS (automatic):\n\
         - Heartbeat: 120s no events → process killed\n\
         - Token budget: >150K tokens → process killed\n\
         - Context rotation: 3 consecutive fails → fresh session (no --continue)\n\n\
         AUTO-VERIFY: Todos can have verify conditions (file_exists, grep_match, command_exits, git_changed).\n\
         Sensor checks conditions every 30s — marks todos done automatically without LLM.\n\
         Add verify: [PLAN:x]Project: task (agent) [verify: grep \"stripe\" src/**/*.ts][/PLAN]\n\n\
         CURRENT SESSION: {}min uptime, {} active delegations\n\
         Use structured commands. Be concise. Think in strategies and cross-project impact.\n\
         [END IDENTITY]\n",
        project_count, seg_count, uptime_min, active_deleg
    )
}

fn build_queue_context(state: &AppState) -> String {
    let path = state.root.join("tasks").join("queue.md");
    match std::fs::read_to_string(&path) {
        Ok(content) if !content.trim().is_empty() => {
            format!("[QUEUE]\n{}\n[END QUEUE]", content.trim())
        }
        _ => String::new(),
    }
}

fn build_pa_memory(state: &AppState) -> String {
    let path = state.root.join("tasks").join("pa-memory.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let recent = &lines[lines.len().saturating_sub(20)..];
    let mut notes = Vec::new();
    for line in recent {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(note) = entry.get("note").and_then(|v| v.as_str()) {
                notes.push(format!("- {}", note));
            }
        }
    }
    if notes.is_empty() {
        return String::new();
    }
    format!("[YOUR MEMORY]\n{}\n[END MEMORY]\n", notes.join("\n"))
}

fn build_chat_history(state: &AppState) -> String {
    let path = state.chats_dir.join("_orchestrator.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let recent = &lines[lines.len().saturating_sub(25)..];
    let mut history = Vec::new();
    for line in recent {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or("?");
            let role_label = if entry.get("mode").and_then(|v| v.as_str()) == Some("duo") {
                if role == "assistant" {
                    let meta = entry
                        .get("meta")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
                        .trim();
                    if meta.is_empty() {
                        "assistant[duo]".to_string()
                    } else {
                        format!("assistant[{}]", meta)
                    }
                } else if role == "user" {
                    "user[duo]".to_string()
                } else {
                    format!("{}[duo]", role)
                }
            } else {
                role.to_string()
            };
            let msg = entry.get("msg").and_then(|v| v.as_str()).unwrap_or("");
            let short: String = msg.chars().take(500).collect();
            history.push(format!("{}: {}", role_label, short));
        }
    }
    if history.is_empty() {
        return String::new();
    }
    format!(
        "[RECENT CONVERSATION]\n{}\n[END CONVERSATION]\n",
        history.join("\n")
    )
}

fn build_delegation_status(state: &AppState) -> String {
    let delegations = match state.delegations.lock() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let active: Vec<String> = delegations
        .values()
        .filter(|d| !d.status.is_terminal())
        .map(|d| {
            format!(
                "  - {} [{}]: {}",
                d.project,
                d.status,
                d.task.chars().take(60).collect::<String>()
            )
        })
        .collect();
    let recent_done: Vec<String> = delegations
        .values()
        .filter(|d| d.status.is_terminal())
        .take(20) // Limit context size (#19)
        .map(|d| {
            let resp: String = d
                .response
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(100)
                .collect();
            format!("  - {} [{}]: {}", d.project, d.status, resp)
        })
        .collect();
    if active.is_empty() && recent_done.is_empty() {
        return String::new();
    }
    let mut s = "[DELEGATIONS]\n".to_string();
    if !active.is_empty() {
        s += &format!("Active:\n{}\n", active.join("\n"));
    }
    if !recent_done.is_empty() {
        s += &format!("Recent results:\n{}\n", recent_done.join("\n"));
    }
    s += "[END DELEGATIONS]\n";
    s
}

fn build_project_context(state: &AppState) -> String {
    // Use full scanner data for rich context
    let ps = state
        .project_segment
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    let projects = {
        let cache = state.scan_cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(data) = &cache.data {
            if let Some(arr) = data
                .get("agents")
                .and_then(|a| a.as_array())
                .or_else(|| data.as_array())
            {
                arr.iter()
                    .filter_map(|v| {
                        serde_json::from_value::<crate::scanner::ProjectInfo>(v.clone()).ok()
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        } else {
            drop(cache);
            crate::scanner::scan_projects(&state.docs_dir, &ps)
        }
    };

    // Group by category
    let mut lines = vec![format!("[PROJECTS BY CATEGORY ({} total)]", projects.len())];
    for (cat_name, cat_projects) in segments.iter() {
        let in_cat: Vec<&crate::scanner::ProjectInfo> = projects
            .iter()
            .filter(|p| cat_projects.contains(&p.name))
            .collect();
        if in_cat.is_empty() {
            continue;
        }
        let working = in_cat.iter().filter(|p| p.status == "working").count();
        let blocked = in_cat.iter().filter(|p| p.blockers).count();
        lines.push(format!(
            "\n{} ({}, {} working, {} blocked):",
            cat_name,
            in_cat.len(),
            working,
            blocked
        ));
        for p in &in_cat {
            let mut info = format!(
                "  {} [{}{}{}d]",
                p.name,
                p.status,
                if p.blockers { " BLOCKED" } else { "" },
                p.days
            );
            if !p.blocker_text.is_empty() {
                info += &format!(
                    " blocker: {}",
                    p.blocker_text.chars().take(80).collect::<String>()
                );
            }
            if !p.task.is_empty() {
                info += &format!(" task: {}", p.task.chars().take(60).collect::<String>());
            }
            if !p.mcp_servers.is_empty() {
                info += &format!(" mcp: {}", p.mcp_servers.join(","));
            }
            lines.push(info);
        }
    }
    // Unassigned projects
    let assigned: std::collections::HashSet<&str> = segments
        .values()
        .flat_map(|v| v.iter().map(|s| s.as_str()))
        .collect();
    let unassigned: Vec<&crate::scanner::ProjectInfo> = projects
        .iter()
        .filter(|p| !assigned.contains(p.name.as_str()))
        .collect();
    if !unassigned.is_empty() {
        lines.push(format!("\nOther ({}):", unassigned.len()));
        for p in &unassigned {
            lines.push(format!("  {} [{}{}d]", p.name, p.status, p.days));
        }
    }
    lines.push("[END PROJECTS]".to_string());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::{build_response_policy_context, extract_template_section};

    #[test]
    fn response_policy_detects_russian_user_language() {
        let policy = build_response_policy_context("Проверь чат и отвечай нормально");

        assert!(policy.contains("Answer in Russian"));
        assert!(policy.contains("reply in Russian"));
        assert!(policy.contains("Keep PA command tags"));
    }

    #[test]
    fn template_section_extraction_stops_at_next_heading() {
        let content = "Intro\n## Work Report Style\nResult first\nDetails\n## Next\nOther";

        let section = extract_template_section(content, "## Work Report Style").unwrap();

        assert!(section.contains("Result first"));
        assert!(!section.contains("## Next"));
    }
}
