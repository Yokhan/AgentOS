use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

const MAX_EVENTS_PER_OPERATION: usize = 200;
const MAX_OPERATION_EVENTS_RESPONSE: usize = 250;
const MAX_OPERATIONS_IN_SNAPSHOT: usize = 40;
const MAX_EVENTS_IN_SNAPSHOT_OPERATION: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationEvent {
    pub id: String,
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub root_id: String,
    pub ts: String,
    pub actor: String,
    pub project: String,
    pub kind: String,
    pub phase: String,
    pub status: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub semantic: bool,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub root_id: String,
    pub actor: String,
    pub project: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,
    pub phase: String,
    pub status: String,
    pub current_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_semantic_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_semantic_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_beat: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_for: Option<String>,
    #[serde(default)]
    pub children: Vec<String>,
    pub started_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub events: Vec<OperationEvent>,
}

#[derive(Clone, Debug)]
pub struct OperationEventInput {
    pub operation_id: String,
    pub parent_id: Option<String>,
    pub root_id: Option<String>,
    pub actor: String,
    pub project: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<String>,
    pub access: Option<String>,
    pub kind: String,
    pub phase: String,
    pub status: String,
    pub title: String,
    pub detail: Option<String>,
    pub current_tool: Option<String>,
    pub waiting_for: Option<String>,
    pub blocked_by: Option<String>,
    pub semantic: bool,
    pub payload: Value,
}

impl OperationEventInput {
    pub fn new(
        operation_id: impl Into<String>,
        actor: impl Into<String>,
        project: impl Into<String>,
        kind: impl Into<String>,
        phase: impl Into<String>,
        status: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            parent_id: None,
            root_id: None,
            actor: actor.into(),
            project: project.into(),
            provider: None,
            model: None,
            effort: None,
            mode: None,
            access: None,
            kind: kind.into(),
            phase: phase.into(),
            status: status.into(),
            title: title.into(),
            detail: None,
            current_tool: None,
            waiting_for: None,
            blocked_by: None,
            semantic: true,
            payload: json!({}),
        }
    }

    pub fn provider(
        mut self,
        provider: Option<&str>,
        model: Option<&str>,
        effort: Option<&str>,
    ) -> Self {
        self.provider = clean_opt(provider);
        self.model = clean_opt(model);
        self.effort = clean_opt(effort);
        self
    }

    pub fn mode(mut self, mode: Option<&str>, access: Option<&str>) -> Self {
        self.mode = clean_opt(mode);
        self.access = clean_opt(access);
        self
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn waiting_for(mut self, waiting_for: impl Into<String>) -> Self {
        self.waiting_for = Some(waiting_for.into());
        self
    }

    pub fn blocked_by(mut self, blocked_by: impl Into<String>) -> Self {
        self.blocked_by = Some(blocked_by.into());
        self
    }

    pub fn current_tool(mut self, current_tool: impl Into<String>) -> Self {
        self.current_tool = Some(current_tool.into());
        self
    }

    pub fn heartbeat(mut self) -> Self {
        self.semantic = false;
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }
}

fn clean_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(String::from)
}

fn is_terminal(status: &str) -> bool {
    matches!(status, "done" | "failed" | "cancelled")
}

fn event_id(ts: &str, operation_id: &str, kind: &str, len: usize) -> String {
    format!(
        "evt-{}-{}-{}-{}",
        operation_id.replace(&[':', '\\', '/', ' '][..], "-"),
        kind.replace(&[':', '\\', '/', ' '][..], "-"),
        ts.replace(&[':', '.', 'T', 'Z', '-'][..], ""),
        len
    )
}

fn should_store_event(input: &OperationEventInput) -> bool {
    if input.kind != "provider_heartbeat" {
        return true;
    }
    let beat = input.payload.get("beat").and_then(|value| value.as_u64());
    matches!(beat, None | Some(0 | 1)) || beat.is_some_and(|value| value % 15 == 0)
}

pub fn emit(state: &AppState, input: OperationEventInput) {
    let ts = state.now_iso();
    let root_id = input
        .root_id
        .clone()
        .or_else(|| input.parent_id.clone())
        .unwrap_or_else(|| input.operation_id.clone());
    let store_event = should_store_event(&input);

    let event = {
        let mut operations = match state.operations.lock() {
            Ok(ops) => ops,
            Err(e) => e.into_inner(),
        };
        let existing_events_len = operations
            .get(&input.operation_id)
            .map(|op| op.events.len())
            .unwrap_or(0);
        let event = OperationEvent {
            id: event_id(&ts, &input.operation_id, &input.kind, existing_events_len),
            operation_id: input.operation_id.clone(),
            parent_id: input.parent_id.clone(),
            root_id: root_id.clone(),
            ts: ts.clone(),
            actor: input.actor.clone(),
            project: input.project.clone(),
            kind: input.kind.clone(),
            phase: input.phase.clone(),
            status: input.status.clone(),
            title: input.title.clone(),
            detail: input.detail.clone(),
            semantic: input.semantic,
            payload: input.payload.clone(),
        };

        let op = operations
            .entry(input.operation_id.clone())
            .or_insert_with(|| OperationRecord {
                operation_id: input.operation_id.clone(),
                parent_id: input.parent_id.clone(),
                root_id: root_id.clone(),
                actor: input.actor.clone(),
                project: input.project.clone(),
                provider: input.provider.clone(),
                model: input.model.clone(),
                effort: input.effort.clone(),
                mode: input.mode.clone(),
                access: input.access.clone(),
                phase: input.phase.clone(),
                status: input.status.clone(),
                current_action: input.title.clone(),
                current_tool: input.current_tool.clone(),
                last_semantic_event: None,
                last_semantic_ts: None,
                heartbeat_ts: None,
                heartbeat_beat: None,
                blocked_by: input.blocked_by.clone(),
                waiting_for: input.waiting_for.clone(),
                children: Vec::new(),
                started_at: ts.clone(),
                updated_at: ts.clone(),
                completed_at: None,
                events: Vec::new(),
            });

        op.actor = input.actor.clone();
        op.project = input.project.clone();
        op.phase = input.phase.clone();
        op.status = input.status.clone();
        op.current_action = input.title.clone();
        op.current_tool = input
            .current_tool
            .clone()
            .or_else(|| op.current_tool.clone());
        op.waiting_for = input.waiting_for.clone();
        op.blocked_by = input.blocked_by.clone();
        op.updated_at = ts.clone();
        if input.provider.is_some() {
            op.provider = input.provider.clone();
        }
        if input.model.is_some() {
            op.model = input.model.clone();
        }
        if input.effort.is_some() {
            op.effort = input.effort.clone();
        }
        if input.mode.is_some() {
            op.mode = input.mode.clone();
        }
        if input.access.is_some() {
            op.access = input.access.clone();
        }
        if input.kind == "provider_heartbeat" {
            op.heartbeat_ts = Some(ts.clone());
            op.heartbeat_beat = input.payload.get("beat").and_then(|v| v.as_u64());
        }
        if input.semantic {
            op.last_semantic_event = Some(input.title.clone());
            op.last_semantic_ts = Some(ts.clone());
        }
        if is_terminal(&input.status) {
            op.completed_at = Some(ts.clone());
        }
        if store_event {
            op.events.push(event.clone());
            if op.events.len() > MAX_EVENTS_PER_OPERATION {
                let drain_count = op.events.len() - MAX_EVENTS_PER_OPERATION;
                op.events.drain(0..drain_count);
            }
        }

        if let Some(parent_id) = input.parent_id.as_ref() {
            if let Some(parent) = operations.get_mut(parent_id) {
                if !parent.children.contains(&input.operation_id) {
                    parent.children.push(input.operation_id.clone());
                }
                parent.updated_at = ts.clone();
            }
        }

        if store_event {
            Some(event)
        } else {
            None
        }
    };

    if let Some(event) = event {
        let audit_path = state.tasks_dir.join(".operations.jsonl");
        super::jsonl::append_jsonl_logged(
            &audit_path,
            &json!({
                "type": "operation_event",
                "event": event
            }),
            "operation event",
        );
    }
}

pub fn chat_actor(chat_key: &str) -> &'static str {
    if chat_key == "_orchestrator" {
        "orchestrator"
    } else {
        "project_agent"
    }
}

pub fn chat_operation_id(run_id: &str) -> String {
    format!("chat:{}", run_id)
}

pub fn delegation_operation_id(id: &str) -> String {
    format!("delegation:{}", id)
}

pub fn build_operation_context(state: &AppState) -> String {
    let operations = match state.operations.lock() {
        Ok(ops) => ops,
        Err(e) => e.into_inner(),
    };
    let mut active: Vec<&OperationRecord> = operations
        .values()
        .filter(|op| !is_terminal(&op.status))
        .collect();
    active.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    if active.is_empty() {
        return String::new();
    }

    let provider_waits = active
        .iter()
        .filter(|op| op.waiting_for.as_deref() == Some("provider_output"))
        .count();
    let needs_user = active
        .iter()
        .filter(|op| op.status == "needs_user" || op.waiting_for.as_deref() == Some("user"))
        .count();
    let heartbeat_only = active
        .iter()
        .filter(|op| {
            op.waiting_for.as_deref() == Some("provider_output") && op.last_semantic_event.is_none()
        })
        .count();

    let mut lines = Vec::new();
    for op in active.iter().take(8) {
        let provider = op.provider.as_deref().unwrap_or(op.actor.as_str());
        let model = op.model.as_deref().unwrap_or("auto");
        let waiting = op.waiting_for.as_deref().unwrap_or("-");
        let last = op
            .last_semantic_event
            .as_deref()
            .unwrap_or("no semantic event yet");
        let heartbeat = op
            .heartbeat_beat
            .map(|beat| format!("beat #{beat}"))
            .unwrap_or_else(|| "no heartbeat".to_string());
        lines.push(format!(
            "- {} [{} {}] status={} phase={} waiting={} action=\"{}\" last=\"{}\" heartbeat={}",
            op.project,
            provider,
            model,
            op.status,
            op.phase,
            waiting,
            crate::commands::claude_runner::safe_truncate(&op.current_action, 120),
            crate::commands::claude_runner::safe_truncate(last, 120),
            heartbeat
        ));
    }

    format!(
        "[OPERATION HEALTH]\n\
         Active operations: {}. Provider waits: {}. Needs user: {}. Heartbeat-only waits: {}.\n\
         Heartbeat is only process liveness, not meaningful progress. Do not treat heartbeat-only waits as completed work.\n\
         If only heartbeat is changing and there is no semantic event, explicitly say the provider is still silent, then either continue an independent ready route or state the blocker.\n\
         If needs_user > 0, summarize the exact user decision needed before starting more work.\n\
         If the execution map looks noisy, prefer cleanup/status commands over repeating broad dashboard scans.\n\
         {}\n\
         [END OPERATION HEALTH]\n",
        active.len(),
        provider_waits,
        needs_user,
        heartbeat_only,
        lines.join("\n")
    )
}

#[tauri::command]
pub fn get_operation_snapshot(state: State<Arc<AppState>>) -> Value {
    let operations = match state.operations.lock() {
        Ok(ops) => ops,
        Err(e) => e.into_inner(),
    };
    let mut records: Vec<OperationRecord> = operations.values().cloned().collect();
    records.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let active_count = records.iter().filter(|op| !is_terminal(&op.status)).count();
    let needs_user_count = records
        .iter()
        .filter(|op| op.status == "needs_user" || op.waiting_for.as_deref() == Some("user"))
        .count();
    let total_count = records.len();
    for record in records.iter_mut() {
        if record.events.len() > MAX_EVENTS_IN_SNAPSHOT_OPERATION {
            let drain_count = record.events.len() - MAX_EVENTS_IN_SNAPSHOT_OPERATION;
            record.events.drain(0..drain_count);
        }
    }
    if records.len() > MAX_OPERATIONS_IN_SNAPSHOT {
        records.truncate(MAX_OPERATIONS_IN_SNAPSHOT);
    }
    json!({
        "operations": records,
        "active_count": active_count,
        "needs_user_count": needs_user_count,
        "total_count": total_count,
        "updated_at": state.now_iso()
    })
}

#[tauri::command]
pub fn get_operation_events(state: State<Arc<AppState>>, limit: Option<usize>) -> Value {
    let max = limit.unwrap_or(120).min(MAX_OPERATION_EVENTS_RESPONSE);
    let operations = match state.operations.lock() {
        Ok(ops) => ops,
        Err(e) => e.into_inner(),
    };
    let mut events: Vec<OperationEvent> = operations
        .values()
        .flat_map(|op| op.events.iter().cloned())
        .collect();
    events.sort_by(|a, b| b.ts.cmp(&a.ts));
    if events.len() > max {
        events.truncate(max);
    }
    json!({
        "events": events,
        "limit": max,
        "updated_at": state.now_iso()
    })
}

#[tauri::command]
pub fn clear_terminal_operations(state: State<Arc<AppState>>) -> Value {
    let mut operations = match state.operations.lock() {
        Ok(ops) => ops,
        Err(e) => e.into_inner(),
    };
    let before = operations.len();
    operations.retain(|_, op| !is_terminal(&op.status));
    json!({
        "status": "ok",
        "removed": before.saturating_sub(operations.len()),
        "remaining": operations.len()
    })
}
