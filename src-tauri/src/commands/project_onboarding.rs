//! Project onboarding: make "connect this repo to AgentOS" a first-class operation.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectOnboardingItem {
    pub name: String,
    pub path: String,
    pub managed: bool,
    pub template_version: String,
    pub latest_template_version: String,
    pub template_outdated: bool,
    pub segment: String,
    pub permission: String,
    pub has_claude_md: bool,
    pub has_agents_md: bool,
    pub has_current_task: bool,
    pub has_check_drift: bool,
    pub needs_segment: bool,
    pub needs_permission: bool,
    pub needs_template: bool,
    pub ready: bool,
    pub next_actions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConnectOptions {
    pub project: String,
    #[serde(default)]
    pub segment: Option<String>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub deploy_template: bool,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConnectMissingOptions {
    #[serde(default)]
    pub segment: Option<String>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectOnboardingPlanOptions {
    #[serde(default)]
    pub segment: Option<String>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

fn segments_path(state: &AppState) -> PathBuf {
    state.segments_path.clone()
}

fn valid_permission(profile: &str) -> bool {
    matches!(profile, "restrictive" | "balanced" | "permissive")
}

fn read_segments(path: &Path) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let value = serde_json::from_str::<Value>(&content).unwrap_or_else(|_| json!({}));
    if let Some(map) = value.get("segments").and_then(Value::as_object) {
        for (segment, projects) in map {
            let list = projects
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            out.insert(segment.clone(), list);
        }
    }
    out
}

fn write_segments(path: &Path, segments: &BTreeMap<String, Vec<String>>) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Invalid segments path: {}", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let value = json!({
        "_comment": "Project segments for dashboard grouping. Edit to customize.",
        "segments": segments,
    });
    let content = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    super::claude_runner::atomic_write(path, &content).map_err(|e| e.to_string())
}

fn attach_project_to_segment(
    segments: &mut BTreeMap<String, Vec<String>>,
    project: &str,
    target_segment: &str,
) -> bool {
    let mut changed = false;
    for projects in segments.values_mut() {
        let before = projects.len();
        projects.retain(|p| p != project);
        changed |= projects.len() != before;
    }
    let target = segments.entry(target_segment.to_string()).or_default();
    if !target.iter().any(|p| p == project) {
        target.push(project.to_string());
        target.sort();
        changed = true;
    }
    changed
}

fn template_version(project_dir: &Path) -> String {
    let manifest = project_dir.join(".template-manifest.json");
    if !manifest.exists() {
        return "none".to_string();
    }
    std::fs::read_to_string(manifest)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .and_then(|value| {
            value
                .get("template_version")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "?".to_string())
}

fn permission_for(config: &Value, project: &str) -> String {
    config
        .get("project_permissions")
        .and_then(|v| v.get(project))
        .and_then(Value::as_str)
        .unwrap_or("none")
        .to_string()
}

fn project_dirs(docs_dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(docs_dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.join(".git").exists())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn git_dirty_count(project_dir: &Path) -> Option<usize> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
    )
}

fn build_item(
    project_dir: &Path,
    segments: &BTreeMap<String, Vec<String>>,
    config: &Value,
    latest_template_version: &str,
) -> Option<ProjectOnboardingItem> {
    let name = project_dir.file_name()?.to_string_lossy().to_string();
    let segment = segments
        .iter()
        .find_map(|(segment, projects)| {
            projects
                .iter()
                .any(|p| p == &name)
                .then(|| segment.to_string())
        })
        .unwrap_or_else(|| "Unassigned".to_string());
    let permission = permission_for(config, &name);
    let version = template_version(project_dir);
    let managed = version != "none";
    let has_claude_md = project_dir.join("CLAUDE.md").exists();
    let has_agents_md = project_dir.join("AGENTS.md").exists();
    let has_current_task = project_dir.join("tasks").join("current.md").exists();
    let has_check_drift = project_dir.join("scripts").join("check-drift.sh").exists();
    let needs_segment = segment == "Unassigned";
    let needs_permission = permission == "none";
    let template_outdated = managed
        && version != "?"
        && !latest_template_version.is_empty()
        && version != latest_template_version;
    let needs_template = !managed || !has_current_task || !has_check_drift || template_outdated;
    let ready = !needs_segment && !needs_permission && !needs_template;
    let mut next_actions = Vec::new();
    if needs_segment {
        next_actions.push("assign segment".to_string());
    }
    if needs_permission {
        next_actions.push("set permission profile".to_string());
    }
    if needs_template {
        next_actions.push("deploy/sync agent template".to_string());
    }

    Some(ProjectOnboardingItem {
        name,
        path: project_dir.display().to_string(),
        managed,
        template_version: version,
        latest_template_version: latest_template_version.to_string(),
        template_outdated,
        segment,
        permission,
        has_claude_md,
        has_agents_md,
        has_current_task,
        has_check_drift,
        needs_segment,
        needs_permission,
        needs_template,
        ready,
        next_actions,
    })
}

pub fn audit_projects(state: &AppState) -> Vec<ProjectOnboardingItem> {
    let segments = read_segments(&segments_path(state));
    let config = state.config();
    let latest_template_version = template_version(&state.docs_dir.join("agent-project-template"));
    let latest_template_version =
        if latest_template_version == "none" || latest_template_version == "?" {
            String::new()
        } else {
            latest_template_version
        };
    let mut items = project_dirs(&state.docs_dir)
        .iter()
        .filter_map(|dir| build_item(dir, &segments, &config, &latest_template_version))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        a.ready
            .cmp(&b.ready)
            .then_with(|| b.next_actions.len().cmp(&a.next_actions.len()))
            .then_with(|| a.name.cmp(&b.name))
    });
    items
}

#[tauri::command]
pub fn project_onboarding_audit(state: State<Arc<AppState>>) -> Value {
    let items = audit_projects(&state);
    let ready = items.iter().filter(|i| i.ready).count();
    let unmanaged = items.iter().filter(|i| i.needs_template).count();
    let unassigned = items.iter().filter(|i| i.needs_segment).count();
    let no_permission = items.iter().filter(|i| i.needs_permission).count();
    json!({
        "status": "ok",
        "documents_dir": state.docs_dir,
        "total": items.len(),
        "ready": ready,
        "unmanaged": unmanaged,
        "unassigned": unassigned,
        "no_permission": no_permission,
        "items": items,
    })
}

fn refresh_segment_state(state: &AppState, segments: &BTreeMap<String, Vec<String>>) {
    let mut segment_map = std::collections::HashMap::new();
    let mut project_segment = std::collections::HashMap::new();
    for (segment, projects) in segments {
        segment_map.insert(segment.clone(), projects.clone());
        for project in projects {
            project_segment.insert(project.clone(), segment.clone());
        }
    }
    *state.segments.lock().unwrap_or_else(|e| e.into_inner()) = segment_map;
    *state
        .project_segment
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = project_segment;
}

fn invalidate_scan_cache(state: &AppState) {
    if let Ok(mut cache) = state.scan_cache.lock() {
        cache.data = None;
        cache.updated = None;
    }
}

pub fn connect_project_inline(
    state: &AppState,
    opts: ProjectConnectOptions,
) -> Result<Value, String> {
    if opts.project.trim().is_empty() {
        return Err("Project is required".to_string());
    }
    let project_dir = state.validate_project(&opts.project)?;
    let segment = opts
        .segment
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Other".to_string());
    let permission = opts
        .permission
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "balanced".to_string());
    if !valid_permission(&permission) {
        return Err(format!(
            "Invalid permission profile `{}`. Use restrictive, balanced, or permissive.",
            permission
        ));
    }

    let seg_path = segments_path(state);
    let mut segments = read_segments(&seg_path);
    let config = state.config();
    let mut planned = Vec::new();
    let old_segment = segments
        .iter()
        .find_map(|(name, projects)| projects.iter().any(|p| p == &opts.project).then(|| name))
        .cloned()
        .unwrap_or_else(|| "Unassigned".to_string());
    if old_segment != segment {
        planned.push(format!("segment: {} -> {}", old_segment, segment));
    }
    let old_permission = permission_for(&config, &opts.project);
    if old_permission != permission {
        planned.push(format!("permission: {} -> {}", old_permission, permission));
    }
    if opts.deploy_template {
        planned.push("template: deploy/sync".to_string());
    }

    if opts.dry_run {
        return Ok(json!({
            "status": "dry_run",
            "project": opts.project,
            "path": project_dir,
            "planned": planned,
        }));
    }

    attach_project_to_segment(&mut segments, &opts.project, &segment);
    write_segments(&seg_path, &segments)?;
    refresh_segment_state(state, &segments);

    state.update_config(|config| {
        if !config.is_object() {
            *config = json!({});
        }
        if config.get("project_permissions").is_none() {
            config["project_permissions"] = json!({});
        }
        config["project_permissions"][&opts.project] = json!(permission);
        Ok(())
    })?;

    let template_result = if opts.deploy_template {
        Some(super::ops::execute_deploy_inline(
            &state.root,
            &state.docs_dir,
            &opts.project,
        ))
    } else {
        None
    };

    invalidate_scan_cache(state);
    let latest = template_version(&state.docs_dir.join("agent-project-template"));
    let latest = if matches!(latest.as_str(), "none" | "?") {
        String::new()
    } else {
        latest
    };
    let item = build_item(&project_dir, &segments, &state.config(), &latest);
    Ok(json!({
        "status": "ok",
        "project": opts.project,
        "planned": planned,
        "template_result": template_result,
        "item": item,
    }))
}

pub fn connect_missing_inline(
    state: &AppState,
    opts: ProjectConnectMissingOptions,
) -> Result<Value, String> {
    let segment = opts
        .segment
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Other".to_string());
    let permission = opts
        .permission
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "balanced".to_string());
    if !valid_permission(&permission) {
        return Err(format!(
            "Invalid permission profile `{}`. Use restrictive, balanced, or permissive.",
            permission
        ));
    }

    let seg_path = segments_path(state);
    let mut segments = read_segments(&seg_path);
    let mut config = state.config();
    if config.get("project_permissions").is_none() {
        config["project_permissions"] = json!({});
    }

    let mut planned = Vec::new();
    let mut changed_segments = false;
    let mut changed_permissions = false;
    let mut permission_projects = Vec::new();
    for project_dir in project_dirs(&state.docs_dir) {
        let Some(name) = project_dir
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
        else {
            continue;
        };
        let old_segment = segments
            .iter()
            .find_map(|(seg, projects)| projects.iter().any(|p| p == &name).then(|| seg))
            .cloned()
            .unwrap_or_else(|| "Unassigned".to_string());
        let old_permission = permission_for(&config, &name);
        let mut actions = Vec::new();
        if old_segment == "Unassigned" {
            actions.push(format!("segment: Unassigned -> {}", segment));
            if !opts.dry_run {
                changed_segments |= attach_project_to_segment(&mut segments, &name, &segment);
            }
        }
        if old_permission == "none" {
            actions.push(format!("permission: none -> {}", permission));
            if !opts.dry_run {
                permission_projects.push(name.clone());
                changed_permissions = true;
            }
        }
        if !actions.is_empty() {
            planned.push(json!({
                "project": name,
                "actions": actions,
            }));
        }
    }

    if opts.dry_run {
        return Ok(json!({
            "status": "dry_run",
            "segment": segment,
            "permission": permission,
            "planned": planned,
        }));
    }

    if changed_segments {
        write_segments(&seg_path, &segments)?;
        refresh_segment_state(state, &segments);
    }
    if changed_permissions {
        state.update_config(|config| {
            if !config.is_object() {
                *config = json!({});
            }
            if config.get("project_permissions").is_none() {
                config["project_permissions"] = json!({});
            }
            for project in &permission_projects {
                config["project_permissions"][project] = json!(permission);
            }
            Ok(())
        })?;
    }
    invalidate_scan_cache(state);

    Ok(json!({
        "status": "ok",
        "segment": segment,
        "permission": permission,
        "updated": planned.len(),
        "planned": planned,
    }))
}

pub fn format_onboarding_plan(state: &AppState, opts: ProjectOnboardingPlanOptions) -> String {
    let segment = opts
        .segment
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Other".to_string());
    let permission = opts
        .permission
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "balanced".to_string());
    let limit = opts.limit.unwrap_or(5).clamp(1, 20);
    let items = audit_projects(state);
    if items.is_empty() {
        return "Project onboarding plan: git projects were not found under documents_dir."
            .to_string();
    }

    let ready = items.iter().filter(|item| item.ready).count();
    let metadata_candidates = items
        .iter()
        .filter(|item| item.needs_segment || item.needs_permission)
        .collect::<Vec<_>>();
    let template_candidates = items
        .iter()
        .filter(|item| item.needs_template)
        .collect::<Vec<_>>();
    let mut safe_template = Vec::new();
    let mut blocked_template = Vec::new();
    for item in &template_candidates {
        let dirty = git_dirty_count(Path::new(&item.path));
        if dirty == Some(0) {
            safe_template.push((item, dirty));
        } else {
            blocked_template.push((item, dirty));
        }
    }

    let mut lines = vec![
        format!(
            "**Project onboarding wave plan:** {}/{} ready",
            ready,
            items.len()
        ),
        format!("- metadata fixes: {} project(s)", metadata_candidates.len()),
        format!(
            "- template candidates: {} clean canary / {} blocked or dirty",
            safe_template.len(),
            blocked_template.len()
        ),
        String::new(),
        "**Safe next commands:**".to_string(),
    ];
    if !metadata_candidates.is_empty() {
        lines.push(format!(
            "1. Preview metadata repair: `[PROJECT_CONNECT_MISSING:{}:{}:dry]`",
            segment, permission
        ));
        lines.push(format!(
            "2. Apply metadata repair after review: `[PROJECT_CONNECT_MISSING:{}:{}]`",
            segment, permission
        ));
    } else {
        lines.push("1. Metadata is already assigned for all discovered git projects.".to_string());
    }

    if let Some((item, _)) = safe_template.first() {
        lines.push(format!(
            "3. Canary template sync: `[PROJECT_CONNECT:{}:{}:{}:deploy,dry]`, then `[PROJECT_CONNECT:{}:{}:{}:deploy]`",
            item.name, item.segment, item.permission, item.name, item.segment, item.permission
        ));
    } else if !template_candidates.is_empty() {
        lines.push(
            "3. No clean canary for template sync. Resolve dirty/blocked projects before deploy."
                .to_string(),
        );
    } else {
        lines.push("3. No template sync needed by onboarding audit.".to_string());
    }

    if !safe_template.is_empty() {
        lines.push(String::new());
        lines.push(format!("**Clean canary candidates ({} max):**", limit));
        for (item, _) in safe_template.iter().take(limit) {
            lines.push(format!(
                "- {}: template={}, segment={}, permission={}",
                item.name, item.template_version, item.segment, item.permission
            ));
        }
    }
    if !blocked_template.is_empty() {
        lines.push(String::new());
        lines.push(format!("**Blocked template candidates ({} max):**", limit));
        for (item, dirty) in blocked_template.iter().take(limit) {
            let dirty_label = dirty
                .map(|count| format!("{} dirty file(s)", count))
                .unwrap_or_else(|| "git status unavailable".to_string());
            lines.push(format!(
                "- {}: {}, next={}",
                item.name,
                dirty_label,
                item.next_actions.join(", ")
            ));
        }
    }
    lines.push(String::new());
    lines.push(
        "Rule: do metadata repair in bulk, but deploy templates only as canary waves after clean git status."
            .to_string(),
    );
    lines.join("\n")
}

#[tauri::command]
pub async fn project_onboarding_plan(
    state: State<'_, Arc<AppState>>,
    opts: ProjectOnboardingPlanOptions,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        Ok(json!({
            "status": "ok",
            "text": format_onboarding_plan(&state_arc, opts),
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn project_connect(
    state: State<'_, Arc<AppState>>,
    opts: ProjectConnectOptions,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || connect_project_inline(&state_arc, opts))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn project_connect_missing(
    state: State<'_, Arc<AppState>>,
    opts: ProjectConnectMissingOptions,
) -> Result<Value, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || connect_missing_inline(&state_arc, opts))
        .await
        .map_err(|e| e.to_string())?
}

pub fn format_onboarding_audit(state: &AppState) -> String {
    let items = audit_projects(state);
    if items.is_empty() {
        return "No git projects found under documents_dir.".to_string();
    }
    let mut lines = Vec::new();
    let ready = items.iter().filter(|i| i.ready).count();
    lines.push(format!(
        "**Project Onboarding Audit:** {}/{} ready",
        ready,
        items.len()
    ));
    for item in items.iter().filter(|i| !i.ready).take(20) {
        lines.push(format!(
            "- {}: segment={}, permission={}, template={} -> {}",
            item.name,
            item.segment,
            item.permission,
            item.template_version,
            item.next_actions.join(", ")
        ));
    }
    let hidden = items.iter().filter(|i| !i.ready).count().saturating_sub(20);
    if hidden > 0 {
        lines.push(format!("- ...{} more projects need onboarding", hidden));
    }
    lines.push(String::new());
    lines.push("Use `[PROJECT_CONNECT:Project:Segment:balanced]` for one project, or add `:deploy` to sync the template.".to_string());
    lines.join("\n")
}

pub fn format_connect_result(result: &Value) -> String {
    let status = result
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let project = result
        .get("project")
        .and_then(Value::as_str)
        .unwrap_or("project");
    let planned = result
        .get("planned")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default();
    if status == "dry_run" {
        return format!("**Project connect dry-run:** {} -> {}", project, planned);
    }
    let template = result
        .get("template_result")
        .and_then(Value::as_str)
        .unwrap_or("");
    if template.is_empty() {
        format!("**Project connected:** {} -> {}", project, planned)
    } else {
        format!(
            "**Project connected:** {} -> {}\nTemplate: {}",
            project, planned, template
        )
    }
}

pub fn format_connect_missing_result(result: &Value) -> String {
    let status = result
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let planned = result
        .get("planned")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let prefix = if status == "dry_run" {
        "**Project bulk connect dry-run**"
    } else {
        "**Project bulk connect complete**"
    };
    let mut lines = vec![format!("{}: {} project(s)", prefix, planned.len())];
    for item in planned.iter().take(20) {
        let project = item
            .get("project")
            .and_then(Value::as_str)
            .unwrap_or("project");
        let actions = item
            .get("actions")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();
        lines.push(format!("- {}: {}", project, actions));
    }
    if planned.len() > 20 {
        lines.push(format!("- ...{} more", planned.len() - 20));
    }
    lines.join("\n")
}

#[allow(dead_code)]
fn _unique_sorted(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_project_moves_between_segments_without_duplicates() {
        let mut segments = BTreeMap::from([
            (
                "Old".to_string(),
                vec!["A".to_string(), "Project".to_string()],
            ),
            ("New".to_string(), vec!["B".to_string()]),
        ]);
        assert!(attach_project_to_segment(&mut segments, "Project", "New"));
        assert_eq!(segments.get("Old").unwrap(), &vec!["A".to_string()]);
        assert_eq!(
            segments.get("New").unwrap(),
            &vec!["B".to_string(), "Project".to_string()]
        );
    }

    #[test]
    fn permission_profile_is_restricted_to_known_profiles() {
        assert!(valid_permission("balanced"));
        assert!(valid_permission("restrictive"));
        assert!(valid_permission("permissive"));
        assert!(!valid_permission("root"));
    }

    #[test]
    fn onboarding_plan_returns_safe_commands_and_blockers() {
        let root = std::env::temp_dir().join(format!(
            "agentos-onboarding-plan-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let docs = root.join("docs");
        let project = docs.join("SampleProject");
        std::fs::create_dir_all(project.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string_pretty(&json!({
                "documents_dir": docs.to_string_lossy(),
                "project_root": root.to_string_lossy(),
                "project_permissions": {}
            }))
            .unwrap(),
        )
        .unwrap();

        let state = AppState::new(root.clone());
        let plan = format_onboarding_plan(
            &state,
            ProjectOnboardingPlanOptions {
                segment: Some("Other".to_string()),
                permission: Some("balanced".to_string()),
                limit: Some(3),
            },
        );

        assert!(plan.contains("Project onboarding wave plan"));
        assert!(plan.contains("[PROJECT_CONNECT_MISSING:Other:balanced:dry]"));
        assert!(plan.contains("[PROJECT_CONNECT_MISSING:Other:balanced]"));
        assert!(plan.contains("Blocked template candidates"));
        assert!(plan.contains("SampleProject"));

        let _ = std::fs::remove_dir_all(root);
    }
}
