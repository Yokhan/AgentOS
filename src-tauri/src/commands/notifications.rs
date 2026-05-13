//! Persistent notification center for operational noise.
//!
//! Chat stays for conversation. AgentOS command/status output goes here unless
//! it is part of the current live stream.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

const NOTIFICATIONS_FILE: &str = ".notifications.jsonl";
const NOTIFICATIONS_ARCHIVE_FILE: &str = ".notifications-archive.jsonl";
const MAX_MESSAGE_CHARS: usize = 2400;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationInput {
    pub severity: String,
    pub source: String,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub project: Option<String>,
    pub command: Option<String>,
    pub operation_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NotificationRecord {
    id: String,
    ts: String,
    severity: String,
    source: String,
    kind: String,
    title: String,
    message: String,
    project: String,
    command: String,
    operation_id: String,
}

fn notifications_path(state: &AppState) -> PathBuf {
    state.root.join("tasks").join(NOTIFICATIONS_FILE)
}

fn notifications_archive_path(state: &AppState) -> PathBuf {
    state.root.join("tasks").join(NOTIFICATIONS_ARCHIVE_FILE)
}

fn trim_message(value: &str) -> String {
    let mut chars = value.trim().chars();
    let mut out = String::new();
    for _ in 0..MAX_MESSAGE_CHARS {
        let Some(ch) = chars.next() else {
            return out;
        };
        out.push(ch);
    }
    if chars.next().is_some() {
        out.push_str("\n... [truncated]");
    }
    out
}

fn normalize_severity(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "critical" => "critical",
        "warning" | "warn" | "error" | "failed" => "warning",
        "success" | "done" | "ok" => "success",
        _ => "info",
    }
}

fn read_notification_records(path: &Path, limit: usize) -> Vec<NotificationRecord> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut rows = Vec::new();
    for line in content.lines().rev() {
        if let Ok(record) = serde_json::from_str::<NotificationRecord>(line) {
            rows.push(record);
            if rows.len() >= limit {
                break;
            }
        }
    }
    rows.reverse();
    rows
}

fn should_persist(input: &NotificationInput) -> bool {
    if normalize_severity(&input.severity) != "info" {
        return true;
    }
    if input.kind == "command_result" {
        return true;
    }
    let text = input.message.trim();
    if text.starts_with("Waiting coordinator:") {
        return true;
    }
    let lower = text.to_ascii_lowercase();
    lower.contains("stopped")
        || lower.contains("blocked")
        || lower.contains("waiting coordinator")
        || lower.contains("missing command")
        || lower.contains("no commands")
}

pub fn append_notification(state: &AppState, input: NotificationInput) {
    if !should_persist(&input) {
        return;
    }
    let _ = std::fs::create_dir_all(state.root.join("tasks"));
    let ts = state.now_iso();
    let id = format!(
        "ntf-{}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| chrono::Utc::now().timestamp_micros() * 1000)
    );
    let record = NotificationRecord {
        id,
        ts,
        severity: normalize_severity(&input.severity).to_string(),
        source: input.source.trim().to_string(),
        kind: input.kind.trim().to_string(),
        title: input.title.trim().to_string(),
        message: trim_message(&input.message),
        project: input.project.unwrap_or_default(),
        command: input.command.unwrap_or_default(),
        operation_id: input.operation_id.unwrap_or_default(),
    };
    crate::commands::jsonl::append_jsonl_logged(
        &notifications_path(state),
        &json!(record),
        "notification append",
    );
}

fn notifications_value(state: &AppState, limit: usize, severity: &str) -> Value {
    let limit = limit.clamp(10, 500);
    let severity_filter = severity.trim().to_ascii_lowercase();
    let mut rows = read_notification_records(&notifications_path(state), limit);
    if !severity_filter.is_empty() && severity_filter != "all" {
        rows.retain(|row| row.severity == severity_filter);
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        *counts.entry(row.severity.clone()).or_default() += 1;
    }
    json!({
        "status": "ok",
        "count": rows.len(),
        "counts": counts,
        "items": rows
    })
}

#[tauri::command]
pub fn get_notifications(
    state: State<Arc<AppState>>,
    limit: Option<usize>,
    severity: Option<String>,
) -> Value {
    notifications_value(
        &state,
        limit.unwrap_or(120).clamp(10, 500),
        severity.unwrap_or_default().as_str(),
    )
}

#[tauri::command]
pub fn clear_notifications(state: State<Arc<AppState>>) -> Value {
    let path = notifications_path(&state);
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if !content.trim().is_empty() {
        let archive = notifications_archive_path(&state);
        if let Err(err) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&archive)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(content.as_bytes())
            })
        {
            return json!({"status":"error","error":err.to_string()});
        }
    }
    match std::fs::write(&path, "") {
        Ok(_) => json!({"status":"ok"}),
        Err(err) => json!({"status":"error","error":err.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::{append_notification, notifications_value, NotificationInput};
    use crate::state::AppState;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-notifications-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("tasks")).expect("create temp tasks");
        path
    }

    #[test]
    fn routine_status_is_not_persisted_but_warning_is() {
        let root = test_root("filter");
        let state = AppState::new(root.clone());
        append_notification(
            &state,
            NotificationInput {
                severity: "info".into(),
                source: "agentos".into(),
                kind: "command_status".into(),
                title: "[DASHBOARD_FULL]".into(),
                message: "Running [DASHBOARD_FULL]".into(),
                project: None,
                command: Some("[DASHBOARD_FULL]".into()),
                operation_id: None,
            },
        );
        append_notification(
            &state,
            NotificationInput {
                severity: "warning".into(),
                source: "agentos".into(),
                kind: "command_warning".into(),
                title: "Malformed command".into(),
                message: "Delegation not parsed".into(),
                project: None,
                command: None,
                operation_id: None,
            },
        );

        let result = notifications_value(&state, 50, "");
        assert_eq!(result["status"], "ok");
        assert_eq!(result["count"], 1);
        assert_eq!(result["items"][0]["severity"], "warning");

        let _ = std::fs::remove_dir_all(root);
    }
}
