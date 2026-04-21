//! Strategy engine: goals → strategies → plans → steps
//! Models in strategy_models.rs. This file = Tauri commands only.

use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

use super::strategy_models::*;

pub fn build_strategy_prompt(
    goal: &str,
    context: Option<&str>,
    project_list: &[String],
    category_context: Option<&str>,
) -> String {
    let cat_section = category_context
        .map(|c| format!("\n[CATEGORY]\n{}\n", c))
        .unwrap_or_default();
    format!(
        "[STRATEGY MODE]\nYou are a strategic orchestrator. The user's goal:\n\nGOAL: {}\n{}\n{}\nAvailable projects:\n{}\n\n\
        Generate a JSON strategy with this EXACT format (nothing else, just JSON):\n\
        {{\"title\":\"Strategy title\",\"plans\":[{{\"project\":\"ProjectName\",\"priority\":\"HIGH\",\"depends_on\":[],\"context\":\"Why\",\
        \"steps\":[{{\"task\":\"Specific task\",\"depends_on\":[]}}]}}]}}\n\n\
        Rules:\n- Only projects that contribute to this goal\n- Order by dependency\n- Each step = one atomic task\n- Max 5 steps per project\n\
        - Steps execute via delegation queue\n- Be token-efficient\n",
        goal, context.unwrap_or(""), cat_section, project_list.join("\n"),
    )
}

/// Get all goals from goals.md
#[tauri::command]
pub fn get_goals(state: State<Arc<AppState>>) -> Value {
    let path = goals_path(&state);
    let mut goals: Vec<Goal> = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&path) {
        let mut current: Option<Goal> = None;
        for line in content.lines() {
            if line.starts_with("## ") {
                if let Some(g) = current.take() {
                    goals.push(g);
                }
                let title = line[3..].trim();
                let id = title.to_lowercase().replace(' ', "-");
                current = Some(Goal {
                    id,
                    title: title.to_string(),
                    description: String::new(),
                    deadline: None,
                    status: "active".to_string(),
                    projects: Vec::new(),
                });
            } else if let Some(ref mut g) = current {
                let trimmed = line.trim();
                if trimmed.starts_with("Deadline:") {
                    g.deadline = Some(trimmed[9..].trim().to_string());
                } else if trimmed.starts_with("Status:") {
                    g.status = trimmed[7..].trim().to_lowercase();
                } else if trimmed.starts_with("Projects:") {
                    g.projects = trimmed[9..]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if !g.description.is_empty() {
                        g.description.push('\n');
                    }
                    g.description.push_str(trimmed);
                }
            }
        }
        if let Some(g) = current {
            goals.push(g);
        }
    }

    json!({"goals": goals})
}

/// Save a goal
#[tauri::command]
pub fn save_goal(
    state: State<Arc<AppState>>,
    title: String,
    description: String,
    deadline: Option<String>,
    projects: Vec<String>,
) -> Value {
    let path = goals_path(&state);
    let entry = format!(
        "\n## {}\n{}\nDeadline: {}\nStatus: active\nProjects: {}\n",
        title,
        description,
        deadline.as_deref().unwrap_or("none"),
        projects.join(", "),
    );

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| {
            use std::io::Write;
            write!(f, "{}", entry)
        }) {
        Ok(_) => json!({"status": "ok"}),
        Err(e) => json!({"status": "error", "error": e.to_string()}),
    }
}

/// Get all strategies
#[tauri::command]
pub fn get_strategies(state: State<Arc<AppState>>) -> Value {
    json!({"strategies": load_strategies(&state)})
}

/// Generate a strategy via PA (claude -p)
/// PA analyzes goal + projects → creates multi-project plan
#[tauri::command]
pub async fn generate_strategy(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    goal: String,
    context: Option<String>,
    room_session_id: Option<String>,
) -> Result<Value, String> {
    let (_, pa_dir) = state.get_orch_dir();
    let agents = {
        let ps = state
            .project_segment
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::scanner::scan_projects(&state.docs_dir, &ps)
    };
    let project_list: Vec<String> = agents
        .iter()
        .map(|a| {
            format!(
                "- {}: {} (branch: {}, {} uncommitted, {})",
                a.name,
                if a.task.is_empty() { "idle" } else { &a.task },
                if a.branch.is_empty() { "?" } else { &a.branch },
                a.uncommitted,
                if a.blockers { "BLOCKED" } else { "ok" }
            )
        })
        .collect();

    let prompt = build_strategy_prompt(&goal, context.as_deref(), &project_list, None);

    let perm_path = super::claude_runner::get_permission_path(&state, "_orchestrator");
    let pa_dir_owned = pa_dir.to_path_buf();
    let text = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let tmp = super::claude_runner::unique_tmp("strategy");
        std::fs::write(&tmp, &prompt).map_err(|e| e.to_string())?;
        let stdin_file = std::fs::File::open(&tmp).map_err(|e| e.to_string())?;
        let claude_bin = super::claude_runner::find_claude();
        let output = super::claude_runner::silent_cmd(&claude_bin)
            .args(["-p", "--settings", &perm_path])
            .current_dir(&pa_dir_owned)
            .stdin(std::process::Stdio::from(stdin_file))
            .env("PYTHONIOENCODING", "utf-8")
            .output()
            .map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&tmp);
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Try to parse JSON from response (might be wrapped in markdown)
    let json_str = text
        .find('{')
        .and_then(|start| text.rfind('}').map(|end| &text[start..=end]))
        .unwrap_or(&text);

    let strategy_data: Value = serde_json::from_str(json_str)
        .unwrap_or(json!({"error": "PA did not return valid JSON", "raw": text}));

    if strategy_data.get("error").is_some() {
        return Ok(strategy_data);
    }

    // Build Strategy struct
    let id = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    let plans: Vec<Plan> = strategy_data
        .get("plans")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let project = p.get("project")?.as_str()?.to_string();
                    let priority = p.get("priority")?.as_str().unwrap_or("MED").to_string();
                    let depends_on: Vec<String> = p
                        .get("depends_on")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let steps: Vec<Step> = p
                        .get("steps")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .enumerate()
                                .map(|(i, s)| Step {
                                    id: format!("{}-{}", project, i),
                                    task: s
                                        .get("task")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    status: super::status::StepStatus::Pending,
                                    response: None,
                                    depends_on: s
                                        .get("depends_on")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .filter_map(|v| v.as_str().map(String::from))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    delegation_id: None,
                                    assignee: super::strategy_models::Assignee::default(),
                                    verify: None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let ctx = p
                        .get("context")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(Plan {
                        project,
                        steps,
                        priority,
                        depends_on,
                        category: None,
                        context: ctx,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let strategy = Strategy {
        id: id.clone(),
        goal_id: goal.to_lowercase().replace(' ', "-"),
        title: strategy_data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string(),
        tactics: vec![],
        plans,
        status: super::status::StrategyStatus::Draft,
        created: state.now_iso(),
        room_session_id: room_session_id.clone(),
        category: None,
        deadline: None,
        metrics: None,
    };

    // Save
    let mut strategies = load_strategies(&state);
    strategies.push(strategy.clone());
    save_strategies(&state, &strategies);
    if let Some(session_id) = room_session_id.as_deref() {
        super::multi_agent::link_strategy_to_session(
            &state,
            session_id,
            &strategy.id,
            &strategy.title,
        );
    }

    let _ = tauri::Emitter::emit(&app, "strategy-generated", json!({"id": id}));

    Ok(json!({"status": "ok", "strategy": strategy}))
}

/// Approve specific steps in a strategy
#[tauri::command]
pub fn approve_strategy_steps(
    state: State<Arc<AppState>>,
    strategy_id: String,
    approved_steps: Vec<String>,
) -> Value {
    let mut strategies = load_strategies(&state);
    let s = match strategies.iter_mut().find(|s| s.id == strategy_id) {
        Some(s) => s,
        None => return json!({"status": "error", "error": "Strategy not found"}),
    };

    for plan in &mut s.plans {
        for step in &mut plan.steps {
            if approved_steps.contains(&step.id) {
                step.status = super::status::StepStatus::Approved;
            } else if step.status == super::status::StepStatus::Pending {
                step.status = super::status::StepStatus::Skipped;
            }
        }
    }
    s.status = super::status::StrategyStatus::Approved;
    save_strategies(&state, &strategies);

    json!({"status": "ok"})
}

/// Execute next available step in an approved strategy
#[tauri::command]
pub async fn execute_strategy_step(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || execute_strategy_step_core(&state_arc, &strategy_id))
        .await
        .map_err(|e| e.to_string())
}

fn execute_strategy_step_core(state: &AppState, strategy_id: &str) -> Value {
    let mut strategies = load_strategies(state);

    // Check for circular project dependencies
    if let Some(s) = strategies.iter().find(|s| s.id == strategy_id) {
        for plan in &s.plans {
            for dep in &plan.depends_on {
                if dep == &plan.project {
                    return json!({"status": "error", "error": format!("Circular dependency: {} depends on itself", plan.project)});
                }
                if let Some(dep_plan) = s.plans.iter().find(|p| p.project == *dep) {
                    if dep_plan.depends_on.contains(&plan.project) {
                        return json!({"status": "error", "error": format!("Circular dependency between {} and {}", plan.project, dep)});
                    }
                }
            }
        }
    }

    let s = match strategies.iter_mut().find(|s| s.id == strategy_id) {
        Some(s)
            if s.status == super::status::StrategyStatus::Approved
                || s.status == super::status::StrategyStatus::Executing =>
        {
            s
        }
        _ => return json!({"status": "error", "error": "Strategy not found or not approved"}),
    };

    // Determine parallel vs sequential from category
    let cat_meta = s
        .category
        .as_deref()
        .and_then(|c| super::category::load_categories(&state.root).remove(c));
    let is_sequential = cat_meta
        .as_ref()
        .map(|m| m.delegation_strategy == "sequential")
        .unwrap_or(true);

    // Find all ready steps (approved + deps met)
    let ready = find_all_ready_steps(s);
    if ready.is_empty() {
        let all_terminal = s
            .plans
            .iter()
            .flat_map(|p| &p.steps)
            .all(|st| st.status.is_terminal());
        if all_terminal {
            s.status = super::status::StrategyStatus::Done;
            save_strategies(state, &strategies);
            return json!({"status": "complete", "message": "All steps executed"});
        }
        return json!({"status": "waiting", "message": "No steps ready (dependencies pending)"});
    }

    let to_queue = if is_sequential {
        &ready[..1]
    } else {
        &ready[..]
    };
    let mut queued = Vec::new();

    // Build contexts before mutating
    let contexts: Vec<String> = to_queue
        .iter()
        .map(|(pi, si, _, _)| build_step_delegation_context(state, s, *pi, *si))
        .collect();

    s.status = super::status::StrategyStatus::Executing;

    // Queue delegations and link to strategy steps atomically
    let mut delegation_ids: Vec<(usize, usize, String, String)> = Vec::new(); // (plan_idx, step_idx, did, project)

    for (i, (plan_idx, step_idx, project, task)) in to_queue.iter().enumerate() {
        let enriched_task = format!("{}\n\n[TASK]\n{}", contexts[i], task);
        let did = super::delegation::queue_delegation_internal(state, project, &enriched_task);
        if did.is_empty() {
            continue;
        }
        delegation_ids.push((*plan_idx, *step_idx, did, project.clone()));
    }

    // Link all delegations to strategy steps in a single lock acquisition
    if !delegation_ids.is_empty() {
        if let Ok(mut delegations) = state.delegations.lock() {
            for (plan_idx, step_idx, did, _) in &delegation_ids {
                if let Some(del) = delegations.get_mut(did) {
                    del.strategy_id = Some(strategy_id.to_string());
                    del.strategy_step_id = Some(s.plans[*plan_idx].steps[*step_idx].id.clone());
                    del.room_session_id = s.room_session_id.clone();
                }
            }
        }
        state.save_delegations();
    }

    for (plan_idx, step_idx, did, project) in &delegation_ids {
        s.plans[*plan_idx].steps[*step_idx].status = super::status::StepStatus::Queued;
        s.plans[*plan_idx].steps[*step_idx].delegation_id = Some(did.clone());
        queued.push(json!({"project": project, "step": s.plans[*plan_idx].steps[*step_idx].id, "delegation": did}));
        crate::log_info!(
            "[strategy] queued step {} for {} as delegation {}",
            s.plans[*plan_idx].steps[*step_idx].id,
            project,
            did
        );
        if let Some(session_id) = s.room_session_id.as_deref() {
            super::multi_agent::link_delegation_to_session(
                state,
                session_id,
                did,
                project,
                &s.plans[*plan_idx].steps[*step_idx].task,
            );
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "strategy_step_queued",
                "system",
                &format!(
                    "Queued strategy step {} for {} as delegation {}",
                    s.plans[*plan_idx].steps[*step_idx].id, project, did
                ),
                json!({
                    "strategy_id": strategy_id,
                    "step_id": s.plans[*plan_idx].steps[*step_idx].id,
                    "delegation_id": did,
                    "project": project,
                }),
            );
        }
    }

    save_strategies(state, &strategies);

    json!({"status": "queued", "count": queued.len(), "delegations": queued})
}

/// Find all steps that are approved and have all dependencies met
fn find_all_ready_steps(strategy: &Strategy) -> Vec<(usize, usize, String, String)> {
    let mut ready = Vec::new();
    for (pi, plan) in strategy.plans.iter().enumerate() {
        // Check project-level dependencies
        let project_deps_done = plan.depends_on.iter().all(|dep_proj| {
            strategy
                .plans
                .iter()
                .filter(|p| p.project == *dep_proj)
                .all(|p| p.steps.iter().all(|st| st.status.is_terminal()))
        });
        if !project_deps_done {
            continue;
        }

        for (si, step) in plan.steps.iter().enumerate() {
            if step.status != super::status::StepStatus::Approved {
                continue;
            }
            let step_deps_done = step.depends_on.iter().all(|dep_id| {
                plan.steps
                    .iter()
                    .any(|st| st.id == *dep_id && st.status.is_terminal())
            });
            if step_deps_done {
                ready.push((pi, si, plan.project.clone(), step.task.clone()));
            }
        }
    }
    ready
}

/// Create Strategy from [STRATEGY:goal]context[/STRATEGY] PA command.
/// Parses TACTIC blocks, deadline, assignee from context body.
pub fn create_strategy_from_command(state: &AppState, goal: &str, context: &str) -> Option<String> {
    crate::log_info!(
        "[pa_cmd] strategy: {}",
        goal.chars().take(50).collect::<String>()
    );
    let id = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let goal_slug: String = goal
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .take(50)
        .collect();
    let goal_id = if goal_slug.is_empty() {
        format!("strat-{}", &id)
    } else {
        goal_slug
    };

    let mut deadline = None;
    let mut lines_iter = context.lines().peekable();
    if let Some(first) = lines_iter.peek() {
        if first.to_lowercase().starts_with("deadline:") {
            deadline = Some(
                first
                    .trim_start_matches("deadline:")
                    .trim_start_matches("Deadline:")
                    .trim()
                    .to_string(),
            );
            lines_iter.next();
        }
    }

    let mut tactics = Vec::new();
    let mut cur: Option<(String, Option<String>, Vec<(String, String, String)>)> = None;
    for line in lines_iter {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("TACTIC ") {
            if let Some((title, cat, steps)) = cur.take() {
                tactics.push(build_tactic(&id, &title, cat, steps));
            }
            let rest = t.strip_prefix("TACTIC ").unwrap_or("");
            let (cat, title) = if let Some(idx) = rest.find(':') {
                (
                    Some(rest[..idx].trim().to_string()),
                    rest[idx + 1..].trim().to_string(),
                )
            } else {
                (None, rest.trim().to_string())
            };
            cur = Some((title, cat, Vec::new()));
            continue;
        }
        if let Some((_, _, ref mut steps)) = cur {
            if let Some(idx) = t.find(':') {
                let proj = t[..idx].trim().trim_start_matches("- ").trim().to_string();
                let task_raw = t[idx + 1..].trim().to_string();
                let assignee = if task_raw.contains("(user)") {
                    "user"
                } else {
                    "agent"
                };
                let task = task_raw
                    .replace("(agent)", "")
                    .replace("(user)", "")
                    .trim()
                    .to_string();
                steps.push((proj, task, assignee.to_string()));
            }
        }
    }
    if let Some((title, cat, steps)) = cur.take() {
        tactics.push(build_tactic(&id, &title, cat, steps));
    }

    let tc = tactics.len();
    let td: usize = tactics
        .iter()
        .flat_map(|t| &t.plans)
        .map(|p| p.steps.len())
        .sum();
    let strategy = Strategy {
        id: id.clone(),
        goal_id,
        title: goal.to_string(),
        tactics,
        plans: vec![],
        status: super::status::StrategyStatus::Draft,
        created: state.now_iso(),
        room_session_id: None,
        category: None,
        deadline,
        metrics: None,
    };
    let mut all = load_strategies(state);
    all.push(strategy);
    save_strategies(state, &all);

    let gp = super::strategy_models::goals_path(state);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gp)
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "\n## {}\nStatus: active", goal)
        });

    Some(format!(
        "Strategy '{}': {} tactics, {} todos. Approve in Strategy view.",
        goal, tc, td
    ))
}

fn build_tactic(
    sid: &str,
    title: &str,
    cat: Option<String>,
    steps: Vec<(String, String, String)>,
) -> Tactic {
    use std::collections::HashMap;
    let mut by_proj: Vec<(String, Vec<(usize, String, String)>)> = Vec::new();
    let mut pi: HashMap<String, usize> = HashMap::new();
    for (idx, (proj, task, asgn)) in steps.iter().enumerate() {
        if let Some(&i) = pi.get(proj) {
            by_proj[i].1.push((idx, task.clone(), asgn.clone()));
        } else {
            pi.insert(proj.clone(), by_proj.len());
            by_proj.push((proj.clone(), vec![(idx, task.clone(), asgn.clone())]));
        }
    }
    let plans = by_proj
        .into_iter()
        .map(|(proj, tasks)| Plan {
            project: proj.clone(),
            steps: tasks
                .iter()
                .map(|(i, task, asgn)| Step {
                    id: format!("{}-{}-{}", sid, proj, i),
                    task: task.clone(),
                    status: crate::commands::status::StepStatus::Pending,
                    response: None,
                    depends_on: vec![],
                    delegation_id: None,
                    assignee: if asgn == "user" {
                        Assignee::User
                    } else {
                        Assignee::Agent
                    },
                    verify: None,
                })
                .collect(),
            priority: "MED".to_string(),
            depends_on: vec![],
            category: cat.clone(),
            context: String::new(),
        })
        .collect();
    Tactic {
        id: format!("{}-t{}", sid, title.len()),
        title: title.to_string(),
        category: cat,
        plans,
        status: TacticStatus::Active,
    }
}

/// Try to queue next steps after a delegation completes (called from delegation.rs)
pub fn try_queue_next_steps(state: &AppState, strategy_id: &str) -> usize {
    let result = execute_strategy_step_core(state, strategy_id);
    result.get("count").and_then(|c| c.as_u64()).unwrap_or(0) as usize
}
