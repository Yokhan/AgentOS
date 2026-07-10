//! Reverse channel for project agents.
//!
//! Project-side agents can report status, blockers, and results without
//! pretending to be the main orchestrator chat. The report is persisted to the
//! project chat, routed to inbox when needed, and reflected in operation state.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectAgentReportRequest {
    pub project: String,
    pub message: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub delegation_id: Option<String>,
    #[serde(default)]
    pub needs_user: Option<bool>,
}

fn normalize_kind(value: Option<&str>) -> String {
    match value
        .unwrap_or("status")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "blocker" | "blocked" => "blocker".to_string(),
        "question" | "needs_user" => "question".to_string(),
        "error" | "failed" | "failure" => "error".to_string(),
        "result" | "done" | "success" => "result".to_string(),
        "progress" => "progress".to_string(),
        _ => "status".to_string(),
    }
}

fn normalize_status(value: Option<&str>, kind: &str) -> String {
    match value.unwrap_or(kind).trim().to_ascii_lowercase().as_str() {
        "done" | "success" | "ok" | "completed" => "done".to_string(),
        "failed" | "error" | "failure" => "failed".to_string(),
        "blocked" | "needs_user" | "question" => "needs_user".to_string(),
        "running" | "progress" | "working" => "running".to_string(),
        _ => "info".to_string(),
    }
}

fn report_needs_user(request: &ProjectAgentReportRequest, kind: &str, status: &str) -> bool {
    request.needs_user.unwrap_or_else(|| {
        kind == "question" || kind == "blocker" || status == "needs_user" || status == "failed"
    })
}

pub fn project_agent_report_core(
    state: &AppState,
    request: ProjectAgentReportRequest,
) -> Result<Value, String> {
    let project = request.project.trim();
    if project.is_empty() {
        return Err("project is required".to_string());
    }
    let project = state
        .validate_project_name_from_llm(project)
        .or_else(|| {
            state
                .validate_project(project)
                .ok()
                .map(|_| project.to_string())
        })
        .ok_or_else(|| format!("unknown project: {}", project))?;

    let message = request.message.trim();
    if message.is_empty() {
        return Err("message is required".to_string());
    }

    let kind = normalize_kind(request.kind.as_deref());
    let status = normalize_status(request.status.as_deref(), &kind);
    let needs_user = report_needs_user(&request, &kind, &status);
    let ts = state.now_iso();
    let id = format!(
        "project-report-{}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| chrono::Utc::now().timestamp_micros() * 1000)
    );

    super::inbox::push_inbox(
        state,
        &project,
        &kind,
        message,
        needs_user,
        request.delegation_id.as_deref(),
        None,
    );

    let chat_file = state.chats_dir.join(format!("{}.jsonl", project));
    super::jsonl::append_jsonl_logged(
        &chat_file,
        &json!({
            "ts": ts,
            "role": "system",
            "kind": "project_agent_report",
            "source": request.source.as_deref().unwrap_or("project_agent"),
            "status": status,
            "msg": message,
            "delegation_id": request.delegation_id,
        }),
        "project agent report chat append",
    );

    super::operation_state::emit(
        state,
        super::operation_state::OperationEventInput::new(
            format!("project-report:{}", id),
            "project_agent",
            &project,
            "project_agent_report",
            &kind,
            &status,
            super::claude_runner::safe_truncate(message, 140),
        )
        .waiting_for(if needs_user { "user" } else { "orchestrator" })
        .payload(json!({
            "report_id": id,
            "delegation_id": request.delegation_id,
            "needs_user": needs_user,
            "source": request.source,
        })),
    );
    Ok(json!({
        "status": "ok",
        "id": id,
        "project": project,
        "kind": kind,
        "state": status,
        "needs_user": needs_user
    }))
}

#[tauri::command]
pub fn project_agent_report(
    state: State<Arc<AppState>>,
    request: ProjectAgentReportRequest,
) -> Result<Value, String> {
    project_agent_report_core(&state, request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "agentos-project-report-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("n8n")).expect("config dir");
        std::fs::write(
            path.join("n8n").join("config.json"),
            serde_json::json!({"documents_dir": path.to_string_lossy()}).to_string(),
        )
        .expect("config");
        std::fs::create_dir_all(path.join("ProjectA")).expect("project");
        std::fs::create_dir_all(path.join(".agentos").join("chats")).expect("chats");
        path
    }

    #[test]
    fn report_routes_blocker_to_inbox_and_operation_state() {
        let root = test_root("blocker");
        let state = AppState::new(root.clone());
        let result = project_agent_report_core(
            &state,
            ProjectAgentReportRequest {
                project: "ProjectA".to_string(),
                message: "Need approval".to_string(),
                kind: Some("blocker".to_string()),
                status: None,
                source: None,
                delegation_id: Some("d-1".to_string()),
                needs_user: None,
            },
        )
        .expect("report");

        assert_eq!(result["needs_user"], true);
        let inbox = state.inbox.lock().expect("inbox");
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].project, "ProjectA");
        assert!(state.chats_dir.join("ProjectA.jsonl").exists());

        let _ = std::fs::remove_dir_all(root);
    }
}
