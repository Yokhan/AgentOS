//! Graph scanning: overview (projects+segments) and per-project (files+imports).

use super::graph_collect::*;
use super::graph_models::*;
use crate::state::AppState;
use std::collections::{HashMap, HashSet};

/// Build overview graph: projects as nodes, segments as groups, shared resources as edges.
pub fn build_overview_graph(state: &AppState) -> GraphData {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    let categories = super::category::load_categories(&state.root);

    for (seg_name, projects) in segments.iter() {
        nodes.push(GraphNode {
            id: format!("seg:{}", seg_name),
            label: seg_name.clone(),
            kind: "segment".to_string(),
            path: None,
            group: None,
            layer: 0,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            metrics: NodeMetrics {
                file_count: Some(projects.len() as u32),
                ..Default::default()
            },
        });
        for proj in projects {
            let proj_id = format!("proj:{}", proj);
            if !nodes.iter().any(|n| n.id == proj_id) {
                nodes.push(GraphNode {
                    id: proj_id.clone(),
                    label: proj.clone(),
                    kind: "project".to_string(),
                    path: None,
                    group: Some(seg_name.clone()),
                    layer: 0,
                    x: 0.0,
                    y: 0.0,
                    w: 0.0,
                    h: 0.0,
                    metrics: Default::default(),
                });
            }
            edges.push(GraphEdge {
                source: format!("seg:{}", seg_name),
                target: proj_id,
                kind: "contains".to_string(),
                weight: 1,
            });
        }
    }

    // Fallback: if segments empty, use scan_cache
    if nodes.is_empty() {
        if let Ok(cache) = state.scan_cache.lock() {
            if let Some(data) = &cache.data {
                if let Some(arr) = data
                    .get("agents")
                    .and_then(|a| a.as_array())
                    .or_else(|| data.as_array())
                {
                    for v in arr {
                        if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                            nodes.push(GraphNode {
                                id: format!("proj:{}", name),
                                label: name.to_string(),
                                path: None,
                                kind: "project".to_string(),
                                group: Some("unsegmented".to_string()),
                                layer: 0,
                                x: 0.0,
                                y: 0.0,
                                w: 0.0,
                                h: 0.0,
                                metrics: Default::default(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Shared resource edges — intra-group (within same category)
    for (cat_name, meta) in &categories {
        if meta.shared_resources.is_empty() {
            continue;
        }
        if let Some(projects) = segments.get(cat_name) {
            for (i, p1) in projects.iter().enumerate() {
                for p2 in projects.iter().skip(i + 1) {
                    edges.push(GraphEdge {
                        source: format!("proj:{}", p1),
                        target: format!("proj:{}", p2),
                        kind: "shared".to_string(),
                        weight: meta.shared_resources.len() as u32,
                    });
                }
            }
        }
    }
    // Inter-group edges: if a shared_resource name matches a project, link them
    let all_proj_names: HashSet<String> = nodes
        .iter()
        .filter(|n| n.kind == "project")
        .map(|n| n.label.clone())
        .collect();
    for (cat_name, meta) in &categories {
        for res in &meta.shared_resources {
            let res_name = res.trim_end_matches('/').rsplit('/').next().unwrap_or(res);
            if let Some(target_proj) = all_proj_names
                .iter()
                .find(|p| p.eq_ignore_ascii_case(res_name))
            {
                if let Some(projects) = segments.get(cat_name) {
                    for proj in projects {
                        if proj != target_proj {
                            let src = format!("proj:{}", proj);
                            let tgt = format!("proj:{}", target_proj);
                            if !edges
                                .iter()
                                .any(|e| e.source == src && e.target == tgt && e.kind == "shared")
                            {
                                edges.push(GraphEdge {
                                    source: src,
                                    target: tgt,
                                    kind: "shared".to_string(),
                                    weight: 1,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Overview: no isolated filtering, compute metrics, no layout (JS does it)
    let cycles = tarjan_scc(&nodes, &edges);
    compute_metrics(&mut nodes, &edges, &cycles);
    let mut groups = Vec::new();
    compute_overview_layout(&mut nodes, &mut groups);
    let stats = GraphStats {
        total_nodes: nodes.len(),
        total_edges: edges.len(),
        cycle_count: cycles.len(),
    };
    GraphData {
        nodes,
        edges,
        cycles,
        stats,
        groups,
    }
}

/// Build file-level graph for a single project using regex import parsing.
pub fn build_project_graph(state: &AppState, project: &str) -> Result<GraphData, String> {
    let project_dir = state.validate_project(project)?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut file_map: HashMap<String, String> = HashMap::new();
    let mut label_counts: HashMap<String, u32> = HashMap::new();

    let files = collect_files(&project_dir);
    crate::log_info!("[graph] {} collected {} files", project, files.len());
    for (rel_path, _) in &files {
        *label_counts.entry(file_label(rel_path)).or_default() += 1;
    }

    // Create file nodes
    for (rel_path, abs_path) in &files {
        let id = format!("file:{}", rel_path);
        let loc = std::fs::read_to_string(abs_path)
            .map(|c| c.lines().count() as u32)
            .unwrap_or(0);
        let layer = detect_layer(rel_path);
        nodes.push(GraphNode {
            id: id.clone(),
            label: display_label(rel_path, &label_counts),
            path: Some(rel_path.clone()),
            kind: "file".to_string(),
            group: Some(layer_name(layer)),
            layer,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            metrics: NodeMetrics {
                loc: Some(loc),
                ..Default::default()
            },
        });
        file_map.insert(rel_path.clone(), id);
    }

    // Parse imports
    let re_js = regex::Regex::new(
        r#"(?:import\s.*?from\s+['"]([^'"]+)['"]|require\(\s*['"]([^'"]+)['"]\s*\))"#,
    )
    .unwrap();
    let re_rs = regex::Regex::new(r"(?:use\s+(crate::\w[\w:]*)|mod\s+(\w+))").unwrap();
    let re_py = regex::Regex::new(r"(?:from\s+(\S+)\s+import|import\s+(\S+))").unwrap();
    let re_gd = regex::Regex::new(r#"(?:preload|load)\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();

    for (rel_path, abs_path) in &files {
        let content = match std::fs::read_to_string(abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let source_id = match file_map.get(rel_path) {
            Some(id) => id.clone(),
            None => continue,
        };
        let ext = rel_path.rsplit('.').next().unwrap_or("");
        let re = match ext {
            "ts" | "tsx" | "js" | "jsx" => &re_js,
            "rs" => &re_rs,
            "py" => &re_py,
            "gd" => &re_gd,
            _ => continue,
        };

        for caps in re.captures_iter(&content) {
            let import_path = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            if import_path.is_empty() {
                continue;
            }
            if let Some(tid) = resolve_import(rel_path, &import_path, &file_map, ext) {
                if !edges
                    .iter()
                    .any(|e: &GraphEdge| e.source == source_id && e.target == tid)
                {
                    edges.push(GraphEdge {
                        source: source_id.clone(),
                        target: tid,
                        kind: "import".to_string(),
                        weight: 1,
                    });
                }
            }
        }
    }

    // MODULE.json enrichment + external services
    for (rel_path, abs_path) in &files {
        let module_json = abs_path.parent().unwrap_or(abs_path).join("MODULE.json");
        if module_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&module_json) {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(fid) = file_map.get(rel_path) {
                        if let Some(node) = nodes.iter_mut().find(|n| n.id == *fid) {
                            if let Some(layer) = meta.get("layer").and_then(|v| v.as_str()) {
                                node.group = Some(layer.to_string());
                            }
                        }
                    }
                    if let Some(services) = meta.get("external_services").and_then(|v| v.as_array())
                    {
                        for svc in services.iter().filter_map(|v| v.as_str()) {
                            let svc_id = format!("ext:{}", svc);
                            if !nodes.iter().any(|n| n.id == svc_id) {
                                nodes.push(GraphNode {
                                    id: svc_id.clone(),
                                    label: format!("☁ {}", svc),
                                    path: None,
                                    kind: "external".to_string(),
                                    group: Some("external".to_string()),
                                    layer: 99,
                                    x: 0.0,
                                    y: 0.0,
                                    w: 130.0,
                                    h: 28.0,
                                    metrics: Default::default(),
                                });
                            }
                            if let Some(fid) = file_map.get(rel_path) {
                                edges.push(GraphEdge {
                                    source: fid.clone(),
                                    target: svc_id,
                                    kind: "import".to_string(),
                                    weight: 1,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Filter isolated nodes (0 connections) — keep only connected subgraph
    let connected: HashSet<&str> = edges
        .iter()
        .flat_map(|e| [e.source.as_str(), e.target.as_str()])
        .collect();
    let total_before = nodes.len();
    nodes.retain(|n| connected.contains(n.id.as_str()));
    let filtered = total_before - nodes.len();
    if filtered > 0 {
        crate::log_info!(
            "[graph] {} filtered {} isolated nodes, {} remaining",
            project,
            filtered,
            nodes.len()
        );
    }

    let cycles = tarjan_scc(&nodes, &edges);
    compute_metrics(&mut nodes, &edges, &cycles);
    // Grid layout: files grouped by architectural layer (ui/logic/data/infra/other)
    let stats = GraphStats {
        total_nodes: nodes.len(),
        total_edges: edges.len(),
        cycle_count: cycles.len(),
    };
    let mut groups = Vec::new();
    compute_overview_layout(&mut nodes, &mut groups);
    Ok(GraphData {
        nodes,
        edges,
        cycles,
        stats,
        groups,
    })
}
