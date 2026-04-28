use crate::state::AppState;
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct WaitCoordinatorSnapshot {
    pub items: Vec<String>,
    pub actionable: usize,
    pub warnings: usize,
    pub summary: String,
}

fn short(value: &str, limit: usize) -> String {
    crate::commands::claude_runner::safe_truncate(value, limit).to_string()
}

fn count(map: &Value, key: &str) -> usize {
    map.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn route_phase(route: &Value) -> &str {
    route
        .get("progress")
        .and_then(|progress| progress.get("phase"))
        .and_then(Value::as_str)
        .or_else(|| route.get("route_state").and_then(Value::as_str))
        .unwrap_or("idle")
}

pub fn build_wait_coordinator_snapshot(
    state: &AppState,
    project: Option<&str>,
    room_session_id: Option<&str>,
    limit: usize,
) -> Option<WaitCoordinatorSnapshot> {
    let map = super::scope::resolve_orchestration_map(
        state,
        project.map(str::to_string),
        room_session_id.map(str::to_string),
    );
    if map.get("status").and_then(Value::as_str) != Some("ok") {
        return None;
    }

    let progress = map.get("route_progress").unwrap_or(&Value::Null);
    let active = count(progress, "active");
    let queueable = count(progress, "queueable");
    let needs_user = count(progress, "needs_user");
    let blocked = count(progress, "blocked");
    let total = count(progress, "total");
    if total == 0 {
        return None;
    }

    let mut snapshot = WaitCoordinatorSnapshot {
        summary: progress
            .get("headline")
            .and_then(Value::as_str)
            .unwrap_or("Обновлен контекст ожидания project agents")
            .to_string(),
        ..Default::default()
    };

    if queueable > 0 {
        snapshot.actionable += queueable;
        snapshot.items.push(format!(
            "Есть {} независимых route, готовых к запуску. Не простаивай из-за других running/pending проектов: если задача все еще приоритетна и нет конфликта, эмитни [WORK_ITEM_QUEUE:id].",
            queueable
        ));
    }

    if active > 0 {
        snapshot.items.push(format!(
            "{} route уже выполняются project agents. Не блокируй весь раунд на них: продолжай независимые route или дай явный статус ожидания.",
            active
        ));
    }

    if needs_user > 0 {
        snapshot.warnings += needs_user;
        snapshot.items.push(format!(
            "{} route ждут решения пользователя. Если это единственный оставшийся блокер, скажи это явно и не делай вид, что агент работает.",
            needs_user
        ));
    }

    if blocked > 0 {
        snapshot.warnings += blocked;
        snapshot.items.push(format!(
            "{} route заблокированы. Разбирай failed/permission blocker, а не повторяй общий dashboard без следующего действия.",
            blocked
        ));
    }

    if let Some(routes) = map.get("project_agent_routes").and_then(Value::as_array) {
        for route in routes
            .iter()
            .filter(|route| route.get("can_queue_next").and_then(Value::as_bool) == Some(true))
            .take(limit)
        {
            let Some(item) = route.get("next_work_item") else {
                continue;
            };
            let id = item.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() {
                continue;
            }
            let project = route.get("project").and_then(Value::as_str).unwrap_or("?");
            let title = item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("ready work item");
            let intent = item
                .get("write_intent")
                .and_then(Value::as_str)
                .unwrap_or("read_only");
            let task = item.get("task").and_then(Value::as_str).unwrap_or("");
            snapshot.items.push(format!(
                "Ready route: {} / {} / {}. Команда запуска: [WORK_ITEM_QUEUE:{}]. Task: {}",
                project,
                short(title, 80),
                intent,
                id,
                short(task, 220)
            ));
        }

        for route in routes
            .iter()
            .filter(|route| matches!(route_phase(route), "running" | "verifying" | "reviewing"))
            .take(limit)
        {
            let project = route.get("project").and_then(Value::as_str).unwrap_or("?");
            let phase = route_phase(route);
            let delegation_id = route
                .get("progress")
                .and_then(|progress| progress.get("active_delegation_id"))
                .and_then(Value::as_str)
                .unwrap_or("");
            snapshot.items.push(format!(
                "Active route: {} сейчас {}{}.",
                project,
                phase,
                if delegation_id.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", short(delegation_id, 12))
                }
            ));
        }
    }

    if snapshot.items.is_empty() {
        None
    } else {
        Some(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::build_wait_coordinator_snapshot;
    use crate::commands::provider_runner::ProviderKind;
    use crate::state::{AppState, WorkItem, WorkItemAssignee, WorkItemStatus, WorkItemWriteIntent};

    fn test_root(name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agentos-wait-coordinator-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("tasks")).expect("create temp tasks");
        path
    }

    #[test]
    fn snapshot_points_orchestrator_to_queueable_work_item() {
        let root = test_root("queueable");
        let state = AppState::new(root.clone());
        let ts = state.now_iso();
        state.work_items.lock().unwrap().insert(
            "wi-ready".to_string(),
            WorkItem {
                id: "wi-ready".to_string(),
                parent_room_session_id: "room-1".to_string(),
                project_session_id: None,
                project: "AgentOS".to_string(),
                title: "Async polish".to_string(),
                task: "Continue independent UX work while another route is running".to_string(),
                executor_provider: ProviderKind::Codex,
                reviewer_provider: None,
                assignee: WorkItemAssignee::Agent,
                write_intent: WorkItemWriteIntent::ReadOnly,
                declared_paths: Vec::new(),
                verify: None,
                status: WorkItemStatus::Ready,
                delegation_id: None,
                result: None,
                review_verdict: None,
                source_kind: None,
                source_id: None,
                created_at: ts.clone(),
                updated_at: ts,
            },
        );

        let snapshot =
            build_wait_coordinator_snapshot(&state, None, None, 3).expect("coordinator snapshot");

        assert!(snapshot.actionable >= 1);
        assert!(snapshot
            .items
            .iter()
            .any(|item| item.contains("[WORK_ITEM_QUEUE:wi-ready]")));

        let _ = std::fs::remove_dir_all(root);
    }
}
