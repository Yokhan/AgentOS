use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
    pub docs_dir: PathBuf,
    pub config_path: PathBuf,
    pub chats_dir: PathBuf,
    pub delegations_path: PathBuf,
    pub sessions_path: PathBuf,
    pub session_events_path: PathBuf,
    pub project_sessions_path: PathBuf,
    pub work_items_path: PathBuf,
    pub file_leases_path: PathBuf,
    pub n8n_url: String,
    pub start_time: Instant,

    // Per-directory busy flags: prevents two claude processes in same project dir
    pub dir_busy: Mutex<std::collections::HashSet<String>>,

    // Caches
    pub scan_cache: Mutex<ScanCache>,
    pub segments: Mutex<HashMap<String, Vec<String>>>,
    pub project_segment: Mutex<HashMap<String, String>>,

    // Runtime state
    pub delegations: Mutex<HashMap<String, Delegation>>,
    pub sessions: Mutex<HashMap<String, MultiAgentSession>>,
    pub project_sessions: Mutex<HashMap<String, ProjectSession>>,
    pub work_items: Mutex<HashMap<String, WorkItem>>,
    pub file_leases: Mutex<HashMap<String, FileLease>>,
    /// Running child process PIDs by chat_key — used to kill zombies
    pub running_pids: Mutex<HashMap<String, u32>>,
    /// Chat keys explicitly cancelled by the user. Checked by agent loops between actions.
    pub chat_cancellations: Mutex<HashSet<String>>,
    /// Running activities by project — in-memory, no file race conditions
    pub activities: Mutex<HashMap<String, serde_json::Value>>,
    /// Agent feedback inbox — delegation results accumulate here
    pub inbox: Mutex<Vec<crate::commands::inbox::InboxItem>>,
    /// Cached config.json — refreshed every 5 seconds or after writes
    pub config_cache: Mutex<(serde_json::Value, Instant)>,
    /// Cached Codex ACP status to avoid repeated adapter startups during UI refreshes
    pub codex_acp_status_cache: Mutex<(Option<serde_json::Value>, Instant)>,
    /// Bearer token for HTTP API authentication (generated at startup)
    pub api_token: String,
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        let config_path = root.join("n8n").join("config.json");
        let docs_dir = Self::load_docs_dir(&config_path);
        let chats_dir = root.join("tasks").join("chats");
        let n8n_url = Self::load_n8n_url(&config_path);

        // Load segments
        let segments_file = root.join("n8n").join("dashboard").join("segments.json");
        let (segments, project_segment) = Self::load_segments(&segments_file);

        // Ensure chats dir exists
        let _ = std::fs::create_dir_all(&chats_dir);

        // Update project_root in config to match detected root
        Self::update_project_root(&config_path, &root);

        let delegations_path = root.join("tasks").join(".delegations.json");
        let sessions_path = root.join("tasks").join(".sessions.json");
        let session_events_path = root.join("tasks").join(".session-events.jsonl");
        let project_sessions_path = root.join("tasks").join(".project-sessions.json");
        let work_items_path = root.join("tasks").join(".work-items.json");
        let file_leases_path = root.join("tasks").join(".file-leases.json");

        // Load persisted delegations, reset "running" to "pending"
        let delegations = Self::load_delegations(&delegations_path);
        let sessions = Self::load_sessions(&sessions_path);
        let project_sessions = Self::load_project_sessions(&project_sessions_path);
        let work_items = Self::load_work_items(&work_items_path);
        let file_leases = Self::load_file_leases(&file_leases_path);

        Self {
            root,
            docs_dir,
            config_path,
            chats_dir,
            delegations_path,
            sessions_path,
            session_events_path,
            project_sessions_path,
            work_items_path,
            file_leases_path,
            n8n_url,
            start_time: Instant::now(),
            scan_cache: Mutex::new(ScanCache::default()),
            segments: Mutex::new(segments),
            project_segment: Mutex::new(project_segment),
            delegations: Mutex::new(delegations),
            sessions: Mutex::new(sessions),
            project_sessions: Mutex::new(project_sessions),
            work_items: Mutex::new(work_items),
            file_leases: Mutex::new(file_leases),
            running_pids: Mutex::new(HashMap::new()),
            chat_cancellations: Mutex::new(HashSet::new()),
            dir_busy: Mutex::new(std::collections::HashSet::new()),
            activities: Mutex::new(HashMap::new()),
            inbox: Mutex::new(Vec::new()),
            config_cache: Mutex::new((
                serde_json::json!({}),
                Instant::now() - std::time::Duration::from_secs(100),
            )),
            codex_acp_status_cache: Mutex::new((None, Instant::now() - Duration::from_secs(300))),
            api_token: Self::generate_token(),
        }
    }

    fn generate_token() -> String {
        // More entropy than DefaultHasher: multiple sources + high-res time
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut h);
        std::process::id().hash(&mut h);
        let a = h.finish();
        // Second hash with different seed
        std::time::Instant::now().hash(&mut h);
        let b = h.finish();
        format!("{:016x}{:016x}", a, b)
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
                    let _ = crate::commands::claude_runner::atomic_write(
                        config_path,
                        &serde_json::to_string_pretty(&cfg).unwrap_or_default(),
                    );
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
            if let Ok(mut map) = serde_json::from_str::<HashMap<String, Delegation>>(&content) {
                // Reset "running" to "pending" on restart
                for d in map.values_mut() {
                    if d.status == crate::commands::status::DelegationStatus::Running {
                        d.status = crate::commands::status::DelegationStatus::Pending;
                    }
                }
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
        if let Ok(delegations) = self.delegations.lock() {
            let _ = crate::commands::claude_runner::atomic_write(
                &self.delegations_path,
                &serde_json::to_string_pretty(&*delegations).unwrap_or_default(),
            );
        }
    }

    pub fn save_sessions(&self) {
        if let Ok(sessions) = self.sessions.lock() {
            let _ = crate::commands::claude_runner::atomic_write(
                &self.sessions_path,
                &serde_json::to_string_pretty(&*sessions).unwrap_or_default(),
            );
        }
    }

    pub fn save_project_sessions(&self) {
        if let Ok(project_sessions) = self.project_sessions.lock() {
            let _ = crate::commands::claude_runner::atomic_write(
                &self.project_sessions_path,
                &serde_json::to_string_pretty(&*project_sessions).unwrap_or_default(),
            );
        }
    }

    pub fn save_work_items(&self) {
        if let Ok(work_items) = self.work_items.lock() {
            let _ = crate::commands::claude_runner::atomic_write(
                &self.work_items_path,
                &serde_json::to_string_pretty(&*work_items).unwrap_or_default(),
            );
        }
    }

    pub fn save_file_leases(&self) {
        if let Ok(file_leases) = self.file_leases.lock() {
            let _ = crate::commands::claude_runner::atomic_write(
                &self.file_leases_path,
                &serde_json::to_string_pretty(&*file_leases).unwrap_or_default(),
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
        if let Ok(content) = std::fs::read_to_string(&self.session_events_path) {
            for line in content.lines().rev() {
                if let Ok(evt) = serde_json::from_str::<SessionEvent>(line) {
                    if evt.session_id == session_id {
                        events.push(evt);
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

    /// Invalidate config cache (call after writing config)
    pub fn invalidate_config(&self) {
        if let Ok(mut cache) = self.config_cache.lock() {
            cache.1 = Instant::now() - std::time::Duration::from_secs(100);
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Wait until directory is not busy, then mark it busy.
    /// Call `release_dir` when done. Prevents concurrent claude in same project.
    pub fn acquire_dir_lock(&self, dir_key: &str) {
        loop {
            if let Ok(mut busy) = self.dir_busy.lock() {
                if !busy.contains(dir_key) {
                    busy.insert(dir_key.to_string());
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn release_dir_lock(&self, dir_key: &str) {
        if let Ok(mut busy) = self.dir_busy.lock() {
            busy.remove(dir_key);
        }
    }
}
