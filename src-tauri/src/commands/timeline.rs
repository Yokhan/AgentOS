//! Execution timeline: read-only normalized view over chat, Duo, and delegation events.

use crate::state::{AppState, Delegation, SessionEvent};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tauri::State;

fn short(value: &str, max: usize) -> String {
    let trimmed = value.trim().replace('\n', " ");
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

fn row(
    source: &str,
    kind: &str,
    status: &str,
    title: String,
    detail: String,
    project: &str,
    ts: String,
) -> Value {
    json!({
        "source": source,
        "kind": kind,
        "status": status,
        "title": title,
        "detail": detail,
        "project": project,
        "ts": ts
    })
}

fn normalize_stream_event(evt: &Value, project: &str, fallback_ts: &str) -> Option<Value> {
    let typ = evt.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let ts = event_ts(evt, fallback_ts);
    match typ {
        "run_started" => Some(row(
            "chat",
            "run",
            "running",
            "Run started".to_string(),
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
        "run_progress" | "run_heartbeat" => Some(row(
            "chat",
            "progress",
            evt.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("running"),
            evt.get("phase")
                .and_then(|v| v.as_str())
                .unwrap_or("progress")
                .to_string(),
            evt.get("detail")
                .and_then(|v| v.as_str())
                .map(short_detail)
                .unwrap_or_default(),
            project,
            ts,
        )),
        "tool_use" => Some(row(
            "chat",
            "tool",
            evt.get("status").and_then(|v| v.as_str()).unwrap_or("done"),
            format!(
                "Tool: {}",
                evt.get("tool").and_then(|v| v.as_str()).unwrap_or("tool")
            ),
            compact_json(evt.get("input")),
            project,
            ts,
        )),
        "tool_result" => Some(row(
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
            "Tool result".to_string(),
            evt.get("content")
                .and_then(|v| v.as_str())
                .map(short_detail)
                .unwrap_or_default(),
            project,
            ts,
        )),
        "pa_status" | "pa_result" | "warning" => Some(row(
            "agentos",
            "command",
            if typ == "warning" { "warning" } else { "done" },
            evt.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("AgentOS command")
                .to_string(),
            evt.get("text")
                .and_then(|v| v.as_str())
                .map(short_detail)
                .unwrap_or_default(),
            project,
            ts,
        )),
        "delegation" => Some(row(
            "delegation",
            "queued",
            "pending",
            format!(
                "Delegated to {}",
                evt.get("project")
                    .and_then(|v| v.as_str())
                    .unwrap_or(project)
            ),
            evt.get("task")
                .and_then(|v| v.as_str())
                .map(short_detail)
                .unwrap_or_default(),
            evt.get("project")
                .and_then(|v| v.as_str())
                .unwrap_or(project),
            ts,
        )),
        "run_done" | "done" => {
            let outcome = evt
                .get("outcome")
                .or_else(|| evt.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("done");
            Some(row(
                "chat",
                "done",
                outcome,
                "Run finished".to_string(),
                evt.get("detail")
                    .or_else(|| evt.get("text"))
                    .and_then(|v| v.as_str())
                    .map(short_detail)
                    .unwrap_or_else(|| outcome.to_string()),
                project,
                ts,
            ))
        }
        "thinking_start" => Some(row(
            "chat",
            "thinking",
            "running",
            "Thinking started".to_string(),
            String::new(),
            project,
            ts,
        )),
        "thinking_stop" => Some(row(
            "chat",
            "thinking",
            "done",
            "Thinking finished".to_string(),
            String::new(),
            project,
            ts,
        )),
        "system" => Some(row(
            "chat",
            "system",
            "info",
            evt.get("system")
                .and_then(|v| v.as_str())
                .unwrap_or("System")
                .to_string(),
            String::new(),
            project,
            ts,
        )),
        _ => None,
    }
}

fn short_detail(value: &str) -> String {
    short(value, 180)
}

fn read_recent_jsonl(path: &Path, limit: usize) -> Vec<Value> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut rows = Vec::new();
    for line in content.lines().rev() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            rows.push(value);
            if rows.len() >= limit {
                break;
            }
        }
    }
    rows.reverse();
    rows
}

fn chat_stream_rows(state: &AppState, project: &str, limit: usize) -> Vec<Value> {
    let chat_key = if project.trim().is_empty() {
        "_orchestrator"
    } else {
        project
    };
    let path = state
        .root
        .join("tasks")
        .join(format!(".stream-{}.jsonl", chat_key));
    read_recent_jsonl(&path, limit)
        .iter()
        .filter_map(|evt| normalize_stream_event(evt, chat_key, ""))
        .collect()
}

fn session_rows(events: &[SessionEvent], project: &str) -> Vec<Value> {
    events
        .iter()
        .map(|event| {
            row(
                "duo",
                &event.kind,
                "info",
                event.kind.replace('_', " "),
                short(&event.payload.to_string(), 180),
                project,
                event.ts.clone(),
            )
        })
        .collect()
}

fn delegation_state_row(delegation: &Delegation) -> Value {
    row(
        "delegation",
        "state",
        &delegation.status.to_string(),
        format!("{}: {}", delegation.project, delegation.status),
        short_detail(&delegation.task),
        &delegation.project,
        delegation.ts.clone(),
    )
}

fn delegation_stream_rows(state: &AppState, delegation: &Delegation, limit: usize) -> Vec<Value> {
    let path = state
        .root
        .join("tasks")
        .join(format!(".stream-deleg-{}.jsonl", delegation.id));
    read_recent_jsonl(&path, limit)
        .iter()
        .filter_map(|evt| {
            let typ = evt.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match typ {
                "stage" => Some(row(
                    "delegation",
                    "stage",
                    "running",
                    evt.get("stage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("stage")
                        .to_string(),
                    evt.get("label")
                        .and_then(|v| v.as_str())
                        .map(short_detail)
                        .unwrap_or_default(),
                    &delegation.project,
                    event_ts(evt, &delegation.ts),
                )),
                "done" => Some(row(
                    "delegation",
                    "done",
                    evt.get("status").and_then(|v| v.as_str()).unwrap_or("done"),
                    format!(
                        "Delegation {}",
                        delegation.id.chars().take(8).collect::<String>()
                    ),
                    evt.get("response")
                        .and_then(|v| v.as_str())
                        .map(short_detail)
                        .unwrap_or_default(),
                    &delegation.project,
                    event_ts(evt, &delegation.ts),
                )),
                _ => None,
            }
        })
        .collect()
}

fn delegation_rows(state: &AppState, project: &str, limit: usize) -> Vec<Value> {
    let mut delegations: Vec<Delegation> = state
        .delegations
        .lock()
        .map(|items| {
            items
                .values()
                .filter(|item| project.is_empty() || item.project == project)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    delegations.sort_by(|a, b| b.ts.cmp(&a.ts));
    let mut rows = Vec::new();
    for delegation in delegations.into_iter().take(limit.min(8)) {
        rows.push(delegation_state_row(&delegation));
        rows.extend(delegation_stream_rows(state, &delegation, 4));
    }
    rows
}

pub fn build_execution_timeline(
    state: &AppState,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: usize,
) -> Value {
    let project = project.unwrap_or_default();
    let limit = limit.clamp(10, 120);
    let mut rows = chat_stream_rows(state, &project, limit);

    if let Some(session_id) = room_session_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
    {
        rows.extend(session_rows(
            &state.get_session_events(session_id, limit.min(40)),
            &project,
        ));
    }
    rows.extend(delegation_rows(state, &project, limit.min(24)));
    rows.sort_by(|a, b| {
        let at = a.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let bt = b.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        at.cmp(bt)
    });
    if rows.len() > limit {
        rows = rows[rows.len() - limit..].to_vec();
    }
    let warnings = rows
        .iter()
        .filter(|row| {
            matches!(
                row.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                "warning" | "failed" | "cancelled" | "error"
            )
        })
        .count();
    json!({
        "status": "ok",
        "project": if project.is_empty() { "_orchestrator" } else { project.as_str() },
        "big_plan": {
            "stage": "timeline",
            "stage_index": 4,
            "stage_total": 6,
            "label": "Execution timeline + event normalization"
        },
        "counts": {
            "items": rows.len(),
            "warnings": warnings
        },
        "items": rows
    })
}

#[tauri::command]
pub fn get_execution_timeline(
    state: State<Arc<AppState>>,
    project: Option<String>,
    room_session_id: Option<String>,
    limit: Option<usize>,
) -> Value {
    build_execution_timeline(&state, project, room_session_id, limit.unwrap_or(60))
}

#[cfg(test)]
mod tests {
    use super::build_execution_timeline;
    use crate::state::AppState;
    use serde_json::json;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-timeline-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("tasks")).expect("create temp tasks");
        path
    }

    #[test]
    fn stream_events_are_normalized_into_timeline() {
        let root = test_root("stream");
        let state = AppState::new(root.clone());
        let path = root.join("tasks").join(".stream-_orchestrator.jsonl");
        crate::commands::jsonl::append_jsonl_logged(
            &path,
            &json!({"type":"run_started","provider":"codex","model":"gpt-5.5","mode":"act","ts":"2026-04-26T10:00:00Z"}),
            "test stream start",
        );
        crate::commands::jsonl::append_jsonl_logged(
            &path,
            &json!({"type":"tool_use","tool":"Read","status":"started","ts":"2026-04-26T10:00:01Z"}),
            "test stream tool",
        );

        let result = build_execution_timeline(&state, None, None, 10);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["big_plan"]["stage"], "timeline");
        assert_eq!(result["items"][0]["kind"], "run");
        assert_eq!(result["items"][1]["kind"], "tool");

        let _ = std::fs::remove_dir_all(root);
    }
}
