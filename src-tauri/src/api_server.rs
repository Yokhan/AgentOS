//! Lightweight HTTP API server running alongside Tauri.
//! Provides REST endpoints for browser mode, testing, orchestrator, and n8n.
//! Shares AppState with Tauri commands via Arc.

use crate::state::AppState;
use axum::{
    extract::State as AxState,
    extract::{Json, Path, Query, Request},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Auth middleware — checks Bearer token on all non-health endpoints
async fn auth_check(AxState(state): AxState<Arc<AppState>>, req: Request, next: Next) -> Response {
    // Allow health endpoint without auth
    if req.uri().path() == "/api/health" {
        return next.run(req).await;
    }
    let token_ok = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v) == state.api_token)
        .unwrap_or(false);
    if token_ok {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized. Use Bearer token from get_api_token command."})),
        )
            .into_response()
    }
}

pub async fn start(state: Arc<AppState>, port: u16) {
    let token_for_log = state.api_token.clone();
    let app = Router::new()
        // Project scanning
        .route("/api/agents", get(get_agents))
        .route("/api/segments", get(get_segments))
        // Chat
        .route("/api/chat/{project}", get(get_chat_history))
        .route("/api/chat", post(send_chat))
        .route("/api/chats", get(get_chats))
        // Feed & status
        .route("/api/feed", get(get_feed))
        .route("/api/activity", get(get_activity))
        .route("/api/plan", get(get_plan))
        // Config
        .route("/api/config", get(get_config))
        .route("/api/config", post(set_config))
        .route("/api/permissions", get(get_permissions))
        // Health
        .route("/api/health", get(health))
        // Delegations
        .route("/api/delegations", get(get_delegations))
        .route("/api/delegation/approve", post(approve_delegation))
        .route("/api/delegation/reject", post(reject_delegation))
        // Graph API (for MCP / Agent Protocol)
        .route("/api/graph/overview", get(graph_overview))
        .route("/api/graph/project/{project}", get(graph_project))
        .route("/api/graph/context/{project}", get(graph_context))
        .route(
            "/api/graph/dependents/{project}/{file}",
            get(graph_dependents),
        )
        .route("/api/graph/dependents", post(graph_dependents_post))
        .route("/api/graph/impact/{project}/{file}", get(graph_impact))
        .route("/api/graph/impact", post(graph_impact_post))
        .route("/api/graph/verify/{project}", get(graph_verify))
        .layer(axum::extract::DefaultBodyLimit::max(1_048_576)) // 1MB max request body
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_check))
        .with_state(state);

    // Try port, fallback to port+1, port+2
    for p in [port, port + 1, port + 2] {
        let addr = format!("127.0.0.1:{}", p);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                crate::log_info!(
                    "HTTP API: http://{} (token: {}...)",
                    addr,
                    &token_for_log[..8.min(token_for_log.len())]
                );
                if let Err(e) = axum::serve(listener, app).await {
                    crate::log_error!("API server crashed: {}", e);
                }
                return;
            }
            Err(e) => {
                eprintln!("Port {} busy ({}), trying next...", p, e);
            }
        }
    }
    eprintln!("WARNING: HTTP API server could not start — all ports busy");
}

async fn health(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    Json(crate::commands::feed::health_snapshot(&state))
}

async fn get_agents(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    Json(crate::commands::agents::get_agents_cached(&state))
}

async fn get_segments(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    let segs = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    Json(json!({
        "segments": *segs,
        "project_segment": *state.project_segment.lock().unwrap_or_else(|e| e.into_inner()),
    }))
}

async fn get_chats(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    Json(crate::commands::chat_core::get_chats_core(&state))
}

#[derive(Deserialize)]
struct ChatHistoryQuery {
    before: Option<usize>,
    limit: Option<usize>,
}

async fn get_chat_history(
    AxState(state): AxState<Arc<AppState>>,
    Path(project): Path<String>,
    Query(query): Query<ChatHistoryQuery>,
) -> impl IntoResponse {
    Json(crate::commands::chat_core::get_chat_history_page_core(
        &state,
        &project,
        query.before,
        query.limit,
    ))
}

async fn send_chat(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let message = body
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let project = body
        .get("project")
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .to_string();

    if message.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Empty message"})),
        );
    }

    let (cwd, chat_key, chat_file) =
        match crate::commands::chat_core::resolve_chat_context(&state, &project) {
            Ok(ctx) => ctx,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e}))),
        };
    let _ = crate::commands::chat_core::prepare_chat(
        &state,
        &chat_key,
        &chat_file,
        &message,
        project.is_empty(),
    );

    let perm_path = crate::commands::claude_runner::get_permission_path(&state, &chat_key);
    let cwd_owned = cwd.to_path_buf();
    let msg_owned = message.clone();
    let response = tokio::task::spawn_blocking(move || {
        crate::commands::claude_runner::run_claude(&cwd_owned, &msg_owned, &perm_path)
    })
    .await
    .unwrap_or_else(|e| format!("Task failed: {}", e));

    let ts2 = state.now_iso();
    let asst_entry = json!({"ts": ts2, "role": "assistant", "msg": response});
    crate::commands::jsonl::append_jsonl_logged(&chat_file, &asst_entry, "api asst response");

    (
        StatusCode::OK,
        Json(json!({"status": "complete", "response": response})),
    )
}

async fn get_feed(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    let feed_path = state.root.join("tasks").join(".chat-history.jsonl");
    let mut items = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&feed_path) {
        for line in content.lines().rev().take(20) {
            if let Ok(item) = serde_json::from_str::<Value>(line) {
                items.push(item);
            }
        }
    }
    Json(json!({"items": items}))
}

async fn get_activity(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    let tasks_file = state.root.join("tasks").join(".running-tasks.json");
    let tasks: Value = std::fs::read_to_string(&tasks_file)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or(json!({}));
    Json(json!({"tasks": tasks}))
}

async fn get_plan(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    // Reuse cached agents data to get project info
    let agents_data = crate::commands::agents::get_agents_cached(&state);
    let empty = vec![];
    let projects = agents_data
        .get("agents")
        .and_then(|a| a.as_array())
        .unwrap_or(&empty);
    let issues: Vec<Value> = projects.iter()
        .filter(|p| p.get("uncommitted").and_then(|v| v.as_u64()).unwrap_or(0) > 20 || p.get("blockers").and_then(|v| v.as_bool()).unwrap_or(false))
        .map(|p| {
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let blockers = p.get("blockers").and_then(|v| v.as_bool()).unwrap_or(false);
            let uncommitted = p.get("uncommitted").and_then(|v| v.as_u64()).unwrap_or(0);
            json!({"project": name, "issue": if blockers {"has blockers"} else {"needs commit"}, "uncommitted": uncommitted})
        })
        .collect();
    Json(json!({"issues": issues, "total": issues.len()}))
}

async fn get_config(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    Json(state.config())
}

async fn set_config(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Merge incoming fields into existing config instead of overwriting
    let mut cfg = state.config();
    if let (Some(existing), Some(incoming)) = (cfg.as_object_mut(), body.as_object()) {
        for (k, v) in incoming {
            existing.insert(k.clone(), v.clone());
        }
    }
    let _ = crate::commands::claude_runner::atomic_write(
        &state.config_path,
        &serde_json::to_string_pretty(&cfg).unwrap_or_default(),
    );
    state.invalidate_config();
    Json(json!({"status": "saved"}))
}

async fn get_permissions(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    Json(crate::commands::config::permissions_snapshot(&state))
}

async fn get_delegations(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    let delegations = match state.delegations.lock() {
        Ok(d) => d
            .values()
            .filter(|d| d.status == crate::commands::status::DelegationStatus::Pending)
            .map(|d| serde_json::to_value(d).unwrap_or_default())
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };
    Json(json!({"delegations": delegations}))
}

async fn approve_delegation(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let id = body
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return Json(json!({"error": "Missing id"}));
    }
    let state_clone = Arc::clone(&state);
    let result = tokio::task::spawn_blocking(move || {
        crate::commands::delegation::approve_delegation_core(&state_clone, &id)
    })
    .await
    .unwrap_or_else(|e| json!({"status": "error", "error": e.to_string()}));
    Json(result)
}

async fn reject_delegation(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let id = body
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();
    if let Ok(mut delegations) = state.delegations.lock() {
        if let Some(del) = delegations.get_mut(&id) {
            del.status = crate::commands::status::DelegationStatus::Rejected;
            let room_session_id = del.room_session_id.clone();
            let work_item_id = del.work_item_id.clone();
            drop(delegations);
            state.save_delegations();
            if let Some(work_item_id) = work_item_id.as_deref() {
                if let Ok(mut work_items) = state.work_items.lock() {
                    if let Some(item) = work_items.get_mut(work_item_id) {
                        item.status = crate::state::WorkItemStatus::Cancelled;
                        item.updated_at = state.now_iso();
                        item.result = Some("Delegation rejected".to_string());
                    }
                }
                state.save_work_items();
                if let Some(session_id) = room_session_id.as_deref() {
                    crate::commands::multi_agent::release_work_item_leases(
                        &state,
                        session_id,
                        work_item_id,
                        "delegation_rejected",
                    );
                }
            }
            return Json(json!({"status": "rejected"}));
        }
    }
    Json(json!({"error": "Not found"}))
}

// === Graph API handlers (Agent Protocol / MCP) ===

async fn graph_overview(AxState(state): AxState<Arc<AppState>>) -> impl IntoResponse {
    let graph = crate::commands::graph_scan::build_overview_graph(&state);
    Json(serde_json::to_value(&graph).unwrap_or_default())
}

async fn graph_project(
    AxState(state): AxState<Arc<AppState>>,
    Path(project): Path<String>,
) -> impl IntoResponse {
    match crate::commands::graph_scan::build_project_graph(&state, &project) {
        Ok(g) => Json(serde_json::to_value(&g).unwrap_or_default()),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn graph_context(
    AxState(state): AxState<Arc<AppState>>,
    Path(project): Path<String>,
) -> impl IntoResponse {
    let ctx = crate::commands::graph_ops::build_graph_context(&state, &project);
    Json(json!({"context": ctx}))
}

async fn graph_dependents(
    AxState(state): AxState<Arc<AppState>>,
    Path((project, file)): Path<(String, String)>,
) -> impl IntoResponse {
    graph_dependents_value(&state, &project, &file)
}

async fn graph_dependents_post(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let project = body
        .get("project")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let file = body
        .get("file")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    graph_dependents_value(&state, &project, &file)
}

fn graph_dependents_value(state: &AppState, project: &str, file: &str) -> Json<Value> {
    match crate::commands::graph_scan::build_project_graph(&state, &project) {
        Ok(graph) => {
            let node = resolve_graph_node(&graph, &file);
            match node {
                Some(n) => {
                    let deps: Vec<Value> = graph
                        .edges
                        .iter()
                        .filter(|e| e.target == n.id && e.kind == "import")
                        .filter_map(|e| graph.nodes.iter().find(|nn| nn.id == e.source))
                        .map(graph_node_summary)
                        .collect();
                    Json(
                        json!({"file": graph_node_ref(n), "dependents": deps, "count": deps.len()}),
                    )
                }
                None => Json(json!({"error": "File not found in graph"})),
            }
        }
        Err(e) => Json(json!({"error": e})),
    }
}

async fn graph_impact(
    AxState(state): AxState<Arc<AppState>>,
    Path((project, file)): Path<(String, String)>,
) -> impl IntoResponse {
    graph_impact_value(&state, &project, &file)
}

async fn graph_impact_post(
    AxState(state): AxState<Arc<AppState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let project = body
        .get("project")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let file = body
        .get("file")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    graph_impact_value(&state, &project, &file)
}

fn graph_impact_value(state: &AppState, project: &str, file: &str) -> Json<Value> {
    // Reuse PA command logic
    match crate::commands::graph_scan::build_project_graph(&state, &project) {
        Ok(graph) => {
            let node = resolve_graph_node(&graph, &file);
            match node {
                Some(seed) => {
                    let mut visited = std::collections::HashSet::new();
                    let mut queue = std::collections::VecDeque::new();
                    let mut impact = Vec::new();
                    queue.push_back((seed.id.clone(), 0u32));
                    visited.insert(seed.id.clone());
                    while let Some((current, depth)) = queue.pop_front() {
                        if depth > 0 {
                            if let Some(n) = graph.nodes.iter().find(|n| n.id == current) {
                                impact.push(json!({
                                    "file": graph_node_ref(n),
                                    "label": n.label,
                                    "id": n.id,
                                    "depth": depth
                                }));
                            }
                        }
                        if depth >= 3 {
                            continue;
                        }
                        for edge in graph
                            .edges
                            .iter()
                            .filter(|e| e.target == current && e.kind == "import")
                        {
                            if !visited.contains(&edge.source) {
                                visited.insert(edge.source.clone());
                                queue.push_back((edge.source.clone(), depth + 1));
                            }
                        }
                    }
                    Json(
                        json!({"file": graph_node_ref(seed), "impact": impact, "total": impact.len()}),
                    )
                }
                None => Json(json!({"error": "File not found"})),
            }
        }
        Err(e) => Json(json!({"error": e})),
    }
}

async fn graph_verify(
    AxState(state): AxState<Arc<AppState>>,
    Path(project): Path<String>,
) -> impl IntoResponse {
    match crate::commands::graph_scan::build_project_graph(&state, &project) {
        Ok(graph) => Json(json!({
            "status": if graph.cycles.is_empty() { "ok" } else { "warnings" },
            "nodes": graph.stats.total_nodes,
            "edges": graph.stats.total_edges,
            "cycles": graph.cycles,
        })),
        Err(e) => Json(json!({"error": e})),
    }
}

fn resolve_graph_node<'a>(
    graph: &'a crate::commands::graph_models::GraphData,
    selector: &str,
) -> Option<&'a crate::commands::graph_models::GraphNode> {
    let selector = selector.trim();
    let selector_lower = selector.to_lowercase();
    let selector_id = if selector_lower.starts_with("file:") {
        selector_lower.clone()
    } else {
        format!("file:{}", selector_lower)
    };

    graph
        .nodes
        .iter()
        .find(|n| n.id.to_lowercase() == selector_id || n.id.to_lowercase() == selector_lower)
        .or_else(|| {
            graph.nodes.iter().find(|n| {
                n.path
                    .as_deref()
                    .map(|p| p.eq_ignore_ascii_case(selector))
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            let mut matches = graph
                .nodes
                .iter()
                .filter(|n| n.label.eq_ignore_ascii_case(selector));
            let first = matches.next()?;
            if matches.next().is_some() {
                None
            } else {
                Some(first)
            }
        })
}

fn graph_node_ref(node: &crate::commands::graph_models::GraphNode) -> &str {
    node.path.as_deref().unwrap_or(&node.label)
}

fn graph_node_summary(node: &crate::commands::graph_models::GraphNode) -> Value {
    json!({
        "id": node.id,
        "label": node.label,
        "path": node.path,
    })
}
