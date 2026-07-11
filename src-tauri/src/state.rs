use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const RUNTIME_SNAPSHOT_SCHEMA: u32 = 1;

#[derive(Clone, Serialize, Deserialize)]
struct RuntimeSnapshot {
    schema_version: u32,
    revision: u64,
    saved_at: String,
    delegations: HashMap<String, Delegation>,
    sessions: HashMap<String, MultiAgentSession>,
    project_sessions: HashMap<String, ProjectSession>,
    work_items: HashMap<String, WorkItem>,
    file_leases: HashMap<String, FileLease>,
}

pub struct DirLockGuard<'a> {
    state: &'a AppState,
    key: String,
}

impl Drop for DirLockGuard<'_> {
    fn drop(&mut self) {
        self.state.release_dir_lock(&self.key);
    }
}

/// Cached scan result
pub struct ScanCache {
    pub data: Option<serde_json::Value>,
    pub updated: Option<Instant>,
}

impl Default for ScanCache {
    fn default() -> Self {
        Self {
            data: None,
            updated: None,
        }
    }
}

/// Pending delegation
#[derive(Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub id: String,
    pub project: String,
    pub task: String,
    pub ts: String,
    /// Actual execution start time. `ts` is the queue time and can be much older.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub status: crate::commands::status::DelegationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(default)]
    pub retries: u32,
    /// Link to plan step (if delegation was created from a plan)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_step: Option<usize>,
    /// Escalation reason (if L2/L3 triggered)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_info: Option<String>,
    /// Link to strategy (if delegation was created from a strategy step)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy_step_id: Option<String>,
    /// Linked live-room session for dual-agent orchestration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_session_id: Option<String>,
    /// Linked child project session
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_session_id: Option<String>,
    /// Linked work item that produced this delegation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    /// Provider chosen to execute this delegation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_provider: Option<crate::commands::provider_runner::ProviderKind>,
    /// Optional reviewer provider for this delegation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_provider: Option<crate::commands::provider_runner::ProviderKind>,
    /// Git changes captured after successful delegation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_diff: Option<String>,
    /// Token usage and cost
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<crate::commands::usage::UsageInfo>,
    /// Scheduled execution time (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
    /// Batch ID (groups related delegations)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Priority ordering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<crate::commands::delegation_models::DelegationPriority>,
    /// Per-delegation timeout override (seconds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Gate pipeline result (post-delegation verification)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<crate::commands::gate::GateResult>,
    /// Structured reviewer verdict, if a review lane completed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_verdict: Option<ReviewVerdict>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Review,
    Debate,
    Parallel,
    Arbitration,
}

impl Default for SessionMode {
    fn default() -> Self {
        Self::Review
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Closed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceState {
    Idle,
    Thinking,
    Replying,
    Waiting,
    Blocked,
}

impl Default for PresenceState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionParticipant {
    pub id: String,
    pub label: String,
    pub provider: crate::commands::provider_runner::ProviderKind,
    pub role: String,
    #[serde(default)]
    pub write_enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MultiAgentSession {
    pub id: String,
    pub title: String,
    pub project: String,
    #[serde(default)]
    pub status: SessionStatus,
    #[serde(default)]
    pub mode: SessionMode,
    #[serde(default)]
    pub participants: Vec<SessionParticipant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestrator_participant_id: Option<String>,
    #[serde(default)]
    pub current_working_set: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_round_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_speaker: Option<String>,
    #[serde(default)]
    pub presence: HashMap<String, PresenceState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_challenge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_rebuttal: Option<String>,
    #[serde(default)]
    pub linked_strategy_ids: Vec<String>,
    #[serde(default)]
    pub linked_project_session_ids: Vec<String>,
    #[serde(default)]
    pub linked_work_item_ids: Vec<String>,
    #[serde(default)]
    pub linked_tactic_ids: Vec<String>,
    #[serde(default)]
    pub linked_plan_ids: Vec<String>,
    #[serde(default)]
    pub linked_delegation_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: String,
    pub session_id: String,
    pub ts: String,
    pub kind: String,
    pub actor: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSessionStatus {
    Active,
    Paused,
    Closed,
}

impl Default for ProjectSessionStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectSession {
    pub id: String,
    pub parent_room_session_id: String,
    pub project: String,
    pub title: String,
    #[serde(default)]
    pub status: ProjectSessionStatus,
    pub executor_provider: crate::commands::provider_runner::ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_provider: Option<crate::commands::provider_runner::ProviderKind>,
    #[serde(default)]
    pub linked_work_item_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemStatus {
    Draft,
    Ready,
    Queued,
    Running,
    Reviewing,
    Completed,
    Failed,
    Cancelled,
}

impl Default for WorkItemStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdictStatus {
    Approve,
    Warn,
    Fail,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReviewVerdict {
    pub status: ReviewVerdictStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    pub source_response: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemAssignee {
    Agent,
    User,
}

impl Default for WorkItemAssignee {
    fn default() -> Self {
        Self::Agent
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemWriteIntent {
    ReadOnly,
    ProposeWrite,
    ExclusiveWrite,
}

impl Default for WorkItemWriteIntent {
    fn default() -> Self {
        Self::ReadOnly
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub parent_room_session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_session_id: Option<String>,
    pub project: String,
    pub title: String,
    pub task: String,
    pub executor_provider: crate::commands::provider_runner::ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_provider: Option<crate::commands::provider_runner::ProviderKind>,
    #[serde(default)]
    pub assignee: WorkItemAssignee,
    #[serde(default)]
    pub write_intent: WorkItemWriteIntent,
    #[serde(default)]
    pub declared_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify: Option<crate::commands::strategy_models::VerifyCondition>,
    #[serde(default)]
    pub status: WorkItemStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_verdict: Option<ReviewVerdict>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileLeaseStatus {
    Active,
    Released,
}

impl Default for FileLeaseStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileLease {
    pub id: String,
    pub session_id: String,
    pub work_item_id: String,
    pub project: String,
    pub participant_id: String,
    pub provider: crate::commands::provider_runner::ProviderKind,
    pub write_intent: WorkItemWriteIntent,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub status: FileLeaseStatus,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,
}

/// Shared application state
pub struct AppState {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub tasks_dir: PathBuf,
    pub docs_dir: PathBuf,
    pub config_path: PathBuf,
    pub segments_path: PathBuf,
    pub chats_dir: PathBuf,
    pub delegations_path: PathBuf,
    pub sessions_path: PathBuf,
    pub session_events_path: PathBuf,
    pub project_sessions_path: PathBuf,
    pub work_items_path: PathBuf,
    pub file_leases_path: PathBuf,
    pub runtime_snapshot_path: PathBuf,
    pub n8n_url: String,
    pub start_time: Instant,

    // Per-directory busy flags: prevents two claude processes in same project dir
    pub dir_busy: Mutex<std::collections::HashSet<String>>,

    // Caches
    pub scan_cache: Mutex<ScanCache>,
    /// Serializes expensive repository scans without holding the cache mutex during I/O.
    pub scan_refresh_lock: Mutex<()>,
    pub segments: Mutex<HashMap<String, Vec<String>>>,
    pub project_segment: Mutex<HashMap<String, String>>,

    // Runtime state
    pub delegations: Mutex<HashMap<String, Delegation>>,
    pub sessions: Mutex<HashMap<String, MultiAgentSession>>,
    pub project_sessions: Mutex<HashMap<String, ProjectSession>>,
    pub work_items: Mutex<HashMap<String, WorkItem>>,
    pub file_leases: Mutex<HashMap<String, FileLease>>,
    /// Serializes cross-entity snapshots and provides monotonic revisions.
    runtime_persistence: Mutex<u64>,
    /// Running child process PIDs by chat_key — used to kill zombies
    pub running_pids: Mutex<HashMap<String, u32>>,
    /// Join handles for long-lived chat/runtime workers.
    pub background_tasks: Mutex<HashMap<String, std::thread::JoinHandle<()>>>,
    pub shutdown_requested: AtomicBool,
    /// Chat keys explicitly cancelled by the user. Checked by agent loops between actions.
    pub chat_cancellations: Mutex<HashSet<String>>,
    /// Running activities by project — in-memory, no file race conditions
    pub activities: Mutex<HashMap<String, serde_json::Value>>,
    pub operations: Mutex<HashMap<String, crate::commands::operation_state::OperationRecord>>,
    /// Agent feedback inbox — delegation results accumulate here
    pub inbox: Mutex<Vec<crate::commands::inbox::InboxItem>>,
    /// Cached config.json — refreshed every 5 seconds or after writes
    pub config_cache: Mutex<(serde_json::Value, Instant)>,
    /// Serializes config read-modify-write transactions across command surfaces.
    pub config_write_lock: Mutex<()>,
    /// Cached Codex ACP status to avoid repeated adapter startups during UI refreshes
    pub codex_acp_status_cache: Mutex<(Option<serde_json::Value>, Instant)>,
    /// Bearer token for HTTP API authentication (generated at startup)
    pub api_token: String,
}

#[cfg(not(test))]
pub(crate) fn runtime_data_dir(_root: &std::path::Path) -> PathBuf {
    if let Ok(path) = std::env::var("AGENT_OS_DATA_DIR") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join("AppData")
                .join("Local")
        })
        .join("AgentOS")
}

#[cfg(test)]
pub(crate) fn runtime_data_dir(root: &std::path::Path) -> PathBuf {
    root.to_path_buf()
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        #[cfg(test)]
        let (data_dir, config_path, segments_file, tasks_dir) = (
            root.clone(),
            root.join("n8n").join("config.json"),
            root.join("n8n").join("dashboard").join("segments.json"),
            root.join("tasks"),
        );
        #[cfg(not(test))]
        let (data_dir, config_path, segments_file, tasks_dir) = {
            let data_dir = runtime_data_dir(&root);
            let config_path = data_dir.join("config.json");
            let segments_file = data_dir.join("segments.json");
            let tasks_dir = data_dir.join("tasks");
            if let Err(error) = Self::migrate_runtime_data(
                &root,
                &data_dir,
                &config_path,
                &segments_file,
                &tasks_dir,
            ) {
                crate::log_error!("[state] runtime data migration failed: {}", error);
            }
            (data_dir, config_path, segments_file, tasks_dir)
        };
        let docs_dir = Self::load_docs_dir(&config_path);
        let chats_dir = tasks_dir.join("chats");
        let n8n_url = Self::load_n8n_url(&config_path);

        // Load segments
        let (segments, project_segment) = Self::load_segments(&segments_file);

        // Ensure chats dir exists
        let _ = std::fs::create_dir_all(&chats_dir);
        let _ = std::fs::create_dir_all(&tasks_dir);
        if let Err(error) = std::fs::write(tasks_dir.join(".running-tasks.json"), "{}") {
            crate::log_error!("[state] cannot initialize running task state: {}", error);
        }

        // Update project_root in config to match detected root
        Self::update_project_root(&config_path, &root);

        let delegations_path = tasks_dir.join(".delegations.json");
        let sessions_path = tasks_dir.join(".sessions.json");
        let session_events_path = tasks_dir.join(".session-events.jsonl");
        let project_sessions_path = tasks_dir.join(".project-sessions.json");
        let work_items_path = tasks_dir.join(".work-items.json");
        let file_leases_path = tasks_dir.join(".file-leases.json");
        let runtime_snapshot_path = tasks_dir.join(".runtime-state.json");

        let (delegations, sessions, project_sessions, work_items, file_leases, revision) =
            Self::load_runtime_state(
                &runtime_snapshot_path,
                &delegations_path,
                &sessions_path,
                &project_sessions_path,
                &work_items_path,
                &file_leases_path,
            );
        let api_token = Self::load_or_create_api_token(&data_dir);

        let state = Self {
            root,
            data_dir,
            tasks_dir,
            docs_dir,
            config_path,
            segments_path: segments_file,
            chats_dir,
            delegations_path,
            sessions_path,
            session_events_path,
            project_sessions_path,
            work_items_path,
            file_leases_path,
            runtime_snapshot_path,
            n8n_url,
            start_time: Instant::now(),
            scan_cache: Mutex::new(ScanCache::default()),
            scan_refresh_lock: Mutex::new(()),
            segments: Mutex::new(segments),
            project_segment: Mutex::new(project_segment),
            delegations: Mutex::new(delegations),
            sessions: Mutex::new(sessions),
            project_sessions: Mutex::new(project_sessions),
            work_items: Mutex::new(work_items),
            file_leases: Mutex::new(file_leases),
            runtime_persistence: Mutex::new(revision),
            running_pids: Mutex::new(HashMap::new()),
            background_tasks: Mutex::new(HashMap::new()),
            shutdown_requested: AtomicBool::new(false),
            chat_cancellations: Mutex::new(HashSet::new()),
            dir_busy: Mutex::new(std::collections::HashSet::new()),
            activities: Mutex::new(HashMap::new()),
            operations: Mutex::new(HashMap::new()),
            inbox: Mutex::new(Vec::new()),
            config_cache: Mutex::new((
                serde_json::json!({}),
                Instant::now() - std::time::Duration::from_secs(100),
            )),
            config_write_lock: Mutex::new(()),
            codex_acp_status_cache: Mutex::new((None, Instant::now() - Duration::from_secs(300))),
            api_token,
        };
        state.save_runtime_snapshot();
        state
    }

    fn migrate_runtime_data(
        root: &std::path::Path,
        data_dir: &std::path::Path,
        config_path: &std::path::Path,
        segments_path: &std::path::Path,
        tasks_dir: &std::path::Path,
    ) -> Result<(), String> {
        std::fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
        Self::copy_file_if_missing(&root.join("n8n").join("config.json"), config_path)?;
        Self::copy_file_if_missing(
            &root.join("n8n").join("dashboard").join("segments.json"),
            segments_path,
        )?;
        Self::copy_directory(&root.join("tasks"), tasks_dir)?;
        std::fs::create_dir_all(tasks_dir).map_err(|error| error.to_string())?;
        if !config_path.exists() {
            std::fs::write(config_path, "{}").map_err(|error| error.to_string())?;
        }
        if !segments_path.exists() {
            std::fs::write(segments_path, r#"{"segments":{}}"#)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn copy_file_if_missing(
        source: &std::path::Path,
        target: &std::path::Path,
    ) -> Result<(), String> {
        if target.exists() || !source.exists() {
            return Ok(());
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::copy(source, target)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn copy_directory(source: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
        if !source.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(target).map_err(|error| error.to_string())?;
        for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());
            if source_path.is_dir() {
                Self::copy_directory(&source_path, &target_path)?;
            } else if !target_path.exists() {
                std::fs::copy(&source_path, &target_path).map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }

    fn generate_token() -> String {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).expect("OS random source unavailable");
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn load_or_create_api_token(data_dir: &std::path::Path) -> String {
        let path = data_dir.join("api-token");
        if let Ok(token) = std::fs::read_to_string(&path) {
            let token = token.trim();
            if token.len() >= 32 && token.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return token.to_string();
            }
        }
        let token = Self::generate_token();
        if let Err(error) = crate::commands::claude_runner::atomic_write(&path, &token) {
            crate::log_error!("[state] cannot persist API token: {}", error);
        }
        token
    }

    fn load_docs_dir(config_path: &PathBuf) -> PathBuf {
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(dir) = cfg.get("documents_dir").and_then(|v| v.as_str()) {
                    return PathBuf::from(dir);
                }
            }
        }
        dirs::document_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Documents"))
    }

    fn update_project_root(config_path: &PathBuf, root: &PathBuf) {
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(mut cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                let current = cfg
                    .get("project_root")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let root_str = root.to_string_lossy();
                if current != root_str.as_ref() {
                    cfg["project_root"] = serde_json::json!(root_str.as_ref());
                    if let Err(error) = crate::commands::claude_runner::atomic_write(
                        config_path,
                        &serde_json::to_string_pretty(&cfg).unwrap_or_default(),
                    ) {
                        crate::log_warn!("[state] project root save failed: {}", error);
                    }
                }
            }
        }
    }

    fn load_n8n_url(config_path: &PathBuf) -> String {
        // Priority: config.json n8n_url → N8N_URL env → default
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(url) = cfg.get("n8n_url").and_then(|v| v.as_str()) {
                    if !url.is_empty() {
                        return url.to_string();
                    }
                }
            }
        }
        std::env::var("N8N_URL").unwrap_or_else(|_| "http://localhost:5678".to_string())
    }

    fn load_segments(path: &PathBuf) -> (HashMap<String, Vec<String>>, HashMap<String, String>) {
        let mut segments: HashMap<String, Vec<String>> = HashMap::new();
        let mut project_segment: HashMap<String, String> = HashMap::new();

        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(segs) = data.get("segments").and_then(|v| v.as_object()) {
                    for (seg_name, projects) in segs {
                        if let Some(arr) = projects.as_array() {
                            let names: Vec<String> = arr
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                            for name in &names {
                                project_segment.insert(name.clone(), seg_name.clone());
                            }
                            segments.insert(seg_name.clone(), names);
                        }
                    }
                }
            }
        }

        (segments, project_segment)
    }

    fn load_delegations(path: &PathBuf) -> HashMap<String, Delegation> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, Delegation>>(&content) {
                return map;
            }
        }
        HashMap::new()
    }

    fn load_sessions(path: &PathBuf) -> HashMap<String, MultiAgentSession> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, MultiAgentSession>>(&content) {
                return map;
            }
        }
        HashMap::new()
    }

    fn load_project_sessions(path: &PathBuf) -> HashMap<String, ProjectSession> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, ProjectSession>>(&content) {
                return map;
            }
        }
        HashMap::new()
    }

    fn load_work_items(path: &PathBuf) -> HashMap<String, WorkItem> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, WorkItem>>(&content) {
                return map;
            }
        }
        HashMap::new()
    }

    fn load_file_leases(path: &PathBuf) -> HashMap<String, FileLease> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, FileLease>>(&content) {
                return map;
            }
        }
        HashMap::new()
    }

    #[allow(clippy::type_complexity)]
    fn load_runtime_state(
        snapshot_path: &PathBuf,
        delegations_path: &PathBuf,
        sessions_path: &PathBuf,
        project_sessions_path: &PathBuf,
        work_items_path: &PathBuf,
        file_leases_path: &PathBuf,
    ) -> (
        HashMap<String, Delegation>,
        HashMap<String, MultiAgentSession>,
        HashMap<String, ProjectSession>,
        HashMap<String, WorkItem>,
        HashMap<String, FileLease>,
        u64,
    ) {
        let loaded = std::fs::read_to_string(snapshot_path)
            .ok()
            .and_then(|content| serde_json::from_str::<RuntimeSnapshot>(&content).ok())
            .filter(|snapshot| snapshot.schema_version == RUNTIME_SNAPSHOT_SCHEMA);
        let (
            mut delegations,
            sessions,
            project_sessions,
            mut work_items,
            mut file_leases,
            revision,
        ) = match loaded {
            Some(snapshot) => (
                snapshot.delegations,
                snapshot.sessions,
                snapshot.project_sessions,
                snapshot.work_items,
                snapshot.file_leases,
                snapshot.revision,
            ),
            None => (
                Self::load_delegations(delegations_path),
                Self::load_sessions(sessions_path),
                Self::load_project_sessions(project_sessions_path),
                Self::load_work_items(work_items_path),
                Self::load_file_leases(file_leases_path),
                0,
            ),
        };

        // No provider process survives an application restart. Requeue the unit of work and
        // release every stale write lease so the next run can make forward progress.
        for delegation in delegations.values_mut() {
            if matches!(
                delegation.status,
                crate::commands::status::DelegationStatus::Running
                    | crate::commands::status::DelegationStatus::Escalated
                    | crate::commands::status::DelegationStatus::Deciding
                    | crate::commands::status::DelegationStatus::Verifying
            ) {
                delegation.status = crate::commands::status::DelegationStatus::Pending;
                delegation.started_at = None;
                delegation.escalation_info = Some(
                    "Recovered after AgentOS restart; execution was safely requeued".to_string(),
                );
            }
        }
        for item in work_items.values_mut() {
            if matches!(
                item.status,
                WorkItemStatus::Running | WorkItemStatus::Reviewing
            ) {
                item.status = if item.delegation_id.is_some() {
                    WorkItemStatus::Queued
                } else {
                    WorkItemStatus::Ready
                };
                item.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
        let recovered_at = chrono::Utc::now().to_rfc3339();
        for lease in file_leases.values_mut() {
            if matches!(lease.status, FileLeaseStatus::Active) {
                lease.status = FileLeaseStatus::Released;
                lease.updated_at = recovered_at.clone();
                lease.released_at = Some(recovered_at.clone());
            }
        }
        (
            delegations,
            sessions,
            project_sessions,
            work_items,
            file_leases,
            revision,
        )
    }

    /// Validate project name — no path traversal, must exist under docs_dir
    pub fn validate_project(&self, project: &str) -> Result<std::path::PathBuf, String> {
        // Block path traversal
        if project.contains("..")
            || project.contains('/')
            || project.contains('\\')
            || project.contains(':')
            || project.contains('\0')
        {
            return Err(format!("Invalid project name: {}", project));
        }
        let path = self.docs_dir.join(project);
        if !path.exists() {
            return Err(format!("Project not found: {}", project));
        }
        // Canonicalize and verify containment
        let canon = path.canonicalize().map_err(|e| e.to_string())?;
        let docs_canon = self.docs_dir.canonicalize().map_err(|e| e.to_string())?;
        if !canon.starts_with(&docs_canon) {
            return Err(format!("Project path escapes documents dir: {}", project));
        }
        Ok(canon)
    }

    /// Validate project name from LLM output against known project list (uses cache, no rescan)
    pub fn validate_project_name_from_llm(&self, name: &str) -> Option<String> {
        // Try cache first (fast path)
        let cache = self.scan_cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(data) = &cache.data {
            if let Some(arr) = data
                .get("agents")
                .and_then(|a| a.as_array())
                .or_else(|| data.as_array())
            {
                for v in arr {
                    if let Some(pname) = v.get("name").and_then(|n| n.as_str()) {
                        if pname.eq_ignore_ascii_case(name) {
                            return Some(pname.to_string());
                        }
                    }
                }
            }
            return None; // cache exists but name not found
        }
        drop(cache);
        // Cache empty — fallback to scan (rare, only on first call)
        let ps = self
            .project_segment
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let projects = crate::scanner::scan_projects(&self.docs_dir, &ps);
        projects
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .map(|p| p.name.clone())
    }

    pub fn save_delegations(&self) {
        self.save_runtime_snapshot();
        if let Ok(delegations) = self.delegations.lock() {
            self.save_json(&self.delegations_path, &*delegations, "delegations");
        }
    }

    pub fn save_sessions(&self) {
        self.save_runtime_snapshot();
        if let Ok(sessions) = self.sessions.lock() {
            self.save_json(&self.sessions_path, &*sessions, "sessions");
        }
    }

    pub fn save_project_sessions(&self) {
        self.save_runtime_snapshot();
        if let Ok(project_sessions) = self.project_sessions.lock() {
            self.save_json(
                &self.project_sessions_path,
                &*project_sessions,
                "project sessions",
            );
        }
    }

    pub fn save_work_items(&self) {
        self.save_runtime_snapshot();
        if let Ok(work_items) = self.work_items.lock() {
            self.save_json(&self.work_items_path, &*work_items, "work items");
        }
    }

    pub fn save_file_leases(&self) {
        self.save_runtime_snapshot();
        if let Ok(file_leases) = self.file_leases.lock() {
            self.save_json(&self.file_leases_path, &*file_leases, "file leases");
        }
    }

    pub fn save_runtime_snapshot(&self) {
        let mut revision = match self.runtime_persistence.lock() {
            Ok(value) => value,
            Err(error) => error.into_inner(),
        };
        let snapshot = RuntimeSnapshot {
            schema_version: RUNTIME_SNAPSHOT_SCHEMA,
            revision: revision.saturating_add(1),
            saved_at: self.now_iso(),
            delegations: self
                .delegations
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            sessions: self
                .sessions
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            project_sessions: self
                .project_sessions
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            work_items: self
                .work_items
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            file_leases: self
                .file_leases
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
        };
        let content = match serde_json::to_string_pretty(&snapshot) {
            Ok(content) => content,
            Err(error) => {
                crate::log_error!("[state] runtime snapshot serialization failed: {}", error);
                return;
            }
        };
        match crate::commands::claude_runner::atomic_write(&self.runtime_snapshot_path, &content) {
            Ok(()) => *revision = snapshot.revision,
            Err(error) => crate::log_error!(
                "[state] runtime snapshot save failed at {}: {}",
                self.runtime_snapshot_path.display(),
                error
            ),
        }
    }

    fn save_json<T: serde::Serialize>(&self, path: &std::path::Path, value: &T, label: &str) {
        let content = match serde_json::to_string_pretty(value) {
            Ok(content) => content,
            Err(error) => {
                crate::log_error!("[state] {} serialization failed: {}", label, error);
                return;
            }
        };
        if let Err(error) = crate::commands::claude_runner::atomic_write(path, &content) {
            crate::log_error!(
                "[state] {} save failed at {}: {}",
                label,
                path.display(),
                error
            );
        }
    }

    pub fn append_session_event(&self, event: &SessionEvent) {
        if let Ok(value) = serde_json::to_value(event) {
            crate::commands::jsonl::append_jsonl_logged(
                &self.session_events_path,
                &value,
                "session event",
            );
        }
    }

    pub fn get_session_events(&self, session_id: &str, limit: usize) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        let scan_lines = limit.saturating_mul(20).max(200);
        if let Ok(lines) = crate::commands::jsonl::read_recent_lines(
            &self.session_events_path,
            scan_lines,
            crate::commands::jsonl::RECENT_READ_MAX_BYTES,
        ) {
            for line in lines {
                if let Ok(event) = serde_json::from_str::<SessionEvent>(&line) {
                    if event.session_id == session_id {
                        events.push(event);
                        if events.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }
        events.reverse();
        events
    }

    pub fn get_orch_dir(&self) -> (String, PathBuf) {
        let mut orch_name = String::new();
        {
            let cfg = self.config();
            if let Some(name) = cfg.get("orchestrator_project").and_then(|v| v.as_str()) {
                orch_name = name.to_string();
            }
        }

        if !orch_name.is_empty() {
            let orch_dir = self.docs_dir.join(&orch_name);
            if orch_dir.exists() {
                return (orch_name, orch_dir);
            }
        }

        (String::new(), self.root.clone())
    }

    pub fn now_iso(&self) -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    /// Read config.json with 5-second cache TTL. Avoids 13+ disk reads per message.
    pub fn config(&self) -> serde_json::Value {
        let mut cache = self.config_cache.lock().unwrap_or_else(|e| e.into_inner());
        if cache.1.elapsed().as_secs() < 5 {
            return cache.0.clone();
        }
        let val = std::fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(serde_json::json!({}));
        *cache = (val.clone(), Instant::now());
        val
    }

    pub fn update_config<F>(&self, update: F) -> Result<serde_json::Value, String>
    where
        F: FnOnce(&mut serde_json::Value) -> Result<(), String>,
    {
        let _write_guard = self
            .config_write_lock
            .lock()
            .map_err(|_| "config write lock poisoned".to_string())?;
        let mut value = std::fs::read_to_string(&self.config_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        update(&mut value)?;
        let content = serde_json::to_string_pretty(&value)
            .map_err(|error| format!("Cannot serialize config: {error}"))?;
        crate::commands::claude_runner::atomic_write(&self.config_path, &content)
            .map_err(|error| format!("Cannot save config: {error}"))?;
        let mut cache = self
            .config_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *cache = (value.clone(), Instant::now());
        Ok(value)
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
    }

    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Acquire)
    }

    /// Wait until directory is not busy, then mark it busy.
    /// Call `release_dir` when done. Prevents concurrent claude in same project.
    pub fn acquire_dir_lock(&self, dir_key: &str) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if self.is_shutdown_requested() {
                return Err("AgentOS is shutting down".to_string());
            }
            if let Ok(mut busy) = self.dir_busy.lock() {
                if !busy.contains(dir_key) {
                    busy.insert(dir_key.to_string());
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(format!("Timed out waiting for project lock: {dir_key}"));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    pub fn acquire_dir_guard(&self, dir_key: &str) -> Result<DirLockGuard<'_>, String> {
        self.acquire_dir_lock(dir_key)?;
        Ok(DirLockGuard {
            state: self,
            key: dir_key.to_string(),
        })
    }

    pub fn release_dir_lock(&self, dir_key: &str) {
        if let Ok(mut busy) = self.dir_busy.lock() {
            busy.remove(dir_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agentos-state-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn concurrent_config_updates_preserve_every_field() {
        let root = temp_root("config");
        std::fs::create_dir_all(root.join("n8n")).expect("create config directory");
        std::fs::write(root.join("n8n").join("config.json"), "{}").expect("seed config");
        let state = Arc::new(AppState::new(root.clone()));
        let mut workers = Vec::new();
        for index in 0..8 {
            let state = state.clone();
            workers.push(std::thread::spawn(move || {
                state
                    .update_config(|config| {
                        config[format!("field_{index}")] = serde_json::json!(index);
                        Ok(())
                    })
                    .expect("config update");
            }));
        }
        for worker in workers {
            worker.join().expect("config worker");
        }
        let saved: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("n8n").join("config.json")).expect("read config"),
        )
        .expect("parse config");
        for index in 0..8 {
            assert_eq!(saved[format!("field_{index}")], index);
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn api_tokens_are_random_256_bit_hex_values() {
        let first = AppState::generate_token();
        let second = AppState::generate_token();
        assert_eq!(first.len(), 64);
        assert!(first.chars().all(|character| character.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn api_token_is_stable_for_the_same_data_directory() {
        let root = temp_root("api-token");
        std::fs::create_dir_all(root.join("n8n")).expect("create config directory");
        std::fs::write(root.join("n8n").join("config.json"), "{}").expect("seed config");
        let first = AppState::new(root.clone()).api_token;
        let second = AppState::new(root.clone()).api_token;
        assert_eq!(first, second);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restart_requeues_active_work_and_releases_leases() {
        let root = temp_root("runtime-recovery");
        std::fs::create_dir_all(root.join("n8n")).expect("create config directory");
        std::fs::create_dir_all(root.join("tasks")).expect("create tasks directory");
        std::fs::write(root.join("n8n").join("config.json"), "{}").expect("seed config");
        let snapshot = serde_json::json!({
            "schema_version": RUNTIME_SNAPSHOT_SCHEMA,
            "revision": 7,
            "saved_at": "2026-07-11T00:00:00Z",
            "delegations": {"d1": {
                "id":"d1", "project":"ProjectA", "task":"work", "ts":"2026-07-11T00:00:00Z",
                "status":"verifying", "retries":0
            }},
            "sessions": {},
            "project_sessions": {},
            "work_items": {"w1": {
                "id":"w1", "parent_room_session_id":"s1", "project":"ProjectA",
                "title":"work", "task":"work", "executor_provider":"codex", "status":"running",
                "delegation_id":"d1", "created_at":"2026-07-11T00:00:00Z", "updated_at":"2026-07-11T00:00:00Z"
            }},
            "file_leases": {"l1": {
                "id":"l1", "session_id":"s1", "work_item_id":"w1", "project":"ProjectA",
                "participant_id":"p1", "provider":"codex", "write_intent":"exclusive_write",
                "paths":["src"], "status":"active", "created_at":"2026-07-11T00:00:00Z",
                "updated_at":"2026-07-11T00:00:00Z"
            }}
        });
        std::fs::write(
            root.join("tasks").join(".runtime-state.json"),
            serde_json::to_string_pretty(&snapshot).unwrap(),
        )
        .expect("seed runtime snapshot");

        let state = AppState::new(root.clone());
        assert_eq!(
            state.delegations.lock().unwrap()["d1"].status,
            crate::commands::status::DelegationStatus::Pending
        );
        assert!(matches!(
            state.work_items.lock().unwrap()["w1"].status,
            WorkItemStatus::Queued
        ));
        assert!(matches!(
            state.file_leases.lock().unwrap()["l1"].status,
            FileLeaseStatus::Released
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_migration_copies_legacy_data_without_removing_source() {
        let root = temp_root("migration");
        let data_dir = root.join("local-data");
        std::fs::create_dir_all(root.join("n8n").join("dashboard")).expect("create legacy config");
        std::fs::create_dir_all(root.join("tasks").join("chats")).expect("create legacy tasks");
        std::fs::write(root.join("n8n").join("config.json"), r#"{"value":1}"#)
            .expect("seed config");
        std::fs::write(
            root.join("n8n").join("dashboard").join("segments.json"),
            r#"{"segments":{"Active":["ProjectA"]}}"#,
        )
        .expect("seed segments");
        let legacy_chat = root.join("tasks").join("chats").join("ProjectA.jsonl");
        std::fs::write(&legacy_chat, "{\"role\":\"user\",\"msg\":\"hello\"}\n").expect("seed chat");

        AppState::migrate_runtime_data(
            &root,
            &data_dir,
            &data_dir.join("config.json"),
            &data_dir.join("segments.json"),
            &data_dir.join("tasks"),
        )
        .expect("migrate data");

        assert!(data_dir.join("config.json").exists());
        assert!(data_dir.join("segments.json").exists());
        assert!(data_dir
            .join("tasks")
            .join("chats")
            .join("ProjectA.jsonl")
            .exists());
        assert!(legacy_chat.exists());
        std::fs::write(data_dir.join("config.json"), r#"{"value":"local"}"#)
            .expect("change migrated config");
        std::fs::write(
            root.join("n8n").join("config.json"),
            r#"{"value":"legacy-new"}"#,
        )
        .expect("change legacy config");
        AppState::migrate_runtime_data(
            &root,
            &data_dir,
            &data_dir.join("config.json"),
            &data_dir.join("segments.json"),
            &data_dir.join("tasks"),
        )
        .expect("repeat migration");
        assert!(std::fs::read_to_string(data_dir.join("config.json"))
            .expect("read local config")
            .contains("local"));
        let _ = std::fs::remove_dir_all(root);
    }
}
