//! Task-scoped code context bundles for orchestrator and project agents.
//!
//! The graph subsystem answers "what depends on what"; this module packages
//! that into a bounded, prompt-safe contract that agents can use directly.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

pub const CODE_CONTEXT_SCHEMA: &str = "agentos.code_context.v1";
const DEFAULT_MAX_CHARS: usize = 12_000;
const MIN_MAX_CHARS: usize = 2_000;
const HARD_MAX_CHARS: usize = 50_000;
const MAX_PROJECTS: usize = 8;
const MAX_REQUESTED_FILES: usize = 12;
const PER_FILE_CHAR_LIMIT: usize = 3_500;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeContextRequest {
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub focus: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub max_chars: Option<usize>,
    #[serde(default)]
    pub include_files: Option<bool>,
}

#[tauri::command]
pub async fn get_code_context_bundle(
    state: State<'_, Arc<AppState>>,
    request: CodeContextRequest,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || build_code_context_bundle_core(&state_arc, &request))
        .await
        .map_err(|e| e.to_string())?
}

pub fn build_task_code_context(
    state: &AppState,
    project: &str,
    task: &str,
    max_chars: usize,
) -> String {
    let request = CodeContextRequest {
        project: Some(project.to_string()),
        focus: Some(task_focus(task)),
        max_chars: Some(max_chars),
        include_files: Some(false),
        ..Default::default()
    };
    match build_code_context_bundle_core(state, &request) {
        Ok(value) => value
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        Err(e) => {
            crate::log_warn!("[code-context] task context failed for {}: {}", project, e);
            String::new()
        }
    }
}

pub fn build_code_context_bundle_core(
    state: &AppState,
    request: &CodeContextRequest,
) -> Result<Value, String> {
    let max_chars = request
        .max_chars
        .unwrap_or(DEFAULT_MAX_CHARS)
        .clamp(MIN_MAX_CHARS, HARD_MAX_CHARS);
    let projects = normalize_projects(state, request)?;
    if projects.is_empty() {
        return Err("project is required".to_string());
    }

    let focus = request.focus.as_deref().unwrap_or("").trim();
    let include_files = request.include_files.unwrap_or(!request.files.is_empty());
    let mut warnings = Vec::new();
    let mut body = Vec::new();
    body.push("[CODE CONTEXT BUNDLE]".to_string());
    body.push(format!("schema: {}", CODE_CONTEXT_SCHEMA));
    body.push(format!("projects: {}", projects.join(", ")));
    if !focus.is_empty() {
        body.push(format!("focus: {}", single_line(focus, 280)));
    }
    body.push(
        "contract: use this for architecture, dependency, impact, and shared-feature work."
            .to_string(),
    );
    body.push("if more context is needed: ask the orchestrator for [CODE_CONTEXT:Project]focus[/CODE_CONTEXT], [GRAPH_IMPACT:Project:file], or [GRAPH_DEPENDENTS:Project:file].".to_string());

    for project in &projects {
        match build_project_context_section(state, project, request, include_files) {
            Ok(section) => body.push(section),
            Err(e) => {
                warnings.push(format!("{}: {}", project, e));
                body.push(format!(
                    "\n[PROJECT: {}]\nstatus: unavailable ({})",
                    project, e
                ));
            }
        }
    }

    body.push("[END CODE CONTEXT BUNDLE]".to_string());
    let (context, truncated) = truncate_chars(&body.join("\n"), max_chars);
    Ok(json!({
        "status": "ok",
        "schema": CODE_CONTEXT_SCHEMA,
        "projects": projects,
        "focus": focus,
        "max_chars": max_chars,
        "truncated": truncated,
        "warnings": warnings,
        "context": context,
    }))
}

fn normalize_projects(
    state: &AppState,
    request: &CodeContextRequest,
) -> Result<Vec<String>, String> {
    let mut raw = Vec::new();
    if let Some(project) = request.project.as_deref() {
        if !project.trim().is_empty() {
            raw.push(project.trim().to_string());
        }
    }
    raw.extend(request.projects.iter().map(|p| p.trim().to_string()));

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for project in raw.into_iter().filter(|p| !p.is_empty()) {
        let normalized = state
            .validate_project_name_from_llm(&project)
            .or_else(|| {
                state
                    .validate_project(&project)
                    .ok()
                    .map(|_| project.clone())
            })
            .ok_or_else(|| format!("unknown project: {}", project))?;
        let key = normalized.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(normalized);
        }
        if out.len() >= MAX_PROJECTS {
            break;
        }
    }
    Ok(out)
}

fn build_project_context_section(
    state: &AppState,
    project: &str,
    request: &CodeContextRequest,
    include_files: bool,
) -> Result<String, String> {
    let project_dir = state.validate_project(project)?;
    let graph = super::graph_scan::build_project_graph(state, project)?;
    let focus = request.focus.as_deref().unwrap_or("");
    let focus_terms = focus_terms(focus);

    let mut lines = vec![
        String::new(),
        format!("[PROJECT: {}]", project),
        format!(
            "graph: {} connected files, {} imports, {} cycles",
            graph.stats.total_nodes, graph.stats.total_edges, graph.stats.cycle_count
        ),
        format!("root: {}", project_dir.display()),
    ];

    let mut files: Vec<_> = graph.nodes.iter().filter(|n| n.kind == "file").collect();
    files.sort_by(|a, b| (b.metrics.ca + b.metrics.ce).cmp(&(a.metrics.ca + a.metrics.ce)));
    if files.is_empty() {
        let fallback = super::graph_collect::collect_files(&project_dir);
        lines.push(format!(
            "file_index: {} scannable files, no connected import graph yet",
            fallback.len()
        ));
        for (rel, _) in fallback.iter().take(16) {
            lines.push(format!("  file: {}", rel));
        }
    } else {
        lines.push("hotspots:".to_string());
        for node in files.iter().take(12) {
            lines.push(format!(
                "  - {} [{}] Ca:{} Ce:{} LOC:{}",
                node.path.as_deref().unwrap_or(&node.label),
                node.group.as_deref().unwrap_or("?"),
                node.metrics.ca,
                node.metrics.ce,
                node.metrics.loc.unwrap_or(0)
            ));
        }
    }

    if !focus_terms.is_empty() {
        let mut matches = Vec::new();
        for node in graph.nodes.iter().filter(|n| n.kind == "file") {
            let hay = format!("{} {}", node.path.as_deref().unwrap_or(""), node.label)
                .to_ascii_lowercase();
            if focus_terms.iter().any(|term| hay.contains(term)) {
                matches.push(node.path.as_deref().unwrap_or(&node.label).to_string());
            }
            if matches.len() >= 12 {
                break;
            }
        }
        if !matches.is_empty() {
            lines.push("focus_matches:".to_string());
            for item in matches {
                lines.push(format!("  - {}", item));
            }
        }
    }

    if !graph.cycles.is_empty() {
        lines.push("cycles:".to_string());
        for cycle in graph.cycles.iter().take(5) {
            lines.push(format!("  - {}", cycle.join(" -> ")));
        }
    }

    if include_files && !request.files.is_empty() {
        lines.push("requested_files:".to_string());
        for file in request.files.iter().take(MAX_REQUESTED_FILES) {
            match read_project_file_snippet(&project_dir, file) {
                Ok(snippet) => lines.push(snippet),
                Err(e) => lines.push(format!("  - {}: {}", file, e)),
            }
        }
    }

    Ok(lines.join("\n"))
}

fn read_project_file_snippet(project_dir: &Path, file: &str) -> Result<String, String> {
    let rel = normalize_rel_file(file)?;
    let path = project_dir.join(&rel);
    let canon_root = project_dir.canonicalize().map_err(|e| e.to_string())?;
    let canon_path = path.canonicalize().map_err(|e| e.to_string())?;
    if !canon_path.starts_with(&canon_root) {
        return Err("file escapes project root".to_string());
    }
    let content = std::fs::read_to_string(&canon_path).map_err(|e| e.to_string())?;
    if content.contains('\0') {
        return Err("binary file skipped".to_string());
    }
    let (snippet, truncated) = truncate_chars(&content, PER_FILE_CHAR_LIMIT);
    Ok(format!(
        "\n--- file: {}{} ---\n{}",
        rel.to_string_lossy().replace('\\', "/"),
        if truncated { " (truncated)" } else { "" },
        snippet
    ))
}

fn normalize_rel_file(file: &str) -> Result<PathBuf, String> {
    let trimmed = file.trim().trim_start_matches("file:");
    if trimmed.is_empty()
        || trimmed.contains('\0')
        || trimmed.contains(':')
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
    {
        return Err("invalid relative file path".to_string());
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("invalid relative file path".to_string());
    }
    Ok(path)
}

fn focus_terms(focus: &str) -> Vec<String> {
    focus
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| s.len() >= 3)
        .take(12)
        .collect()
}

fn task_focus(task: &str) -> String {
    let mut focus = single_line(task, 500);
    if focus.is_empty() {
        focus = "implementation context".to_string();
    }
    focus
}

fn single_line(value: &str, max_chars: usize) -> String {
    let clean = value.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(&clean, max_chars).0
}

fn truncate_chars(value: &str, max_chars: usize) -> (String, bool) {
    if value.chars().count() <= max_chars {
        return (value.to_string(), false);
    }
    let mut out: String = value.chars().take(max_chars).collect();
    out.push_str("\n... (truncated)");
    (out, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_state(name: &str) -> (AppState, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("agentos-code-context-{name}-{nonce}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).expect("n8n");
        std::fs::create_dir_all(root.join("Demo").join("src")).expect("demo src");
        std::fs::write(
            root.join("n8n").join("config.json"),
            json!({"documents_dir": root.to_string_lossy()}).to_string(),
        )
        .expect("config");
        std::fs::write(
            root.join("Demo").join("src").join("main.ts"),
            "import { login } from './auth';\nlogin();\n",
        )
        .expect("main");
        std::fs::write(
            root.join("Demo").join("src").join("auth.ts"),
            "export function login() { return true; }\n",
        )
        .expect("auth");
        (AppState::new(root.clone()), root)
    }

    #[test]
    fn bundle_includes_graph_and_protocol() {
        let (state, root) = test_state("bundle");
        let value = build_code_context_bundle_core(
            &state,
            &CodeContextRequest {
                project: Some("Demo".to_string()),
                focus: Some("shared auth login".to_string()),
                max_chars: Some(8000),
                ..Default::default()
            },
        )
        .expect("bundle");
        let context = value["context"].as_str().unwrap_or("");
        assert!(context.contains(CODE_CONTEXT_SCHEMA));
        assert!(context.contains("[PROJECT: Demo]"));
        assert!(context.contains("src/auth.ts"));
        assert!(context.contains("[GRAPH_IMPACT:Project:file]"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn requested_file_snippets_are_root_safe() {
        let (state, root) = test_state("files");
        let ok = build_code_context_bundle_core(
            &state,
            &CodeContextRequest {
                project: Some("Demo".to_string()),
                files: vec!["src/auth.ts".to_string()],
                include_files: Some(true),
                ..Default::default()
            },
        )
        .expect("bundle");
        assert!(ok["context"]
            .as_str()
            .unwrap_or("")
            .contains("export function login"));

        let err = build_code_context_bundle_core(
            &state,
            &CodeContextRequest {
                project: Some("Demo".to_string()),
                files: vec!["../n8n/config.json".to_string()],
                include_files: Some(true),
                ..Default::default()
            },
        )
        .expect("bundle with warning");
        assert!(err["context"]
            .as_str()
            .unwrap_or("")
            .contains("invalid relative file path"));
        let _ = std::fs::remove_dir_all(root);
    }
}
