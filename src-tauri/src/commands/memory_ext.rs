//! Extended memory commands: list, search, delete PA memories.

use crate::state::AppState;
use serde_json::Value;

/// MEMORY_LIST: list saved memories
pub fn memory_list(state: &AppState, filter: &str) -> Option<String> {
    let path = state.root.join("tasks").join("pa-memory.jsonl");
    let content = std::fs::read_to_string(&path).ok()?;
    let mut entries: Vec<(String, String)> = Vec::new();

    for line in content.lines() {
        if let Ok(e) = serde_json::from_str::<Value>(line) {
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let note = e.get("note").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if filter.is_empty() || filter == "?recent" || note.to_lowercase().contains(&filter.to_lowercase()) {
                entries.push((ts, note));
            }
        }
    }

    if filter == "?recent" {
        entries = entries.into_iter().rev().take(20).collect();
    }

    if entries.is_empty() { return Some("No memories found.".to_string()); }
    let lines: Vec<String> = entries.iter()
        .map(|(ts, note)| format!("  {} {}", ts.chars().take(16).collect::<String>(), note.chars().take(80).collect::<String>()))
        .collect();
    Some(format!("**Memories ({}):**\n{}", lines.len(), lines.join("\n")))
}

/// MEMORY_SEARCH: full-text search
pub fn memory_search(state: &AppState, query: &str) -> Option<String> {
    let filter = if query.starts_with("?search:") { &query[8..] } else { query };
    memory_list(state, filter)
}

/// MEMORY_DELETE: remove by timestamp or age
pub fn memory_delete(state: &AppState, filter: &str) -> Option<String> {
    let path = state.root.join("tasks").join("pa-memory.jsonl");
    let archive = state.root.join("tasks").join("pa-memory-archive.jsonl");
    let content = std::fs::read_to_string(&path).ok()?;
    let mut kept = Vec::new();
    let mut archived = 0;

    for line in content.lines() {
        if let Ok(e) = serde_json::from_str::<Value>(line) {
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("");
            let should_delete = if filter.starts_with("?older:") {
                let days: i64 = filter[7..].trim_end_matches('d').parse().unwrap_or(30);
                let threshold = chrono::Utc::now() - chrono::Duration::days(days);
                chrono::DateTime::parse_from_rfc3339(ts)
                    .or_else(|_| chrono::DateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%SZ"))
                    .map(|dt| dt < threshold).unwrap_or(false)
            } else {
                ts == filter
            };

            if should_delete {
                super::jsonl::append_jsonl_logged(&archive, &e, "memory archive");
                archived += 1;
            } else {
                kept.push(line.to_string());
            }
        }
    }

    let _ = std::fs::write(&path, kept.join("\n") + if kept.is_empty() { "" } else { "\n" });
    Some(format!("**Memory cleanup:** archived {} entries", archived))
}
