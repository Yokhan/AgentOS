//! Graph data types, layout algorithm, cycle detection, coupling metrics.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub kind: String,
    pub group: Option<String>,
    pub layer: u32,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub metrics: NodeMetrics,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NodeMetrics {
    pub ca: u32,
    pub ce: u32,
    pub instability: f64,
    pub in_cycle: bool,
    pub file_count: Option<u32>,
    pub loc: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub weight: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GroupRect {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub cycles: Vec<Vec<String>>,
    pub stats: GraphStats,
    #[serde(default)]
    pub groups: Vec<GroupRect>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub cycle_count: usize,
}

/// Deterministic layout: groups in 3-column grid, nodes listed vertically inside groups.
pub fn compute_overview_layout(nodes: &mut [GraphNode], groups_out: &mut Vec<GroupRect>) {
    const NODE_W: f64 = 180.0;
    const NODE_H: f64 = 28.0;
    const ROW_GAP: f64 = 4.0;
    const PAD: f64 = 16.0;
    const HEADER_H: f64 = 28.0;
    const COL_GAP: f64 = 48.0;
    const ROW_GAP_OUTER: f64 = 48.0;
    const COLS: usize = 3;
    let group_w = NODE_W + PAD * 2.0;

    // Group nodes by group name (alphabetical for determinism)
    let mut group_names: Vec<String> = Vec::new();
    let mut group_members: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.kind == "segment" {
            continue;
        }
        let g = node.group.clone().unwrap_or_else(|| "other".to_string());
        if !group_members.contains_key(&g) {
            group_names.push(g.clone());
        }
        group_members.entry(g).or_default().push(i);
    }
    group_names.sort();

    // Layout groups in COLS-column grid
    let mut col_bottom = vec![0.0_f64; COLS];
    for (gi, gname) in group_names.iter().enumerate() {
        let members = match group_members.get(gname) {
            Some(m) => m,
            None => continue,
        };
        let col = gi % COLS;
        let col_x = col as f64 * (group_w + COL_GAP);
        let gy = col_bottom[col];
        let group_h = HEADER_H + PAD + members.len() as f64 * (NODE_H + ROW_GAP) + PAD;

        groups_out.push(GroupRect {
            name: gname.clone(),
            x: col_x,
            y: gy,
            w: group_w,
            h: group_h,
        });

        for (mi, &ni) in members.iter().enumerate() {
            nodes[ni].x = col_x + PAD;
            nodes[ni].y = gy + HEADER_H + PAD + mi as f64 * (NODE_H + ROW_GAP);
            nodes[ni].w = NODE_W;
            nodes[ni].h = NODE_H;
        }
        col_bottom[col] = gy + group_h + ROW_GAP_OUTER;
    }
}

/// Tarjan's SCC — returns strongly connected components with >1 node (cycles)
pub fn tarjan_scc(nodes: &[GraphNode], edges: &[GraphEdge]) -> Vec<Vec<String>> {
    let n = nodes.len();
    let idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for e in edges {
        if let (Some(&si), Some(&ti)) = (idx.get(e.source.as_str()), idx.get(e.target.as_str())) {
            adj[si].push(ti);
        }
    }

    let mut index = 0u32;
    let mut stack: Vec<usize> = Vec::new();
    let mut on_stack = vec![false; n];
    let mut indices = vec![u32::MAX; n];
    let mut lowlinks = vec![0u32; n];
    let mut sccs: Vec<Vec<String>> = Vec::new();

    fn strongconnect(
        v: usize,
        adj: &[Vec<usize>],
        index: &mut u32,
        stack: &mut Vec<usize>,
        on_stack: &mut [bool],
        indices: &mut [u32],
        lowlinks: &mut [u32],
        sccs: &mut Vec<Vec<String>>,
        nodes: &[GraphNode],
    ) {
        indices[v] = *index;
        lowlinks[v] = *index;
        *index += 1;
        stack.push(v);
        on_stack[v] = true;

        for &w in &adj[v] {
            if indices[w] == u32::MAX {
                strongconnect(
                    w, adj, index, stack, on_stack, indices, lowlinks, sccs, nodes,
                );
                lowlinks[v] = lowlinks[v].min(lowlinks[w]);
            } else if on_stack[w] {
                lowlinks[v] = lowlinks[v].min(indices[w]);
            }
        }

        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack[w] = false;
                scc.push(nodes[w].id.clone());
                if w == v {
                    break;
                }
            }
            if scc.len() > 1 {
                sccs.push(scc);
            }
        }
    }

    for i in 0..n {
        if indices[i] == u32::MAX {
            strongconnect(
                i,
                &adj,
                &mut index,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlinks,
                &mut sccs,
                nodes,
            );
        }
    }
    sccs
}

/// Compute coupling metrics: Ca (afferent), Ce (efferent), Instability
pub fn compute_metrics(nodes: &mut [GraphNode], edges: &[GraphEdge], cycles: &[Vec<String>]) {
    let cycle_nodes: HashSet<&str> = cycles
        .iter()
        .flat_map(|c| c.iter().map(|s| s.as_str()))
        .collect();

    for node in nodes.iter_mut() {
        let ca = edges
            .iter()
            .filter(|e| e.target == node.id && e.kind == "import")
            .count() as u32;
        let ce = edges
            .iter()
            .filter(|e| e.source == node.id && e.kind == "import")
            .count() as u32;
        node.metrics.ca = ca;
        node.metrics.ce = ce;
        node.metrics.instability = if ca + ce > 0 {
            ce as f64 / (ca + ce) as f64
        } else {
            0.0
        };
        node.metrics.in_cycle = cycle_nodes.contains(node.id.as_str());
    }
}
