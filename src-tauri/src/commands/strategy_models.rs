//! Strategy data models, persistence, and context builders.

use crate::commands::status::{StepStatus, StrategyStatus};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub deadline: Option<String>,
    pub status: String, // Goals stay as String — parsed from markdown, not JSON
    pub projects: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    /// New: tactics layer (Strategy → Tactic → Plan → Todo)
    #[serde(default)]
    pub tactics: Vec<Tactic>,
    /// Legacy: direct plans (backward compat — wrapped into tactic "main" at load time)
    #[serde(default)]
    pub plans: Vec<Plan>,
    pub status: StrategyStatus,
    pub created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_session_id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    /// Strategy-level deadline (user's real-world goal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// User-reported metrics description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<String>,
}

impl Strategy {
    /// Get all tactics — wraps legacy plans into a "main" tactic if no tactics defined.
    pub fn all_tactics(&self) -> Vec<Tactic> {
        if !self.tactics.is_empty() {
            return self.tactics.clone();
        }
        if !self.plans.is_empty() {
            return vec![Tactic {
                id: format!("{}-main", self.id),
                title: "main".to_string(),
                category: self.category.clone(),
                plans: self.plans.clone(),
                status: TacticStatus::Active,
            }];
        }
        vec![]
    }
    /// Get ALL steps/todos across all tactics/plans (flat, cloned).
    pub fn all_steps_flat(&self) -> Vec<Step> {
        self.all_tactics()
            .into_iter()
            .flat_map(|t| t.plans.into_iter().flat_map(|p| p.steps))
            .collect()
    }
}

/// Tactic: direction for achieving strategy goal, maps to a category of projects.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tactic {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub plans: Vec<Plan>,
    #[serde(default)]
    pub status: TacticStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticStatus {
    Active,
    Done,
    Blocked,
}
impl Default for TacticStatus {
    fn default() -> Self {
        Self::Active
    }
}
impl std::fmt::Display for TacticStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Done => write!(f, "done"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Plan {
    pub project: String,
    pub steps: Vec<Step>,
    pub priority: String,
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub context: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub task: String,
    pub status: StepStatus,
    pub response: Option<String>,
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub delegation_id: Option<String>,
    /// Who executes: agent (→ delegation) or user (→ manual checkbox)
    #[serde(default)]
    pub assignee: Assignee,
    /// Auto-verify condition (checked by sensor, not LLM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyCondition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assignee {
    Agent,
    User,
}
impl Default for Assignee {
    fn default() -> Self {
        Self::Agent
    }
}

/// Scriptable verify condition — checked by gate pipeline and periodic sensor.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VerifyCondition {
    FileExists { path: String },
    GrepMatch { glob: String, pattern: String },
    CommandExits { cmd: String, exit_code: i32 },
    GitChanged { path: String },
}

pub fn strategies_path(state: &AppState) -> std::path::PathBuf {
    state.root.join("tasks").join(".strategies.json")
}

pub fn goals_path(state: &AppState) -> std::path::PathBuf {
    state.root.join("tasks").join("goals.md")
}

pub fn load_strategies(state: &AppState) -> Vec<Strategy> {
    let path = strategies_path(state);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_strategies(state: &AppState, strategies: &[Strategy]) {
    let path = strategies_path(state);
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(strategies).unwrap_or_default(),
    );
}

/// Build [STRATEGIES] context for PA prompt (~150 tokens)
pub fn build_strategy_context(state: &AppState) -> String {
    let strategies = load_strategies(state);
    let active: Vec<&Strategy> = strategies.iter().filter(|s| s.status.is_active()).collect();
    if active.is_empty() {
        return String::new();
    }

    let mut lines = vec!["[STRATEGIES]".to_string()];
    for s in &active {
        let tactics = s.all_tactics();
        let total: usize = tactics
            .iter()
            .flat_map(|t| &t.plans)
            .map(|p| p.steps.len())
            .sum();
        let done = tactics
            .iter()
            .flat_map(|t| &t.plans)
            .flat_map(|p| &p.steps)
            .filter(|st| st.status == StepStatus::Done)
            .count();
        let failed = tactics
            .iter()
            .flat_map(|t| &t.plans)
            .flat_map(|p| &p.steps)
            .filter(|st| st.status == StepStatus::Failed)
            .count();
        let deadline = s.deadline.as_deref().unwrap_or("");

        lines.push(format!(
            "Strategy: \"{}\" [{}] ({}/{} done, {} failed{})",
            s.title,
            s.status,
            done,
            total,
            failed,
            if deadline.is_empty() {
                String::new()
            } else {
                format!(", deadline: {}", deadline)
            }
        ));

        for tactic in &tactics {
            let tcat = tactic.category.as_deref().unwrap_or("");
            let tdone = tactic
                .plans
                .iter()
                .flat_map(|p| &p.steps)
                .filter(|st| st.status == StepStatus::Done)
                .count();
            let ttotal: usize = tactic.plans.iter().map(|p| p.steps.len()).sum();
            lines.push(format!(
                "  Tactic: \"{}\" [{}]{} ({}/{})",
                tactic.title,
                tactic.status,
                if tcat.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", tcat)
                },
                tdone,
                ttotal
            ));

            for plan in &tactic.plans {
                if plan.steps.is_empty() {
                    continue;
                }
                let pdone = plan
                    .steps
                    .iter()
                    .filter(|st| st.status == StepStatus::Done)
                    .count();
                lines.push(format!(
                    "    {} ({}, {}/{}):",
                    plan.project,
                    plan.priority,
                    pdone,
                    plan.steps.len()
                ));
                for step in &plan.steps {
                    let icon = step.status.icon();
                    let assignee_tag = if step.assignee == Assignee::User {
                        " [user]"
                    } else {
                        ""
                    };
                    let verify_tag = if step.verify.is_some() {
                        " [auto-verify]"
                    } else {
                        ""
                    };
                    let extra = if step.status == StepStatus::Queued {
                        step.delegation_id
                            .as_deref()
                            .map(|d| format!(" (del:{})", &d[..d.len().min(8)]))
                            .unwrap_or_default()
                    } else if step.status == StepStatus::Done || step.status == StepStatus::Failed {
                        step.response
                            .as_deref()
                            .map(|r| format!(" → {}", r.chars().take(60).collect::<String>()))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    lines.push(format!(
                        "      {} {}{}{}{}",
                        icon,
                        step.task.chars().take(50).collect::<String>(),
                        assignee_tag,
                        verify_tag,
                        extra
                    ));
                }
            }
        }
    }
    lines.push("[END STRATEGIES]".to_string());
    lines.join("\n") + "\n"
}

/// STRATEGY_PROGRESS: detailed progress with evidence
pub fn strategy_progress(state: &AppState, filter: &str) -> Option<String> {
    let strategies = load_strategies(state);
    let filtered: Vec<&Strategy> = if filter.is_empty() {
        strategies.iter().filter(|s| s.status.is_active()).collect()
    } else {
        strategies
            .iter()
            .filter(|s| s.title.to_lowercase().contains(&filter.to_lowercase()) || s.id == filter)
            .collect()
    };
    if filtered.is_empty() {
        return Some("No matching strategies.".to_string());
    }

    let mut lines = Vec::new();
    for s in &filtered {
        let total: usize = s.plans.iter().map(|p| p.steps.len()).sum();
        let done = s
            .plans
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|st| st.status == StepStatus::Done)
            .count();
        let failed = s
            .plans
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|st| st.status == StepStatus::Failed)
            .count();
        let pct = if total > 0 { done * 100 / total } else { 0 };
        lines.push(format!(
            "**{}** [{}] — {}/{} done ({}%), {} failed",
            s.title, s.status, done, total, pct, failed
        ));
        for plan in &s.plans {
            let pd = plan
                .steps
                .iter()
                .filter(|st| st.status == StepStatus::Done)
                .count();
            lines.push(format!(
                "  {} ({}/{}): {}",
                plan.project,
                pd,
                plan.steps.len(),
                plan.steps
                    .iter()
                    .map(|st| st.status.icon())
                    .collect::<Vec<_>>()
                    .join("")
            ));
        }
    }
    Some(lines.join("\n"))
}

/// STRATEGY_MILESTONE: mark milestone with evidence
pub fn strategy_milestone(
    state: &AppState,
    strategy_filter: &str,
    milestone: &str,
    evidence: &str,
) -> Option<String> {
    // Log milestone to PA memory
    let note = format!(
        "MILESTONE [{}]: {} — {}",
        strategy_filter, milestone, evidence
    );
    let mem_path = state.root.join("tasks").join("pa-memory.jsonl");
    super::jsonl::append_jsonl_logged(
        &mem_path,
        &serde_json::json!({"ts": state.now_iso(), "note": note}),
        "milestone",
    );

    // Log to orchestrator chat
    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    let msg = format!(
        "🏁 Milestone: {} — {}",
        milestone,
        evidence.chars().take(200).collect::<String>()
    );
    super::jsonl::append_jsonl_logged(
        &orch_file,
        &serde_json::json!({"ts": state.now_iso(), "role": "system", "msg": msg}),
        "milestone",
    );

    Some(format!(
        "**Milestone recorded:** {} → {}",
        milestone,
        evidence.chars().take(100).collect::<String>()
    ))
}

/// Update strategy step when its delegation completes
pub fn update_step_from_delegation(
    state: &AppState,
    strategy_id: &str,
    step_id: &str,
    status: &str,
    response: Option<&str>,
) {
    let mut strategies = load_strategies(state);
    let s = match strategies.iter_mut().find(|s| s.id == strategy_id) {
        Some(s) => s,
        None => {
            crate::log_warn!("[strategy] update_step: strategy {} not found", strategy_id);
            return;
        }
    };

    // Update the step
    let mut found = false;
    for plan in &mut s.plans {
        for step in &mut plan.steps {
            if step.id == step_id {
                step.status = if status == "complete" {
                    StepStatus::Done
                } else {
                    StepStatus::Failed
                };
                step.response = response.map(|r| r.chars().take(300).collect());
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    if !found {
        crate::log_warn!(
            "[strategy] update_step: step {} not found in strategy {}",
            step_id,
            strategy_id
        );
        return;
    }

    // Check if all steps terminal → strategy done
    let all_terminal = s
        .plans
        .iter()
        .flat_map(|p| &p.steps)
        .all(|st| st.status.is_terminal());
    if all_terminal {
        s.status = StrategyStatus::Done;
        crate::log_info!("[strategy] '{}' completed (all steps terminal)", s.title);

        // Check if goal should auto-complete
        let goal_id = s.goal_id.clone();
        let all_strats_done = strategies
            .iter()
            .filter(|st| st.goal_id == goal_id)
            .all(|st| st.status == StrategyStatus::Done);
        if all_strats_done {
            mark_goal_completed(state, &goal_id);
        }
    }

    save_strategies(state, &strategies);

    // Save result to PA memory
    if let Some(resp) = response {
        let note = format!(
            "Strategy step {}: {}",
            step_id,
            resp.chars().take(100).collect::<String>()
        );
        let mem_path = state.root.join("tasks").join("pa-memory.jsonl");
        super::jsonl::append_jsonl_logged(
            &mem_path,
            &json!({"ts": state.now_iso(), "note": note}),
            "strategy memory",
        );
    }
}

/// Build context string for a delegation created from a strategy step
pub fn build_step_delegation_context(
    state: &AppState,
    strategy: &Strategy,
    plan_idx: usize,
    step_idx: usize,
) -> String {
    if plan_idx >= strategy.plans.len() {
        crate::log_error!(
            "[strategy] build_step_context: plan_idx {} out of bounds ({})",
            plan_idx,
            strategy.plans.len()
        );
        return String::new();
    }
    let plan = &strategy.plans[plan_idx];
    if step_idx >= plan.steps.len() {
        crate::log_error!(
            "[strategy] build_step_context: step_idx {} out of bounds ({})",
            step_idx,
            plan.steps.len()
        );
        return String::new();
    }
    let done_count = plan
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::Done)
        .count();

    let mut ctx = format!(
        "[STRATEGY CONTEXT]\nGoal: {}\nProject tactic: {} ({} priority, {}/{} done)",
        strategy.title,
        plan.project,
        plan.priority,
        done_count,
        plan.steps.len()
    );

    // Previous step results in this project
    let prev: Vec<String> = plan.steps[..step_idx]
        .iter()
        .filter(|s| s.status == StepStatus::Done && s.response.is_some())
        .map(|s| {
            format!(
                "  - {}: {}",
                s.task.chars().take(40).collect::<String>(),
                s.response
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect::<String>()
            )
        })
        .collect();
    if !prev.is_empty() {
        ctx += &format!("\nPrevious results:\n{}", prev.join("\n"));
    }

    // Category context
    let cat_ctx = super::category::enrich_delegation_with_category(state, &plan.project);
    if !cat_ctx.is_empty() {
        ctx += &cat_ctx;
    }

    ctx
}

fn mark_goal_completed(state: &AppState, goal_id: &str) {
    let path = goals_path(state);
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            crate::log_warn!(
                "[strategy] cannot read goals.md to mark '{}' completed: {}",
                goal_id,
                e
            );
            return;
        }
    };

    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut in_goal = false;
    let mut updated = false;
    for line in &mut lines {
        if line.starts_with("## ") {
            let title = line[3..].trim();
            in_goal = title.to_lowercase().replace(' ', "-") == goal_id;
        }
        if in_goal && line.trim().starts_with("Status:") {
            *line = "Status: completed".to_string();
            updated = true;
            crate::log_info!("[strategy] goal '{}' marked completed", goal_id);
            break;
        }
    }

    if !updated {
        crate::log_warn!(
            "[strategy] goal '{}' not found in goals.md or missing Status field",
            goal_id
        );
        return;
    }
    if let Err(e) = std::fs::write(&path, lines.join("\n")) {
        crate::log_error!("[strategy] failed to write goals.md: {}", e);
    }
}

// build_strategy_prompt moved to strategy.rs
