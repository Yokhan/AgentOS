//! Orchestrator plans — PA creates and tracks multi-step execution plans.
//! Plans persist in tasks/plans/*.json and are visible in PA context.

use super::status::{PlanStatus, PlanStepStatus};
use super::strategy_models::{Assignee, VerifyCondition};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlanStep {
    #[serde(default)]
    pub id: String,
    pub project: String,
    pub task: String,
    #[serde(default)]
    pub assignee: Assignee,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    pub status: PlanStepStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub status: PlanStatus,
    pub steps: Vec<PlanStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_session_id: Option<String>,
    pub created: String,
    pub updated: String,
}

fn plans_dir(state: &AppState) -> std::path::PathBuf {
    let dir = state.root.join("tasks").join("plans");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn step_id_for(plan_id: &str, index: usize) -> String {
    format!("{}-step-{}", plan_id, index + 1)
}

pub fn plan_step_source_id(plan_id: &str, step_id: &str) -> String {
    format!("{}::{}", plan_id, step_id)
}

pub fn parse_plan_step_source_id(source_id: &str) -> Option<(String, String)> {
    source_id.split_once("::").and_then(|(plan_id, step_id)| {
        let plan_id = plan_id.trim();
        let step_id = step_id.trim();
        if plan_id.is_empty() || step_id.is_empty() {
            None
        } else {
            Some((plan_id.to_string(), step_id.to_string()))
        }
    })
}

fn parse_assignee_value(step: &Value) -> Assignee {
    step.get("assignee")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn parse_verify_value(step: &Value) -> Option<VerifyCondition> {
    step.get("verify")
        .cloned()
        .filter(|v| !v.is_null())
        .and_then(|v| serde_json::from_value(v).ok())
}

fn parse_plan_step(step: &Value, plan_id: &str, index: usize) -> PlanStep {
    PlanStep {
        id: step
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| step_id_for(plan_id, index)),
        project: step
            .get("project")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        task: step
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        assignee: parse_assignee_value(step),
        verify: parse_verify_value(step),
        work_item_id: step
            .get("work_item_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        status: step
            .get("status")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(PlanStepStatus::Pending),
        delegation_id: step
            .get("delegation_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        result: step
            .get("result")
            .and_then(|v| v.as_str())
            .map(String::from),
    }
}

fn normalize_plan(plan: &mut Plan) -> bool {
    let mut changed = false;
    for (index, step) in plan.steps.iter_mut().enumerate() {
        if step.id.trim().is_empty() {
            step.id = step_id_for(&plan.id, index);
            changed = true;
        }
    }
    changed
}

pub fn find_plan_step_index(plan: &Plan, step_id: &str) -> Option<usize> {
    plan.steps.iter().position(|step| step.id == step_id)
}

pub fn update_plan_step_linked_work_item(
    state: &AppState,
    plan_id: &str,
    step_id: &str,
    work_item_id: &str,
    assignee: Assignee,
    verify: Option<VerifyCondition>,
) -> Result<(), String> {
    let mut plans = load_all_plans_internal(state);
    let plan = plans
        .iter_mut()
        .find(|p| p.id == plan_id)
        .ok_or_else(|| "Plan not found".to_string())?;
    let step_index =
        find_plan_step_index(plan, step_id).ok_or_else(|| "Plan step not found".to_string())?;
    let step = &mut plan.steps[step_index];
    step.work_item_id = Some(work_item_id.to_string());
    step.assignee = assignee;
    step.verify = verify;
    plan.updated = state.now_iso();
    save_plan_internal(state, plan);
    Ok(())
}

pub fn sync_plan_step_from_work_item(
    state: &AppState,
    plan_id: &str,
    step_id: &str,
    status: PlanStepStatus,
    result: Option<String>,
    work_item_id: Option<String>,
    delegation_id: Option<String>,
) -> Result<(), String> {
    let mut plans = load_all_plans_internal(state);
    let plan = plans
        .iter_mut()
        .find(|p| p.id == plan_id)
        .ok_or_else(|| "Plan not found".to_string())?;
    let step_index =
        find_plan_step_index(plan, step_id).ok_or_else(|| "Plan step not found".to_string())?;
    let step = &mut plan.steps[step_index];
    step.status = status;
    if let Some(work_item_id) = work_item_id {
        step.work_item_id = Some(work_item_id);
    }
    if let Some(delegation_id) = delegation_id {
        step.delegation_id = Some(delegation_id);
    }
    if result.is_some() {
        step.result = result;
    }
    plan.updated = state.now_iso();
    if plan.steps.iter().all(|s| s.status.is_terminal()) {
        plan.status = PlanStatus::Completed;
    }
    save_plan_internal(state, plan);
    Ok(())
}

pub fn load_all_plans_internal(state: &AppState) -> Vec<Plan> {
    let dir = plans_dir(state);
    let mut plans = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut plan) = serde_json::from_str::<Plan>(&content) {
                        if normalize_plan(&mut plan) {
                            save_plan_internal(state, &plan);
                        }
                        plans.push(plan);
                    }
                }
            }
        }
    }
    plans.sort_by(|a, b| b.updated.cmp(&a.updated));
    plans
}

pub fn save_plan_internal(state: &AppState, plan: &Plan) {
    let path = plans_dir(state).join(format!("{}.json", plan.id));
    let _ = crate::commands::claude_runner::atomic_write(
        &path,
        &serde_json::to_string_pretty(plan).unwrap_or_default(),
    );
}

#[tauri::command]
pub fn get_plans(state: State<Arc<AppState>>) -> Value {
    let plans = load_all_plans_internal(&state);
    let active: Vec<&Plan> = plans
        .iter()
        .filter(|p| p.status == PlanStatus::Active)
        .collect();
    let done: Vec<&Plan> = plans
        .iter()
        .filter(|p| p.status == PlanStatus::Completed)
        .collect();
    json!({"plans": plans, "active_count": active.len(), "done_count": done.len()})
}

#[tauri::command]
pub fn create_plan(
    state: State<Arc<AppState>>,
    title: String,
    steps: Vec<Value>,
    room_session_id: Option<String>,
) -> Value {
    let id = format!(
        "plan-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let ts = state.now_iso();
    let plan_steps: Vec<PlanStep> = steps
        .iter()
        .enumerate()
        .map(|(index, s)| parse_plan_step(s, &id, index))
        .collect();

    let plan = Plan {
        id: id.clone(),
        title: title.clone(),
        status: PlanStatus::Active,
        steps: plan_steps,
        room_session_id: room_session_id.clone(),
        created: ts.clone(),
        updated: ts,
    };
    save_plan_internal(&state, &plan);
    crate::log_info!(
        "[plans] created '{}' with {} steps",
        plan.title,
        plan.steps.len()
    );
    if let Some(session_id) = room_session_id.as_deref() {
        super::multi_agent::link_plan_to_session(&state, session_id, &plan.id, &title);
    }
    json!({"status": "ok", "id": id})
}

#[tauri::command]
pub fn update_plan_step(
    state: State<Arc<AppState>>,
    plan_id: String,
    step_index: usize,
    status: String,
    result: Option<String>,
) -> Value {
    let mut plans = load_all_plans_internal(&state);
    let plan = match plans.iter_mut().find(|p| p.id == plan_id) {
        Some(p) => p,
        None => return json!({"status": "error", "error": "Plan not found"}),
    };
    if step_index >= plan.steps.len() {
        return json!({"status": "error", "error": "Step index out of range"});
    }
    // Parse status string from API into enum
    let step_status: PlanStepStatus =
        serde_json::from_value(json!(status)).unwrap_or(PlanStepStatus::Pending);
    plan.steps[step_index].status = step_status;
    plan.steps[step_index].result = result;
    plan.updated = state.now_iso();

    if plan.steps.iter().all(|s| s.status.is_terminal()) {
        plan.status = PlanStatus::Completed;
    }

    save_plan_internal(&state, plan);
    json!({"status": "ok"})
}

/// Build plans context for PA prompt
pub fn build_plans_context(state: &AppState) -> String {
    let plans = load_all_plans_internal(state);
    let active: Vec<&Plan> = plans
        .iter()
        .filter(|p| p.status == PlanStatus::Active)
        .collect();
    if active.is_empty() {
        return String::new();
    }

    let mut lines = vec!["[ACTIVE PLANS]".to_string()];
    for plan in active {
        let done = plan
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Done)
            .count();
        let failed = plan
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Failed)
            .count();
        let total = plan.steps.len();
        lines.push(format!(
            "Plan: \"{}\" ({}/{} done, {} failed)",
            plan.title, done, total, failed
        ));
        for (i, step) in plan.steps.iter().enumerate() {
            let icon = step.status.icon();
            let res: String = step
                .result
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect();
            lines.push(format!(
                "  {} {}. {} → {} {}",
                icon,
                i + 1,
                step.project,
                step.task.chars().take(40).collect::<String>(),
                if res.is_empty() {
                    String::new()
                } else {
                    format!("({})", res)
                }
            ));
        }
    }
    lines.push("[END PLANS]".to_string());
    lines.join("\n") + "\n"
}
