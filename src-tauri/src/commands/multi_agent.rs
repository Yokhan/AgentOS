use crate::state::AppState;
use crate::state::{
    FileLease, FileLeaseStatus, MultiAgentSession, PresenceState, ProjectSession,
    ProjectSessionStatus, ReviewVerdict, ReviewVerdictStatus, SessionEvent, SessionMode,
    SessionParticipant, SessionStatus, WorkItem, WorkItemStatus,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::task::JoinSet;

fn new_id(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", prefix, nanos)
}

fn parse_mode(mode: Option<&str>) -> SessionMode {
    match mode
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "debate" => SessionMode::Debate,
        "parallel" => SessionMode::Parallel,
        "arbitration" => SessionMode::Arbitration,
        _ => SessionMode::Review,
    }
}

fn default_participants(state: &AppState) -> Vec<SessionParticipant> {
    vec![
        SessionParticipant {
            id: "claude_pm".to_string(),
            label: "Claude PM".to_string(),
            provider: super::provider_runner::orchestrator_provider(state),
            role: "product".to_string(),
            write_enabled: true,
        },
        SessionParticipant {
            id: "codex_tech".to_string(),
            label: "Codex Tech".to_string(),
            provider: super::provider_runner::technical_reviewer_provider(state),
            role: "technical".to_string(),
            write_enabled: false,
        },
    ]
}

fn default_orchestrator_participant_id(
    state: &AppState,
    participants: &[SessionParticipant],
) -> Option<String> {
    participants
        .iter()
        .find(|p| {
            p.role == "product"
                && p.provider == super::provider_runner::orchestrator_provider(state)
        })
        .map(|p| p.id.clone())
        .or_else(|| participants.first().map(|p| p.id.clone()))
}

fn session_orchestrator_participant_id(
    state: &AppState,
    session: &MultiAgentSession,
) -> Option<String> {
    session
        .orchestrator_participant_id
        .as_ref()
        .filter(|id| session.participants.iter().any(|p| p.id == **id))
        .cloned()
        .or_else(|| default_orchestrator_participant_id(state, &session.participants))
}

fn session_orchestrator_participant<'a>(
    state: &AppState,
    session: &'a MultiAgentSession,
) -> Option<&'a SessionParticipant> {
    let orchestrator_id = session_orchestrator_participant_id(state, session)?;
    session
        .participants
        .iter()
        .find(|participant| participant.id == orchestrator_id)
}

fn default_presence(participants: &[SessionParticipant]) -> HashMap<String, PresenceState> {
    participants
        .iter()
        .map(|p| (p.id.clone(), PresenceState::Idle))
        .collect()
}

fn write_enabled_participants(session: &MultiAgentSession) -> Vec<&SessionParticipant> {
    session
        .participants
        .iter()
        .filter(|p| p.write_enabled)
        .collect()
}

fn write_enabled_participant_for_provider(
    session: &MultiAgentSession,
    provider: crate::commands::provider_runner::ProviderKind,
) -> Option<&SessionParticipant> {
    session
        .participants
        .iter()
        .find(|p| p.write_enabled && p.provider == provider)
}

fn participant_can_execute_pa_commands(
    state: &AppState,
    session: &MultiAgentSession,
    participant: &SessionParticipant,
    analysis_only: bool,
) -> bool {
    participant_has_round_write_access(participant, analysis_only)
        && session_orchestrator_participant_id(state, session).as_deref()
            == Some(participant.id.as_str())
}

fn participant_has_round_write_access(
    participant: &SessionParticipant,
    analysis_only: bool,
) -> bool {
    !analysis_only && participant.write_enabled
}

fn nonterminal_work_item(item: &Value) -> bool {
    !matches!(
        item.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        "completed" | "failed" | "cancelled"
    )
}

fn overlapping_paths(left: &[String], right: &[String]) -> Vec<String> {
    let mut overlaps = Vec::new();
    for l in left {
        for r in right {
            let same = l == r;
            let prefix = l.starts_with(&(r.clone() + "/")) || r.starts_with(&(l.clone() + "/"));
            if (same || prefix) && !overlaps.iter().any(|p| p == l || p == r) {
                overlaps.push(l.clone());
                if l != r {
                    overlaps.push(r.clone());
                }
            }
        }
    }
    overlaps
}

fn write_conflicts_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let items = linked_work_items_for_session(state, session);
    let scoped: Vec<&Value> = items
        .iter()
        .filter(|item| {
            nonterminal_work_item(item)
                && item
                    .get("write_intent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("read_only")
                    != "read_only"
                && item
                    .get("declared_paths")
                    .and_then(|v| v.as_array())
                    .map(|arr| !arr.is_empty())
                    .unwrap_or(false)
        })
        .collect();
    let mut conflicts = Vec::new();
    for (idx, left) in scoped.iter().enumerate() {
        let left_paths: Vec<String> = left
            .get("declared_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        for right in scoped.iter().skip(idx + 1) {
            let right_paths: Vec<String> = right
                .get("declared_paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let overlaps = overlapping_paths(&left_paths, &right_paths);
            if overlaps.is_empty() {
                continue;
            }
            let left_intent = left
                .get("write_intent")
                .and_then(|v| v.as_str())
                .unwrap_or("read_only");
            let right_intent = right
                .get("write_intent")
                .and_then(|v| v.as_str())
                .unwrap_or("read_only");
            let blocking = left_intent == "exclusive_write" || right_intent == "exclusive_write";
            conflicts.push(json!({
                "blocking": blocking,
                "paths": overlaps,
                "left": {
                    "id": left.get("id").and_then(|v| v.as_str()).unwrap_or_default(),
                    "title": left.get("title").and_then(|v| v.as_str()).unwrap_or_default(),
                    "intent": left_intent,
                    "executor_provider": left.get("executor_provider").cloned().unwrap_or(Value::Null),
                },
                "right": {
                    "id": right.get("id").and_then(|v| v.as_str()).unwrap_or_default(),
                    "title": right.get("title").and_then(|v| v.as_str()).unwrap_or_default(),
                    "intent": right_intent,
                    "executor_provider": right.get("executor_provider").cloned().unwrap_or(Value::Null),
                }
            }));
        }
    }
    conflicts
}

fn conflicting_existing_work_items(
    state: &AppState,
    session: &MultiAgentSession,
    current_work_item_id: Option<&str>,
    declared_paths: &[String],
) -> Vec<Value> {
    if declared_paths.is_empty() {
        return Vec::new();
    }
    linked_work_items_for_session(state, session)
        .into_iter()
        .filter(|item| {
            nonterminal_work_item(item)
                && item
                    .get("write_intent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("read_only")
                    != "read_only"
                && item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| Some(id) != current_work_item_id)
                    .unwrap_or(true)
        })
        .filter_map(|item| {
            let paths: Vec<String> = item
                .get("declared_paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let overlaps = overlapping_paths(declared_paths, &paths);
            if overlaps.is_empty() {
                None
            } else {
                Some(json!({
                    "id": item.get("id").and_then(|v| v.as_str()).unwrap_or_default(),
                    "title": item.get("title").and_then(|v| v.as_str()).unwrap_or_default(),
                    "write_intent": item.get("write_intent").and_then(|v| v.as_str()).unwrap_or("read_only"),
                    "paths": overlaps,
                }))
            }
        })
        .collect()
}

fn blocking_overlaps_between_work_items(items: &[WorkItem]) -> Vec<Value> {
    let mut conflicts = Vec::new();
    for (idx, left) in items.iter().enumerate() {
        if matches!(
            left.write_intent,
            crate::state::WorkItemWriteIntent::ReadOnly
        ) || left.declared_paths.is_empty()
        {
            continue;
        }
        for right in items.iter().skip(idx + 1) {
            if matches!(
                right.write_intent,
                crate::state::WorkItemWriteIntent::ReadOnly
            ) || right.declared_paths.is_empty()
            {
                continue;
            }
            let overlaps = overlapping_paths(&left.declared_paths, &right.declared_paths);
            if overlaps.is_empty() {
                continue;
            }
            conflicts.push(json!({
                "blocking": true,
                "paths": overlaps,
                "left": {
                    "id": left.id,
                    "title": left.title,
                    "executor_provider": left.executor_provider.as_str(),
                    "write_intent": left.write_intent,
                },
                "right": {
                    "id": right.id,
                    "title": right.title,
                    "executor_provider": right.executor_provider.as_str(),
                    "write_intent": right.write_intent,
                }
            }));
        }
    }
    conflicts
}

fn participant_runtime_settings(state: &AppState, participant: &SessionParticipant) -> Value {
    let role_model_key = if participant.role == "product" {
        Some("orchestrator_model")
    } else {
        Some("technical_reviewer_model")
    };
    let role_effort_key = if participant.role == "product" {
        Some("orchestrator_effort")
    } else {
        Some("technical_reviewer_effort")
    };
    json!({
        "participant_id": participant.id,
        "provider": participant.provider.as_str(),
        "role": participant.role,
        "model": super::provider_runner::resolve_provider_model(
            state,
            participant.provider,
            None,
            role_model_key,
        ),
        "reasoning_effort": super::provider_runner::resolve_provider_effort(
            state,
            participant.provider,
            None,
            role_effort_key,
        ),
    })
}

pub fn parse_review_verdict(response: &str) -> Option<ReviewVerdict> {
    let lines: Vec<String> = response
        .lines()
        .map(|line| line.trim().trim_start_matches('#').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    let verdict_line = lines.iter().find(|line| {
        let upper = line.to_ascii_uppercase();
        upper.starts_with("APPROVE") || upper.starts_with("WARN") || upper.starts_with("FAIL")
    })?;
    let verdict_status = {
        let upper = verdict_line.to_ascii_uppercase();
        if upper.starts_with("APPROVE") {
            ReviewVerdictStatus::Approve
        } else if upper.starts_with("WARN") {
            ReviewVerdictStatus::Warn
        } else {
            ReviewVerdictStatus::Fail
        }
    };
    let summary = lines
        .iter()
        .find(|line| *line != verdict_line)
        .cloned()
        .unwrap_or_else(|| super::claude_runner::safe_truncate(verdict_line, 220).to_string());
    let next_action = lines.iter().find_map(|line| {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("next action:") {
            Some(line["next action:".len()..].trim().to_string())
        } else {
            None
        }
    });
    Some(ReviewVerdict {
        status: verdict_status,
        summary: super::claude_runner::safe_truncate(&summary, 300).to_string(),
        next_action: next_action
            .as_deref()
            .map(|line| super::claude_runner::safe_truncate(line, 240).to_string()),
        source_response: super::claude_runner::safe_truncate(response, 1200).to_string(),
    })
}

fn linked_file_leases_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let mut items: Vec<Value> = state
        .file_leases
        .lock()
        .map(|leases| {
            leases
                .values()
                .filter(|lease| {
                    lease.session_id == session.id
                        && matches!(lease.status, FileLeaseStatus::Active)
                })
                .map(|lease| serde_json::to_value(lease).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| {
        let at = a.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    items
}

fn active_lease_for_work_item(
    state: &AppState,
    session_id: &str,
    work_item_id: &str,
) -> Option<FileLease> {
    state.file_leases.lock().ok().and_then(|leases| {
        leases
            .values()
            .find(|lease| {
                lease.session_id == session_id
                    && lease.work_item_id == work_item_id
                    && matches!(lease.status, FileLeaseStatus::Active)
            })
            .cloned()
    })
}

fn release_single_file_lease(
    state: &AppState,
    lease_id: &str,
    reason: &str,
) -> Result<FileLease, String> {
    let released = {
        let mut leases = state.file_leases.lock().map_err(|e| e.to_string())?;
        let lease = leases
            .get_mut(lease_id)
            .ok_or_else(|| "Lease not found".to_string())?;
        if !matches!(lease.status, FileLeaseStatus::Active) {
            return Err("Lease is already released".to_string());
        }
        lease.status = FileLeaseStatus::Released;
        lease.updated_at = state.now_iso();
        lease.released_at = Some(state.now_iso());
        lease.clone()
    };
    state.save_file_leases();
    emit_pipeline_event(
        state,
        &released.session_id,
        "lease_released",
        "system",
        &format!(
            "Released lease {} for {}",
            released.id, released.work_item_id
        ),
        json!({
            "lease_id": released.id,
            "work_item_id": released.work_item_id,
            "participant_id": released.participant_id,
            "provider": released.provider.as_str(),
            "paths": released.paths,
            "reason": reason,
        }),
    );
    Ok(released)
}

fn acquire_work_item_lease_for_participant(
    state: &AppState,
    session: &MultiAgentSession,
    work_item: &WorkItem,
    participant: &SessionParticipant,
) -> Result<Option<FileLease>, String> {
    if matches!(
        work_item.write_intent,
        crate::state::WorkItemWriteIntent::ReadOnly
    ) || work_item.declared_paths.is_empty()
    {
        return Ok(None);
    }
    if !participant.write_enabled {
        return Err(format!(
            "{} is not the active writer for this room",
            participant.label
        ));
    }
    if participant.provider != work_item.executor_provider {
        return Err(format!(
            "Selected writer is {} ({}), but work item executor is {}",
            participant.label,
            participant.provider.as_str(),
            work_item.executor_provider.as_str()
        ));
    }
    if let Some(existing) = active_lease_for_work_item(state, &session.id, &work_item.id) {
        if existing.participant_id == participant.id {
            return Ok(Some(existing));
        }
        return Err(format!(
            "Work item already has an active lease owned by {}",
            existing.participant_id
        ));
    }
    let existing = linked_file_leases_for_session(state, session);
    for lease in existing {
        let same_work_item = lease
            .get("work_item_id")
            .and_then(|v| v.as_str())
            .map(|id| id == work_item.id)
            .unwrap_or(false);
        if same_work_item {
            continue;
        }
        let lease_paths: Vec<String> = lease
            .get("paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let overlaps = overlapping_paths(&work_item.declared_paths, &lease_paths);
        if overlaps.is_empty() {
            continue;
        }
        return Err(format!(
            "Lease conflict on {} with work item {}",
            overlaps.join(", "),
            lease
                .get("work_item_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        ));
    }
    let lease = FileLease {
        id: new_id("lease"),
        session_id: session.id.clone(),
        work_item_id: work_item.id.clone(),
        project: work_item.project.clone(),
        participant_id: participant.id.clone(),
        provider: participant.provider,
        write_intent: work_item.write_intent.clone(),
        paths: work_item.declared_paths.clone(),
        status: FileLeaseStatus::Active,
        created_at: state.now_iso(),
        updated_at: state.now_iso(),
        released_at: None,
    };
    if let Ok(mut leases) = state.file_leases.lock() {
        leases.insert(lease.id.clone(), lease.clone());
    }
    state.save_file_leases();
    emit_pipeline_event(
        state,
        &session.id,
        "lease_acquired",
        "system",
        &format!(
            "{} acquired {:?} lease for {}",
            participant.label, work_item.write_intent, work_item.title
        ),
        json!({
            "lease_id": lease.id,
            "work_item_id": work_item.id,
            "participant_id": participant.id,
            "provider": participant.provider.as_str(),
            "paths": work_item.declared_paths,
            "write_intent": work_item.write_intent,
        }),
    );
    Ok(Some(lease))
}

fn acquire_work_item_lease(
    state: &AppState,
    session: &MultiAgentSession,
    work_item: &WorkItem,
) -> Result<Option<FileLease>, String> {
    if matches!(
        work_item.write_intent,
        crate::state::WorkItemWriteIntent::ReadOnly
    ) || work_item.declared_paths.is_empty()
    {
        return Ok(None);
    }
    let writer = write_enabled_participant_for_provider(session, work_item.executor_provider)
        .ok_or_else(|| {
            format!(
                "{} does not currently have write access",
                work_item.executor_provider.as_str()
            )
        })?;
    acquire_work_item_lease_for_participant(state, session, work_item, writer)
}

pub fn release_work_item_leases(
    state: &AppState,
    session_id: &str,
    work_item_id: &str,
    reason: &str,
) -> Vec<String> {
    let mut released = Vec::new();
    if let Ok(mut leases) = state.file_leases.lock() {
        for lease in leases.values_mut() {
            if lease.session_id == session_id
                && lease.work_item_id == work_item_id
                && matches!(lease.status, FileLeaseStatus::Active)
            {
                lease.status = FileLeaseStatus::Released;
                lease.updated_at = state.now_iso();
                lease.released_at = Some(state.now_iso());
                released.push(lease.id.clone());
            }
        }
    }
    if !released.is_empty() {
        state.save_file_leases();
        emit_pipeline_event(
            state,
            session_id,
            "lease_released",
            "system",
            &format!("Released {} lease(s) for {}", released.len(), work_item_id),
            json!({
                "work_item_id": work_item_id,
                "lease_ids": released,
                "reason": reason,
            }),
        );
    }
    released
}

fn session_title(project: &str, title: Option<&str>) -> String {
    if let Some(title) = title.filter(|t| !t.trim().is_empty()) {
        return title.trim().to_string();
    }
    if project.is_empty() {
        "Dual-agent orchestration".to_string()
    } else {
        format!("{} dual-agent session", project)
    }
}

fn set_session_state<F>(state: &AppState, session_id: &str, updater: F)
where
    F: FnOnce(&mut MultiAgentSession),
{
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(session_id) {
            updater(session);
            session.updated_at = state.now_iso();
        }
    }
    state.save_sessions();
}

fn log_presence_event(
    state: &AppState,
    session_id: &str,
    participant: &SessionParticipant,
    presence: PresenceState,
    summary: &str,
) {
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session_id.to_string(),
        ts: state.now_iso(),
        kind: "agent_presence".to_string(),
        actor: participant.id.clone(),
        payload: json!({
            "presence": presence,
            "summary": summary,
            "label": participant.label,
        }),
    });
}

pub fn emit_pipeline_event(
    state: &AppState,
    session_id: &str,
    kind: &str,
    actor: &str,
    summary: &str,
    payload: Value,
) {
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session_id.to_string(),
        ts: state.now_iso(),
        kind: kind.to_string(),
        actor: actor.to_string(),
        payload: json!({
            "summary": summary,
            "data": payload,
        }),
    });
}

pub fn link_strategy_to_session(
    state: &AppState,
    session_id: &str,
    strategy_id: &str,
    title: &str,
) {
    set_session_state(state, session_id, |session| {
        if !session
            .linked_strategy_ids
            .iter()
            .any(|id| id == strategy_id)
        {
            session.linked_strategy_ids.push(strategy_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "strategy_linked",
        "system",
        &format!("Linked strategy {} ({})", title, strategy_id),
        json!({"strategy_id": strategy_id, "title": title}),
    );
}

pub fn link_project_session_to_session(
    state: &AppState,
    session_id: &str,
    project_session_id: &str,
    title: &str,
    project: &str,
) {
    set_session_state(state, session_id, |session| {
        if !session
            .linked_project_session_ids
            .iter()
            .any(|id| id == project_session_id)
        {
            session
                .linked_project_session_ids
                .push(project_session_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "project_session_linked",
        "system",
        &format!(
            "Linked project session {} ({}) for {}",
            title, project_session_id, project
        ),
        json!({
            "project_session_id": project_session_id,
            "title": title,
            "project": project,
        }),
    );
}

pub fn link_work_item_to_session(
    state: &AppState,
    session_id: &str,
    work_item_id: &str,
    title: &str,
    project: &str,
) {
    set_session_state(state, session_id, |session| {
        if !session
            .linked_work_item_ids
            .iter()
            .any(|id| id == work_item_id)
        {
            session.linked_work_item_ids.push(work_item_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "work_item_linked",
        "system",
        &format!(
            "Linked work item {} ({}) for {}",
            title, work_item_id, project
        ),
        json!({
            "work_item_id": work_item_id,
            "title": title,
            "project": project,
        }),
    );
}

#[allow(dead_code)]
pub fn link_tactic_to_session(state: &AppState, session_id: &str, tactic_id: &str, title: &str) {
    set_session_state(state, session_id, |session| {
        if !session.linked_tactic_ids.iter().any(|id| id == tactic_id) {
            session.linked_tactic_ids.push(tactic_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "tactic_linked",
        "system",
        &format!("Linked tactic {} ({})", title, tactic_id),
        json!({"tactic_id": tactic_id, "title": title}),
    );
}

pub fn link_plan_to_session(state: &AppState, session_id: &str, plan_id: &str, title: &str) {
    set_session_state(state, session_id, |session| {
        if !session.linked_plan_ids.iter().any(|id| id == plan_id) {
            session.linked_plan_ids.push(plan_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "plan_linked",
        "system",
        &format!("Linked plan {} ({})", title, plan_id),
        json!({"plan_id": plan_id, "title": title}),
    );
}

pub fn link_delegation_to_session(
    state: &AppState,
    session_id: &str,
    delegation_id: &str,
    project: &str,
    task: &str,
) {
    set_session_state(state, session_id, |session| {
        if !session
            .linked_delegation_ids
            .iter()
            .any(|id| id == delegation_id)
        {
            session
                .linked_delegation_ids
                .push(delegation_id.to_string());
        }
    });
    emit_pipeline_event(
        state,
        session_id,
        "delegation_linked",
        "system",
        &format!("Linked delegation {} for {}", delegation_id, project),
        json!({
            "delegation_id": delegation_id,
            "project": project,
            "task": super::claude_runner::safe_truncate(task, 240),
        }),
    );
}

fn latest_assistant_message(
    state: &AppState,
    session_id: &str,
    participant_id: &str,
) -> Option<String> {
    let chat_key = session_chat_key(session_id, participant_id);
    let path = state.chats_dir.join(format!("{}.jsonl", chat_key));
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines().rev() {
        let entry = serde_json::from_str::<Value>(line).ok()?;
        if entry.get("role").and_then(|v| v.as_str()) == Some("assistant") {
            if let Some(msg) = entry.get("msg").and_then(|v| v.as_str()) {
                return Some(msg.to_string());
            }
        }
    }
    None
}

fn linked_delegations_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let mut items: Vec<Value> = state
        .delegations
        .lock()
        .map(|delegations| {
            session
                .linked_delegation_ids
                .iter()
                .filter_map(|id| delegations.get(id))
                .map(|d| serde_json::to_value(d).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| {
        let at = a.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    items
}

fn parallel_batches_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let delegations = linked_delegations_for_session(state, session);
    let mut grouped: HashMap<String, Vec<Value>> = HashMap::new();
    for delegation in delegations {
        let Some(batch_id) = delegation
            .get("batch_id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        grouped.entry(batch_id).or_default().push(delegation);
    }
    let mut batches: Vec<Value> = grouped
        .into_iter()
        .map(|(batch_id, items)| {
            let mut pending = 0usize;
            let mut running = 0usize;
            let mut done = 0usize;
            let mut failed = 0usize;
            let mut rejected = 0usize;
            let mut projects: Vec<String> = Vec::new();
            let mut providers: Vec<String> = Vec::new();
            let mut work_item_ids: Vec<String> = Vec::new();
            let mut project_session_ids: Vec<String> = Vec::new();
            let newest_ts = items
                .iter()
                .filter_map(|item| item.get("ts").and_then(|v| v.as_str()))
                .max()
                .unwrap_or("")
                .to_string();
            for item in &items {
                match item.get("status").and_then(|v| v.as_str()).unwrap_or("") {
                    "pending" | "needs_permission" => pending += 1,
                    "running" => running += 1,
                    "done" => done += 1,
                    "failed" => failed += 1,
                    "rejected" => rejected += 1,
                    _ => {}
                }
                if let Some(project) = item.get("project").and_then(|v| v.as_str()) {
                    if !projects.iter().any(|p| p == project) {
                        projects.push(project.to_string());
                    }
                }
                if let Some(provider) = item.get("executor_provider").and_then(|v| v.as_str()) {
                    if !providers.iter().any(|p| p == provider) {
                        providers.push(provider.to_string());
                    }
                }
                if let Some(work_item_id) = item.get("work_item_id").and_then(|v| v.as_str()) {
                    if !work_item_ids.iter().any(|id| id == work_item_id) {
                        work_item_ids.push(work_item_id.to_string());
                    }
                }
                if let Some(project_session_id) =
                    item.get("project_session_id").and_then(|v| v.as_str())
                {
                    if !project_session_ids
                        .iter()
                        .any(|id| id == project_session_id)
                    {
                        project_session_ids.push(project_session_id.to_string());
                    }
                }
            }
            let total = items.len();
            let terminal = done + failed + rejected;
            let status = if terminal == total {
                if failed > 0 || rejected > 0 {
                    "completed_with_failures"
                } else {
                    "completed"
                }
            } else if running > 0 {
                "running"
            } else {
                "queued"
            };
            json!({
                "batch_id": batch_id,
                "status": status,
                "total": total,
                "pending": pending,
                "running": running,
                "done": done,
                "failed": failed,
                "rejected": rejected,
                "projects": projects,
                "providers": providers,
                "work_item_ids": work_item_ids,
                "project_session_ids": project_session_ids,
                "delegations": items,
                "updated_at": newest_ts,
            })
        })
        .collect();
    batches.sort_by(|a, b| {
        let at = a.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    batches
}

fn linked_project_sessions_for_session(
    state: &AppState,
    session: &MultiAgentSession,
) -> Vec<Value> {
    let mut items: Vec<Value> = state
        .project_sessions
        .lock()
        .map(|project_sessions| {
            session
                .linked_project_session_ids
                .iter()
                .filter_map(|id| project_sessions.get(id))
                .map(|item| serde_json::to_value(item).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| {
        let at = a.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    items
}

fn linked_work_items_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let mut items: Vec<Value> = state
        .work_items
        .lock()
        .map(|work_items| {
            session
                .linked_work_item_ids
                .iter()
                .filter_map(|id| work_items.get(id))
                .map(|item| serde_json::to_value(item).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| {
        let at = a.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    items
}

fn linked_inbox_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let linked_ids: std::collections::HashSet<String> =
        session.linked_delegation_ids.iter().cloned().collect();
    let mut items: Vec<Value> = state
        .inbox
        .lock()
        .map(|inbox| {
            inbox
                .iter()
                .filter(|item| {
                    item.room_session_id.as_deref() == Some(session.id.as_str())
                        || item
                            .delegation_id
                            .as_ref()
                            .map(|id| linked_ids.contains(id))
                            .unwrap_or(false)
                        || (!session.project.is_empty() && item.project == session.project)
                })
                .map(|item| serde_json::to_value(item).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    items.sort_by(|a, b| {
        let at = a.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        bt.cmp(at)
    });
    items
}

fn linked_signals_for_session(state: &AppState, session: &MultiAgentSession) -> Vec<Value> {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let acked: std::collections::HashSet<String> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("ack"))
        .filter_map(|v| {
            v.get("signal_id")
                .and_then(|i| i.as_str())
                .map(String::from)
        })
        .collect();
    let linked_ids: std::collections::HashSet<String> =
        session.linked_delegation_ids.iter().cloned().collect();
    content
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("ack"))
        .filter(|v| !acked.contains(v.get("id").and_then(|i| i.as_str()).unwrap_or("")))
        .filter(|v| {
            v.get("delegation_id")
                .and_then(|id| id.as_str())
                .map(|id| linked_ids.contains(id))
                .unwrap_or(false)
                || (!session.project.is_empty()
                    && v.get("project").and_then(|p| p.as_str()) == Some(session.project.as_str()))
        })
        .take(20)
        .collect()
}

fn build_agent_prompt(
    state: &AppState,
    session: &MultiAgentSession,
    participant: &SessionParticipant,
    message: &str,
    analysis_only: bool,
) -> String {
    let mut prompt = String::new();
    let orchestrator = session_orchestrator_participant(state, session);
    let is_orchestrator =
        orchestrator.map(|current| current.id.as_str()) == Some(participant.id.as_str());
    prompt.push_str("You are operating inside AgentOS multi-agent orchestration.\n");
    prompt.push_str(&format!(
        "Session: {} ({})\nMode: {:?}\nProject: {}\nRole: {}\nParticipant: {}\n\n",
        session.title,
        session.id,
        session.mode,
        if session.project.is_empty() {
            "_orchestrator"
        } else {
            &session.project
        },
        participant.role,
        participant.label
    ));

    let others: Vec<String> = session
        .participants
        .iter()
        .filter(|p| p.id != participant.id)
        .map(|p| format!("{} ({}, {})", p.label, p.role, p.provider.as_str()))
        .collect();
    if !others.is_empty() {
        prompt.push_str("Other visible participants:\n");
        for other in others {
            prompt.push_str(&format!("- {}\n", other));
        }
        prompt.push('\n');
    }

    match participant.role.as_str() {
        "product" => {
            prompt.push_str(
                "Focus on product framing, decomposition, priorities, and arbitration-ready proposals.\n",
            );
        }
        "technical" => {
            prompt.push_str(
                "Focus on architecture, implementation risk, correctness, verification, and technical critique.\n",
            );
        }
        _ => {}
    }

    if participant.write_enabled {
        prompt.push_str("You currently have write access in this room.\n");
        prompt.push_str("Parallel write is allowed only on disjoint leased paths. Do not assume global exclusive ownership.\n");
        if !analysis_only && !is_orchestrator {
            let owner = orchestrator
                .map(|current| current.label.clone())
                .unwrap_or_else(|| "another participant".to_string());
            prompt.push_str(&format!(
                "Execution mode can write through you, but orchestration currently belongs to {}. You may still execute assigned work, however PA commands will not run until you are set as orchestrator.\n",
                owner
            ));
        }
    } else {
        prompt.push_str("You are review-oriented and should challenge weak assumptions before proposing changes.\n");
        if !analysis_only {
            prompt.push_str("Execution mode is active for the room, but you still do not have write access. Stay read-only unless the user explicitly grants writer status.\n");
        }
    }

    if analysis_only {
        prompt.push_str("This is an analysis-only round. Do not modify files, run migrations, or execute write operations.\n");
        prompt.push_str("If code changes are needed, end your response with a single line in this exact format:\n");
        prompt.push_str("FILES: path/to/file.ext, another/path.ext\n");
        prompt.push_str("If no file changes are needed, end with:\nFILES: none\n");
    } else {
        prompt.push_str("If you expect file changes, end your response with a single FILES: line listing the likely touched files.\n");
    }

    if participant_can_execute_pa_commands(state, session, participant, analysis_only) {
        prompt.push_str("\nExecution mode is live for this participant. Plain analysis does not trigger orchestration.\n");
        prompt
            .push_str("The user selected you as the execution lead/orchestrator for this room.\n");
        prompt.push_str("If the user approved execution or asks to proceed, do not only propose a plan. Act through structured PA commands.\n");
        prompt.push_str("Executable PA command tags must be on their own lines outside fenced code blocks. Tags inside fenced code blocks are examples only and will be ignored.\n");
        prompt.push_str("Common commands:\n");
        prompt.push_str("- [DASHBOARD_FULL]\n");
        prompt.push_str("- [DELEGATE_STATUS:?failed]\n");
        prompt.push_str("- [GIT_STATUS_ALL]\n");
        prompt.push_str("- [TEMPLATE_AUDIT]\n");
        prompt.push_str("- [DELEGATE:Project]task[/DELEGATE]\n");
        prompt.push_str("- [DELEGATE_BATCH:p1,p2]task[/DELEGATE_BATCH]\n");
        prompt.push_str("- [DELEGATE_CHAIN:Project]step1\\nstep2[/DELEGATE_CHAIN]\n");
        prompt.push_str("- [PLAN:title]Project: task\\n...[/PLAN]\n");
        prompt.push_str("- [STRATEGY:goal]context[/STRATEGY]\n");
        prompt.push_str("- [NOTIFY:message]\n");
        prompt
            .push_str("If you are only reviewing, say so plainly and do not emit command tags.\n");
    }

    if !session.current_working_set.is_empty() {
        prompt.push_str("\nCurrent working set:\n");
        for path in &session.current_working_set {
            prompt.push_str(&format!("- {}\n", path));
        }
    }

    if let Some(challenge) = session.pending_challenge.as_deref() {
        prompt.push_str(&format!("\nPending challenge target: {}\n", challenge));
    }
    if let Some(rebuttal) = session.pending_rebuttal.as_deref() {
        prompt.push_str(&format!("Pending rebuttal target: {}\n", rebuttal));
    }

    let recent_events = state.get_session_events(&session.id, 8);
    if !recent_events.is_empty() {
        prompt.push_str("\nRecent session ledger:\n");
        for evt in recent_events {
            let summary = evt
                .payload
                .get("summary")
                .and_then(|v| v.as_str())
                .or_else(|| evt.payload.get("response").and_then(|v| v.as_str()))
                .or_else(|| evt.payload.get("message").and_then(|v| v.as_str()))
                .unwrap_or("");
            prompt.push_str(&format!("- [{}] {} {}\n", evt.actor, evt.kind, summary));
        }
    }

    prompt.push_str("\nUser request:\n");
    prompt.push_str(message);
    prompt
}

const ROOM_AUTO_CONTINUE_TURNS: usize = 20;
const ROOM_AUTO_CONTINUE_REPEAT_LIMIT: usize = 2;

#[derive(Default)]
struct RoomPaFeedback {
    items: Vec<String>,
    fragments: Vec<String>,
    actionable: usize,
    warnings: usize,
}

impl RoomPaFeedback {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn signature(&self) -> String {
        self.items.join("\n")
    }
}

fn execute_room_pa_commands(state: &AppState, response: &str) -> RoomPaFeedback {
    let commands = super::pa_commands::parse_pa_commands(response, state);
    let warnings = super::pa_commands::detect_malformed_commands(response);
    let mut feedback = RoomPaFeedback::default();

    for parsed in &commands {
        if !parsed.valid {
            if let Some(err) = &parsed.error {
                feedback.warnings += 1;
                feedback.items.push(format!("warning -> {}", err));
                feedback.fragments.push(format!("**Warning:** {}", err));
            }
            continue;
        }
        let command_label = super::pa_commands::describe_pa_command(&parsed.cmd);
        feedback.actionable += 1;
        match super::pa_commands::execute_pa_command(state, &parsed.cmd) {
            Some(text) => {
                let preview = super::claude_runner::safe_truncate(&text, 900);
                feedback
                    .items
                    .push(format!("{} -> {}", command_label, preview));
                feedback
                    .fragments
                    .push(format!("{}:\n{}", command_label, text));
            }
            None => {
                let done = format!("Completed {} (no output)", command_label);
                feedback.items.push(done.clone());
                feedback.fragments.push(done);
            }
        }
    }

    for warning in warnings {
        feedback.warnings += 1;
        feedback.items.push(format!("warning -> {}", warning));
        feedback.fragments.push(format!("**Note:** {}", warning));
    }

    feedback
}

fn build_room_auto_continue_message(turn: usize, feedback: &RoomPaFeedback) -> String {
    format!(
        "[AUTO-CONTINUE AFTER AGENTOS COMMANDS]\n\
         AgentOS executed the PA commands from your previous response.\n\
         Results:\n{}\n\n\
         Continue the room task autonomously. Stop by returning a final status with no PA command tags when complete or blocked. \
         Emit the next PA command tags only when another AgentOS action is actually required. Do not ask the user to type continue.\n\
         Auto-continue turn: {}/{} safety ceiling. Actionable commands: {}. Warnings: {}.",
        feedback.items
            .iter()
            .enumerate()
            .map(|(idx, item)| format!("{}. {}", idx + 1, item))
            .collect::<Vec<_>>()
            .join("\n"),
        turn,
        ROOM_AUTO_CONTINUE_TURNS,
        feedback.actionable,
        feedback.warnings
    )
}

fn normalize_file_intent(raw: &str) -> Option<String> {
    let normalized = raw
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim()
        .to_string();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("none") {
        return None;
    }
    if normalized.starts_with('/') || normalized.contains("..") {
        return None;
    }
    Some(normalized)
}

fn extract_file_intents(response: &str) -> Vec<String> {
    let mut intents = Vec::new();
    for line in response.lines().rev() {
        let trimmed = line.trim();
        let upper = trimmed.to_ascii_uppercase();
        let payload = if upper.starts_with("FILES:") {
            Some(trimmed[6..].trim())
        } else if upper.starts_with("[FILES:") && trimmed.ends_with(']') {
            Some(trimmed[7..trimmed.len().saturating_sub(1)].trim())
        } else {
            None
        };
        let Some(payload) = payload else {
            continue;
        };
        if payload.eq_ignore_ascii_case("none") {
            break;
        }
        for item in payload.split(',') {
            if let Some(path) = normalize_file_intent(item) {
                if !intents.iter().any(|p| p == &path) {
                    intents.push(path);
                }
            }
        }
        break;
    }
    intents
}

fn update_session_working_set(state: &AppState, session_id: &str, ts: &str, intents: &[String]) {
    if intents.is_empty() {
        return;
    }
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(existing) = sessions.get_mut(session_id) {
            existing.updated_at = ts.to_string();
            for path in intents {
                if !existing.current_working_set.iter().any(|p| p == path) {
                    existing.current_working_set.push(path.clone());
                }
            }
            if existing.current_working_set.len() > 24 {
                let drain = existing.current_working_set.len() - 24;
                existing.current_working_set.drain(0..drain);
            }
        }
    }
    state.save_sessions();
}

fn session_chat_key(session_id: &str, participant_id: &str) -> String {
    format!("session-{}-{}", session_id, participant_id)
}

fn shared_chat_key_for_session(session: &MultiAgentSession) -> String {
    if session.project.trim().is_empty() {
        "_orchestrator".to_string()
    } else {
        session.project.clone()
    }
}

fn append_shared_user_message(
    state: &AppState,
    session: &MultiAgentSession,
    message: &str,
    round_id: Option<&str>,
    analysis_only: bool,
) {
    let chat_key = shared_chat_key_for_session(session);
    let chat_file = state.chats_dir.join(format!("{}.jsonl", chat_key));
    super::jsonl::append_jsonl_logged(
        &chat_file,
        &json!({
            "ts": state.now_iso(),
            "role": "user",
            "msg": message,
            "mode": "duo",
            "room_session_id": session.id,
            "analysis_only": analysis_only,
            "round_id": round_id,
        }),
        "shared duo user msg",
    );
}

fn append_shared_assistant_message(
    state: &AppState,
    session: &MultiAgentSession,
    participant: &SessionParticipant,
    response: &str,
    round_id: Option<&str>,
    analysis_only: bool,
    file_intents: &[String],
) {
    let chat_key = shared_chat_key_for_session(session);
    let chat_file = state.chats_dir.join(format!("{}.jsonl", chat_key));
    super::jsonl::append_jsonl_logged(
        &chat_file,
        &json!({
            "ts": state.now_iso(),
            "role": "assistant",
            "msg": response,
            "mode": "duo",
            "room_session_id": session.id,
            "participant": participant.id,
            "provider": participant.provider.as_str(),
            "meta": format!(" · {}", participant.label),
            "analysis_only": analysis_only,
            "round_id": round_id,
            "file_intents": file_intents,
        }),
        "shared duo assistant msg",
    );
}

fn parse_work_item_assignee(value: Option<&str>) -> crate::state::WorkItemAssignee {
    match value
        .unwrap_or("agent")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "user" => crate::state::WorkItemAssignee::User,
        _ => crate::state::WorkItemAssignee::Agent,
    }
}

fn parse_work_item_write_intent(value: Option<&str>) -> crate::state::WorkItemWriteIntent {
    match value
        .unwrap_or("read_only")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "exclusive_write" => crate::state::WorkItemWriteIntent::ExclusiveWrite,
        "propose_write" => crate::state::WorkItemWriteIntent::ProposeWrite,
        _ => crate::state::WorkItemWriteIntent::ReadOnly,
    }
}

fn parse_declared_paths(paths: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = Vec::new();
    for raw in paths.unwrap_or_default() {
        if let Some(path) = normalize_file_intent(&raw) {
            if !normalized.iter().any(|existing| existing == &path) {
                normalized.push(path);
            }
        }
    }
    normalized
}

fn parse_verify_condition(
    value: Option<Value>,
) -> Result<Option<crate::commands::strategy_models::VerifyCondition>, String> {
    match value {
        Some(v) if !v.is_null() => serde_json::from_value(v)
            .map(Some)
            .map_err(|e| format!("Invalid verify condition: {}", e)),
        _ => Ok(None),
    }
}

fn create_project_session_record(
    state: &AppState,
    parent_room_session_id: &str,
    project: &str,
    title: Option<&str>,
    executor_provider: super::provider_runner::ProviderKind,
    reviewer_provider: Option<super::provider_runner::ProviderKind>,
) -> ProjectSession {
    ProjectSession {
        id: new_id("ps"),
        parent_room_session_id: parent_room_session_id.to_string(),
        project: project.to_string(),
        title: title
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().to_string())
            .unwrap_or_else(|| format!("{} work session", project)),
        status: ProjectSessionStatus::Active,
        executor_provider,
        reviewer_provider,
        linked_work_item_ids: Vec::new(),
        created_at: state.now_iso(),
        updated_at: state.now_iso(),
    }
}

fn create_work_item_record(
    state: &AppState,
    parent_room_session_id: &str,
    project_session_id: Option<&str>,
    project: &str,
    title: Option<&str>,
    task: &str,
    executor_provider: super::provider_runner::ProviderKind,
    reviewer_provider: Option<super::provider_runner::ProviderKind>,
    assignee: crate::state::WorkItemAssignee,
    write_intent: crate::state::WorkItemWriteIntent,
    declared_paths: Vec<String>,
    verify: Option<crate::commands::strategy_models::VerifyCondition>,
    source_kind: Option<&str>,
    source_id: Option<&str>,
) -> WorkItem {
    let fallback_title = task
        .lines()
        .next()
        .unwrap_or("work item")
        .chars()
        .take(80)
        .collect::<String>();
    WorkItem {
        id: new_id("wi"),
        parent_room_session_id: parent_room_session_id.to_string(),
        project_session_id: project_session_id.map(|s| s.to_string()),
        project: project.to_string(),
        title: title
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().to_string())
            .unwrap_or(fallback_title),
        task: task.trim().to_string(),
        executor_provider,
        reviewer_provider,
        assignee,
        write_intent,
        declared_paths,
        verify,
        status: WorkItemStatus::Ready,
        delegation_id: None,
        result: None,
        review_verdict: None,
        source_kind: source_kind.map(|s| s.to_string()),
        source_id: source_id.map(|s| s.to_string()),
        created_at: state.now_iso(),
        updated_at: state.now_iso(),
    }
}

fn existing_work_item_by_source(
    state: &AppState,
    parent_room_session_id: &str,
    source_kind: &str,
    source_id: &str,
) -> Option<WorkItem> {
    state.work_items.lock().ok().and_then(|items| {
        items
            .values()
            .find(|item| {
                item.parent_room_session_id == parent_room_session_id
                    && item.source_kind.as_deref() == Some(source_kind)
                    && item.source_id.as_deref() == Some(source_id)
            })
            .cloned()
    })
}

pub fn work_item_plan_step_source(work_item: &WorkItem) -> Option<(String, String)> {
    if work_item.source_kind.as_deref() != Some("plan_step") {
        return None;
    }
    work_item
        .source_id
        .as_deref()
        .and_then(super::plans::parse_plan_step_source_id)
}

fn load_session_and_participant(
    state: &AppState,
    session_id: &str,
    participant_id: &str,
) -> Result<(MultiAgentSession, SessionParticipant), String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get(session_id)
        .cloned()
        .ok_or_else(|| "Session not found".to_string())?;
    let participant = session
        .participants
        .iter()
        .find(|p| p.id == participant_id)
        .cloned()
        .ok_or_else(|| "Participant not found".to_string())?;
    Ok((session, participant))
}

pub fn auto_queue_delegation_review(
    state: &AppState,
    delegation: &crate::state::Delegation,
    final_response: &str,
    effective_status: &str,
    gate_result: &Option<crate::commands::gate::GateResult>,
) -> Result<Option<Value>, String> {
    let Some(room_session_id) = delegation.room_session_id.as_deref() else {
        return Ok(None);
    };
    let Some(reviewer_provider) = delegation.reviewer_provider else {
        return Ok(None);
    };
    if let Some(work_item_id) = delegation.work_item_id.as_deref() {
        if let Some(item) = state
            .work_items
            .lock()
            .ok()
            .and_then(|items| items.get(work_item_id).cloned())
        {
            if item.source_kind.as_deref() == Some("delegation_review") {
                return Ok(None);
            }
        }
    }
    if existing_work_item_by_source(state, room_session_id, "delegation_review", &delegation.id)
        .is_some()
    {
        return Ok(None);
    }
    let gate_summary = gate_result
        .as_ref()
        .map(|gate| {
            let mut parts = vec![format!("gate status: {:?}", gate.status)];
            if !gate.errors.is_empty() {
                parts.push(format!(
                    "gate errors: {}",
                    super::claude_runner::safe_truncate(&gate.errors.join("; "), 300)
                ));
            }
            if let Some(verify_output) = gate.verify_output.as_deref() {
                parts.push(format!(
                    "verify output: {}",
                    super::claude_runner::safe_truncate(verify_output, 400)
                ));
            }
            parts.join("\n")
        })
        .unwrap_or_else(|| "gate status: none".to_string());
    let review_task = format!(
        "Read-only review for completed delegation {delegation_id} in project {project}.\n\
Do not modify files. This is a reviewer lane.\n\
Assess whether the result is acceptable, whether gate output indicates hidden risk, and whether follow-up work is needed.\n\
Reply with a concise verdict header APPROVE, WARN, or FAIL, then rationale and next action.\n\n\
Original task:\n{task}\n\n\
Execution outcome:\n{effective_status}\n\n\
Delegation response summary:\n{response}\n\n\
{gate_summary}",
        delegation_id = delegation.id,
        project = delegation.project,
        task = delegation.task,
        effective_status = effective_status,
        response = super::claude_runner::safe_truncate(final_response, 1200),
        gate_summary = gate_summary,
    );
    let review_title = format!(
        "Review delegation {}",
        delegation.id.chars().take(8).collect::<String>()
    );
    let review_item = create_work_item_record(
        state,
        room_session_id,
        delegation.project_session_id.as_deref(),
        &delegation.project,
        Some(&review_title),
        &review_task,
        reviewer_provider,
        None,
        crate::state::WorkItemAssignee::Agent,
        crate::state::WorkItemWriteIntent::ReadOnly,
        Vec::new(),
        None,
        Some("delegation_review"),
        Some(&delegation.id),
    );
    if let Ok(mut work_items) = state.work_items.lock() {
        work_items.insert(review_item.id.clone(), review_item.clone());
    }
    if let Some(project_session_id) = delegation.project_session_id.as_deref() {
        if let Ok(mut project_sessions) = state.project_sessions.lock() {
            if let Some(session) = project_sessions.get_mut(project_session_id) {
                if !session
                    .linked_work_item_ids
                    .iter()
                    .any(|id| id == &review_item.id)
                {
                    session.linked_work_item_ids.push(review_item.id.clone());
                }
                session.updated_at = state.now_iso();
            }
        }
        state.save_project_sessions();
    }
    state.save_work_items();
    link_work_item_to_session(
        state,
        room_session_id,
        &review_item.id,
        &review_item.title,
        &review_item.project,
    );
    emit_pipeline_event(
        state,
        room_session_id,
        "review_work_item_created",
        "system",
        &format!(
            "Created reviewer lane {} for delegation {}",
            review_item.id, delegation.id
        ),
        json!({
            "work_item_id": review_item.id,
            "delegation_id": delegation.id,
            "provider": reviewer_provider.as_str(),
            "project": delegation.project,
        }),
    );
    let queued = queue_work_item_execution_internal(state, &review_item.id, None);
    if queued.get("status").and_then(|v| v.as_str()) == Some("ok") {
        emit_pipeline_event(
            state,
            room_session_id,
            "review_work_item_queued",
            "system",
            &format!(
                "Queued reviewer lane {} for delegation {}",
                review_item.id, delegation.id
            ),
            json!({
                "work_item_id": review_item.id,
                "delegation_id": delegation.id,
                "provider": reviewer_provider.as_str(),
                "queue": queued,
            }),
        );
        Ok(Some(queued))
    } else {
        emit_pipeline_event(
            state,
            room_session_id,
            "review_work_item_queue_failed",
            "system",
            &format!(
                "Reviewer lane {} was created but queue failed for delegation {}",
                review_item.id, delegation.id
            ),
            json!({
                "work_item_id": review_item.id,
                "delegation_id": delegation.id,
                "provider": reviewer_provider.as_str(),
                "queue": queued,
            }),
        );
        Ok(Some(queued))
    }
}

pub fn project_review_verdict(state: &AppState, work_item: &WorkItem, verdict: &ReviewVerdict) {
    let room_session_id = work_item.parent_room_session_id.as_str();
    let source_delegation_id = work_item.source_id.as_deref().unwrap_or_default();
    let mut source_work_item_id: Option<String> = None;
    if !source_delegation_id.is_empty() {
        if let Ok(mut delegations) = state.delegations.lock() {
            if let Some(source_delegation) = delegations.get_mut(source_delegation_id) {
                source_delegation.review_verdict = Some(verdict.clone());
                source_work_item_id = source_delegation.work_item_id.clone();
            }
        }
        state.save_delegations();
    }
    if let Some(source_work_item_id) = source_work_item_id.as_deref() {
        if let Ok(mut work_items) = state.work_items.lock() {
            if let Some(source_work_item) = work_items.get_mut(source_work_item_id) {
                source_work_item.review_verdict = Some(verdict.clone());
                source_work_item.updated_at = state.now_iso();
            }
        }
        state.save_work_items();
    }
    let status_label = match verdict.status {
        ReviewVerdictStatus::Approve => "approve",
        ReviewVerdictStatus::Warn => "warn",
        ReviewVerdictStatus::Fail => "fail",
    };
    emit_pipeline_event(
        state,
        room_session_id,
        "review_verdict",
        "system",
        &format!(
            "Reviewer verdict for delegation {} -> {}",
            source_delegation_id, status_label
        ),
        json!({
            "work_item_id": work_item.id,
            "delegation_id": source_delegation_id,
            "status": status_label,
            "summary": verdict.summary,
            "next_action": verdict.next_action,
            "provider": work_item.executor_provider.as_str(),
            "source_work_item_id": source_work_item_id,
        }),
    );
    let needs_user = !matches!(verdict.status, ReviewVerdictStatus::Approve);
    super::inbox::push_inbox(
        state,
        &work_item.project,
        "review_verdict",
        &format!(
            "{} [{}] {}",
            source_delegation_id, status_label, verdict.summary
        ),
        needs_user,
        if source_delegation_id.is_empty() {
            None
        } else {
            Some(source_delegation_id)
        },
        Some(room_session_id),
    );
    match verdict.status {
        ReviewVerdictStatus::Approve => {}
        ReviewVerdictStatus::Warn => super::signals::emit_signal(
            state,
            super::signals::SignalSource::Reviewer,
            super::signals::Severity::Warn,
            Some(&work_item.project),
            &format!(
                "Reviewer WARN on {}: {}",
                source_delegation_id, verdict.summary
            ),
            if source_delegation_id.is_empty() {
                None
            } else {
                Some(source_delegation_id)
            },
        ),
        ReviewVerdictStatus::Fail => super::signals::emit_signal(
            state,
            super::signals::SignalSource::Reviewer,
            super::signals::Severity::Critical,
            Some(&work_item.project),
            &format!(
                "Reviewer FAIL on {}: {}",
                source_delegation_id, verdict.summary
            ),
            if source_delegation_id.is_empty() {
                None
            } else {
                Some(source_delegation_id)
            },
        ),
    }
}

fn resolve_session_cwd(
    state: &AppState,
    session: &MultiAgentSession,
) -> Result<(std::path::PathBuf, String, String), String> {
    if session.project.is_empty() {
        let (_, cwd) = state.get_orch_dir();
        Ok((
            cwd,
            "_orchestrator".to_string(),
            "_orchestrator".to_string(),
        ))
    } else {
        let cwd = state.validate_project(&session.project)?;
        Ok((cwd, session.project.clone(), session.project.clone()))
    }
}

fn run_session_agent_core(
    state: &Arc<AppState>,
    session: &MultiAgentSession,
    participant: &SessionParticipant,
    message: &str,
    shared_thread_user_message: Option<&str>,
    model: Option<String>,
    reasoning_effort: Option<String>,
    analysis_only: bool,
    round_id: Option<&str>,
) -> Result<Value, String> {
    let (cwd, lock_key, permission_project) = resolve_session_cwd(state, session)?;
    let chat_key = session_chat_key(&session.id, &participant.id);
    let chat_file = state.chats_dir.join(format!("{}.jsonl", chat_key));
    let prompt = build_agent_prompt(state, session, participant, message, analysis_only);
    let role_model_key = if participant.role == "product" {
        Some("orchestrator_model")
    } else {
        Some("technical_reviewer_model")
    };
    let role_effort_key = if participant.role == "product" {
        Some("orchestrator_effort")
    } else {
        Some("technical_reviewer_effort")
    };
    let resolved_model = super::provider_runner::resolve_provider_model(
        state,
        participant.provider,
        model.as_deref(),
        role_model_key,
    );
    let resolved_effort = super::provider_runner::resolve_provider_effort(
        state,
        participant.provider,
        reasoning_effort.as_deref(),
        role_effort_key,
    );
    let write_access = participant_has_round_write_access(participant, analysis_only);
    let perm_path = if analysis_only || !write_access {
        super::claude_runner::get_permission_path_for_profile(state, "restrictive")
    } else {
        super::claude_runner::get_permission_path(state, &permission_project)
    };
    let now = state.now_iso();

    if round_id.is_none() {
        if let Some(public_message) = shared_thread_user_message {
            append_shared_user_message(state, session, public_message, None, analysis_only);
        }
        state.append_session_event(&SessionEvent {
            id: new_id("evt"),
            session_id: session.id.clone(),
            ts: now.clone(),
            kind: "user_message".to_string(),
            actor: "user".to_string(),
            payload: json!({
                "message": message,
                "analysis_only": analysis_only,
                "round_id": round_id,
                "summary": super::claude_runner::safe_truncate(message, 200),
            }),
        });
    }
    super::jsonl::append_jsonl_logged(
        &chat_file,
        &json!({
            "ts": now,
            "role": "user",
            "msg": message,
            "participant": participant.id,
            "analysis_only": analysis_only,
            "round_id": round_id,
        }),
        "session user msg",
    );
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: now.clone(),
        kind: "message_requested".to_string(),
        actor: participant.id.clone(),
        payload: json!({
            "message": message,
            "provider": participant.provider.as_str(),
            "analysis_only": analysis_only,
            "round_id": round_id,
            "summary": format!("{} received a new prompt", participant.label),
        }),
    });

    set_session_state(state, &session.id, |existing| {
        existing.active_round_id = round_id.map(|id| id.to_string());
        existing.active_speaker = Some(participant.id.clone());
        existing
            .presence
            .insert(participant.id.clone(), PresenceState::Thinking);
    });
    log_presence_event(
        state,
        &session.id,
        participant,
        PresenceState::Thinking,
        &format!("{} is thinking", participant.label),
    );

    let activity_label = if analysis_only {
        format!("{} analysis", participant.label)
    } else if !write_access {
        format!("{} execute-review", participant.label)
    } else {
        participant.label.clone()
    };
    super::process_manager::set_activity(state, &chat_key, "multi-agent", &activity_label);
    if write_access {
        state.acquire_dir_lock(&lock_key);
    }
    let response = super::provider_runner::run_provider_with_opts(
        state,
        participant.provider,
        &cwd,
        &prompt,
        Some(&perm_path),
        resolved_model.as_deref(),
        resolved_effort.as_deref(),
    );
    if write_access {
        state.release_dir_lock(&lock_key);
    }
    super::process_manager::clear_activity(state, &chat_key);

    let mut final_response = response.clone();
    let mut loop_response = response;
    if participant_can_execute_pa_commands(state, session, participant, analysis_only) {
        let mut last_signature = String::new();
        let mut repeat_count = 0usize;
        for turn in 1..=ROOM_AUTO_CONTINUE_TURNS {
            let feedback = execute_room_pa_commands(state, &loop_response);
            if feedback.is_empty() {
                break;
            }

            let signature = feedback.signature();
            if signature == last_signature {
                repeat_count += 1;
            } else {
                last_signature = signature;
                repeat_count = 1;
            }

            if !feedback.fragments.is_empty() {
                final_response += "\n\n---\n";
                final_response += &feedback.fragments.join("\n\n---\n");
            }

            if repeat_count >= ROOM_AUTO_CONTINUE_REPEAT_LIMIT {
                final_response += "\n\n---\nAuto-run stopped because the agent repeated the same command result loop.";
                break;
            }

            let continuation_message = build_room_auto_continue_message(turn, &feedback);
            let continuation_prompt =
                build_agent_prompt(state, session, participant, &continuation_message, false);
            super::process_manager::set_activity(
                state,
                &chat_key,
                "multi-agent",
                &format!("{} auto-continue {}", participant.label, turn),
            );
            if write_access {
                state.acquire_dir_lock(&lock_key);
            }
            loop_response = super::provider_runner::run_provider_with_opts(
                state,
                participant.provider,
                &cwd,
                &continuation_prompt,
                Some(&perm_path),
                resolved_model.as_deref(),
                resolved_effort.as_deref(),
            );
            if write_access {
                state.release_dir_lock(&lock_key);
            }
            super::process_manager::clear_activity(state, &chat_key);
            final_response += "\n\n---\n";
            final_response += &loop_response;
        }
    }

    let ts = state.now_iso();
    set_session_state(state, &session.id, |existing| {
        existing.active_round_id = round_id.map(|id| id.to_string());
        existing.active_speaker = Some(participant.id.clone());
        existing
            .presence
            .insert(participant.id.clone(), PresenceState::Replying);
    });
    log_presence_event(
        state,
        &session.id,
        participant,
        PresenceState::Replying,
        &format!("{} is replying", participant.label),
    );
    let file_intents = extract_file_intents(&final_response);
    super::jsonl::append_jsonl_logged(
        &chat_file,
        &json!({
            "ts": ts,
            "role": "assistant",
            "msg": final_response,
            "participant": participant.id,
            "provider": participant.provider.as_str(),
            "meta": participant.label,
            "analysis_only": analysis_only,
            "round_id": round_id,
            "file_intents": file_intents.clone(),
        }),
        "session assistant msg",
    );
    append_shared_assistant_message(
        state,
        session,
        participant,
        &final_response,
        round_id,
        analysis_only,
        &file_intents,
    );
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: ts.clone(),
        kind: "message_completed".to_string(),
        actor: participant.id.clone(),
        payload: json!({
            "provider": participant.provider.as_str(),
            "analysis_only": analysis_only,
            "round_id": round_id,
            "summary": super::claude_runner::safe_truncate(&final_response, 200),
            "response": final_response.clone(),
            "file_intents": file_intents.clone(),
        }),
    });
    if !file_intents.is_empty() {
        state.append_session_event(&SessionEvent {
            id: new_id("evt"),
            session_id: session.id.clone(),
            ts: ts.clone(),
            kind: "file_intents_declared".to_string(),
            actor: participant.id.clone(),
            payload: json!({
                "summary": format!("{} declared {}", participant.label, file_intents.join(", ")),
                "round_id": round_id,
                "paths": file_intents.clone(),
            }),
        });
    }

    update_session_working_set(state, &session.id, &ts, &file_intents);
    set_session_state(state, &session.id, |existing| {
        existing.updated_at = ts.clone();
        existing.active_speaker = None;
        existing
            .presence
            .insert(participant.id.clone(), PresenceState::Idle);
        if round_id.is_none() {
            existing.active_round_id = None;
        }
        if existing.pending_challenge.as_deref() == Some(participant.id.as_str()) {
            existing.pending_challenge = None;
        }
        if existing.pending_rebuttal.as_deref() == Some(participant.id.as_str()) {
            existing.pending_rebuttal = None;
        }
    });
    log_presence_event(
        state,
        &session.id,
        participant,
        PresenceState::Idle,
        &format!("{} is idle", participant.label),
    );

    Ok(json!({
        "status": "complete",
        "session_id": session.id,
        "participant": participant,
        "response": final_response,
        "model": resolved_model,
        "reasoning_effort": resolved_effort,
        "analysis_only": analysis_only,
        "file_intents": file_intents,
        "round_id": round_id,
        "ts": ts,
    }))
}

#[tauri::command]
pub fn create_multi_agent_session(
    state: State<'_, Arc<AppState>>,
    project: Option<String>,
    title: Option<String>,
    mode: Option<String>,
) -> Value {
    let project = project.unwrap_or_default();
    if !project.is_empty() {
        if let Err(err) = state.validate_project(&project) {
            return json!({"status": "error", "error": err});
        }
    }

    let now = state.now_iso();
    let participants = default_participants(&state);
    let orchestrator_participant_id = default_orchestrator_participant_id(&state, &participants);
    let session = MultiAgentSession {
        id: new_id("mas"),
        title: session_title(&project, title.as_deref()),
        project,
        status: SessionStatus::Active,
        mode: parse_mode(mode.as_deref()),
        participants: participants.clone(),
        orchestrator_participant_id,
        current_working_set: Vec::new(),
        active_round_id: None,
        active_speaker: None,
        presence: default_presence(&participants),
        pending_challenge: None,
        pending_rebuttal: None,
        linked_strategy_ids: Vec::new(),
        linked_project_session_ids: Vec::new(),
        linked_work_item_ids: Vec::new(),
        linked_tactic_ids: Vec::new(),
        linked_plan_ids: Vec::new(),
        linked_delegation_ids: Vec::new(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(session.id.clone(), session.clone());
    }
    state.save_sessions();

    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: now,
        kind: "session_created".to_string(),
        actor: "system".to_string(),
        payload: json!({
            "summary": format!("Created {} in {:?} mode", session.title, session.mode),
            "participants": session.participants,
        }),
    });

    json!({"status": "ok", "session": session})
}

#[tauri::command]
pub fn list_multi_agent_sessions(state: State<'_, Arc<AppState>>) -> Value {
    let mut sessions: Vec<MultiAgentSession> = state
        .sessions
        .lock()
        .map(|map| map.values().cloned().collect())
        .unwrap_or_default();
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    json!({"sessions": sessions})
}

#[tauri::command]
pub fn get_multi_agent_session(state: State<'_, Arc<AppState>>, session_id: String) -> Value {
    let session = state
        .sessions
        .lock()
        .ok()
        .and_then(|map| map.get(&session_id).cloned());
    match session {
        Some(session) => {
            let mut hydrated_session = session.clone();
            if hydrated_session.orchestrator_participant_id.is_none() {
                hydrated_session.orchestrator_participant_id =
                    session_orchestrator_participant_id(&state, &session);
            }
            let active_writers: Vec<SessionParticipant> = write_enabled_participants(&session)
                .into_iter()
                .cloned()
                .collect();
            let active_writer = if active_writers.len() == 1 {
                active_writers.first().cloned()
            } else {
                None
            };
            let active_orchestrator = hydrated_session
                .orchestrator_participant_id
                .as_ref()
                .and_then(|id| {
                    hydrated_session
                        .participants
                        .iter()
                        .find(|participant| participant.id == *id)
                        .cloned()
                });
            let write_conflicts = write_conflicts_for_session(&state, &session);
            let participant_runtime: Vec<Value> = session
                .participants
                .iter()
                .map(|participant| participant_runtime_settings(&state, participant))
                .collect();
            json!({
                "status": "ok",
                "session": hydrated_session,
                "events": state.get_session_events(&session_id, 200),
                "active_writer": active_writer,
                "active_writers": active_writers,
                "active_orchestrator": active_orchestrator,
                "write_conflicts": write_conflicts,
                "participant_runtime": participant_runtime,
                "active_leases": linked_file_leases_for_session(&state, &session),
                "linked_project_sessions": linked_project_sessions_for_session(&state, &session),
                "linked_work_items": linked_work_items_for_session(&state, &session),
                "linked_delegations": linked_delegations_for_session(&state, &session),
                "parallel_batches": parallel_batches_for_session(&state, &session),
                "linked_inbox_items": linked_inbox_for_session(&state, &session),
                "linked_signals": linked_signals_for_session(&state, &session),
            })
        }
        None => json!({"status": "error", "error": "Session not found"}),
    }
}

#[tauri::command]
pub fn get_session_agent_history(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    participant_id: String,
) -> Value {
    let chat_key = session_chat_key(&session_id, &participant_id);
    let path = state.chats_dir.join(format!("{}.jsonl", chat_key));
    let mut messages = Vec::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                messages.push(entry);
            }
        }
    }
    json!({"session_id": session_id, "participant_id": participant_id, "messages": messages})
}

#[tauri::command]
pub fn set_session_writer(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    participant_id: String,
) -> Value {
    let mut updated = None;
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            let mut found = false;
            for participant in &mut session.participants {
                if participant.id == participant_id {
                    found = true;
                    participant.write_enabled = true;
                }
            }
            if !found {
                return json!({"status": "error", "error": "Participant not found"});
            }
            session.updated_at = state.now_iso();
            updated = Some(session.clone());
        }
    }
    let Some(session) = updated else {
        return json!({"status": "error", "error": "Session not found"});
    };
    state.save_sessions();
    let label = session
        .participants
        .iter()
        .find(|p| p.id == participant_id)
        .map(|p| p.label.clone())
        .unwrap_or(participant_id.clone());
    emit_pipeline_event(
        &state,
        &session_id,
        "write_access_granted",
        "user",
        &format!("Granted write access to {}", label),
        json!({
            "participant_id": participant_id,
            "label": label,
        }),
    );
    json!({"status": "ok", "session": session})
}

#[tauri::command]
pub fn set_session_orchestrator(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    participant_id: String,
) -> Value {
    let mut updated = None;
    let mut previous_label = None;
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            previous_label =
                session_orchestrator_participant(&state, session).map(|p| p.label.clone());
            let mut found_label = None;
            for participant in &mut session.participants {
                if participant.id == participant_id {
                    participant.write_enabled = true;
                    found_label = Some(participant.label.clone());
                    break;
                }
            }
            let Some(label) = found_label else {
                return json!({"status": "error", "error": "Participant not found"});
            };
            session.orchestrator_participant_id = Some(participant_id.clone());
            session.updated_at = state.now_iso();
            updated = Some((session.clone(), label));
        }
    }
    let Some((session, label)) = updated else {
        return json!({"status": "error", "error": "Session not found"});
    };
    state.save_sessions();
    emit_pipeline_event(
        &state,
        &session_id,
        "orchestrator_switched",
        "user",
        &format!("Set {} as the room orchestrator", label),
        json!({
            "participant_id": participant_id,
            "label": label,
            "previous_label": previous_label,
        }),
    );
    json!({"status": "ok", "session": session})
}

#[tauri::command]
pub fn revoke_session_writer(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    participant_id: String,
) -> Value {
    let blocking_leases: Vec<Value> = state
        .file_leases
        .lock()
        .map(|leases| {
            leases
                .values()
                .filter(|lease| {
                    lease.session_id == session_id
                        && matches!(lease.status, FileLeaseStatus::Active)
                        && lease.participant_id == participant_id
                })
                .map(|lease| {
                    json!({
                        "lease_id": lease.id,
                        "work_item_id": lease.work_item_id,
                        "participant_id": lease.participant_id,
                        "paths": lease.paths,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    if !blocking_leases.is_empty() {
        return json!({
            "status": "error",
            "error": "Cannot revoke write access while this participant holds active leases",
            "leases": blocking_leases,
        });
    }
    let mut updated = None;
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            if session_orchestrator_participant_id(&state, session).as_deref()
                == Some(participant_id.as_str())
            {
                return json!({
                    "status": "error",
                    "error": "Current orchestrator must retain write access. Set another orchestrator first",
                });
            }
            let enabled_count = session
                .participants
                .iter()
                .filter(|p| p.write_enabled)
                .count();
            let mut found = false;
            for participant in &mut session.participants {
                if participant.id == participant_id {
                    found = true;
                    if participant.write_enabled && enabled_count == 1 {
                        return json!({
                            "status": "error",
                            "error": "At least one participant must retain write access",
                        });
                    }
                    participant.write_enabled = false;
                }
            }
            if !found {
                return json!({"status": "error", "error": "Participant not found"});
            }
            session.updated_at = state.now_iso();
            updated = Some(session.clone());
        }
    }
    let Some(session) = updated else {
        return json!({"status": "error", "error": "Session not found"});
    };
    state.save_sessions();
    let label = session
        .participants
        .iter()
        .find(|p| p.id == participant_id)
        .map(|p| p.label.clone())
        .unwrap_or(participant_id.clone());
    emit_pipeline_event(
        &state,
        &session_id,
        "write_access_revoked",
        "user",
        &format!("Revoked write access from {}", label),
        json!({
            "participant_id": participant_id,
            "label": label,
        }),
    );
    json!({"status": "ok", "session": session})
}

#[tauri::command]
pub fn acquire_work_item_lease_manual(
    state: State<'_, Arc<AppState>>,
    work_item_id: String,
    participant_id: Option<String>,
) -> Value {
    let work_item = match state
        .work_items
        .lock()
        .ok()
        .and_then(|items| items.get(&work_item_id).cloned())
    {
        Some(item) => item,
        None => return json!({"status": "error", "error": "Work item not found"}),
    };
    if matches!(
        work_item.write_intent,
        crate::state::WorkItemWriteIntent::ReadOnly
    ) {
        return json!({"status": "error", "error": "Read-only work items do not need leases"});
    }
    if work_item.declared_paths.is_empty() {
        return json!({"status": "error", "error": "Write-scoped work item requires declared paths"});
    }
    let session = match state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&work_item.parent_room_session_id).cloned())
    {
        Some(session) => session,
        None => return json!({"status": "error", "error": "Parent room session not found"}),
    };
    let participant = if let Some(participant_id) = participant_id.as_deref() {
        match session
            .participants
            .iter()
            .find(|participant| participant.id == participant_id)
            .cloned()
        {
            Some(participant) => participant,
            None => return json!({"status": "error", "error": "Participant not found"}),
        }
    } else {
        match write_enabled_participant_for_provider(&session, work_item.executor_provider).cloned()
        {
            Some(participant) => participant,
            None => {
                return json!({
                    "status": "error",
                    "error": format!(
                        "{} does not currently have write access",
                        work_item.executor_provider.as_str()
                    )
                })
            }
        }
    };
    match acquire_work_item_lease_for_participant(&state, &session, &work_item, &participant) {
        Ok(Some(lease)) => json!({"status": "ok", "lease": lease}),
        Ok(None) => json!({"status": "error", "error": "No lease was acquired"}),
        Err(err) => json!({"status": "error", "error": err}),
    }
}

#[tauri::command]
pub fn release_file_lease(
    state: State<'_, Arc<AppState>>,
    lease_id: String,
    force: Option<bool>,
) -> Value {
    let force = force.unwrap_or(false);
    let lease = match state
        .file_leases
        .lock()
        .ok()
        .and_then(|leases| leases.get(&lease_id).cloned())
    {
        Some(lease) => lease,
        None => return json!({"status": "error", "error": "Lease not found"}),
    };
    if !matches!(lease.status, FileLeaseStatus::Active) {
        return json!({"status": "error", "error": "Lease is already released"});
    }
    let work_item = state
        .work_items
        .lock()
        .ok()
        .and_then(|items| items.get(&lease.work_item_id).cloned());
    if !force {
        if let Some(work_item) = work_item.as_ref() {
            if matches!(
                work_item.status,
                WorkItemStatus::Queued | WorkItemStatus::Running | WorkItemStatus::Reviewing
            ) {
                return json!({
                    "status": "error",
                    "error": "Work item is still active. Use force release if you need to break the lease.",
                });
            }
        }
    }
    match release_single_file_lease(
        &state,
        &lease_id,
        if force {
            "force_manual_release"
        } else {
            "manual_release"
        },
    ) {
        Ok(released) => json!({"status": "ok", "lease": released}),
        Err(err) => json!({"status": "error", "error": err}),
    }
}

#[tauri::command]
pub async fn run_session_agent(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    participant_id: String,
    message: String,
    model: Option<String>,
    reasoning_effort: Option<String>,
    analysis_only: Option<bool>,
) -> Result<Value, String> {
    if message.trim().is_empty() {
        return Ok(json!({"status": "error", "error": "Empty message"}));
    }

    let (session, participant) =
        load_session_and_participant(&state, &session_id, &participant_id)?;
    run_session_agent_core(
        &state.inner().clone(),
        &session,
        &participant,
        &message,
        Some(&message),
        model,
        reasoning_effort,
        analysis_only.unwrap_or(false),
        None,
    )
}

#[tauri::command]
pub async fn run_session_round(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    message: String,
    model: Option<String>,
    reasoning_effort: Option<String>,
    analysis_only: Option<bool>,
) -> Result<Value, String> {
    if message.trim().is_empty() {
        return Ok(json!({"status": "error", "error": "Empty message"}));
    }

    let session = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| "Session not found".to_string())?
    };

    let round_id = new_id("round");
    let analysis_only = analysis_only.unwrap_or(true);
    let now = state.now_iso();
    append_shared_user_message(&state, &session, &message, Some(&round_id), analysis_only);
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: now.clone(),
        kind: "user_message".to_string(),
        actor: "user".to_string(),
        payload: json!({
            "message": message,
            "analysis_only": analysis_only,
            "round_id": round_id,
            "summary": super::claude_runner::safe_truncate(&message, 200),
        }),
    });
    set_session_state(&state, &session.id, |existing| {
        existing.active_round_id = Some(round_id.clone());
        existing.active_speaker = None;
        let participant_ids: Vec<String> =
            existing.participants.iter().map(|p| p.id.clone()).collect();
        for participant_id in participant_ids {
            existing
                .presence
                .insert(participant_id, PresenceState::Thinking);
        }
    });
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: now,
        kind: "round_started".to_string(),
        actor: "system".to_string(),
        payload: json!({
            "summary": format!(
                "Started {} round for {} participants",
                if analysis_only { "analysis" } else { "execution" },
                session.participants.len()
            ),
            "round_id": round_id,
            "analysis_only": analysis_only,
        }),
    });

    let mut join_set = JoinSet::new();
    for participant in session.participants.clone() {
        let state = Arc::clone(state.inner());
        let session = session.clone();
        let message = message.clone();
        let model = model.clone();
        let reasoning_effort = reasoning_effort.clone();
        let round_id = round_id.clone();
        join_set.spawn_blocking(move || {
            run_session_agent_core(
                &state,
                &session,
                &participant,
                &message,
                None,
                model,
                reasoning_effort,
                analysis_only,
                Some(&round_id),
            )
        });
    }

    let mut results = Vec::new();
    let mut errors = Vec::new();
    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(Ok(value)) => results.push(value),
            Ok(Err(err)) => errors.push(err),
            Err(err) => errors.push(err.to_string()),
        }
    }

    let ts = state.now_iso();
    state.append_session_event(&SessionEvent {
        id: new_id("evt"),
        session_id: session.id.clone(),
        ts: ts.clone(),
        kind: "round_completed".to_string(),
        actor: "system".to_string(),
        payload: json!({
            "summary": format!(
                "Completed round with {} responses{}",
                results.len(),
                if errors.is_empty() {
                    "".to_string()
                } else {
                    format!(", {} errors", errors.len())
                }
            ),
            "round_id": round_id,
            "analysis_only": analysis_only,
            "errors": errors.clone(),
        }),
    });

    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(existing) = sessions.get_mut(&session.id) {
            existing.updated_at = ts;
            existing.active_round_id = None;
            existing.active_speaker = None;
            let participant_ids: Vec<String> =
                existing.participants.iter().map(|p| p.id.clone()).collect();
            for participant_id in participant_ids {
                existing
                    .presence
                    .insert(participant_id, PresenceState::Idle);
            }
        }
    }
    state.save_sessions();

    Ok(json!({
        "status": if errors.is_empty() { "complete" } else { "partial" },
        "session_id": session.id,
        "round_id": round_id,
        "analysis_only": analysis_only,
        "results": results,
        "errors": errors,
    }))
}

#[tauri::command]
pub async fn run_session_room_action(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    action: String,
    message: String,
    target_participant_id: Option<String>,
) -> Result<Value, String> {
    {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        if !sessions.contains_key(&session_id) {
            return Err("Session not found".to_string());
        }
    }
    let action = action.trim().to_ascii_lowercase();
    let trimmed = message.trim();

    match action.as_str() {
        "mention" => {
            let target_id = target_participant_id
                .ok_or_else(|| "Target participant is required".to_string())?;
            let (session, participant) =
                load_session_and_participant(&state, &session_id, &target_id)?;
            let prompt = format!(
                "Direct mention for {}.\nRespond in room context and keep awareness of the other visible agent.\n\nUser message:\n{}",
                participant.label, trimmed
            );
            let public_message = format!("@{}: {}", participant.label, trimmed);
            run_session_agent_core(
                &state.inner().clone(),
                &session,
                &participant,
                &prompt,
                Some(&public_message),
                None,
                None,
                true,
                None,
            )
        }
        "challenge" => {
            let target_id = target_participant_id
                .ok_or_else(|| "Target participant is required".to_string())?;
            let (session, participant) =
                load_session_and_participant(&state, &session_id, &target_id)?;
            let source = session
                .participants
                .iter()
                .find(|p| p.id != participant.id)
                .cloned()
                .ok_or_else(|| "No counterpart participant found".to_string())?;
            let source_msg = latest_assistant_message(&state, &session_id, &source.id)
                .unwrap_or_else(|| {
                    "No prior response from the counterpart is available.".to_string()
                });
            set_session_state(&state, &session_id, |existing| {
                existing.pending_challenge = Some(participant.id.clone());
                existing.pending_rebuttal = None;
            });
            state.append_session_event(&SessionEvent {
                id: new_id("evt"),
                session_id: session.id.clone(),
                ts: state.now_iso(),
                kind: "challenge_requested".to_string(),
                actor: "user".to_string(),
                payload: json!({
                    "summary": format!("Challenge {} against {}", participant.label, source.label),
                    "target": participant.id,
                    "source": source.id,
                    "message": trimmed,
                }),
            });
            let prompt = format!(
                "Challenge turn.\nYou are {target_label}. Critique the latest visible message from {source_label}.\nUse this structure:\nclaim\nevidence\nrisk\nproposal\n\nUser instruction:\n{instruction}\n\nLatest message from {source_label}:\n{source_msg}",
                target_label = participant.label,
                source_label = source.label,
                instruction = if trimmed.is_empty() {
                    "Challenge the counterpart and surface the strongest technical or product disagreement."
                } else {
                    trimmed
                },
                source_msg = source_msg,
            );
            let public_message = format!(
                "Challenge {}: {}",
                participant.label,
                if trimmed.is_empty() {
                    "challenge the counterpart and surface the strongest disagreement"
                } else {
                    trimmed
                }
            );
            run_session_agent_core(
                &state.inner().clone(),
                &session,
                &participant,
                &prompt,
                Some(&public_message),
                None,
                None,
                true,
                None,
            )
        }
        "rebuttal" => {
            let target_id = target_participant_id
                .ok_or_else(|| "Target participant is required".to_string())?;
            let (session, participant) =
                load_session_and_participant(&state, &session_id, &target_id)?;
            let source = session
                .participants
                .iter()
                .find(|p| p.id != participant.id)
                .cloned()
                .ok_or_else(|| "No counterpart participant found".to_string())?;
            let source_msg = latest_assistant_message(&state, &session_id, &source.id)
                .unwrap_or_else(|| {
                    "No prior response from the counterpart is available.".to_string()
                });
            set_session_state(&state, &session_id, |existing| {
                existing.pending_rebuttal = Some(participant.id.clone());
                existing.pending_challenge = None;
            });
            state.append_session_event(&SessionEvent {
                id: new_id("evt"),
                session_id: session.id.clone(),
                ts: state.now_iso(),
                kind: "rebuttal_requested".to_string(),
                actor: "user".to_string(),
                payload: json!({
                    "summary": format!("Rebuttal {} against {}", participant.label, source.label),
                    "target": participant.id,
                    "source": source.id,
                    "message": trimmed,
                }),
            });
            let prompt = format!(
                "Rebuttal turn.\nYou are {target_label}. Respond to the latest visible criticism from {source_label}.\nAddress the critique directly and finish with a concrete proposal.\n\nUser instruction:\n{instruction}\n\nLatest message from {source_label}:\n{source_msg}",
                target_label = participant.label,
                source_label = source.label,
                instruction = if trimmed.is_empty() {
                    "Respond directly to the counterpart's critique and clarify your final proposal."
                } else {
                    trimmed
                },
                source_msg = source_msg,
            );
            let public_message = format!(
                "Rebuttal {}: {}",
                participant.label,
                if trimmed.is_empty() {
                    "respond to the counterpart critique and clarify the final proposal"
                } else {
                    trimmed
                }
            );
            run_session_agent_core(
                &state.inner().clone(),
                &session,
                &participant,
                &prompt,
                Some(&public_message),
                None,
                None,
                true,
                None,
            )
        }
        _ => Ok(json!({"status": "error", "error": "Unsupported room action"})),
    }
}

#[tauri::command]
pub fn create_project_session(
    state: State<'_, Arc<AppState>>,
    parent_room_session_id: String,
    project: String,
    title: Option<String>,
    executor_provider: Option<String>,
    reviewer_provider: Option<String>,
) -> Value {
    if let Err(err) = state
        .sessions
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|sessions| {
            if sessions.contains_key(&parent_room_session_id) {
                Ok(())
            } else {
                Err("Parent room session not found".to_string())
            }
        })
    {
        return json!({"status": "error", "error": err});
    }
    if let Err(err) = state.validate_project(&project) {
        return json!({"status": "error", "error": err});
    }
    let executor = super::provider_runner::parse_provider(
        executor_provider.as_deref(),
        super::provider_runner::orchestrator_provider(&state),
    );
    let reviewer = reviewer_provider.as_deref().map(|value| {
        super::provider_runner::parse_provider(
            Some(value),
            super::provider_runner::technical_reviewer_provider(&state),
        )
    });
    let project_session = create_project_session_record(
        &state,
        &parent_room_session_id,
        &project,
        title.as_deref(),
        executor,
        reviewer,
    );
    if let Ok(mut project_sessions) = state.project_sessions.lock() {
        project_sessions.insert(project_session.id.clone(), project_session.clone());
    }
    state.save_project_sessions();
    link_project_session_to_session(
        &state,
        &parent_room_session_id,
        &project_session.id,
        &project_session.title,
        &project,
    );
    json!({"status": "ok", "project_session": project_session})
}

#[tauri::command]
pub fn create_work_item(
    state: State<'_, Arc<AppState>>,
    parent_room_session_id: String,
    project_session_id: Option<String>,
    project: String,
    title: Option<String>,
    task: String,
    assignee: Option<String>,
    write_intent: Option<String>,
    declared_paths: Option<Vec<String>>,
    verify: Option<Value>,
    executor_provider: Option<String>,
    reviewer_provider: Option<String>,
    source_kind: Option<String>,
    source_id: Option<String>,
) -> Value {
    if task.trim().is_empty() {
        return json!({"status": "error", "error": "Empty task"});
    }
    if let Err(err) = state
        .sessions
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|sessions| {
            if sessions.contains_key(&parent_room_session_id) {
                Ok(())
            } else {
                Err("Parent room session not found".to_string())
            }
        })
    {
        return json!({"status": "error", "error": err});
    }
    let session = match state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&parent_room_session_id).cloned())
    {
        Some(session) => session,
        None => return json!({"status": "error", "error": "Parent room session not found"}),
    };
    if let Err(err) = state.validate_project(&project) {
        return json!({"status": "error", "error": err});
    }
    if let Some(project_session_id) = project_session_id.as_deref() {
        let valid_project_session = state
            .project_sessions
            .lock()
            .ok()
            .and_then(|map| map.get(project_session_id).cloned());
        match valid_project_session {
            Some(ps)
                if ps.parent_room_session_id == parent_room_session_id && ps.project == project => {
            }
            Some(_) => {
                return json!({"status": "error", "error": "Project session does not match parent/project"});
            }
            None => return json!({"status": "error", "error": "Project session not found"}),
        }
    }
    let executor = super::provider_runner::parse_provider(
        executor_provider.as_deref(),
        super::provider_runner::orchestrator_provider(&state),
    );
    let reviewer = reviewer_provider.as_deref().map(|value| {
        super::provider_runner::parse_provider(
            Some(value),
            super::provider_runner::technical_reviewer_provider(&state),
        )
    });
    let assignee = parse_work_item_assignee(assignee.as_deref());
    let write_intent = parse_work_item_write_intent(write_intent.as_deref());
    let declared_paths = parse_declared_paths(declared_paths);
    if !matches!(write_intent, crate::state::WorkItemWriteIntent::ReadOnly)
        && declared_paths.is_empty()
    {
        return json!({"status": "error", "error": "Write-intent work items require declared paths"});
    }
    let verify = match parse_verify_condition(verify) {
        Ok(v) => v,
        Err(err) => return json!({"status": "error", "error": err}),
    };
    if matches!(assignee, crate::state::WorkItemAssignee::Agent)
        && !matches!(write_intent, crate::state::WorkItemWriteIntent::ReadOnly)
    {
        let Some(_) = write_enabled_participant_for_provider(&session, executor) else {
            return json!({
                "status": "error",
                "error": format!(
                    "{} does not currently have write access. Grant write before creating {} write work.",
                    executor.as_str(),
                    executor.as_str()
                )
            });
        };
    }
    let existing_conflicts =
        conflicting_existing_work_items(&state, &session, None, &declared_paths);
    if !existing_conflicts.is_empty()
        && matches!(
            write_intent,
            crate::state::WorkItemWriteIntent::ExclusiveWrite
        )
    {
        return json!({
            "status": "error",
            "error": "Exclusive write conflicts with existing scoped work items",
            "conflicts": existing_conflicts,
        });
    }
    if source_kind.as_deref() == Some("plan_step") {
        let source = match source_id.as_deref() {
            Some(source) => source,
            None => {
                return json!({"status": "error", "error": "Plan step source requires source_id"})
            }
        };
        let (plan_id, step_id) = match super::plans::parse_plan_step_source_id(source) {
            Some(ids) => ids,
            None => return json!({"status": "error", "error": "Invalid plan step source id"}),
        };
        let plans = super::plans::load_all_plans_internal(&state);
        let plan = match plans.iter().find(|p| p.id == plan_id) {
            Some(plan) => plan,
            None => return json!({"status": "error", "error": "Linked plan not found"}),
        };
        let step_index = match super::plans::find_plan_step_index(plan, &step_id) {
            Some(index) => index,
            None => return json!({"status": "error", "error": "Linked plan step not found"}),
        };
        let step = &plan.steps[step_index];
        if step.project != project || step.task.trim() != task.trim() {
            return json!({"status": "error", "error": "Work item does not match linked plan step"});
        }
        if step.work_item_id.is_some() {
            return json!({"status": "error", "error": "Plan step already has a linked work item"});
        }
    }
    let work_item = create_work_item_record(
        &state,
        &parent_room_session_id,
        project_session_id.as_deref(),
        &project,
        title.as_deref(),
        &task,
        executor,
        reviewer,
        assignee,
        write_intent,
        declared_paths.clone(),
        verify,
        source_kind.as_deref(),
        source_id.as_deref(),
    );
    if let Ok(mut work_items) = state.work_items.lock() {
        work_items.insert(work_item.id.clone(), work_item.clone());
    }
    if let Some(project_session_id) = project_session_id.as_deref() {
        if let Ok(mut project_sessions) = state.project_sessions.lock() {
            if let Some(session) = project_sessions.get_mut(project_session_id) {
                if !session
                    .linked_work_item_ids
                    .iter()
                    .any(|id| id == &work_item.id)
                {
                    session.linked_work_item_ids.push(work_item.id.clone());
                }
                session.updated_at = state.now_iso();
            }
        }
        state.save_project_sessions();
    }
    if let Some((plan_id, step_id)) = work_item_plan_step_source(&work_item) {
        let mapped_assignee = match work_item.assignee {
            crate::state::WorkItemAssignee::Agent => {
                crate::commands::strategy_models::Assignee::Agent
            }
            crate::state::WorkItemAssignee::User => {
                crate::commands::strategy_models::Assignee::User
            }
        };
        if let Err(err) = super::plans::update_plan_step_linked_work_item(
            &state,
            &plan_id,
            &step_id,
            &work_item.id,
            mapped_assignee,
            work_item.verify.clone(),
        ) {
            return json!({"status": "error", "error": err});
        }
        emit_pipeline_event(
            &state,
            &parent_room_session_id,
            "plan_step_work_item_linked",
            "system",
            &format!(
                "Linked plan step {} in {} to work item {}",
                step_id, plan_id, work_item.id
            ),
            json!({
                "plan_id": plan_id,
                "step_id": step_id,
                "work_item_id": work_item.id,
            }),
        );
    }
    state.save_work_items();
    if !declared_paths.is_empty() {
        update_session_working_set(
            &state,
            &parent_room_session_id,
            &state.now_iso(),
            &declared_paths,
        );
        emit_pipeline_event(
            &state,
            &parent_room_session_id,
            "work_item_scope_declared",
            "user",
            &format!(
                "Declared {} path(s) for work item {}",
                declared_paths.len(),
                work_item.title
            ),
            json!({
                "work_item_id": work_item.id,
                "write_intent": work_item.write_intent,
                "paths": declared_paths,
            }),
        );
        if !existing_conflicts.is_empty() {
            emit_pipeline_event(
                &state,
                &parent_room_session_id,
                "write_conflict_detected",
                "system",
                &format!(
                    "Scoped work item {} overlaps with existing write-scoped work",
                    work_item.title
                ),
                json!({
                    "work_item_id": work_item.id,
                    "conflicts": existing_conflicts,
                }),
            );
        }
    }
    link_work_item_to_session(
        &state,
        &parent_room_session_id,
        &work_item.id,
        &work_item.title,
        &project,
    );
    json!({"status": "ok", "work_item": work_item})
}

#[tauri::command]
pub fn create_plan_step_work_item(
    state: State<'_, Arc<AppState>>,
    plan_id: String,
    step_index: usize,
    room_session_id: Option<String>,
    project_session_id: Option<String>,
    executor_provider: Option<String>,
    reviewer_provider: Option<String>,
) -> Value {
    let plans = super::plans::load_all_plans_internal(&state);
    let plan = match plans.iter().find(|plan| plan.id == plan_id) {
        Some(plan) => plan,
        None => return json!({"status": "error", "error": "Plan not found"}),
    };
    let step = match plan.steps.get(step_index) {
        Some(step) => step,
        None => return json!({"status": "error", "error": "Plan step index out of range"}),
    };
    if step.work_item_id.is_some() {
        return json!({"status": "error", "error": "Plan step already has work item"});
    }
    let parent_room_session_id = match room_session_id.or_else(|| plan.room_session_id.clone()) {
        Some(id) => id,
        None => {
            return json!({
                "status": "error",
                "error": "Plan step is not linked to a room session"
            })
        }
    };
    let source_id = super::plans::plan_step_source_id(&plan.id, &step.id);
    create_work_item(
        state,
        parent_room_session_id,
        project_session_id,
        step.project.clone(),
        Some(step.task.chars().take(80).collect()),
        step.task.clone(),
        Some(
            match step.assignee {
                crate::commands::strategy_models::Assignee::Agent => "agent",
                crate::commands::strategy_models::Assignee::User => "user",
            }
            .to_string(),
        ),
        Some("read_only".to_string()),
        None,
        step.verify
            .clone()
            .map(|verify| serde_json::to_value(verify).unwrap_or(Value::Null)),
        executor_provider,
        reviewer_provider,
        Some("plan_step".to_string()),
        Some(source_id),
    )
}

#[tauri::command]
pub fn queue_session_delegation(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    project_session_id: Option<String>,
    project: String,
    task: String,
) -> Value {
    let work_item_value = create_work_item(
        state.clone(),
        session_id.clone(),
        project_session_id,
        project.clone(),
        None,
        task.clone(),
        Some("agent".to_string()),
        Some("read_only".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    if work_item_value.get("status").and_then(|v| v.as_str()) != Some("ok") {
        return work_item_value;
    }
    let work_item = match work_item_value.get("work_item").cloned() {
        Some(v) => v,
        None => return json!({"status": "error", "error": "Work item creation failed"}),
    };
    let work_item_id = work_item
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let task_text = work_item
        .get("task")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let delegation_id = super::delegation::queue_delegation_internal(&state, &project, &task_text);
    if delegation_id.is_empty() {
        return json!({"status": "error", "error": "Delegation queue failed"});
    }
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(&delegation_id) {
            del.room_session_id = Some(session_id.clone());
            del.project_session_id = work_item
                .get("project_session_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            del.work_item_id = Some(work_item_id.clone());
            del.executor_provider = work_item
                .get("executor_provider")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok());
            del.reviewer_provider = work_item
                .get("reviewer_provider")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok());
        }
    }
    if let Ok(mut work_items) = state.work_items.lock() {
        if let Some(item) = work_items.get_mut(&work_item_id) {
            item.delegation_id = Some(delegation_id.clone());
            item.status = WorkItemStatus::Queued;
            item.updated_at = state.now_iso();
        }
    }
    state.save_delegations();
    state.save_work_items();
    link_delegation_to_session(&state, &session_id, &delegation_id, &project, &task_text);
    emit_pipeline_event(
        &state,
        &session_id,
        "todo_queued_as_delegation",
        "user",
        &format!(
            "Queued agent task for {} as delegation {}",
            project, delegation_id
        ),
        json!({
            "project": project,
            "task": super::claude_runner::safe_truncate(&task_text, 240),
            "delegation_id": delegation_id,
            "work_item_id": work_item_id,
            "assignee": "agent",
        }),
    );
    json!({"status": "ok", "delegation_id": delegation_id, "project": project, "work_item_id": work_item_id})
}

#[tauri::command]
fn queue_work_item_execution_internal(
    state: &AppState,
    work_item_id: &str,
    batch_id: Option<&str>,
) -> Value {
    let work_item = state
        .work_items
        .lock()
        .ok()
        .and_then(|items| items.get(work_item_id).cloned());
    let work_item = match work_item {
        Some(item) => item,
        None => return json!({"status": "error", "error": "Work item not found"}),
    };
    if !matches!(work_item.assignee, crate::state::WorkItemAssignee::Agent) {
        return json!({"status": "error", "error": "Only agent work items can be queued"});
    }
    if work_item.delegation_id.is_some() {
        return json!({"status": "error", "error": "Work item already has delegation"});
    }
    let session = match state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&work_item.parent_room_session_id).cloned())
    {
        Some(session) => session,
        None => return json!({"status": "error", "error": "Parent room session not found"}),
    };
    if !matches!(
        work_item.write_intent,
        crate::state::WorkItemWriteIntent::ReadOnly
    ) {
        let Some(_) = write_enabled_participant_for_provider(&session, work_item.executor_provider)
        else {
            return json!({
                "status": "error",
                "error": format!(
                    "{} does not currently have write access. Grant write before queuing this work item.",
                    work_item.executor_provider.as_str()
                )
            });
        };
    }
    let existing_conflicts = conflicting_existing_work_items(
        &state,
        &session,
        Some(&work_item.id),
        &work_item.declared_paths,
    );
    if !existing_conflicts.is_empty()
        && matches!(
            work_item.write_intent,
            crate::state::WorkItemWriteIntent::ExclusiveWrite
        )
    {
        return json!({
            "status": "error",
            "error": "Exclusive write conflicts with existing scoped work items",
            "conflicts": existing_conflicts,
        });
    }
    let lease = match acquire_work_item_lease(&state, &session, &work_item) {
        Ok(lease) => lease,
        Err(err) => return json!({"status": "error", "error": err}),
    };
    let delegation_id =
        super::delegation::queue_delegation_internal(&state, &work_item.project, &work_item.task);
    if delegation_id.is_empty() {
        if lease.is_some() {
            release_work_item_leases(
                &state,
                &work_item.parent_room_session_id,
                &work_item.id,
                "delegation_queue_failed",
            );
        }
        return json!({"status": "error", "error": "Delegation queue failed"});
    }
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(&delegation_id) {
            del.room_session_id = Some(work_item.parent_room_session_id.clone());
            del.project_session_id = work_item.project_session_id.clone();
            del.work_item_id = Some(work_item.id.clone());
            del.executor_provider = Some(work_item.executor_provider.clone());
            del.reviewer_provider = work_item.reviewer_provider.clone();
            del.batch_id = batch_id.map(str::to_string);
            if let Some((plan_id, step_id)) = work_item_plan_step_source(&work_item) {
                let plans = super::plans::load_all_plans_internal(&state);
                if let Some(plan) = plans.iter().find(|plan| plan.id == plan_id) {
                    if let Some(step_index) = super::plans::find_plan_step_index(plan, &step_id) {
                        del.plan_id = Some(plan_id);
                        del.plan_step = Some(step_index);
                    }
                }
            }
        }
    }
    if let Ok(mut work_items) = state.work_items.lock() {
        if let Some(item) = work_items.get_mut(work_item_id) {
            item.delegation_id = Some(delegation_id.clone());
            item.status = WorkItemStatus::Queued;
            item.updated_at = state.now_iso();
        }
    }
    state.save_delegations();
    state.save_work_items();
    link_delegation_to_session(
        &state,
        &work_item.parent_room_session_id,
        &delegation_id,
        &work_item.project,
        &work_item.task,
    );
    emit_pipeline_event(
        &state,
        &work_item.parent_room_session_id,
        "todo_queued_as_delegation",
        "user",
        &format!(
            "Queued agent work item '{}' for {} as delegation {}",
            work_item.title, work_item.project, delegation_id
        ),
        json!({
            "project": work_item.project,
            "task": super::claude_runner::safe_truncate(&work_item.task, 240),
            "delegation_id": delegation_id,
            "work_item_id": work_item.id,
            "assignee": "agent",
        }),
    );
    json!({
        "status": "ok",
        "delegation_id": delegation_id,
        "project": work_item.project,
        "work_item_id": work_item.id,
        "lease_id": lease.map(|lease| lease.id),
    })
}

#[tauri::command]
pub fn queue_work_item_execution(state: State<'_, Arc<AppState>>, work_item_id: String) -> Value {
    queue_work_item_execution_internal(&state, &work_item_id, None)
}

#[tauri::command]
pub fn queue_parallel_work_items(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    work_item_ids: Vec<String>,
) -> Value {
    let selected_ids: Vec<String> = work_item_ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .fold(Vec::new(), |mut acc, id| {
            if !acc.iter().any(|existing| existing == &id) {
                acc.push(id);
            }
            acc
        });
    if selected_ids.is_empty() {
        return json!({"status": "error", "error": "No work items selected"});
    }
    let session = match state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&session_id).cloned())
    {
        Some(session) => session,
        None => return json!({"status": "error", "error": "Session not found"}),
    };
    let selected_items: Vec<WorkItem> = match state.work_items.lock() {
        Ok(items) => {
            let mut out = Vec::new();
            for id in &selected_ids {
                let Some(item) = items.get(id).cloned() else {
                    return json!({"status": "error", "error": format!("Work item not found: {}", id)});
                };
                if item.parent_room_session_id != session_id {
                    return json!({"status": "error", "error": format!("Work item {} does not belong to session", id)});
                }
                if !matches!(item.assignee, crate::state::WorkItemAssignee::Agent) {
                    return json!({"status": "error", "error": format!("Work item {} is not agent-assigned", id)});
                }
                if item.delegation_id.is_some() || !matches!(item.status, WorkItemStatus::Ready) {
                    return json!({"status": "error", "error": format!("Work item {} is not queueable", id)});
                }
                if !matches!(
                    item.write_intent,
                    crate::state::WorkItemWriteIntent::ReadOnly
                ) && write_enabled_participant_for_provider(&session, item.executor_provider)
                    .is_none()
                {
                    return json!({
                        "status": "error",
                        "error": format!(
                            "{} does not currently have write access for work item {}",
                            item.executor_provider.as_str(),
                            item.id
                        )
                    });
                }
                out.push(item);
            }
            out
        }
        Err(e) => return json!({"status": "error", "error": e.to_string()}),
    };
    let batch_conflicts = blocking_overlaps_between_work_items(&selected_items);
    if !batch_conflicts.is_empty() {
        return json!({
            "status": "error",
            "error": "Selected work items overlap and cannot run in the same safe parallel batch",
            "conflicts": batch_conflicts,
        });
    }
    let batch_id = new_id("parallel");
    emit_pipeline_event(
        &state,
        &session_id,
        "parallel_batch_started",
        "user",
        &format!("Queueing safe parallel batch {}", batch_id),
        json!({
            "batch_id": batch_id,
            "work_item_ids": selected_ids,
        }),
    );
    let mut queued = Vec::new();
    let mut errors = Vec::new();
    for item in &selected_items {
        let res = queue_work_item_execution_internal(&state, &item.id, Some(&batch_id));
        if res.get("status").and_then(|v| v.as_str()) == Some("ok") {
            queued.push(res);
        } else {
            errors.push(json!({
                "work_item_id": item.id,
                "error": res.get("error").cloned().unwrap_or(Value::String("Unknown queue error".to_string())),
            }));
        }
    }
    emit_pipeline_event(
        &state,
        &session_id,
        if errors.is_empty() {
            "parallel_batch_queued"
        } else {
            "parallel_batch_partial"
        },
        "system",
        &format!(
            "Parallel batch {} queued {} item(s){}",
            batch_id,
            queued.len(),
            if errors.is_empty() {
                "".to_string()
            } else {
                format!(", {} failed", errors.len())
            }
        ),
        json!({
            "batch_id": batch_id,
            "queued": queued,
            "errors": errors,
        }),
    );
    json!({
        "status": if errors.is_empty() { "ok" } else { "partial" },
        "batch_id": batch_id,
        "queued": queued,
        "errors": errors,
    })
}

#[tauri::command]
pub fn queue_provider_parallel_round(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    provider: String,
) -> Value {
    let session = match state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&session_id).cloned())
    {
        Some(session) => session,
        None => return json!({"status": "error", "error": "Session not found"}),
    };
    let provider = super::provider_runner::parse_provider(
        Some(provider.as_str()),
        super::provider_runner::orchestrator_provider(&state),
    );
    let Some(_) = write_enabled_participant_for_provider(&session, provider) else {
        return json!({
            "status": "error",
            "error": format!("{} does not currently have write access", provider.as_str()),
        });
    };
    let mut candidates: Vec<WorkItem> = match state.work_items.lock() {
        Ok(items) => items
            .values()
            .filter(|item| {
                item.parent_room_session_id == session_id
                    && matches!(item.assignee, crate::state::WorkItemAssignee::Agent)
                    && item.executor_provider == provider
                    && item.delegation_id.is_none()
                    && matches!(item.status, WorkItemStatus::Ready)
            })
            .cloned()
            .collect(),
        Err(e) => return json!({"status": "error", "error": e.to_string()}),
    };
    if candidates.is_empty() {
        return json!({
            "status": "error",
            "error": format!("No queueable {} work items found", provider.as_str()),
        });
    }
    candidates.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let mut selected: Vec<WorkItem> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    for item in candidates {
        if matches!(
            item.write_intent,
            crate::state::WorkItemWriteIntent::ReadOnly
        ) || item.declared_paths.is_empty()
        {
            selected.push(item);
            continue;
        }
        let overlaps_existing = selected.iter().any(|chosen| {
            !matches!(
                chosen.write_intent,
                crate::state::WorkItemWriteIntent::ReadOnly
            ) && !overlapping_paths(&item.declared_paths, &chosen.declared_paths).is_empty()
        });
        if overlaps_existing {
            skipped.push(json!({
                "work_item_id": item.id,
                "title": item.title,
                "reason": "overlapping declared_paths inside provider round",
                "paths": item.declared_paths,
            }));
            continue;
        }
        selected.push(item);
    }
    if selected.is_empty() {
        return json!({
            "status": "error",
            "error": format!("No safe {} work items available for a parallel round", provider.as_str()),
            "skipped": skipped,
        });
    }
    let batch_id = new_id(&format!("provider-{}", provider.as_str()));
    emit_pipeline_event(
        &state,
        &session_id,
        "provider_parallel_round_started",
        "user",
        &format!(
            "Queueing {} provider round {} with {} item(s)",
            provider.as_str(),
            batch_id,
            selected.len()
        ),
        json!({
            "batch_id": batch_id,
            "provider": provider.as_str(),
            "selected_work_item_ids": selected.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            "skipped": skipped,
        }),
    );
    let mut queued = Vec::new();
    let mut errors = Vec::new();
    for item in &selected {
        let res = queue_work_item_execution_internal(&state, &item.id, Some(&batch_id));
        if res.get("status").and_then(|v| v.as_str()) == Some("ok") {
            queued.push(res);
        } else {
            errors.push(json!({
                "work_item_id": item.id,
                "error": res.get("error").cloned().unwrap_or(Value::String("Unknown queue error".to_string())),
            }));
        }
    }
    emit_pipeline_event(
        &state,
        &session_id,
        if errors.is_empty() {
            "provider_parallel_round_queued"
        } else {
            "provider_parallel_round_partial"
        },
        "system",
        &format!(
            "{} round {} queued {} item(s){}",
            provider.as_str(),
            batch_id,
            queued.len(),
            if errors.is_empty() {
                "".to_string()
            } else {
                format!(", {} failed", errors.len())
            }
        ),
        json!({
            "batch_id": batch_id,
            "provider": provider.as_str(),
            "queued": queued,
            "errors": errors,
            "skipped": skipped,
        }),
    );
    json!({
        "status": if errors.is_empty() { "ok" } else { "partial" },
        "batch_id": batch_id,
        "provider": provider.as_str(),
        "queued": queued,
        "errors": errors,
        "skipped": skipped,
    })
}

#[tauri::command]
pub fn complete_user_work_item(
    state: State<'_, Arc<AppState>>,
    work_item_id: String,
    result: Option<String>,
) -> Value {
    let (room_session_id, work_item_title, project, final_result, plan_step_source) = {
        let mut work_items = match state.work_items.lock() {
            Ok(items) => items,
            Err(e) => return json!({"status": "error", "error": e.to_string()}),
        };
        let item = match work_items.get_mut(&work_item_id) {
            Some(item) => item,
            None => return json!({"status": "error", "error": "Work item not found"}),
        };
        if !matches!(item.assignee, crate::state::WorkItemAssignee::User) {
            return json!({"status": "error", "error": "Only user work items can be completed manually"});
        }
        item.status = WorkItemStatus::Completed;
        item.result = result
            .clone()
            .or_else(|| Some("Completed manually by user".to_string()));
        item.updated_at = state.now_iso();
        (
            item.parent_room_session_id.clone(),
            item.title.clone(),
            item.project.clone(),
            item.result.clone().unwrap_or_default(),
            work_item_plan_step_source(item),
        )
    };
    state.save_work_items();
    release_work_item_leases(&state, &room_session_id, &work_item_id, "user_completed");
    if let Some((plan_id, step_id)) = plan_step_source {
        let _ = super::plans::sync_plan_step_from_work_item(
            &state,
            &plan_id,
            &step_id,
            crate::commands::status::PlanStepStatus::Done,
            Some(final_result.clone()),
            Some(work_item_id.clone()),
            None,
        );
    }
    emit_pipeline_event(
        &state,
        &room_session_id,
        "user_work_item_completed",
        "user",
        &format!(
            "Completed user work item '{}' in {}",
            work_item_title, project
        ),
        json!({
            "work_item_id": work_item_id,
            "project": project,
            "result": final_result,
        }),
    );
    json!({"status": "ok", "work_item_id": work_item_id})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::provider_runner::ProviderKind;
    use crate::state::{PresenceState, SessionMode, SessionStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "agent-os-multi-agent-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(root.join("tasks")).unwrap();
        root
    }

    fn test_state(name: &str) -> AppState {
        AppState::new(temp_root(name))
    }

    fn test_session(participants: Vec<SessionParticipant>) -> MultiAgentSession {
        let mut presence = HashMap::new();
        for participant in &participants {
            presence.insert(participant.id.clone(), PresenceState::Idle);
        }
        let orchestrator_participant_id = participants
            .first()
            .map(|participant| participant.id.clone());
        MultiAgentSession {
            id: "mas-test".to_string(),
            title: "Test Session".to_string(),
            project: String::new(),
            status: SessionStatus::Active,
            mode: SessionMode::Review,
            participants,
            orchestrator_participant_id,
            current_working_set: Vec::new(),
            active_round_id: None,
            active_speaker: None,
            presence,
            pending_challenge: None,
            pending_rebuttal: None,
            linked_strategy_ids: Vec::new(),
            linked_project_session_ids: Vec::new(),
            linked_work_item_ids: Vec::new(),
            linked_tactic_ids: Vec::new(),
            linked_plan_ids: Vec::new(),
            linked_delegation_ids: Vec::new(),
            created_at: "2026-04-22T00:00:00Z".to_string(),
            updated_at: "2026-04-22T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn execution_prompt_includes_pa_commands_for_writer() {
        let state = test_state("writer");
        let participant = SessionParticipant {
            id: "claude_pm".to_string(),
            label: "Claude PM".to_string(),
            provider: ProviderKind::Claude,
            role: "product".to_string(),
            write_enabled: true,
        };
        let session = test_session(vec![participant.clone()]);

        let prompt = build_agent_prompt(&state, &session, &participant, "Start execution", false);

        assert!(prompt.contains("Execution mode is live for this participant."));
        assert!(prompt.contains("selected you as the execution lead/orchestrator"));
        assert!(prompt.contains("outside fenced code blocks"));
        assert!(prompt.contains("[DASHBOARD_FULL]"));
        assert!(prompt.contains("[DELEGATE_STATUS:?failed]"));
        assert!(prompt.contains("[DELEGATE:Project]task[/DELEGATE]"));
        assert!(participant_has_round_write_access(&participant, false));
    }

    #[test]
    fn execution_prompt_keeps_non_writer_read_only() {
        let state = test_state("reviewer");
        let participant = SessionParticipant {
            id: "codex_tech".to_string(),
            label: "Codex Tech".to_string(),
            provider: ProviderKind::Codex,
            role: "technical".to_string(),
            write_enabled: false,
        };
        let session = test_session(vec![participant.clone()]);

        let prompt = build_agent_prompt(&state, &session, &participant, "Start execution", false);

        assert!(prompt.contains("you still do not have write access"));
        assert!(!prompt.contains("[DELEGATE:Project]task[/DELEGATE]"));
        assert!(!participant_has_round_write_access(&participant, false));
    }

    #[test]
    fn analysis_round_never_grants_write_access() {
        let participant = SessionParticipant {
            id: "claude_pm".to_string(),
            label: "Claude PM".to_string(),
            provider: ProviderKind::Claude,
            role: "product".to_string(),
            write_enabled: true,
        };

        assert!(!participant_has_round_write_access(&participant, true));
    }

    #[test]
    fn codex_can_become_orchestrator_when_selected() {
        let state = test_state("codex-orchestrator");
        let claude = SessionParticipant {
            id: "claude_pm".to_string(),
            label: "Claude PM".to_string(),
            provider: ProviderKind::Claude,
            role: "product".to_string(),
            write_enabled: true,
        };
        let codex = SessionParticipant {
            id: "codex_tech".to_string(),
            label: "Codex Tech".to_string(),
            provider: ProviderKind::Codex,
            role: "technical".to_string(),
            write_enabled: true,
        };
        let mut session = test_session(vec![claude.clone(), codex.clone()]);
        session.orchestrator_participant_id = Some(codex.id.clone());

        assert!(participant_can_execute_pa_commands(
            &state, &session, &codex, false
        ));
        assert!(!participant_can_execute_pa_commands(
            &state, &session, &claude, false
        ));
    }

    #[test]
    fn non_orchestrator_writer_does_not_receive_pa_commands() {
        let state = test_state("writer-no-orchestrator");
        let claude = SessionParticipant {
            id: "claude_pm".to_string(),
            label: "Claude PM".to_string(),
            provider: ProviderKind::Claude,
            role: "product".to_string(),
            write_enabled: true,
        };
        let codex = SessionParticipant {
            id: "codex_tech".to_string(),
            label: "Codex Tech".to_string(),
            provider: ProviderKind::Codex,
            role: "technical".to_string(),
            write_enabled: true,
        };
        let mut session = test_session(vec![claude.clone(), codex.clone()]);
        session.orchestrator_participant_id = Some(codex.id.clone());

        let prompt = build_agent_prompt(&state, &session, &claude, "Start execution", false);

        assert!(prompt.contains("orchestration currently belongs to Codex Tech"));
        assert!(!prompt.contains("[DELEGATE:Project]task[/DELEGATE]"));
    }
}
