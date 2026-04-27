use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

pub fn queue_delegation_internal(state: &AppState, project: &str, task: &str) -> String {
    // Validate task content
    let task = task.trim();
    if task.is_empty() {
        crate::log_warn!("[delegation] rejected empty task for {}", project);
        return String::new();
    }
    // Truncate overly long tasks
    let task = if task.len() > 5000 {
        crate::log_warn!(
            "[delegation] truncating task for {} ({}→5000 chars)",
            project,
            task.len()
        );
        &task[..task
            .char_indices()
            .nth(5000)
            .map(|(i, _)| i)
            .unwrap_or(task.len())]
    } else {
        task
    };

    let id = format!(
        "{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        std::process::id()
    );

    let delegation = crate::state::Delegation {
        id: id.clone(),
        project: project.to_string(),
        task: task.to_string(),
        ts: state.now_iso(),
        status: crate::commands::status::DelegationStatus::Pending,
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

    if let Ok(mut delegations) = state.delegations.lock() {
        delegations.insert(id.clone(), delegation);
    }
    state.save_delegations();

    id
}

#[tauri::command]
pub fn get_delegations(state: State<Arc<AppState>>) -> Value {
    // Cleanup old done/failed delegations (#61) — keep last 50
    if let Ok(mut d) = state.delegations.lock() {
        if d.len() > 100 {
            let mut old: Vec<String> = d
                .iter()
                .filter(|(_, v)| {
                    v.status == crate::commands::status::DelegationStatus::Done
                        || v.status == crate::commands::status::DelegationStatus::Failed
                        || v.status == crate::commands::status::DelegationStatus::Rejected
                })
                .map(|(k, _)| k.clone())
                .collect();
            old.sort();
            for key in old.iter().take(old.len().saturating_sub(50)) {
                d.remove(key);
            }
        }
    }
    let delegations = match state.delegations.lock() {
        Ok(d) => d
            .values()
            .filter(|d| !d.status.is_terminal())
            .map(|d| serde_json::to_value(d).unwrap_or_default())
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    json!({"delegations": delegations})
}

/// Core delegation logic — can be called from both Tauri command and API handler
pub fn approve_delegation_core(state: &AppState, id: &str) -> Value {
    crate::log_info!("[delegation] approving id={}", id);
    // Atomic check-and-update
    let d = {
        let mut delegations = match state.delegations.lock() {
            Ok(d) => d,
            Err(_) => return json!({"status": "error", "error": "lock error"}),
        };
        let del = match delegations.get_mut(id) {
            Some(d)
                if d.status == crate::commands::status::DelegationStatus::Pending
                    || d.status == crate::commands::status::DelegationStatus::NeedsPermission =>
            {
                d
            }
            _ => {
                return json!({"status": "error", "error": "Delegation not found or already executed"})
            }
        };
        del.status = crate::commands::status::DelegationStatus::Running;
        del.clone()
    };
    state.save_delegations();
    if let Some(work_item_id) = d.work_item_id.as_deref() {
        if let Ok(mut work_items) = state.work_items.lock() {
            if let Some(item) = work_items.get_mut(work_item_id) {
                item.status = crate::state::WorkItemStatus::Running;
                item.updated_at = state.now_iso();
            }
        }
        state.save_work_items();
    }
    if let (Some(plan_id), Some(step_idx)) = (&d.plan_id, d.plan_step) {
        let mut plans = super::plans::load_all_plans_internal(state);
        if let Some(plan) = plans.iter_mut().find(|p| p.id == *plan_id) {
            if step_idx < plan.steps.len() {
                plan.steps[step_idx].status = crate::commands::status::PlanStepStatus::Running;
                plan.steps[step_idx].delegation_id = Some(id.to_string());
                if let Some(work_item_id) = d.work_item_id.as_deref() {
                    plan.steps[step_idx].work_item_id = Some(work_item_id.to_string());
                }
                plan.updated = state.now_iso();
                super::plans::save_plan_internal(state, plan);
            }
        }
    }
    if let Some(session_id) = d.room_session_id.as_deref() {
        super::multi_agent::emit_pipeline_event(
            state,
            session_id,
            "delegation_started",
            "system",
            &format!("Delegation {} started for {}", id, d.project),
            json!({
                "delegation_id": id,
                "project": d.project,
                "task": super::claude_runner::safe_truncate(&d.task, 240),
            }),
        );
        if let Some(batch_id) = d.batch_id.as_deref() {
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "parallel_batch_member_started",
                "system",
                &format!("Batch {} started delegation {}", batch_id, id),
                json!({
                    "batch_id": batch_id,
                    "delegation_id": id,
                    "project": d.project,
                    "work_item_id": d.work_item_id,
                    "executor_provider": d.executor_provider,
                }),
            );
        }
    }

    let project_dir = match state.validate_project(&d.project) {
        Ok(p) => p,
        Err(e) => return json!({"status": "error", "error": e}),
    };

    let chat_file = state.chats_dir.join(format!("{}.jsonl", d.project));
    let ts = state.now_iso();

    // Enrich task with category context + graph dependency context
    let category_ctx = super::category::enrich_delegation_with_category(state, &d.project);
    let graph_ctx = super::graph_ops::build_graph_context(state, &d.project);
    let mut enriched_task = d.task.clone();
    if !category_ctx.is_empty() {
        enriched_task = format!("{}\n{}", enriched_task, category_ctx);
    }
    if !graph_ctx.is_empty() {
        enriched_task = format!("{}\n{}", enriched_task, graph_ctx);
    }

    let user_entry =
        json!({"ts": ts, "role": "user", "msg": format!("[via PA] {}", enriched_task)});
    super::jsonl::append_jsonl_logged(&chat_file, &user_entry, "delegation user msg");

    // Read delegation model/effort from config
    let cfg_val = state.config();
    let deleg_model: Option<String> = cfg_val
        .get("delegation_model")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let deleg_effort: Option<String> = cfg_val
        .get("delegation_effort")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    // Acquire per-project lock: prevents two claude processes in same directory
    state.acquire_dir_lock(&d.project);
    crate::log_info!("[delegation:{}] acquired project lock", d.project);

    // Stream buffer for real-time progress
    let stream_buf = state
        .root
        .join("tasks")
        .join(format!(".stream-deleg-{}.jsonl", id));
    let _ = std::fs::write(&stream_buf, "");

    // --- LEVEL 1: Execute with balanced permissions (minimum for delegations) ---
    let perm_path =
        super::claude_runner::get_delegation_permission_path(state, &d.project, "balanced");
    crate::log_info!(
        "[delegation:{}] L1 executing with balanced{}",
        d.project,
        deleg_model
            .as_deref()
            .map(|m| format!(" model={}", m))
            .unwrap_or_default()
    );
    super::delegation_stream::emit_stage(&stream_buf, "L1", "balanced permissions");
    let (response, is_perm) = super::delegation_stream::run_delegation_streaming(
        state,
        &project_dir,
        &enriched_task,
        &perm_path,
        deleg_model.as_deref(),
        deleg_effort.as_deref(),
        &stream_buf,
    );

    // Check response: error first, then permission request
    let mut final_response = response.clone();
    let mut execution_status = "complete";

    if response.starts_with("Error:")
        || response.starts_with("Error running")
        || response.starts_with("Error waiting")
    {
        crate::log_error!(
            "[delegation:{}] L1 error: {}",
            d.project,
            response.chars().take(100).collect::<String>()
        );
        execution_status = "failed";
    } else if is_perm {
        crate::log_warn!(
            "[delegation:{}] L1 got permission request, escalating to permissive",
            d.project
        );

        // Update status to escalated
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(id) {
                del.status = crate::commands::status::DelegationStatus::Escalated;
                del.escalation_info = Some(format!(
                    "L1 permission request: {}",
                    response.chars().take(150).collect::<String>()
                ));
            }
        }
        state.save_delegations();
        if let Some(session_id) = d.room_session_id.as_deref() {
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "delegation_verifying",
                "system",
                &format!("Delegation {} entered gate verification", id),
                json!({"delegation_id": id, "project": d.project}),
            );
        }

        // --- LEVEL 2: Retry with permissive permissions ---
        let perm_path_perm =
            super::claude_runner::get_delegation_permission_path(state, &d.project, "permissive");
        crate::log_info!(
            "[delegation:{}] L2 retrying with permissive permissions",
            d.project
        );
        super::delegation_stream::emit_stage(&stream_buf, "L2", "permissive retry");
        let (response2, is_perm2) = super::delegation_stream::run_delegation_streaming(
            state,
            &project_dir,
            &enriched_task,
            &perm_path_perm,
            deleg_model.as_deref(),
            deleg_effort.as_deref(),
            &stream_buf,
        );

        if is_perm2 {
            crate::log_warn!(
                "[delegation:{}] L2 still permission request, escalating to PA",
                d.project
            );

            // --- LEVEL 3: Ask PA for decision ---
            if let Ok(mut delegations) = state.delegations.lock() {
                if let Some(del) = delegations.get_mut(id) {
                    del.status = crate::commands::status::DelegationStatus::Deciding;
                }
            }
            state.save_delegations();

            let (_, pa_dir) = state.get_orch_dir();
            let pa_prompt = format!(
                "[ESCALATION from {}]\nThe agent for '{}' needs permission it doesn't have.\n\
                 Agent's response: \"{}\"\n\n\
                 Task was: \"{}\"\n\n\
                 Decide: respond with [GRANT] to retry with full permissions, or [ABORT] to cancel this delegation.\n\
                 Explain your reasoning briefly.",
                d.project, d.project,
                response2.chars().take(300).collect::<String>(),
                d.task.chars().take(200).collect::<String>(),
            );

            let pa_perm = super::claude_runner::get_permission_path(state, "_orchestrator");
            crate::log_info!("[delegation:{}] L3 asking PA for decision", d.project);
            super::delegation_stream::emit_stage(&stream_buf, "L3", "PA deciding GRANT/ABORT");
            let pa_decision = super::claude_runner::run_claude(&pa_dir, &pa_prompt, &pa_perm);

            // Log PA decision in orchestrator chat
            let orch_file = state.chats_dir.join("_orchestrator.jsonl");
            let decision_entry = json!({"ts": state.now_iso(), "role": "system", "msg": format!("[PA Decision for {}] {}", d.project, pa_decision.chars().take(300).collect::<String>())});
            super::jsonl::append_jsonl_logged(&orch_file, &decision_entry, "PA decision");

            if pa_decision.contains("[GRANT]") {
                crate::log_info!("[delegation:{}] PA granted, L3 retrying", d.project);
                super::delegation_stream::emit_stage(
                    &stream_buf,
                    "L3_retry",
                    "PA granted, retrying",
                );
                let (resp3, is_perm3) = super::delegation_stream::run_delegation_streaming(
                    state,
                    &project_dir,
                    &enriched_task,
                    &perm_path_perm,
                    deleg_model.as_deref(),
                    deleg_effort.as_deref(),
                    &stream_buf,
                );
                final_response = resp3;
                if is_perm3 {
                    execution_status = "failed";
                    crate::log_error!("[delegation:{}] L3 still failed after PA grant", d.project);
                }
            } else {
                final_response = format!(
                    "PA aborted delegation: {}",
                    pa_decision.chars().take(200).collect::<String>()
                );
                execution_status = "failed";
                crate::log_info!("[delegation:{}] PA aborted", d.project);
            }
        } else {
            final_response = response2;
            crate::log_info!("[delegation:{}] L2 succeeded with permissive", d.project);
        }
    }

    // Emit done to stream buffer
    // Save final response to project chat
    let ts2 = state.now_iso();
    let asst_entry = json!({"ts": ts2, "role": "assistant", "msg": final_response});
    super::jsonl::append_jsonl_logged(&chat_file, &asst_entry, "delegation assistant response");

    // Gate pipeline: verify + diff + cost check (#2 #11: atomic status transition)
    let (gate_result, effective_status) = if execution_status == "complete" {
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(del) = delegations.get_mut(id) {
                del.status = crate::commands::status::DelegationStatus::Verifying;
            }
        }
        state.save_delegations();
        if let Some(work_item_id) = d.work_item_id.as_deref() {
            if let Ok(mut work_items) = state.work_items.lock() {
                if let Some(item) = work_items.get_mut(work_item_id) {
                    item.status = crate::state::WorkItemStatus::Reviewing;
                    item.updated_at = state.now_iso();
                }
            }
            state.save_work_items();
        }
        // Run gate (outside lock — may take time)
        let gr = super::gate::run_gate(state, &d.project, id);
        super::signals::emit_gate_signals(state, &d.project, id, &gr);
        if let Some(session_id) = d.room_session_id.as_deref() {
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "gate_result",
                "system",
                &format!("Gate {:?} for delegation {}", gr.status, id),
                json!({
                    "delegation_id": id,
                    "project": d.project,
                    "status": gr.status,
                    "errors": gr.errors,
                }),
            );
        }
        let eff = if gr.status == super::gate::GateStatus::Fail {
            "failed"
        } else {
            "complete"
        };
        (Some(gr), eff)
    } else {
        super::signals::emit_signal(
            state,
            super::signals::SignalSource::Gate,
            super::signals::Severity::Warn,
            Some(&d.project),
            &format!(
                "Delegation failed: {}",
                super::claude_runner::safe_truncate(&final_response, 100)
            ),
            Some(id),
        );
        (None, "failed")
    };

    super::delegation_stream::emit_done(&stream_buf, effective_status, &final_response);

    let orch_file = state.chats_dir.join("_orchestrator.jsonl");
    let icon = if effective_status == "complete" {
        "✓"
    } else {
        "✗"
    };
    let sys_entry = json!({"ts": ts2, "role": "system", "msg": format!("{} {} in {}: {}", icon, effective_status, d.project, final_response.chars().take(500).collect::<String>())});
    super::jsonl::append_jsonl_logged(&orch_file, &sys_entry, "orch status");

    let needs_user = effective_status == "failed"
        || super::claude_runner::is_permission_request(&final_response);
    super::inbox::push_inbox(
        state,
        &d.project,
        "delegation_result",
        &final_response,
        needs_user,
        Some(id),
        d.room_session_id.as_deref(),
    );
    if let Some(session_id) = d.room_session_id.as_deref() {
        super::multi_agent::emit_pipeline_event(
            state,
            session_id,
            "delegation_completed",
            "system",
            &format!("Delegation {} {} for {}", id, effective_status, d.project),
            json!({
                "delegation_id": id,
                "project": d.project,
                "status": effective_status,
                "needs_user": needs_user,
                "response": super::claude_runner::safe_truncate(&final_response, 320),
            }),
        );
        if let Some(batch_id) = d.batch_id.as_deref() {
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "parallel_batch_member_completed",
                "system",
                &format!(
                    "Batch {} completed delegation {} -> {}",
                    batch_id, id, effective_status
                ),
                json!({
                    "batch_id": batch_id,
                    "delegation_id": id,
                    "project": d.project,
                    "work_item_id": d.work_item_id,
                    "status": effective_status,
                }),
            );
        }
    }

    // Check incident pattern
    if effective_status == "failed" {
        super::signals::check_incident(state, &d.project);
    }

    // Atomic status update: set final status + gate result + git diff in one lock (#2)
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(id) {
            del.status = if effective_status == "complete" {
                crate::commands::status::DelegationStatus::Done
            } else {
                crate::commands::status::DelegationStatus::Failed
            };
            del.response = Some(final_response.chars().take(500).collect());
            del.gate_result = gate_result.clone();
            if effective_status == "complete" {
                del.git_diff = super::claude_runner::capture_git_changes(&project_dir);
            }
        }
    }
    state.save_delegations();
    if let Some(work_item_id) = d.work_item_id.as_deref() {
        let mut completed_work_item = None;
        if let Ok(mut work_items) = state.work_items.lock() {
            if let Some(item) = work_items.get_mut(work_item_id) {
                item.status = if effective_status == "complete" {
                    crate::state::WorkItemStatus::Completed
                } else {
                    crate::state::WorkItemStatus::Failed
                };
                item.result = Some(final_response.chars().take(500).collect());
                if item.source_kind.as_deref() == Some("delegation_review") {
                    item.review_verdict = super::multi_agent::parse_review_verdict(&final_response);
                }
                item.updated_at = state.now_iso();
                completed_work_item = Some(item.clone());
            }
        }
        state.save_work_items();
        if let Some(item) = completed_work_item.as_ref() {
            if let Some(verdict) = item.review_verdict.as_ref() {
                super::multi_agent::project_review_verdict(state, item, verdict);
            }
        }
        if let Some(session_id) = d.room_session_id.as_deref() {
            super::multi_agent::release_work_item_leases(
                state,
                session_id,
                work_item_id,
                effective_status,
            );
        }
    }
    let _ = super::multi_agent::auto_queue_delegation_review(
        state,
        &d,
        &final_response,
        effective_status,
        &gate_result,
    );
    super::delegation_analytics::log_delegation(&state.root, &d.project, &d.task, effective_status);

    // E25: Auto-update linked plan step
    if let (Some(plan_id), Some(step_idx)) = (&d.plan_id, d.plan_step) {
        let mut plans = super::plans::load_all_plans_internal(state);
        if let Some(plan) = plans.iter_mut().find(|p| p.id == *plan_id) {
            if step_idx < plan.steps.len() {
                plan.steps[step_idx].status = if effective_status == "complete" {
                    crate::commands::status::PlanStepStatus::Done
                } else {
                    crate::commands::status::PlanStepStatus::Failed
                };
                plan.steps[step_idx].delegation_id = Some(id.to_string());
                if let Some(work_item_id) = d.work_item_id.as_deref() {
                    plan.steps[step_idx].work_item_id = Some(work_item_id.to_string());
                }
                plan.steps[step_idx].result = Some(final_response.chars().take(200).collect());
                plan.updated = state.now_iso();
                if plan.steps.iter().all(|s| s.status.is_terminal()) {
                    plan.status = crate::commands::status::PlanStatus::Completed;
                }
                super::plans::save_plan_internal(state, plan);
                crate::log_info!(
                    "[delegation] updated plan '{}' step {} -> {}",
                    plan.title,
                    step_idx,
                    effective_status
                );
                if let Some(session_id) = d.room_session_id.as_deref() {
                    super::multi_agent::emit_pipeline_event(
                        state,
                        session_id,
                        "plan_step_updated",
                        "system",
                        &format!(
                            "Plan '{}' step {} -> {}",
                            plan.title,
                            step_idx + 1,
                            effective_status
                        ),
                        json!({
                            "plan_id": plan.id,
                            "step_index": step_idx,
                            "status": effective_status,
                            "project": d.project,
                        }),
                    );
                }
                // Notify in orchestrator chat (#50)
                let plan_msg = format!(
                    "📋 Plan '{}' step {}: {} ({})",
                    plan.title,
                    step_idx + 1,
                    effective_status,
                    d.project
                );
                let orch_file = state.chats_dir.join("_orchestrator.jsonl");
                let plan_entry =
                    serde_json::json!({"ts": state.now_iso(), "role": "system", "msg": plan_msg});
                super::jsonl::append_jsonl_logged(&orch_file, &plan_entry, "plan step update");
            }
        }
    }

    // Update linked strategy step
    if let (Some(strat_id), Some(step_id)) = (&d.strategy_id, &d.strategy_step_id) {
        super::strategy_models::update_step_from_delegation(
            state,
            strat_id,
            step_id,
            effective_status,
            Some(&final_response.chars().take(300).collect::<String>()),
        );
        crate::log_info!(
            "[delegation] updated strategy step {} -> {}",
            step_id,
            effective_status
        );
        if let Some(session_id) = d.room_session_id.as_deref() {
            super::multi_agent::emit_pipeline_event(
                state,
                session_id,
                "strategy_step_updated",
                "system",
                &format!("Strategy step {} -> {}", step_id, effective_status),
                json!({
                    "strategy_id": strat_id,
                    "step_id": step_id,
                    "status": effective_status,
                    "project": d.project,
                }),
            );
        }

        // Auto-queue next step if this one succeeded
        if effective_status == "complete" {
            let next = super::strategy::try_queue_next_steps(state, strat_id);
            if next > 0 {
                let orch_msg = format!("⚡ Strategy auto-queued {} next step(s)", next);
                let orch_file = state.chats_dir.join("_orchestrator.jsonl");
                super::jsonl::append_jsonl_logged(
                    &orch_file,
                    &json!({"ts": state.now_iso(), "role": "system", "msg": orch_msg}),
                    "strategy auto-queue",
                );
                if let Some(session_id) = d.room_session_id.as_deref() {
                    super::multi_agent::emit_pipeline_event(
                        state,
                        session_id,
                        "strategy_autoqueued",
                        "system",
                        &format!("Strategy auto-queued {} next step(s)", next),
                        json!({
                            "strategy_id": strat_id,
                            "count": next,
                        }),
                    );
                }
            }
        }
    }

    // Release project lock
    state.release_dir_lock(&d.project);
    crate::log_info!("[delegation:{}] released project lock", d.project);

    json!({
        "status": effective_status,
        "execution_status": execution_status,
        "project": d.project,
        "task": d.task,
        "response": final_response,
        "gate_result": gate_result,
    })
}
