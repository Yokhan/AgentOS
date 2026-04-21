//! Simple file logger for AgentOS with rotation.
//! Writes timestamped lines to tasks/agent-os.log.
//! Rotates when file exceeds 5MB.

use std::path::PathBuf;
use std::sync::OnceLock;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5MB

/// Initialize logger with the app root path. Call once at startup.
pub fn init(root: &std::path::Path) {
    let path = root.join("tasks").join("agent-os.log");
    let _ = LOG_PATH.set(path);
}

/// Write a log line with timestamp and level. Rotates if file > 5MB.
pub fn log(level: &str, msg: &str) {
    let Some(path) = LOG_PATH.get() else { return };
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let line = format!("[{}] {} {}\n", ts, level, msg);
    eprint!("{}", line);

    // Rotate if needed
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > MAX_LOG_SIZE {
            let backup = path.with_extension("log.1");
            let _ = std::fs::rename(path, backup);
        }
    }

    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        });
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logger::log("INFO", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::logger::log("WARN", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logger::log("ERROR", &format!($($arg)*)) };
}
