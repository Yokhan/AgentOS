//! Graph operations: operations overlay and agent protocol context.
//! Extracted from graph_scan.rs for file size.

use super::graph_models::*;
use crate::state::AppState;

/// Overlay operations layer: active delegations, strategies on graph.
pub fn overlay_operations(state: &AppState, graph: &mut GraphData) {
    let max_x = graph
        .nodes
        .iter()
        .map(|n| n.x + n.w)
        .fold(0.0_f64, f64::max);
    let ops_x = max_x + 60.0;
    let mut y_offset = 0.0_f64;

    if let Ok(delegations) = state.delegations.lock() {
        for d in delegations.values().filter(|d| !d.status.is_terminal()) {
            let id = format!("deleg:{}", d.id.chars().take(12).collect::<String>());
            graph.nodes.push(GraphNode {
                id: id.clone(),
                label: format!("⚡{}", d.project),
                kind: "delegation".to_string(),
                path: None,
                group: Some("operations".to_string()),
                layer: 99,
                x: ops_x,
                y: y_offset,
                w: 130.0,
                h: 28.0,
                metrics: Default::default(),
            });
            let proj_id = format!("proj:{}", d.project);
            if graph.nodes.iter().any(|n| n.id == proj_id) {
                graph.edges.push(GraphEdge {
                    source: id,
                    target: proj_id,
                    kind: "works_on".to_string(),
                    weight: 1,
                });
            }
            y_offset += 36.0;
        }
    }

    let strategies = super::strategy_models::load_strategies(state);
    for s in strategies.iter().filter(|s| s.status.is_active()) {
        let id = format!("strat:{}", s.id.chars().take(12).collect::<String>());
        let total: usize = s.plans.iter().map(|p| p.steps.len()).sum();
        let done = s
            .plans
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|st| st.status == crate::commands::status::StepStatus::Done)
            .count();
        graph.nodes.push(GraphNode {
            id: id.clone(),
            label: format!(
                "📋{} {}/{}",
                s.title.chars().take(10).collect::<String>(),
                done,
                total
            ),
            path: None,
            kind: "strategy".to_string(),
            group: Some("operations".to_string()),
            layer: 99,
            x: ops_x + 150.0,
            y: y_offset,
            w: 130.0,
            h: 28.0,
            metrics: Default::default(),
        });
        for plan in &s.plans {
            let proj_id = format!("proj:{}", plan.project);
            if graph.nodes.iter().any(|n| n.id == proj_id) {
                graph.edges.push(GraphEdge {
                    source: id.clone(),
                    target: proj_id,
                    kind: "works_on".to_string(),
                    weight: 1,
                });
            }
        }
        y_offset += 36.0;
    }

    graph.stats.total_nodes = graph.nodes.len();
    graph.stats.total_edges = graph.edges.len();
}

/// Build compact graph context for agent delegation prompt.
pub fn build_graph_context(state: &AppState, project: &str) -> String {
    let graph = match super::graph_scan::build_project_graph(state, project) {
        Ok(g) => g,
        Err(e) => {
            crate::log_warn!("[graph] build_graph_context failed for {}: {}", project, e);
            return String::new();
        }
    };
    if graph.nodes.is_empty() {
        return String::new();
    }

    let mut lines = vec![format!(
        "=== PROJECT GRAPH: {} ({} modules, {} deps) ===",
        project, graph.stats.total_nodes, graph.stats.total_edges
    )];

    let mut sorted: Vec<&GraphNode> = graph.nodes.iter().filter(|n| n.kind == "file").collect();
    sorted.sort_by(|a, b| (b.metrics.ca + b.metrics.ce).cmp(&(a.metrics.ca + a.metrics.ce)));

    for node in sorted.iter().take(10) {
        let deps_out: Vec<String> = graph
            .edges
            .iter()
            .filter(|e| e.source == node.id && e.kind == "import")
            .filter_map(|e| {
                graph
                    .nodes
                    .iter()
                    .find(|n| n.id == e.target)
                    .map(|n| n.label.clone())
            })
            .collect();
        let deps_in: Vec<String> = graph
            .edges
            .iter()
            .filter(|e| e.target == node.id && e.kind == "import")
            .filter_map(|e| {
                graph
                    .nodes
                    .iter()
                    .find(|n| n.id == e.source)
                    .map(|n| n.label.clone())
            })
            .collect();

        lines.push(format!(
            "\n[{}] ({}, Ca:{} Ce:{})",
            node.label,
            node.group.as_deref().unwrap_or("?"),
            node.metrics.ca,
            node.metrics.ce
        ));
        if !deps_out.is_empty() {
            lines.push(format!("  → depends on: {}", deps_out.join(", ")));
        }
        if !deps_in.is_empty() {
            lines.push(format!("  ← depended by: {}", deps_in.join(", ")));
        }
    }

    if !graph.cycles.is_empty() {
        lines.push("\n⚠ CYCLES:".to_string());
        for cycle in &graph.cycles {
            lines.push(format!("  {}", cycle.join(" → ")));
        }
    }

    lines.push("=== END GRAPH ===".to_string());
    let result = lines.join("\n");
    if result.len() > 4000 {
        result.chars().take(4000).collect::<String>() + "\n... (truncated)"
    } else {
        result
    }
}
