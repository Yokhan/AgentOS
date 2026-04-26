//! Token usage and cost tracking for delegations and chat.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct UsageInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub model: String,
}

/// Append usage entry to .usage-log.jsonl (called from delegation_stream when usage events are extracted)
#[allow(dead_code)]
pub fn append_usage(root: &std::path::Path, project: &str, usage: &UsageInfo) {
    let path = root.join("tasks").join(".usage-log.jsonl");
    let entry = json!({
        "ts": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "project": project,
        "input_tokens": usage.input_tokens,
        "output_tokens": usage.output_tokens,
        "cost_usd": usage.cost_usd,
        "model": usage.model,
    });
    super::jsonl::append_jsonl_logged(&path, &entry, "usage log");
}

/// Get usage summary — totals by project and overall
#[tauri::command]
pub fn get_usage_summary(state: State<Arc<AppState>>) -> Value {
    let path = state.root.join("tasks").join(".usage-log.jsonl");
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let mut total_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut by_project: std::collections::HashMap<String, Value> = std::collections::HashMap::new();

    for line in content.lines() {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            let project = entry
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let input = entry
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = entry
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cost = entry
                .get("cost_usd")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            total_input += input;
            total_output += output;
            total_cost += cost;

            let proj = by_project.entry(project.to_string()).or_insert_with(
                || json!({"input_tokens": 0, "output_tokens": 0, "cost_usd": 0.0, "count": 0}),
            );
            if let Some(obj) = proj.as_object_mut() {
                let pi = obj
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let po = obj
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let pc = obj.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let pn = obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                obj.insert("input_tokens".to_string(), json!(pi + input));
                obj.insert("output_tokens".to_string(), json!(po + output));
                obj.insert("cost_usd".to_string(), json!(pc + cost));
                obj.insert("count".to_string(), json!(pn + 1));
            }
        }
    }

    json!({
        "total_input_tokens": total_input,
        "total_output_tokens": total_output,
        "total_cost_usd": total_cost,
        "by_project": by_project,
    })
}
