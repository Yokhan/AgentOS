//! Category management: load categories.json, build PA context, enrich delegations.

use crate::state::AppState;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Deserialize)]
pub struct CategoryMeta {
    pub shared_resources: Vec<String>,
    pub delegation_strategy: String,
    pub description: String,
}

/// Load categories from n8n/dashboard/categories.json
pub fn load_categories(root: &Path) -> HashMap<String, CategoryMeta> {
    let path = root.join("n8n").join("dashboard").join("categories.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Build [CATEGORIES] context block for PA prompt
pub fn build_category_context(state: &AppState) -> String {
    let categories = load_categories(&state.root);
    if categories.is_empty() {
        return String::new();
    }

    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    let mut lines = vec!["[CATEGORIES]".to_string()];

    for (cat_name, meta) in &categories {
        let project_count = segments.get(cat_name).map(|v| v.len()).unwrap_or(0);
        if project_count == 0 {
            continue;
        }

        let shared = if meta.shared_resources.is_empty() {
            "none".to_string()
        } else {
            meta.shared_resources.join(", ")
        };

        lines.push(format!(
            "{} ({} projects, {}):\n  Shared: {}\n  Strategy: {}",
            cat_name, project_count, meta.description, shared, meta.delegation_strategy
        ));
    }

    lines.push("[END CATEGORIES]".to_string());
    if lines.len() <= 2 {
        return String::new();
    }
    lines.join("\n") + "\n"
}

/// Get category name for a project (from segments)
pub fn get_category_for_project(state: &AppState, project: &str) -> Option<String> {
    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    for (cat, projects) in segments.iter() {
        if projects.iter().any(|p| p.eq_ignore_ascii_case(project)) {
            return Some(cat.clone());
        }
    }
    None
}

/// Enrich delegation task with category context (shared resources, related projects)
pub fn enrich_delegation_with_category(state: &AppState, project: &str) -> String {
    let cat_name = match get_category_for_project(state, project) {
        Some(c) => c,
        None => return String::new(),
    };

    let categories = load_categories(&state.root);
    let meta = match categories.get(&cat_name) {
        Some(m) => m,
        None => return String::new(),
    };

    let segments = state.segments.lock().unwrap_or_else(|e| e.into_inner());
    let related: Vec<&String> = segments.get(&cat_name)
        .map(|v| v.iter().filter(|p| !p.eq_ignore_ascii_case(project)).collect())
        .unwrap_or_default();

    let mut ctx = format!("\n[CATEGORY CONTEXT: {}]", cat_name);
    if !related.is_empty() {
        let names: Vec<&str> = related.iter().take(8).map(|s| s.as_str()).collect();
        ctx += &format!("\nRelated projects: {}", names.join(", "));
    }
    if !meta.shared_resources.is_empty() {
        ctx += &format!("\nShared resources: {}", meta.shared_resources.join(", "));
    }
    ctx += &format!("\nDelegation strategy: {}", meta.delegation_strategy);
    ctx += "\n[END CATEGORY CONTEXT]";
    ctx
}
