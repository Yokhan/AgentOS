use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::State;

use super::process_manager::{track_pid, untrack_pid_if_match};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Claude,
    Codex,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexTransport {
    Cli,
    Acp,
}

impl CodexTransport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Acp => "acp",
        }
    }
}

fn configured_codex_transport(state: &AppState) -> Option<CodexTransport> {
    state
        .config()
        .get("codex_transport")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "acp" => CodexTransport::Acp,
            _ => CodexTransport::Cli,
        })
}

fn codex_transport(state: &AppState) -> CodexTransport {
    if let Some(transport) = configured_codex_transport(state) {
        return transport;
    }
    if probe_binary(&resolve_codex_binary(state)).is_ok() {
        return CodexTransport::Cli;
    }
    if autodiscover_codex_acp_command(state).is_some() {
        return CodexTransport::Acp;
    }
    CodexTransport::Cli
}

pub fn parse_provider(value: Option<&str>, default: ProviderKind) -> ProviderKind {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "codex" => ProviderKind::Codex,
        "claude" => ProviderKind::Claude,
        _ => default,
    }
}

fn config_flag_enabled(state: &AppState, key: &str, default: bool) -> bool {
    state
        .config()
        .get(key)
        .and_then(|v| v.as_str())
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            !matches!(
                normalized.as_str(),
                "0" | "false" | "off" | "disabled" | "no"
            )
        })
        .unwrap_or(default)
}

pub fn claude_enabled(state: &AppState) -> bool {
    config_flag_enabled(state, "claude_enabled", true)
}

pub fn provider_enabled(state: &AppState, provider: ProviderKind) -> bool {
    match provider {
        ProviderKind::Claude => claude_enabled(state),
        ProviderKind::Codex => true,
    }
}

fn provider_with_fallback(state: &AppState, provider: ProviderKind) -> ProviderKind {
    if provider_enabled(state, provider) {
        provider
    } else {
        ProviderKind::Codex
    }
}

pub fn orchestrator_provider(state: &AppState) -> ProviderKind {
    let cfg = state.config();
    provider_with_fallback(
        state,
        parse_provider(
            cfg.get("orchestrator_provider").and_then(|v| v.as_str()),
            ProviderKind::Claude,
        ),
    )
}

pub fn technical_reviewer_provider(state: &AppState) -> ProviderKind {
    let cfg = state.config();
    provider_with_fallback(
        state,
        parse_provider(
            cfg.get("technical_reviewer_provider")
                .and_then(|v| v.as_str()),
            ProviderKind::Codex,
        ),
    )
}

pub fn delegation_provider(state: &AppState) -> ProviderKind {
    let cfg = state.config();
    provider_with_fallback(
        state,
        parse_provider(
            cfg.get("delegation_provider").and_then(|v| v.as_str()),
            orchestrator_provider(state),
        ),
    )
}

pub fn single_chat_provider(state: &AppState, explicit_provider: Option<&str>) -> ProviderKind {
    let default = orchestrator_provider(state);
    let explicit = explicit_provider.filter(|value| !value.trim().is_empty());
    provider_with_fallback(state, parse_provider(explicit, default))
}

pub fn resolve_single_chat_settings(
    state: &AppState,
    project: &str,
    explicit_provider: Option<&str>,
    explicit_model: Option<&str>,
    explicit_effort: Option<&str>,
) -> (ProviderKind, Option<String>, Option<String>) {
    let provider = single_chat_provider(state, explicit_provider);
    if project.trim().is_empty() {
        return (
            provider,
            resolve_provider_model(state, provider, explicit_model, Some("orchestrator_model")),
            resolve_provider_effort(
                state,
                provider,
                explicit_effort,
                Some("orchestrator_effort"),
            ),
        );
    }
    (
        provider,
        resolve_provider_model(state, provider, explicit_model, None),
        resolve_provider_effort(state, provider, explicit_effort, None),
    )
}

pub fn resolve_delegation_model(state: &AppState, provider: ProviderKind) -> Option<String> {
    let cfg = state.config();
    match provider {
        ProviderKind::Claude => cfg
            .get("delegation_model")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from),
        ProviderKind::Codex => cfg
            .get("delegation_codex_model")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or_else(|| {
                cfg.get("codex_model")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from)
            }),
    }
}

pub fn resolve_delegation_effort(state: &AppState, provider: ProviderKind) -> Option<String> {
    let cfg = state.config();
    match provider {
        ProviderKind::Claude => cfg
            .get("delegation_effort")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from),
        ProviderKind::Codex => cfg
            .get("delegation_codex_effort")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or_else(|| {
                cfg.get("codex_effort")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from)
            }),
    }
}

pub fn resolve_provider_model(
    state: &AppState,
    provider: ProviderKind,
    explicit_model: Option<&str>,
    role_key: Option<&str>,
) -> Option<String> {
    if let Some(model) = explicit_model.filter(|m| !m.is_empty()) {
        return Some(model.to_string());
    }
    let cfg = state.config();
    if let Some(key) = role_key {
        if let Some(model) = cfg
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            return Some(model.to_string());
        }
    }
    match provider {
        ProviderKind::Claude => None,
        ProviderKind::Codex => cfg
            .get("codex_model")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}

pub fn resolve_provider_effort(
    state: &AppState,
    provider: ProviderKind,
    explicit_effort: Option<&str>,
    role_key: Option<&str>,
) -> Option<String> {
    if let Some(effort) = explicit_effort.filter(|e| !e.is_empty()) {
        return Some(effort.to_string());
    }
    let cfg = state.config();
    if let Some(key) = role_key {
        if let Some(effort) = cfg
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            return Some(effort.to_string());
        }
    }
    match provider {
        ProviderKind::Claude => None,
        ProviderKind::Codex => cfg
            .get("codex_effort")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from),
    }
}

fn resolve_codex_binary(state: &AppState) -> String {
    let cfg = state.config();
    if let Some(bin) = cfg.get("codex_binary").and_then(|v| v.as_str()) {
        if !bin.trim().is_empty() {
            return bin.trim().to_string();
        }
    }
    if let Ok(bin) = std::env::var("CODEX_BIN") {
        if !bin.trim().is_empty() {
            return bin;
        }
    }
    if cfg!(target_os = "windows") {
        "codex.cmd".to_string()
    } else {
        "codex".to_string()
    }
}

fn resolve_codex_acp_command(state: &AppState) -> String {
    let cfg = state.config();
    if let Some(command) = cfg.get("codex_acp_command").and_then(|v| v.as_str()) {
        if !command.trim().is_empty() {
            return command.trim().to_string();
        }
    }
    if let Ok(command) = std::env::var("CODEX_ACP_COMMAND") {
        if !command.trim().is_empty() {
            return command.trim().to_string();
        }
    }
    if let Some(path) = autodiscover_codex_acp_command(state) {
        return path;
    }
    if cfg!(target_os = "windows") {
        "codex-acp.cmd".to_string()
    } else {
        "codex-acp".to_string()
    }
}

fn candidate_codex_acp_paths(base: &Path) -> Vec<PathBuf> {
    let root = base.join("codex-acp");
    let mut candidates = vec![root.join("current").join(codex_acp_exe_name())];
    let mut versioned: Vec<PathBuf> = match std::fs::read_dir(&root) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .map(|path| path.join(codex_acp_exe_name()))
            .filter(|path| path.is_file())
            .collect(),
        Err(_) => Vec::new(),
    };
    versioned.sort_by(|a, b| {
        b.parent()
            .and_then(|p| p.file_name())
            .cmp(&a.parent().and_then(|p| p.file_name()))
    });
    candidates.extend(versioned);
    candidates
}

fn codex_acp_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "codex-acp.exe"
    } else {
        "codex-acp"
    }
}

fn autodiscover_codex_acp_command(state: &AppState) -> Option<String> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        roots.push(PathBuf::from(local_app_data).join("AgentOS").join("tools"));
    }
    roots.push(state.root.join("tools"));

    for root in roots {
        for candidate in candidate_codex_acp_paths(&root) {
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn codex_acp_args(state: &AppState) -> Result<Vec<String>, String> {
    let raw = state
        .config()
        .get("codex_acp_args")
        .and_then(|v| v.as_str())
        .unwrap_or("[]")
        .trim()
        .to_string();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    if raw.starts_with('[') {
        serde_json::from_str::<Vec<String>>(&raw)
            .map_err(|e| format!("Invalid codex_acp_args JSON: {}", e))
    } else {
        Ok(raw
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<String>>())
    }
}

fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;
    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            b'=' => continue,
            _ => return None,
        } as u32;
        buffer = (buffer << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn jwt_payload(token: &str) -> Option<Value> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64url_decode(payload)?;
    serde_json::from_slice(&bytes).ok()
}

fn short_secret_tail(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() <= 8 {
        return trimmed.to_string();
    }
    format!("...{}", &trimmed[trimmed.len().saturating_sub(6)..])
}

fn account_display_from(email: Option<&str>, name: Option<&str>, fallback: Option<&str>) -> String {
    if let Some(email) = email.map(str::trim).filter(|v| !v.is_empty()) {
        return email.to_string();
    }
    if let Some(name) = name.map(str::trim).filter(|v| !v.is_empty()) {
        return name.to_string();
    }
    if let Some(fallback) = fallback.map(str::trim).filter(|v| !v.is_empty()) {
        return format!("account {}", short_secret_tail(fallback));
    }
    "unknown account".to_string()
}

fn codex_account_snapshot_from_auth_path(path: &Path) -> Value {
    let source = path.to_string_lossy().to_string();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return json!({
                "status": "missing",
                "source": source,
                "display": "not signed in",
                "error": error.to_string(),
            });
        }
    };
    let auth: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            return json!({
                "status": "error",
                "source": source,
                "display": "unreadable auth",
                "error": error.to_string(),
            });
        }
    };
    let auth_mode = auth
        .get("auth_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let tokens = auth.get("tokens").cloned().unwrap_or(Value::Null);
    let account_id = tokens
        .get("account_id")
        .and_then(|v| v.as_str())
        .or_else(|| auth.get("account_id").and_then(|v| v.as_str()));
    let payload = tokens
        .get("id_token")
        .and_then(|v| v.as_str())
        .and_then(jwt_payload)
        .unwrap_or(Value::Null);
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .or_else(|| auth.get("email").and_then(|v| v.as_str()));
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .or_else(|| auth.get("name").and_then(|v| v.as_str()));
    let api_key_tail = auth
        .get("OPENAI_API_KEY")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(short_secret_tail);
    let fallback = account_id.or(api_key_tail.as_deref());
    let display = account_display_from(email, name, fallback);

    json!({
        "status": "ok",
        "source": source,
        "auth_mode": auth_mode,
        "display": display,
        "email": email.unwrap_or(""),
        "name": name.unwrap_or(""),
        "account_id_tail": account_id.map(short_secret_tail).unwrap_or_default(),
        "api_key_tail": api_key_tail.unwrap_or_default(),
        "last_refresh": auth
            .get("last_refresh")
            .and_then(|v| v.as_str())
            .or_else(|| tokens.get("last_refresh").and_then(|v| v.as_str()))
            .unwrap_or(""),
    })
}

fn codex_account_snapshot() -> Value {
    let Some(home) = dirs::home_dir() else {
        return json!({
            "status": "missing",
            "display": "home not found",
        });
    };
    codex_account_snapshot_from_auth_path(&home.join(".codex").join("auth.json"))
}

pub fn resolve_provider_binary(state: &AppState, provider: ProviderKind) -> String {
    match provider {
        ProviderKind::Claude => super::claude_runner::find_claude(),
        ProviderKind::Codex => match codex_transport(state) {
            CodexTransport::Cli => resolve_codex_binary(state),
            CodexTransport::Acp => resolve_codex_acp_command(state),
        },
    }
}

fn probe_binary(binary: &str) -> Result<String, String> {
    match super::claude_runner::silent_cmd(binary)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                Ok("ok".to_string())
            } else {
                Ok(stdout)
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                Err(format!("exit code {:?}", output.status.code()))
            } else {
                Err(stderr)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn codex_template(state: &AppState) -> String {
    state
        .config()
        .get("codex_command_template")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn codex_allowed_efforts(model: Option<&str>) -> &'static [&'static str] {
    let model = model.unwrap_or("").trim().to_ascii_lowercase();
    if model.is_empty() {
        return &["none", "low", "medium", "high", "xhigh"];
    }
    if model.starts_with("gpt-5.5") || model.starts_with("gpt-5.4") || model == "gpt-5.2" {
        return &["none", "low", "medium", "high", "xhigh"];
    }
    if model == "gpt-5.3-codex" || model == "gpt-5.3-codex-spark" || model == "gpt-5.2-codex" {
        return &["low", "medium", "high", "xhigh"];
    }
    if model == "gpt-5.1-codex-max" {
        return &["none", "medium", "high", "xhigh"];
    }
    if model.starts_with("gpt-5.1") {
        return &["none", "low", "medium", "high"];
    }
    if model.starts_with("gpt-5") {
        return &["minimal", "low", "medium", "high"];
    }
    &["low", "medium", "high"]
}

fn codex_effort_config_arg(model: Option<&str>, effort: &str) -> Option<String> {
    let normalized = effort.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if !codex_allowed_efforts(model).contains(&normalized.as_str()) {
        return None;
    }
    Some(format!("model_reasoning_effort=\"{}\"", normalized))
}

fn codex_sandbox_for_permission_path(perm_path: Option<&str>) -> &'static str {
    let Some(path) = perm_path.filter(|p| !p.trim().is_empty()) else {
        return "read-only";
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return "read-only";
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return "read-only";
    };
    let allow = value
        .get("permissions")
        .and_then(|permissions| permissions.get("allow"))
        .and_then(|allow| allow.as_array())
        .cloned()
        .unwrap_or_default();
    let allowed: Vec<String> = allow
        .iter()
        .filter_map(|item| item.as_str().map(|s| s.to_ascii_lowercase()))
        .collect();
    let can_write = allowed
        .iter()
        .any(|item| item == "write" || item == "edit" || item == "notebookedit");
    if !can_write {
        return "read-only";
    }
    let full_bash = allowed.iter().any(|item| item == "bash");
    if full_bash {
        "danger-full-access"
    } else {
        "workspace-write"
    }
}

fn codex_wants_runtime_control(model: Option<&str>, reasoning_effort: Option<&str>) -> bool {
    model.filter(|m| !m.trim().is_empty()).is_some()
        || reasoning_effort.filter(|e| !e.trim().is_empty()).is_some()
}

fn provider_timeout(state: &AppState) -> Duration {
    let seconds = state
        .config()
        .get("provider_timeout_seconds")
        .and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse::<u64>().ok()))
        .unwrap_or(45 * 60)
        .clamp(30, 6 * 60 * 60);
    Duration::from_secs(seconds)
}

fn effective_codex_transport(
    state: &AppState,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
) -> CodexTransport {
    let configured = codex_transport(state);
    if configured == CodexTransport::Acp
        && codex_wants_runtime_control(model, reasoning_effort)
        && codex_can_run_via_cli(state)
    {
        return CodexTransport::Cli;
    }
    configured
}

fn extract_error_message_from_json_line(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        let json_part = trimmed
            .strip_prefix("ERROR:")
            .map(str::trim)
            .unwrap_or(trimmed);
        if !json_part.starts_with('{') {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(json_part) else {
            continue;
        };
        if let Some(message) = value
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(|message| message.as_str())
        {
            return Some(message.trim().to_string());
        }
        if let Some(message) = value.get("message").and_then(|message| message.as_str()) {
            return Some(message.trim().to_string());
        }
    }
    None
}

fn trim_provider_echo(raw: &str) -> String {
    let markers = [
        "\n[IDENTITY]",
        "\r\n[IDENTITY]",
        "\n[PROJECTS]",
        "\r\n[PROJECTS]",
        "\n[RECENT CONVERSATION]",
        "\r\n[RECENT CONVERSATION]",
        "\n[USER MESSAGE]",
        "\r\n[USER MESSAGE]",
        "\nuser\n[",
        "\r\nuser\r\n[",
    ];
    let mut cut = raw.len();
    for marker in markers {
        if let Some(index) = raw.find(marker) {
            cut = cut.min(index);
        }
    }
    raw[..cut].trim().to_string()
}

fn compact_provider_error(
    provider: &str,
    model: Option<&str>,
    transport: Option<CodexTransport>,
    raw: &str,
) -> String {
    let extracted = extract_error_message_from_json_line(raw);
    let mut detail = extracted.unwrap_or_else(|| trim_provider_echo(raw));
    if detail.is_empty() {
        detail = "Provider returned an error without details.".to_string();
    }
    if detail.len() > 1200 {
        detail.truncate(1200);
        detail.push_str("...");
    }

    let lower = detail.to_ascii_lowercase();
    let action = if lower.contains("newer version of codex") {
        "Action: update Codex CLI (`npm install -g @openai/codex@latest`) or switch to an older model."
    } else if lower.contains("authentication")
        || lower.contains("not logged in")
        || lower.contains("login")
        || lower.contains("unauthorized")
    {
        "Action: finish provider sign-in, then refresh Settings -> Providers."
    } else if lower.contains("unknown model")
        || lower.contains("model")
        || lower.contains("invalid_request_error")
    {
        "Action: pick a model that this provider account/runtime supports, then retry."
    } else {
        "Action: open Settings -> Providers, check the runtime status, and retry after fixing the provider."
    };

    let model_part = model
        .filter(|m| !m.trim().is_empty())
        .map(|m| format!(" model={}", m.trim()))
        .unwrap_or_default();
    let transport_part = transport
        .map(|t| format!(" transport={}", t.as_str()))
        .unwrap_or_default();

    format!(
        "Provider error: {}{}{}\n{}\nDetails: {}",
        provider, model_part, transport_part, action, detail
    )
}

fn probe_command_exists(binary: &str) -> Result<String, String> {
    if Path::new(binary).is_file() {
        return Ok(binary.to_string());
    }
    let output = if cfg!(target_os = "windows") {
        super::claude_runner::silent_cmd("where")
            .arg(binary)
            .output()
    } else {
        super::claude_runner::silent_cmd("which")
            .arg(binary)
            .output()
    };
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                Ok(binary.to_string())
            } else {
                Ok(stdout.lines().next().unwrap_or(binary).to_string())
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                Err(format!("'{}' not found", binary))
            } else {
                Err(stderr)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn acp_probe_root(state: &AppState) -> &Path {
    &state.root
}

fn codex_acp_initialize(
    state: &AppState,
) -> Result<
    (
        super::acp_client::AcpClient,
        super::acp_client::AcpInitStatus,
    ),
    String,
> {
    let command = resolve_codex_acp_command(state);
    let args = codex_acp_args(state)?;
    let mut client = super::acp_client::AcpClient::spawn(&command, &args, acp_probe_root(state))?;
    let init = client.initialize()?;
    Ok((client, init))
}

fn invalidate_codex_acp_status_cache(state: &AppState) {
    if let Ok(mut cache) = state.codex_acp_status_cache.lock() {
        cache.0 = None;
        cache.1 = Instant::now() - Duration::from_secs(300);
    }
}

fn codex_acp_status_snapshot(state: &AppState) -> Value {
    if let Ok(cache) = state.codex_acp_status_cache.lock() {
        if let Some(value) = &cache.0 {
            if cache.1.elapsed() < Duration::from_secs(20) {
                return value.clone();
            }
        }
    }

    let command = resolve_codex_acp_command(state);
    let args = match codex_acp_args(state) {
        Ok(args) => args,
        Err(e) => {
            return json!({
                "transport": "acp",
                "command": command,
                "args": [],
                "available": false,
                "ready": false,
                "authenticated": false,
                "probe": e,
                "auth_required": false,
                "auth_methods": [],
            });
        }
    };
    let exists = probe_command_exists(&command);
    if let Err(err) = exists {
        return json!({
            "transport": "acp",
            "command": command,
            "args": args,
            "available": false,
            "ready": false,
            "authenticated": false,
            "probe": err,
            "auth_required": false,
            "auth_methods": [],
        });
    }
    let snapshot = match codex_acp_initialize(state) {
        Ok((mut client, init)) => {
            let auth_methods = init.auth_methods.clone();
            match client.new_session(&state.root) {
                Ok(_) => json!({
                    "transport": "acp",
                    "command": command,
                    "args": args,
                    "available": true,
                    "ready": true,
                    "authenticated": Value::Null,
                    "probe": "ACP initialized and session probe passed",
                    "auth_required": false,
                    "auth_methods": auth_methods,
                    "agent_info": init.agent_info,
                    "agent_capabilities": init.agent_capabilities,
                    "protocol_version": init.protocol_version,
                    "session_probe_skipped": false,
                }),
                Err(err) => json!({
                    "transport": "acp",
                    "command": command,
                    "args": args,
                    "available": true,
                    "ready": false,
                    "authenticated": false,
                    "probe": format!("ACP initialized but cannot create a chat session: {}", err),
                    "auth_required": false,
                    "auth_methods": auth_methods,
                    "agent_info": init.agent_info,
                    "agent_capabilities": init.agent_capabilities,
                    "protocol_version": init.protocol_version,
                    "session_probe_skipped": false,
                }),
            }
        }
        Err(err) => json!({
            "transport": "acp",
            "command": command,
            "args": args,
            "available": true,
            "ready": false,
            "authenticated": false,
            "probe": err,
            "auth_required": false,
            "auth_methods": [],
        }),
    };

    if let Ok(mut cache) = state.codex_acp_status_cache.lock() {
        cache.0 = Some(snapshot.clone());
        cache.1 = Instant::now();
    }

    snapshot
}

fn codex_models_cache_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex").join("models_cache.json"))
}

fn model_slug_from_value(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        if trimmed.starts_with("gpt-") || trimmed.contains("codex") {
            return Some(trimmed.to_string());
        }
        return None;
    }
    let obj = value.as_object()?;
    for key in ["slug", "id", "name", "model"] {
        if let Some(slug) = obj.get(key).and_then(|v| v.as_str()) {
            let trimmed = slug.trim();
            if trimmed.starts_with("gpt-") || trimmed.contains("codex") {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn normalize_model_entry(value: &Value, source: &str) -> Option<Value> {
    let slug = model_slug_from_value(value)?;
    let display_name = value
        .get("display_name")
        .or_else(|| value.get("displayName"))
        .or_else(|| value.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or(&slug);
    Some(json!({
        "slug": slug,
        "display_name": display_name,
        "description": value.get("description").cloned().unwrap_or(Value::Null),
        "default_reasoning_level": value
            .get("default_reasoning_level")
            .or_else(|| value.get("defaultReasoningLevel"))
            .cloned()
            .unwrap_or(Value::Null),
        "supported_reasoning_levels": value
            .get("supported_reasoning_levels")
            .or_else(|| value.get("supportedReasoningLevels"))
            .or_else(|| value.get("reasoning_efforts"))
            .or_else(|| value.get("reasoningEfforts"))
            .cloned()
            .unwrap_or(Value::Array(vec![])),
        "source": source,
    }))
}

fn collect_model_entries(value: &Value, source: &str, out: &mut Vec<Value>) {
    match value {
        Value::Array(items) => {
            let model_like_count = items
                .iter()
                .filter(|item| model_slug_from_value(item).is_some())
                .count();
            if model_like_count > 0 {
                for item in items {
                    if let Some(entry) = normalize_model_entry(item, source) {
                        out.push(entry);
                    }
                }
                return;
            }
            for item in items {
                collect_model_entries(item, source, out);
            }
        }
        Value::Object(map) => {
            if let Some(entry) = normalize_model_entry(value, source) {
                out.push(entry);
                return;
            }
            for nested in map.values() {
                collect_model_entries(nested, source, out);
            }
        }
        _ => {}
    }
}

fn codex_cached_models() -> Vec<Value> {
    let Some(path) = codex_models_cache_path() else {
        return vec![];
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
        return vec![];
    };
    let mut models = Vec::new();
    collect_model_entries(
        parsed.get("models").unwrap_or(&parsed),
        "codex-cache",
        &mut models,
    );
    models
}

fn codex_available_models(codex_acp: &Value) -> Vec<Value> {
    let mut models = codex_cached_models();
    if let Some(capabilities) = codex_acp.get("agent_capabilities") {
        collect_model_entries(capabilities, "acp", &mut models);
    }
    let mut seen = HashSet::new();
    models
        .into_iter()
        .filter(|model| {
            let slug = model
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            !slug.is_empty() && seen.insert(slug)
        })
        .collect()
}

fn render_template_args(
    template: &str,
    prompt_file: &Path,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
) -> Result<Vec<String>, String> {
    let raw_args: Vec<String> = if template.trim_start().starts_with('[') {
        serde_json::from_str(template)
            .map_err(|e| format!("Invalid codex_command_template JSON: {}", e))?
    } else {
        template.split_whitespace().map(String::from).collect()
    };
    let prompt_path = prompt_file.to_string_lossy();
    Ok(raw_args
        .into_iter()
        .map(|part| {
            part.replace("{prompt_file}", &prompt_path)
                .replace("{model}", model.unwrap_or(""))
                .replace("{effort}", reasoning_effort.unwrap_or(""))
        })
        .filter(|part| !part.is_empty())
        .collect())
}

fn wait_with_optional_pid_tracking(
    mut cmd: std::process::Command,
    state: &AppState,
    chat_key: Option<&str>,
) -> std::io::Result<std::process::Output> {
    let child = cmd.spawn()?;
    let pid = child.id();
    if let Some(key) = chat_key {
        track_pid(state, key, pid);
        crate::log_info!("[provider:{}] spawned pid={}", key, pid);
    }

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    let started = Instant::now();
    let timeout = provider_timeout(state);
    let mut missing_since: Option<Instant> = None;
    let mut last_process_probe = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);
    let output = loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(output) => break output,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if chat_key
                    .map(|key| super::process_manager::is_cancelled(state, key))
                    .unwrap_or(false)
                {
                    crate::log_warn!(
                        "[provider:{}] cancelling pid={} after user stop",
                        chat_key.unwrap_or("_unknown"),
                        pid
                    );
                    kill_pid_tree(pid);
                    break Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "provider cancelled by user",
                    ));
                }
                if last_process_probe.elapsed() >= Duration::from_secs(1) {
                    last_process_probe = Instant::now();
                    if process_exists(pid) {
                        missing_since = None;
                    } else {
                        let since = missing_since.get_or_insert_with(Instant::now);
                        if since.elapsed() >= Duration::from_secs(3) {
                            crate::log_warn!(
                                "[provider:{}] pid={} disappeared but wait thread did not return",
                                chat_key.unwrap_or("_unknown"),
                                pid
                            );
                            break Err(io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "provider process exited but output pipe did not close",
                            ));
                        }
                    }
                }
                if started.elapsed() >= timeout {
                    crate::log_warn!(
                        "[provider:{}] timeout after {}s, killing pid={}",
                        chat_key.unwrap_or("_unknown"),
                        timeout.as_secs(),
                        pid
                    );
                    kill_pid_tree(pid);
                    break Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("provider timed out after {} seconds", timeout.as_secs()),
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "provider wait thread disconnected",
                ));
            }
        }
    };
    if let Some(key) = chat_key {
        untrack_pid_if_match(state, key, pid);
    }
    output
}

pub(crate) fn process_exists(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let filter = format!("PID eq {}", pid);
        let Ok(output) = super::claude_runner::silent_cmd("tasklist")
            .args(["/FI", &filter, "/FO", "CSV", "/NH"])
            .output()
        else {
            return true;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        output.status.success()
            && stdout
                .lines()
                .any(|line| line.contains(&format!("\",\"{}\",", pid)))
    }
    #[cfg(not(target_os = "windows"))]
    {
        super::claude_runner::silent_cmd("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|status| status.success())
            .unwrap_or(true)
    }
}

fn kill_pid_tree(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = super::claude_runner::silent_cmd("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = super::claude_runner::silent_cmd("kill")
            .args(["-9", &pid.to_string()])
            .output();
    }
}

fn run_codex_with_template(
    state: &AppState,
    cwd: &Path,
    prompt: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    chat_key: Option<&str>,
) -> String {
    let template = codex_template(state);
    if template.is_empty() {
        return compact_provider_error(
            "codex",
            model,
            Some(CodexTransport::Cli),
            "codex provider is selected but codex_command_template is not configured",
        );
    }

    let tmp = super::claude_runner::unique_tmp("codex");
    if std::fs::write(&tmp, prompt).is_err() {
        return compact_provider_error(
            "codex",
            model,
            Some(CodexTransport::Cli),
            "could not write codex temp file",
        );
    }

    let args = match render_template_args(&template, &tmp, model, reasoning_effort) {
        Ok(args) => args,
        Err(err) => {
            let _ = std::fs::remove_file(&tmp);
            return compact_provider_error("codex", model, Some(CodexTransport::Cli), &err);
        }
    };

    let binary = resolve_provider_binary(state, ProviderKind::Codex);
    let mut cmd = super::claude_runner::silent_cmd(&binary);
    cmd.args(args)
        .current_dir(cwd)
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let output = wait_with_optional_pid_tracking(cmd, state, chat_key);

    let _ = std::fs::remove_file(&tmp);

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                if stdout.is_empty() {
                    if stderr.is_empty() {
                        "Agent returned empty response".to_string()
                    } else {
                        stderr
                    }
                } else {
                    stdout
                }
            } else if !stderr.is_empty() {
                compact_provider_error("codex", model, Some(CodexTransport::Cli), &stderr)
            } else if !stdout.is_empty() {
                compact_provider_error("codex", model, Some(CodexTransport::Cli), &stdout)
            } else {
                compact_provider_error(
                    "codex",
                    model,
                    Some(CodexTransport::Cli),
                    &format!("codex exited with {:?}", output.status.code()),
                )
            }
        }
        Err(e) => compact_provider_error(
            "codex",
            model,
            Some(CodexTransport::Cli),
            &format!("Error running codex: {}", e),
        ),
    }
}

fn run_codex_official_cli(
    state: &AppState,
    cwd: &Path,
    prompt: &str,
    perm_path: Option<&str>,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    chat_key: Option<&str>,
) -> String {
    let binary = resolve_codex_binary(state);
    let prompt_file = super::claude_runner::unique_tmp("codex-prompt");
    let output_file = super::claude_runner::unique_tmp("codex-last");

    if std::fs::write(&prompt_file, prompt).is_err() {
        return compact_provider_error(
            "codex",
            model,
            Some(CodexTransport::Cli),
            "could not write Codex temp prompt file",
        );
    }

    let stdin_file = match std::fs::File::open(&prompt_file) {
        Ok(file) => file,
        Err(e) => {
            let _ = std::fs::remove_file(&prompt_file);
            return compact_provider_error(
                "codex",
                model,
                Some(CodexTransport::Cli),
                &format!("Error opening Codex temp prompt file: {}", e),
            );
        }
    };

    let mut cmd = super::claude_runner::silent_cmd(&binary);
    cmd.args(["exec", "--skip-git-repo-check", "-o"])
        .arg(&output_file)
        .args(["--sandbox", codex_sandbox_for_permission_path(perm_path)])
        .arg("-");

    if let Some(model) = model.filter(|m| !m.is_empty()) {
        cmd.args(["-m", model]);
    }
    if let Some(arg) = reasoning_effort.and_then(|effort| codex_effort_config_arg(model, effort)) {
        cmd.args(["-c", &arg]);
    }

    cmd.current_dir(cwd)
        .stdin(std::process::Stdio::from(stdin_file))
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let output = wait_with_optional_pid_tracking(cmd, state, chat_key);

    let response = match output {
        Ok(output) => {
            let last_message = std::fs::read_to_string(&output_file)
                .ok()
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty());
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                last_message
                    .or_else(|| (!stdout.is_empty()).then_some(stdout.clone()))
                    .or_else(|| (!stderr.is_empty()).then_some(stderr.clone()))
                    .unwrap_or_else(|| "Agent returned empty response".to_string())
            } else if !stderr.is_empty() {
                compact_provider_error("codex", model, Some(CodexTransport::Cli), &stderr)
            } else if !stdout.is_empty() {
                compact_provider_error("codex", model, Some(CodexTransport::Cli), &stdout)
            } else {
                compact_provider_error(
                    "codex",
                    model,
                    Some(CodexTransport::Cli),
                    &format!("codex exited with {:?}", output.status.code()),
                )
            }
        }
        Err(e) => compact_provider_error(
            "codex",
            model,
            Some(CodexTransport::Cli),
            &format!("Error running codex: {}", e),
        ),
    };

    let _ = std::fs::remove_file(&prompt_file);
    let _ = std::fs::remove_file(&output_file);
    response
}

fn run_codex_via_acp(
    state: &AppState,
    cwd: &Path,
    prompt: &str,
    model: Option<&str>,
    _reasoning_effort: Option<&str>,
) -> String {
    match codex_acp_initialize(state) {
        Ok((mut client, _init)) => match client.new_session(cwd) {
            Ok(session) => {
                let session_id = session
                    .get("sessionId")
                    .or_else(|| session.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if session_id.is_empty() {
                    return compact_provider_error(
                        "codex",
                        model,
                        Some(CodexTransport::Acp),
                        "ACP session/new did not return sessionId",
                    );
                }
                match client.prompt(&session_id, prompt) {
                    Ok(text) => text,
                    Err(err) => {
                        compact_provider_error("codex", model, Some(CodexTransport::Acp), &err)
                    }
                }
            }
            Err(err) => compact_provider_error("codex", model, Some(CodexTransport::Acp), &err),
        },
        Err(err) => compact_provider_error("codex", model, Some(CodexTransport::Acp), &err),
    }
}

fn codex_can_run_via_cli(state: &AppState) -> bool {
    probe_binary(&resolve_codex_binary(state)).is_ok()
}

pub fn run_provider_with_opts(
    state: &AppState,
    provider: ProviderKind,
    cwd: &Path,
    prompt: &str,
    perm_path: Option<&str>,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
) -> String {
    run_provider_with_chat_control(
        state,
        provider,
        cwd,
        prompt,
        perm_path,
        model,
        reasoning_effort,
        None,
    )
}

pub fn run_provider_with_chat_control(
    state: &AppState,
    provider: ProviderKind,
    cwd: &Path,
    prompt: &str,
    perm_path: Option<&str>,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    chat_key: Option<&str>,
) -> String {
    match provider {
        ProviderKind::Claude => {
            if !claude_enabled(state) {
                return compact_provider_error(
                    "claude",
                    model,
                    None,
                    "Claude is disabled in AgentOS settings. Enable it in Settings -> Providers, or route this task to Codex.",
                );
            }
            super::claude_runner::run_claude_with_opts(
                cwd,
                prompt,
                perm_path.unwrap_or(""),
                model,
                reasoning_effort,
            )
        }
        ProviderKind::Codex => match effective_codex_transport(state, model, reasoning_effort) {
            CodexTransport::Cli => {
                if codex_template(state).is_empty() {
                    run_codex_official_cli(
                        state,
                        cwd,
                        prompt,
                        perm_path,
                        model,
                        reasoning_effort,
                        chat_key,
                    )
                } else {
                    run_codex_with_template(state, cwd, prompt, model, reasoning_effort, chat_key)
                }
            }
            CodexTransport::Acp => run_codex_via_acp(state, cwd, prompt, model, reasoning_effort),
        },
    }
}

pub fn provider_status_snapshot(state: &AppState) -> Value {
    let cfg = state.config();
    let orchestrator = orchestrator_provider(state);
    let technical = technical_reviewer_provider(state);

    let claude_is_enabled = claude_enabled(state);
    let claude_binary = if claude_is_enabled {
        resolve_provider_binary(state, ProviderKind::Claude)
    } else {
        String::new()
    };
    let claude_probe = if claude_is_enabled {
        probe_binary(&claude_binary)
    } else {
        Err("disabled by AgentOS config".to_string())
    };
    let configured_transport = codex_transport(state);
    let codex_template = codex_template(state);
    let codex_binary = resolve_codex_binary(state);
    let codex_probe = probe_binary(&codex_binary);
    let codex_account = codex_account_snapshot();
    let codex_acp = if configured_transport == CodexTransport::Acp {
        codex_acp_status_snapshot(state)
    } else {
        let command = resolve_codex_acp_command(state);
        json!({
            "transport": "acp",
            "command": command,
            "args": [],
            "available": probe_command_exists(&resolve_codex_acp_command(state)).is_ok(),
            "ready": false,
            "authenticated": Value::Null,
            "probe": "ACP is not active. Switch Codex transport to ACP to run a full adapter/session probe.",
            "auth_required": false,
            "auth_methods": [],
            "session_probe_skipped": true,
        })
    };
    let codex_models = codex_available_models(&codex_acp);
    let codex_models_count = codex_models.len();
    let mut codex_model_sources: Vec<String> = Vec::new();
    for model in &codex_models {
        if let Some(source) = model.get("source").and_then(|v| v.as_str()) {
            if !codex_model_sources.iter().any(|item| item == source) {
                codex_model_sources.push(source.to_string());
            }
        }
    }
    let codex_models_source = if codex_model_sources.is_empty() {
        "fallback".to_string()
    } else {
        codex_model_sources.join("+")
    };
    let codex_model = cfg
        .get("codex_model")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let codex_effort = cfg
        .get("codex_effort")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let effective_transport = effective_codex_transport(
        state,
        (!codex_model.is_empty()).then_some(codex_model),
        (!codex_effort.is_empty()).then_some(codex_effort),
    );
    let acp_available = codex_acp
        .get("available")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let acp_ready = codex_acp
        .get("ready")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let cli_available = codex_probe.is_ok();
    let effective_ready = match effective_transport {
        CodexTransport::Cli => cli_available,
        CodexTransport::Acp => acp_ready,
    };
    let effective_available = match effective_transport {
        CodexTransport::Cli => cli_available,
        CodexTransport::Acp => acp_available,
    };
    let route_note = if configured_transport == CodexTransport::Acp
        && effective_transport == CodexTransport::Cli
    {
        "ACP is configured, but selected model/effort needs runtime control. AgentOS will use the Codex CLI fallback for this run."
    } else if effective_transport == CodexTransport::Cli {
        "AgentOS will use the official Codex CLI for this run."
    } else {
        "AgentOS will use the configured Codex ACP adapter for this run."
    };

    json!({
        "roles": {
            "orchestrator_provider": orchestrator.as_str(),
            "technical_reviewer_provider": technical.as_str(),
            "delegation_provider": delegation_provider(state).as_str(),
        },
        "providers": {
            "claude": {
                "enabled": claude_is_enabled,
                "disabled": !claude_is_enabled,
                "binary": claude_binary,
                "available": claude_is_enabled && claude_probe.is_ok(),
                "probe": claude_probe.clone().unwrap_or_else(|e| e),
                "model": cfg.get("orchestrator_model").and_then(|v| v.as_str()).unwrap_or(""),
                "effort": cfg.get("orchestrator_effort").and_then(|v| v.as_str()).unwrap_or(""),
            },
            "codex": {
                "transport": effective_transport.as_str(),
                "configured_transport": configured_transport.as_str(),
                "effective_transport": effective_transport.as_str(),
                "route_note": route_note,
                "binary": if effective_transport == CodexTransport::Cli { Value::String(codex_binary.clone()) } else { Value::Null },
                "available": effective_available,
                "probe": if effective_transport == CodexTransport::Cli { Value::String(codex_probe.clone().unwrap_or_else(|e| e)) } else { codex_acp.get("probe").cloned().unwrap_or(Value::String("ACP unavailable".to_string())) },
                "cli_available": cli_available,
                "cli_probe": codex_probe.clone().unwrap_or_else(|e| e),
                "cli_binary": codex_binary,
                "acp_available": acp_available,
                "acp_ready": acp_ready,
                "model": codex_model,
                "effort": codex_effort,
                "account": codex_account,
                "command_template": codex_template,
                "ready": effective_ready,
                "authenticated": codex_acp.get("authenticated").cloned().unwrap_or(Value::Null),
                "auth_required": codex_acp.get("auth_required").cloned().unwrap_or(Value::Null),
                "auth_methods": codex_acp.get("auth_methods").cloned().unwrap_or(Value::Array(vec![])),
                "acp_command": codex_acp.get("command").cloned().unwrap_or(Value::Null),
                "acp_args": codex_acp.get("args").cloned().unwrap_or(Value::Array(vec![])),
                "acp_probe": codex_acp.get("probe").cloned().unwrap_or(Value::String("ACP unavailable".to_string())),
                "models_count": codex_models_count,
                "models_source": codex_models_source,
                "models": codex_models,
            }
        },
        "notes": [
            "Codex CLI mode works out of the box with the official codex exec command and ChatGPT login.",
            "Optional codex_command_template still supports placeholders {prompt_file}, {model}, {effort}.",
            "Example JSON template: [\"exec\",\"-m\",\"{model}\",\"-c\",\"model_reasoning_effort=\\\"{effort}\\\"\",\"{prompt_file}\"]",
            "Codex ACP mode uses an external ACP adapter and lets the adapter own authentication.",
            "When ACP is selected but AgentOS needs explicit model/effort control, it falls back to official codex exec if CLI is available."
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agentos-provider-runner-{}-{}",
            name,
            std::process::id()
        ))
    }

    #[test]
    fn candidate_codex_acp_paths_prefers_latest_versioned_install() {
        let base = temp_path("codex-acp-candidates");
        let _ = std::fs::remove_dir_all(&base);
        let old = base
            .join("codex-acp")
            .join("v0.4.1")
            .join(codex_acp_exe_name());
        let new = base
            .join("codex-acp")
            .join("v0.4.2")
            .join(codex_acp_exe_name());
        std::fs::create_dir_all(old.parent().unwrap()).unwrap();
        std::fs::create_dir_all(new.parent().unwrap()).unwrap();
        std::fs::write(&old, b"old").unwrap();
        std::fs::write(&new, b"new").unwrap();

        let candidates = candidate_codex_acp_paths(&base);

        assert_eq!(
            candidates
                .into_iter()
                .filter(|path| path.is_file())
                .next()
                .unwrap(),
            new
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn probe_command_exists_accepts_absolute_existing_path() {
        let dir = temp_path("probe");
        let file = dir.join(codex_acp_exe_name());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&file, b"stub").unwrap();

        assert_eq!(
            probe_command_exists(file.to_string_lossy().as_ref()).unwrap(),
            file.to_string_lossy().to_string()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn codex_effort_matrix_is_model_aware() {
        assert!(codex_effort_config_arg(None, "xhigh").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5.4"), "xhigh").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5.5"), "xhigh").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5.2-codex"), "xhigh").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5.3-codex"), "none").is_none());
        assert!(codex_effort_config_arg(Some("gpt-5"), "minimal").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5"), "none").is_none());
        assert!(codex_effort_config_arg(Some("gpt-5.1-codex"), "none").is_some());
        assert!(codex_effort_config_arg(Some("gpt-5.1-codex-max"), "low").is_none());
    }

    #[test]
    fn model_entries_are_collected_from_codex_cache_shape() {
        let mut out = Vec::new();
        collect_model_entries(
            &json!({
                "models": [
                    {
                        "slug": "gpt-5.5",
                        "display_name": "GPT-5.5",
                        "supported_reasoning_levels": [
                            {"effort": "low"},
                            {"effort": "xhigh"}
                        ]
                    }
                ]
            }),
            "test",
            &mut out,
        );

        assert_eq!(
            out.first()
                .and_then(|model| model.get("slug"))
                .and_then(|slug| slug.as_str()),
            Some("gpt-5.5")
        );
        assert_eq!(
            out.first()
                .and_then(|model| model.get("source"))
                .and_then(|source| source.as_str()),
            Some("test")
        );
    }

    #[test]
    fn codex_account_snapshot_reads_safe_identity_fields() {
        let root = temp_path("codex-account");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let auth_path = root.join("auth.json");
        std::fs::write(
            &auth_path,
            serde_json::to_string(&json!({
                "auth_mode": "chatgpt",
                "tokens": {
                    "id_token": "head.eyJlbWFpbCI6Imlnb3JAZXhhbXBsZS5jb20iLCJuYW1lIjoiSWdvciJ9.sig",
                    "access_token": "secret-access-token",
                    "refresh_token": "secret-refresh-token",
                    "account_id": "acc_1234567890abcdef"
                },
                "last_refresh": "2026-05-09T00:00:00Z"
            }))
            .unwrap(),
        )
        .unwrap();

        let snapshot = codex_account_snapshot_from_auth_path(&auth_path);

        assert_eq!(snapshot["status"], "ok");
        assert_eq!(snapshot["auth_mode"], "chatgpt");
        assert_eq!(snapshot["display"], "igor@example.com");
        assert_eq!(snapshot["email"], "igor@example.com");
        assert_eq!(snapshot["account_id_tail"], "...abcdef");
        assert_eq!(snapshot.get("access_token"), None);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn provider_error_is_compact_and_does_not_echo_context() {
        let raw = r#"ERROR: {"type":"error","error":{"message":"Model gpt-5.5 requires a newer version of Codex."}}
user
[IDENTITY]
/queue title="fake" goal="must not parse""#;

        let compact =
            compact_provider_error("codex", Some("gpt-5.5"), Some(CodexTransport::Cli), raw);

        assert!(compact.contains("Provider error: codex model=gpt-5.5 transport=cli"));
        assert!(compact.contains("update Codex CLI"));
        assert!(compact.contains("requires a newer version of Codex"));
        assert!(!compact.contains("[IDENTITY]"));
        assert!(!compact.contains("/queue"));
    }

    #[test]
    fn codex_sandbox_follows_permission_profile() {
        let root = temp_path("codex-sandbox");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let restrictive = root.join("restrictive.json");
        let balanced = root.join("balanced.json");
        let permissive = root.join("permissive.json");
        std::fs::write(&restrictive, r#"{"permissions":{"allow":["Read","Grep"]}}"#).unwrap();
        std::fs::write(
            &balanced,
            r#"{"permissions":{"allow":["Read","Edit","Write","Bash(git status*)"]}}"#,
        )
        .unwrap();
        std::fs::write(
            &permissive,
            r#"{"permissions":{"allow":["Read","Edit","Write","Bash"]}}"#,
        )
        .unwrap();

        assert_eq!(
            codex_sandbox_for_permission_path(Some(restrictive.to_string_lossy().as_ref())),
            "read-only"
        );
        assert_eq!(
            codex_sandbox_for_permission_path(Some(balanced.to_string_lossy().as_ref())),
            "workspace-write"
        );
        assert_eq!(
            codex_sandbox_for_permission_path(Some(permissive.to_string_lossy().as_ref())),
            "danger-full-access"
        );
        assert_eq!(codex_sandbox_for_permission_path(None), "read-only");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn project_solo_chat_can_explicitly_use_codex() {
        let root = temp_path("project-solo-provider");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({
                "orchestrator_provider": "claude",
                "codex_model": "gpt-5.5",
                "codex_effort": "xhigh"
            }))
            .unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());

        let (provider, model, effort) =
            resolve_single_chat_settings(&state, "AgentOS", Some("codex"), None, None);

        assert_eq!(provider, ProviderKind::Codex);
        assert_eq!(model.as_deref(), Some("gpt-5.5"));
        assert_eq!(effort.as_deref(), Some("xhigh"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn project_solo_chat_auto_uses_configured_orchestrator_provider() {
        let root = temp_path("project-solo-auto-provider");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({ "orchestrator_provider": "codex" })).unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());

        let (provider, _, _) = resolve_single_chat_settings(&state, "AgentOS", None, None, None);

        assert_eq!(provider, ProviderKind::Codex);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disabled_claude_routes_to_codex() {
        let root = temp_path("disabled-claude-routes");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({
                "claude_enabled": "false",
                "orchestrator_provider": "claude",
                "technical_reviewer_provider": "claude",
                "codex_model": "gpt-5.5"
            }))
            .unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());

        assert!(!claude_enabled(&state));
        assert_eq!(orchestrator_provider(&state), ProviderKind::Codex);
        assert_eq!(technical_reviewer_provider(&state), ProviderKind::Codex);
        assert_eq!(
            single_chat_provider(&state, Some("claude")),
            ProviderKind::Codex
        );

        let status = provider_status_snapshot(&state);
        assert_eq!(
            status["roles"]["orchestrator_provider"].as_str(),
            Some("codex")
        );
        assert_eq!(
            status["providers"]["claude"]["enabled"].as_bool(),
            Some(false)
        );
        assert_eq!(
            status["providers"]["claude"]["available"].as_bool(),
            Some(false)
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disabled_claude_delegation_routes_to_codex_and_ignores_claude_model() {
        let root = temp_path("disabled-claude-delegation-routes");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({
                "claude_enabled": "false",
                "orchestrator_provider": "claude",
                "delegation_model": "sonnet",
                "delegation_effort": "high",
                "codex_model": "gpt-5.5",
                "codex_effort": "xhigh"
            }))
            .unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());
        let provider = delegation_provider(&state);

        assert_eq!(provider, ProviderKind::Codex);
        assert_eq!(
            resolve_delegation_model(&state, provider).as_deref(),
            Some("gpt-5.5")
        );
        assert_eq!(
            resolve_delegation_effort(&state, provider).as_deref(),
            Some("xhigh")
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disabled_claude_run_returns_actionable_error() {
        let root = temp_path("disabled-claude-run");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({ "claude_enabled": "false" })).unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());

        let output = run_provider_with_opts(
            &state,
            ProviderKind::Claude,
            &root,
            "hello",
            None,
            Some("opus"),
            None,
        );

        assert!(output.contains("Provider error: claude model=opus"));
        assert!(output.contains("Claude is disabled in AgentOS settings"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn provider_timeout_uses_config_with_bounds() {
        let root = temp_path("provider-timeout");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("n8n")).unwrap();
        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({ "provider_timeout_seconds": "3" })).unwrap(),
        )
        .unwrap();
        let state = crate::state::AppState::new(root.clone());
        assert_eq!(provider_timeout(&state).as_secs(), 30);

        std::fs::write(
            root.join("n8n").join("config.json"),
            serde_json::to_string(&json!({ "provider_timeout_seconds": 120 })).unwrap(),
        )
        .unwrap();
        state.invalidate_config();
        assert_eq!(provider_timeout(&state).as_secs(), 120);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn process_exists_reports_current_process_and_missing_pid() {
        assert!(process_exists(std::process::id()));
        assert!(!process_exists(u32::MAX - 123));
    }
}

#[tauri::command]
pub fn get_provider_status(state: State<'_, Arc<AppState>>) -> Value {
    provider_status_snapshot(&state)
}

#[tauri::command]
pub fn codex_acp_authenticate(state: State<'_, Arc<AppState>>, method_id: Option<String>) -> Value {
    let command = resolve_codex_acp_command(&state);
    let args = match codex_acp_args(&state) {
        Ok(args) => args,
        Err(error) => {
            return json!({"status":"error","error": error});
        }
    };
    match codex_acp_initialize(&state) {
        Ok((mut client, init)) => {
            let chosen = method_id
                .or_else(|| {
                    init.auth_methods.iter().find_map(|method| {
                        method.get("id").and_then(|v| v.as_str()).map(String::from)
                    })
                })
                .unwrap_or_default();
            if chosen.is_empty() {
                return json!({
                    "status":"error",
                    "error":"No ACP auth methods advertised by the Codex adapter",
                    "command": command,
                    "args": args,
                    "auth_methods": init.auth_methods,
                });
            }
            match client.authenticate(&chosen) {
                Ok(result) => {
                    invalidate_codex_acp_status_cache(&state);
                    json!({
                        "status":"ok",
                        "method_id": chosen,
                        "result": result,
                        "ready": true,
                        "probe": "Authentication finished. Refresh status to re-check adapter readiness.",
                    })
                }
                Err(error) => json!({
                    "status":"error",
                    "error": error,
                    "method_id": chosen,
                    "auth_methods": init.auth_methods,
                }),
            }
        }
        Err(error) => json!({
            "status":"error",
            "error": error,
            "command": command,
            "args": args,
        }),
    }
}
