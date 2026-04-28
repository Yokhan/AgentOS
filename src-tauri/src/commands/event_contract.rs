//! Shared event contract for chat, Duo, delegation, and project-agent timeline views.

use crate::state::{Delegation, SessionEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

use crate::state::AppState;

pub const EVENT_SCHEMA_VERSION: &str = "agentos.event.v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRow {
    pub source: String,
    pub kind: String,
    pub status: String,
    pub title: String,
    pub detail: String,
    pub project: String,
    pub ts: String,
}

impl EventRow {
    pub fn new(
        source: &str,
        kind: &str,
        status: &str,
        title: impl Into<String>,
        detail: impl Into<String>,
        project: &str,
        ts: impl Into<String>,
    ) -> Self {
        Self {
            source: source.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            title: title.into(),
            detail: detail.into(),
            project: project.to_string(),
            ts: ts.into(),
        }
    }

    pub fn warning_like(&self) -> bool {
        matches!(
            self.status.as_str(),
            "warning" | "failed" | "cancelled" | "error" | "needs_permission"
        )
    }
}

pub fn short(value: &str, max: usize) -> String {
    let trimmed = value.trim().replace(['\r', '\n'], " ");
    if trimmed.chars().count() <= max {
        trimmed
    } else {
        trimmed.chars().take(max).collect::<String>() + "..."
    }
}

fn event_ts(value: &Value, fallback: &str) -> String {
    value
        .get("ts")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback)
        .to_string()
}

fn compact_json(value: Option<&Value>) -> String {
    match value {
        Some(v) if !v.is_null() => short(&v.to_string(), 160),
        _ => String::new(),
    }
}

pub fn normalize_chat_stream_event(
    evt: &Value,
    project: &str,
    fallback_ts: &str,
) -> Option<EventRow> {
    let typ = evt.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let ts = event_ts(evt, fallback_ts);
    match typ {
        "run_started" => Some(EventRow::new(
            "chat",
            "run",
            "running",
            "Run started",
            format!(
                "{} / {} / {}",
                evt.get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent"),
                evt.get("model").and_then(|v| v.as_str()).unwrap_or("auto"),
                evt.get("mode").and_then(|v| v.as_str()).unwrap_or("act")
            ),
            project,
            ts,
        )),
        "run_progress" | "run_heartbeat" => Some(EventRow::new(
            "chat",
            "progress",
            evt.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("running"),
            evt.get("phase")
                .and_then(|v| v.as_str())
                .unwrap_or("progress"),
            evt.get("detail")
                .and_then(|v| v.as_str())
                .map(|v| short(v, 180))
                .unwrap_or_default(),
            project,
            ts,
        )),
        "tool_use" | "tool_start" => Some(EventRow::new(
            "chat",
            "tool",
            evt.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("running"),
            format!(
                "Tool: {}",
                evt.get("tool").and_then(|v| v.as_str()).unwrap_or("tool")
            ),
            compact_json(evt.get("input")),
            project,
            ts,
        )),
        "tool_result" | "tool_stop" => Some(EventRow::new(
            "chat",
            "tool_result",
            if evt
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                "warning"
            } else {
                "done"
            },
            "Tool result",
            evt.get("content")
                .and_then(|v| v.as_str())
                .map(|v| short(v, 180))
                .unwrap_or_default(),
            project,
            ts,
        )),
        "pa_status" | "pa_result" | "warning" => {
            let text = evt.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let waiting = typ == "pa_status" && text.starts_with("Waiting coordinator:");
            Some(EventRow::new(
                "agentos",
                if waiting { "coordination" } else { "command" },
                if waiting {
                    "waiting"
                } else if typ == "warning" {
                    "warning"
                } else {
                    "done"
                },
                if waiting {
                    "Waiting coordinator"
                } else {
                    evt.get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("AgentOS command")
                },
                short(text, 180),
                project,
                ts,
            ))
        }
        "delegation" => {
            let delegated_project = evt
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or(project);
            Some(EventRow::new(
                "delegation",
                "queued",
                "pending",
                format!("Delegated to {}", delegated_project),
                evt.get("task")
                    .and_then(|v| v.as_str())
                    .map(|v| short(v, 180))
                    .unwrap_or_default(),
                delegated_project,
                ts,
            ))
        }
        "run_done" | "done" => {
            let outcome = evt
                .get("outcome")
                .or_else(|| evt.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("done");
            Some(EventRow::new(
                "chat",
                "done",
                outcome,
                "Run finished",
                evt.get("detail")
                    .or_else(|| evt.get("text"))
                    .and_then(|v| v.as_str())
                    .map(|v| short(v, 180))
                    .unwrap_or_else(|| outcome.to_string()),
                project,
                ts,
            ))
        }
        "thinking_start" | "thinking" => Some(EventRow::new(
            "chat",
            "thinking",
            "running",
            "Thinking",
            evt.get("text")
                .and_then(|v| v.as_str())
                .map(|v| short(v, 180))
                .unwrap_or_default(),
            project,
            ts,
        )),
        "thinking_stop" => Some(EventRow::new(
            "chat",
            "thinking",
            "done",
            "Thinking finished",
            "",
            project,
            ts,
        )),
        "usage" | "cost" => Some(EventRow::new(
            "chat",
            typ,
            "info",
            typ.replace('_', " "),
            compact_json(evt.get("usage").or_else(|| evt.get("cost_usd"))),
            project,
            ts,
        )),
        "safety" => Some(EventRow::new(
            "chat",
            "safety",
            "warning",
            "Safety stop",
            evt.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("safety event"),
            project,
            ts,
        )),
        "system" => Some(EventRow::new(
            "chat",
            "system",
            "info",
            evt.get("system")
                .and_then(|v| v.as_str())
                .unwrap_or("System"),
            "",
            project,
            ts,
        )),
        _ => None,
    }
}

pub fn normalize_session_event(event: &SessionEvent, project: &str) -> EventRow {
    EventRow::new(
        "duo",
        &event.kind,
        "info",
        event.kind.replace('_', " "),
        short(&event.payload.to_string(), 180),
        project,
        event.ts.clone(),
    )
}

pub fn normalize_delegation_state(delegation: &Delegation) -> EventRow {
    EventRow::new(
        "delegation",
        "state",
        &delegation.status.to_string(),
        format!("{}: {}", delegation.project, delegation.status),
        short(&delegation.task, 180),
        &delegation.project,
        delegation.ts.clone(),
    )
}

pub fn normalize_delegation_stream_event(evt: &Value, delegation: &Delegation) -> Option<EventRow> {
    let typ = evt.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match typ {
        "stage" => Some(EventRow::new(
            "delegation",
            "stage",
            "running",
            evt.get("stage").and_then(|v| v.as_str()).unwrap_or("stage"),
            evt.get("label")
                .and_then(|v| v.as_str())
                .map(|v| short(v, 180))
                .unwrap_or_default(),
            &delegation.project,
            event_ts(evt, &delegation.ts),
        )),
        "done" => Some(EventRow::new(
            "delegation",
            "done",
            evt.get("status").and_then(|v| v.as_str()).unwrap_or("done"),
            format!(
                "Delegation {}",
                delegation.id.chars().take(8).collect::<String>()
            ),
            evt.get("response")
                .and_then(|v| v.as_str())
                .map(|v| short(v, 180))
                .unwrap_or_default(),
            &delegation.project,
            event_ts(evt, &delegation.ts),
        )),
        "safety" => Some(EventRow::new(
            "delegation",
            "safety",
            "warning",
            "Delegation safety stop",
            evt.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("safety event"),
            &delegation.project,
            event_ts(evt, &delegation.ts),
        )),
        "tool_start" | "tool_stop" | "usage" | "cost" | "thinking" => {
            normalize_chat_stream_event(evt, &delegation.project, &delegation.ts).map(|mut row| {
                row.source = "delegation".to_string();
                row
            })
        }
        _ => None,
    }
}

pub fn event_contract_schema_value() -> Value {
    json!({
        "status": "ok",
        "schema_version": EVENT_SCHEMA_VERSION,
        "big_plan": {
            "stage": "event_contract",
            "stage_index": 5,
            "stage_total": 6,
            "label": "Event contract + normalized source adapters"
        },
        "row_shape": ["source", "kind", "status", "title", "detail", "project", "ts"],
        "sources": [
            {"id": "chat", "label": "Solo/project chat stream", "coverage": ["run_started", "run_progress", "run_done", "tool_use", "tool_result", "thinking", "pa_status", "warning"]},
            {"id": "duo", "label": "Duo room/session events", "coverage": ["session_event"]},
            {"id": "delegation", "label": "Project-agent delegation lifecycle", "coverage": ["state", "stage", "done", "safety", "tool", "usage", "cost"]}
        ],
        "guarantees": [
            "read_only",
            "backwards_compatible_jsonl",
            "single_ui_row_contract"
        ]
    })
}

#[tauri::command]
pub fn get_event_contract_schema(_state: State<Arc<AppState>>) -> Value {
    event_contract_schema_value()
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_chat_stream_event, normalize_delegation_stream_event, normalize_session_event,
    };
    use crate::commands::status::DelegationStatus;
    use crate::state::{Delegation, SessionEvent};
    use serde_json::json;

    #[test]
    fn chat_events_normalize_to_event_row() {
        let row = normalize_chat_stream_event(
            &json!({"type":"run_started","provider":"codex","model":"gpt-5.5","mode":"act","ts":"2026-04-26T10:00:00Z"}),
            "_orchestrator",
            "",
        )
        .expect("normalized row");
        assert_eq!(row.source, "chat");
        assert_eq!(row.kind, "run");
        assert_eq!(row.status, "running");
        assert_eq!(row.project, "_orchestrator");
    }

    #[test]
    fn session_events_normalize_to_duo_rows() {
        let event = SessionEvent {
            id: "evt-1".to_string(),
            session_id: "room-1".to_string(),
            ts: "2026-04-26T10:00:00Z".to_string(),
            kind: "round_started".to_string(),
            actor: "system".to_string(),
            payload: json!({"participants": 2}),
        };
        let row = normalize_session_event(&event, "AgentOS");
        assert_eq!(row.source, "duo");
        assert_eq!(row.kind, "round_started");
        assert_eq!(row.project, "AgentOS");
    }

    #[test]
    fn delegation_stream_events_normalize_to_delegation_rows() {
        let delegation = Delegation {
            id: "delegation-123456".to_string(),
            project: "AgentOS".to_string(),
            task: "Check release".to_string(),
            ts: "2026-04-26T10:00:00Z".to_string(),
            started_at: Some("2026-04-26T10:00:05Z".to_string()),
            status: DelegationStatus::Running,
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
        let row = normalize_delegation_stream_event(
            &json!({"type":"stage","stage":"build","label":"Building release"}),
            &delegation,
        )
        .expect("normalized row");
        assert_eq!(row.source, "delegation");
        assert_eq!(row.kind, "stage");
        assert_eq!(row.status, "running");
    }
}
