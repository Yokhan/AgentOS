//! Signal System: event bus for reactive orchestration.
//! Signals are collected from gates, scanners, timeouts, user feedback.
//! PA sees [SIGNALS] in context. Critical signals can auto-trigger PA.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub source: SignalSource,
    pub severity: Severity,
    pub project: Option<String>,
    pub message: String,
    pub created_at: String,
    #[serde(default)]
    pub acknowledged: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
    Gate,
    Scanner,
    Timeout,
    User,
    Reviewer,
    Incident,
    CostGuard,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Emit a signal — append to signals JSONL + update in-memory cache.
pub fn emit_signal(
    state: &AppState,
    source: SignalSource,
    severity: Severity,
    project: Option<&str>,
    message: &str,
    delegation_id: Option<&str>,
) {
    let id = format!(
        "sig-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let signal = Signal {
        id: id.clone(),
        source,
        severity: severity.clone(),
        project: project.map(|s| s.to_string()),
        message: message.to_string(),
        created_at: state.now_iso(),
        acknowledged: false,
        delegation_id: delegation_id.map(|s| s.to_string()),
    };

    // Persist to JSONL
    let path = state.root.join("tasks").join(".signals.jsonl");
    super::jsonl::append_jsonl_logged(
        &path,
        &serde_json::to_value(&signal).unwrap_or(json!({})),
        "signal",
    );

    crate::log_info!(
        "[signal] {} {} {:?} — {}",
        severity,
        source_label(&signal.source),
        project,
        message
    );
}

/// Emit gate-based signals from GateResult.
pub fn emit_gate_signals(
    state: &AppState,
    project: &str,
    delegation_id: &str,
    gate: &super::gate::GateResult,
) {
    match gate.status {
        super::gate::GateStatus::Fail => {
            emit_signal(
                state,
                SignalSource::Gate,
                Severity::Critical,
                Some(project),
                &format!(
                    "Verify FAILED: {}",
                    gate.errors.first().map(|s| s.as_str()).unwrap_or("unknown")
                ),
                Some(delegation_id),
            );
        }
        super::gate::GateStatus::Warn => {
            for err in &gate.errors {
                emit_signal(
                    state,
                    SignalSource::Gate,
                    Severity::Warn,
                    Some(project),
                    err,
                    Some(delegation_id),
                );
            }
        }
        super::gate::GateStatus::Pass => {
            // No signal for pass — quiet success
        }
    }
}

/// Check for incident pattern: 3+ failures in same project within 10 minutes.
pub fn check_incident(state: &AppState, project: &str) -> bool {
    let fail_count = count_recent_critical(state, Some(project), 10);
    if fail_count >= 3 {
        emit_signal(
            state,
            SignalSource::Incident,
            Severity::Critical,
            Some(project),
            &format!(
                "INCIDENT: {} critical signals in 10min — auto-approve paused for {}",
                fail_count, project
            ),
            None,
        );
        true
    } else {
        false
    }
}

/// Count critical signals in last N minutes, optionally filtered by project.
pub fn count_recent_critical(state: &AppState, project: Option<&str>, minutes: u64) -> usize {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes as i64);
    content
        .lines()
        .rev()
        .take(100)
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| {
            if v.get("severity").and_then(|s| s.as_str()) != Some("critical") {
                return false;
            }
            if v.get("acknowledged")
                .and_then(|a| a.as_bool())
                .unwrap_or(false)
            {
                return false;
            }
            if let Some(proj) = project {
                if v.get("project").and_then(|p| p.as_str()) != Some(proj) {
                    return false;
                }
            }
            v.get("created_at")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| dt > cutoff)
                .unwrap_or(false)
        })
        .count()
}

/// Build [SIGNALS] section for PA context — last 20 unacknowledged signals.
pub fn build_signals_context(state: &AppState) -> String {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    // Collect acked IDs
    let acked: std::collections::HashSet<String> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("ack"))
        .filter_map(|v| {
            v.get("signal_id")
                .and_then(|i| i.as_str())
                .map(String::from)
        })
        .collect();
    let signals: Vec<Signal> = content
        .lines()
        .rev()
        .take(50)
        .filter_map(|line| serde_json::from_str::<Signal>(line).ok())
        .filter(|s| !acked.contains(&s.id))
        .take(20)
        .collect();

    if signals.is_empty() {
        return String::new();
    }

    let mut lines = vec!["[SIGNALS]".to_string()];
    for s in &signals {
        let icon = match s.severity {
            Severity::Critical => "🔴",
            Severity::Warn => "🟡",
            Severity::Info => "🔵",
        };
        let proj = s.project.as_deref().unwrap_or("system");
        lines.push(format!(
            "  {} [{}] {} — {}",
            icon,
            source_label(&s.source),
            proj,
            s.message
        ));
    }
    lines.push("[END SIGNALS]".to_string());
    lines.join("\n")
}

/// Acknowledge a signal by ID — append-only (no file rewrite, no race condition).
pub fn acknowledge_signal(state: &AppState, signal_id: &str) -> bool {
    let path = state.root.join("tasks").join(".signals.jsonl");
    // Append an ack entry instead of rewriting the file
    let ack = json!({"type":"ack","signal_id":signal_id,"ts":state.now_iso()});
    super::jsonl::append_jsonl_logged(&path, &ack, "signal ack");
    // Mark in memory: rebuild signals context will skip acked
    true
}

/// Rotate signals file if > 5000 lines (keep last 1000).
pub fn rotate_signals(state: &AppState) {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 5000 {
        let kept: String = lines[lines.len() - 1000..].join("\n") + "\n";
        let _ = std::fs::write(&path, kept);
        crate::log_info!("[signals] rotated: {} → 1000 lines", lines.len());
    }
}

/// Get count of unacknowledged signals by severity.
pub fn signal_counts(state: &AppState) -> (u32, u32, u32) {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return (0, 0, 0),
    };
    // Collect acked IDs first (append-only ack model)
    let acked: std::collections::HashSet<String> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("ack"))
        .filter_map(|v| {
            v.get("signal_id")
                .and_then(|i| i.as_str())
                .map(String::from)
        })
        .collect();
    let (mut crit, mut warn, mut info) = (0u32, 0u32, 0u32);
    for line in content.lines().rev().take(200) {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if v.get("type").and_then(|t| t.as_str()) == Some("ack") {
                continue;
            }
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
            if acked.contains(id) {
                continue;
            }
            match v.get("severity").and_then(|s| s.as_str()) {
                Some("critical") => crit += 1,
                Some("warn") => warn += 1,
                Some("info") => info += 1,
                _ => {}
            }
        }
    }
    (crit, warn, info)
}

fn source_label(s: &SignalSource) -> &'static str {
    match s {
        SignalSource::Gate => "gate",
        SignalSource::Scanner => "scan",
        SignalSource::Timeout => "timeout",
        SignalSource::User => "user",
        SignalSource::Reviewer => "reviewer",
        SignalSource::Incident => "INCIDENT",
        SignalSource::CostGuard => "cost",
    }
}

/// Tauri command: get recent signals for frontend display.
#[tauri::command]
pub fn get_signals(state: tauri::State<std::sync::Arc<AppState>>) -> serde_json::Value {
    let path = state.root.join("tasks").join(".signals.jsonl");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return json!({"signals":[]}),
    };
    let acked: std::collections::HashSet<String> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("ack"))
        .filter_map(|v| {
            v.get("signal_id")
                .and_then(|i| i.as_str())
                .map(String::from)
        })
        .collect();
    let signals: Vec<Value> = content
        .lines()
        .rev()
        .take(50)
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("ack"))
        .filter(|v| !acked.contains(v.get("id").and_then(|i| i.as_str()).unwrap_or("")))
        .collect();
    let (crit, warn, info) = signal_counts(&state);
    json!({"signals": signals, "counts": {"critical": crit, "warn": warn, "info": info}})
}

/// Tauri command: acknowledge a signal.
#[tauri::command]
pub fn ack_signal(state: tauri::State<std::sync::Arc<AppState>>, id: String) -> serde_json::Value {
    if acknowledge_signal(&state, &id) {
        json!({"status": "ok"})
    } else {
        json!({"status": "error", "error": "signal not found"})
    }
}

// chrono_minus_minutes removed — replaced by proper chrono::Duration in count_recent_critical()
