//! Graph View Tauri commands: overview graph and per-project file graph.

use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

/// Get overview graph: all projects grouped by segments with shared resource edges.
#[tauri::command]
pub fn get_overview_graph(state: State<Arc<AppState>>) -> Value {
    let graph = super::graph_scan::build_overview_graph(&state);
    serde_json::to_value(&graph).unwrap_or_else(|e| json!({"error": format!("serialize: {}", e)}))
}

/// Get file-level graph for a specific project (async — heavy I/O).
#[tauri::command]
pub async fn get_project_graph(
    state: State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    crate::log_info!("[graph] scanning project: {}", project);
    tokio::task::spawn_blocking(move || {
        match super::graph_scan::build_project_graph(&state_arc, &project) {
            Ok(graph) => {
                crate::log_info!(
                    "[graph] {} done: {} nodes, {} edges",
                    project,
                    graph.stats.total_nodes,
                    graph.stats.total_edges
                );
                Ok(serde_json::to_value(&graph).unwrap_or_default())
            }
            Err(e) => {
                crate::log_warn!("[graph] {} error: {}", project, e);
                Err(e)
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Get overview graph with operations layer (delegations, strategies).
#[tauri::command]
pub fn get_overview_graph_ops(state: State<Arc<AppState>>) -> Value {
    let mut graph = super::graph_scan::build_overview_graph(&state);
    super::graph_ops::overlay_operations(&state, &mut graph);
    serde_json::to_value(&graph).unwrap_or_else(|e| json!({"error": format!("serialize: {}", e)}))
}

/// Export graph as Mermaid text
#[tauri::command]
pub fn export_graph_mermaid(state: State<Arc<AppState>>, project: String) -> Value {
    let graph = if project.is_empty() || project == "overview" {
        super::graph_scan::build_overview_graph(&state)
    } else {
        match super::graph_scan::build_project_graph(&state, &project) {
            Ok(g) => g,
            Err(e) => return json!({"error": e}),
        }
    };

    let mut lines = vec!["graph TD".to_string()];
    for node in &graph.nodes {
        let shape = match node.kind.as_str() {
            "segment" => format!("{}[/{}\\]", safe_id(&node.id), node.label),
            "external" => format!("{}{{{{{}}}}}", safe_id(&node.id), node.label),
            _ => format!("{}[{}]", safe_id(&node.id), node.label),
        };
        lines.push(format!("    {}", shape));
    }
    for edge in &graph.edges {
        let arrow = match edge.kind.as_str() {
            "import" => "-->",
            "contains" => "-.->",
            "shared" => "<-->",
            _ => "-->",
        };
        lines.push(format!(
            "    {} {} {}",
            safe_id(&edge.source),
            arrow,
            safe_id(&edge.target)
        ));
    }

    json!({"mermaid": lines.join("\n"), "nodes": graph.stats.total_nodes, "edges": graph.stats.total_edges})
}

/// Verify project: scan graph, return diagnostics (cycles, unresolved imports).
/// Used by PA command and can be called from CLI.
#[tauri::command]
pub fn verify_project(state: State<Arc<AppState>>, project: String) -> Value {
    let graph = match super::graph_scan::build_project_graph(&state, &project) {
        Ok(g) => g,
        Err(e) => return json!({"status": "error", "error": e}),
    };

    let mut diagnostics = Vec::new();

    // Check cycles
    for cycle in &graph.cycles {
        diagnostics.push(json!({
            "type": "circular_dependency",
            "severity": "warning",
            "nodes": cycle,
            "message": format!("Circular dependency: {}", cycle.join(" → ")),
        }));
    }

    // Check high instability modules
    for node in &graph.nodes {
        if node.metrics.instability > 0.8 && node.metrics.ca > 0 {
            diagnostics.push(json!({
                "type": "high_instability",
                "severity": "info",
                "nodes": [node.id],
                "message": format!("{} has instability {:.2} (many dependencies, few dependents)", node.label, node.metrics.instability),
            }));
        }
    }

    let has_errors = graph.cycles.len();
    json!({
        "status": if has_errors > 0 { "warnings" } else { "ok" },
        "project": project,
        "nodes": graph.stats.total_nodes,
        "edges": graph.stats.total_edges,
        "cycles": graph.stats.cycle_count,
        "diagnostics": diagnostics,
    })
}

/// Check if project files changed since last scan (mtime-based).
/// Returns list of changed files for incremental update.
#[tauri::command]
pub fn check_graph_changes(
    state: State<Arc<AppState>>,
    project: String,
    since_ts: String,
) -> Value {
    let project_dir = match state.validate_project(&project) {
        Ok(p) => p,
        Err(e) => return json!({"changed": false, "error": e}),
    };

    let since = chrono::DateTime::parse_from_rfc3339(&since_ts)
        .map(|dt| dt.timestamp() as u64)
        .unwrap_or(0);

    let excludes = [
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        "__pycache__",
    ];
    let extensions = ["rs", "ts", "tsx", "js", "jsx", "py"];
    let mut changed_files: Vec<String> = Vec::new();

    fn check_dir(
        dir: &std::path::Path,
        base: &std::path::Path,
        excludes: &[&str],
        exts: &[&str],
        since: u64,
        out: &mut Vec<String>,
        depth: u32,
    ) {
        if depth > 10 || out.len() > 50 {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if excludes.iter().any(|e| name == *e) {
                continue;
            }
            if path.is_dir() {
                check_dir(&path, base, excludes, exts, since, out, depth + 1);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.contains(&ext) {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        if let Ok(modified) = meta.modified() {
                            let mtime = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            if mtime > since {
                                let rel = path
                                    .strip_prefix(base)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .to_string();
                                out.push(rel);
                            }
                        }
                    }
                }
            }
        }
    }

    check_dir(
        &project_dir,
        &project_dir,
        &excludes,
        &extensions,
        since,
        &mut changed_files,
        0,
    );
    json!({"changed": !changed_files.is_empty(), "files": changed_files, "count": changed_files.len()})
}

/// Diff mode: compare current graph vs a previous snapshot.
/// Returns nodes/edges that were added, removed, or changed.
#[tauri::command]
pub fn graph_diff(state: State<Arc<AppState>>, project: String, previous_json: String) -> Value {
    let current = match super::graph_scan::build_project_graph(&state, &project) {
        Ok(g) => g,
        Err(e) => return json!({"error": e}),
    };

    let previous: super::graph_models::GraphData = match serde_json::from_str(&previous_json) {
        Ok(p) => p,
        Err(_) => return json!({"error": "Invalid previous graph JSON"}),
    };

    let prev_ids: std::collections::HashSet<&str> =
        previous.nodes.iter().map(|n| n.id.as_str()).collect();
    let curr_ids: std::collections::HashSet<&str> =
        current.nodes.iter().map(|n| n.id.as_str()).collect();

    let added: Vec<&str> = curr_ids.difference(&prev_ids).copied().collect();
    let removed: Vec<&str> = prev_ids.difference(&curr_ids).copied().collect();

    // Changed = same id but different edge count
    let mut changed: Vec<String> = Vec::new();
    for node in &current.nodes {
        if prev_ids.contains(node.id.as_str()) {
            let curr_edges = current
                .edges
                .iter()
                .filter(|e| e.source == node.id || e.target == node.id)
                .count();
            let prev_edges = previous
                .edges
                .iter()
                .filter(|e| e.source == node.id || e.target == node.id)
                .count();
            if curr_edges != prev_edges {
                changed.push(node.id.clone());
            }
        }
    }

    json!({
        "added": added, "removed": removed, "changed": changed,
        "added_count": added.len(), "removed_count": removed.len(), "changed_count": changed.len(),
        "current": current,
    })
}

/// Extract subgraph: only nodes within N hops of a seed node.
#[tauri::command]
pub fn get_subgraph(
    state: State<Arc<AppState>>,
    project: String,
    seed: String,
    hops: Option<u32>,
) -> Value {
    let graph = match super::graph_scan::build_project_graph(&state, &project) {
        Ok(g) => g,
        Err(e) => return json!({"error": e}),
    };

    let max_hops = hops.unwrap_or(2);
    let seed_lower = seed.to_lowercase();
    let seed_lookup = if seed_lower.starts_with("file:") {
        seed_lower.clone()
    } else {
        format!("file:{}", seed_lower)
    };
    let seed_node = graph
        .nodes
        .iter()
        .find(|n| n.id.to_lowercase() == seed_lookup || n.id.to_lowercase() == seed_lower)
        .or_else(|| {
            graph.nodes.iter().find(|n| {
                n.path
                    .as_deref()
                    .map(|p| p.eq_ignore_ascii_case(&seed))
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            let mut matches = graph
                .nodes
                .iter()
                .filter(|n| n.label.eq_ignore_ascii_case(&seed));
            let first = matches.next()?;
            if matches.next().is_some() {
                None
            } else {
                Some(first)
            }
        });

    let seed_id = match seed_node {
        Some(n) => n.id.clone(),
        None => return json!({"error": format!("Node '{}' not found", seed)}),
    };

    // BFS to collect nodes within hops
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((seed_id.clone(), 0u32));
    visited.insert(seed_id);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }
        for edge in &graph.edges {
            let neighbor = if edge.source == current {
                &edge.target
            } else if edge.target == current {
                &edge.source
            } else {
                continue;
            };
            if !visited.contains(neighbor.as_str()) {
                visited.insert(neighbor.clone());
                queue.push_back((neighbor.clone(), depth + 1));
            }
        }
    }

    let sub_nodes: Vec<&super::graph_models::GraphNode> = graph
        .nodes
        .iter()
        .filter(|n| visited.contains(n.id.as_str()))
        .collect();
    let sub_edges: Vec<&super::graph_models::GraphEdge> = graph
        .edges
        .iter()
        .filter(|e| visited.contains(e.source.as_str()) && visited.contains(e.target.as_str()))
        .collect();

    json!({
        "nodes": sub_nodes, "edges": sub_edges,
        "seed": seed, "hops": max_hops,
        "stats": {"total_nodes": sub_nodes.len(), "total_edges": sub_edges.len()},
    })
}

/// Get graph at a specific git commit (for timeline/diff).
/// Checks out files at commit, scans, returns graph, restores.
#[tauri::command]
pub fn get_graph_at_commit(state: State<Arc<AppState>>, project: String, commit: String) -> Value {
    let project_dir = match state.validate_project(&project) {
        Ok(p) => p,
        Err(e) => return json!({"error": e}),
    };

    // Get list of files at that commit (without checkout — safe, read-only)
    let output = super::claude_runner::silent_cmd("git")
        .args([
            "-C",
            &project_dir.to_string_lossy(),
            "ls-tree",
            "-r",
            "--name-only",
            &commit,
        ])
        .output();

    let files_at_commit: Vec<String> = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| {
                let ext = l.rsplit('.').next().unwrap_or("");
                ["rs", "ts", "tsx", "js", "jsx", "py", "gd"].contains(&ext)
            })
            .map(String::from)
            .collect(),
        _ => return json!({"error": "Could not read git tree at commit"}),
    };

    // Build a simple node list from the file tree (no import parsing — would need git show per file)
    let nodes: Vec<super::graph_models::GraphNode> = files_at_commit
        .iter()
        .enumerate()
        .map(|(i, f)| super::graph_models::GraphNode {
            id: format!("file:{}", f),
            label: f.rsplit('/').next().unwrap_or(f).to_string(),
            path: Some(f.clone()),
            kind: "file".to_string(),
            group: None,
            layer: 0,
            x: (i % 10) as f64 * 150.0,
            y: (i / 10) as f64 * 50.0,
            w: 130.0,
            h: 36.0,
            metrics: Default::default(),
        })
        .collect();

    json!({
        "commit": commit,
        "file_count": nodes.len(),
        "nodes": nodes,
        "edges": Vec::<String>::new(),
        "stats": {"total_nodes": nodes.len(), "total_edges": 0, "cycle_count": 0},
    })
}

fn safe_id(id: &str) -> String {
    id.replace(':', "_")
        .replace('/', "_")
        .replace('.', "_")
        .replace(' ', "_")
}
