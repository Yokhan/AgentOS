use crate::state::AppState;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tauri::State;

pub fn search_chat_history_core(
    state: &AppState,
    query: &str,
    project_filter: Option<&str>,
    limit: usize,
) -> Value {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return json!({"query": query, "matches": [], "count": 0});
    }
    let project_filter = project_filter
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all");

    let mut matches = Vec::new();
    let max = limit.clamp(1, 200);
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&state.chats_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }
    }
    files.push(state.tasks_dir.join(".chat-history.jsonl"));

    for path in files {
        let file_project = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("chat")
            .trim_start_matches(".")
            .to_string();
        let Ok(file) = std::fs::File::open(&path) else {
            continue;
        };
        let total = BufReader::new(file).lines().count();
        let mut reverse_index = total;
        let _ = super::jsonl::for_each_line_reverse(&path, |line| {
            let line_index = reverse_index;
            reverse_index = reverse_index.saturating_sub(1);
            let Ok(row) = serde_json::from_str::<Value>(line) else {
                return true;
            };
            let row_project = row
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or(file_project.as_str());
            if let Some(filter) = project_filter {
                if !row_project.eq_ignore_ascii_case(filter)
                    && !file_project.eq_ignore_ascii_case(filter)
                {
                    return true;
                }
            }
            let text = row
                .get("msg")
                .or_else(|| row.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !text.to_lowercase().contains(&q) {
                return true;
            }
            let snippet: String = text.chars().take(260).collect();
            matches.push(json!({
                "project": row_project,
                "role": row.get("role").and_then(|v| v.as_str()).unwrap_or(""),
                "ts": row.get("ts").and_then(|v| v.as_str()).unwrap_or(""),
                "line": line_index,
                "snippet": snippet,
            }));
            if matches.len() >= max {
                return false;
            }
            true
        });
        if matches.len() >= max {
            return json!({"query": query, "matches": matches, "count": matches.len()});
        }
    }

    json!({"query": query, "matches": matches, "count": matches.len()})
}

#[tauri::command]
pub fn search_chat_history(
    state: State<Arc<AppState>>,
    query: String,
    project: Option<String>,
    limit: Option<usize>,
) -> Value {
    search_chat_history_core(&state, &query, project.as_deref(), limit.unwrap_or(50))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_project_chat_message() {
        let root = std::env::temp_dir().join(format!(
            "agentos-chat-search-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("chats")).unwrap();
        std::fs::create_dir_all(root.join("tasks")).unwrap();
        let state = AppState::new(root.clone());
        let chat = state.chats_dir.join("ProjectA.jsonl");
        std::fs::write(
            &chat,
            r#"{"ts":"2026-01-01T00:00:00Z","role":"assistant","msg":"needle in project chat"}"#,
        )
        .unwrap();

        let result = search_chat_history_core(&state, "needle", Some("ProjectA"), 10);
        assert_eq!(result["count"], 1);
        assert_eq!(result["matches"][0]["project"], "ProjectA");

        let _ = std::fs::remove_dir_all(root);
    }
}
