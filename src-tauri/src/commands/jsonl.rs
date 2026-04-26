//! JSONL append utility — single shared function replacing 19 duplicate sites.
//! All chat saves, delegation logs, audit entries go through here.

use serde_json::Value;
use std::path::Path;

/// Append a JSON value as a line to a JSONL file. Returns io::Result for explicit error handling.
pub fn append_jsonl(path: &Path, value: &Value) -> std::io::Result<()> {
    let serialized = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    writeln!(f, "{}", serialized)
}

/// Append JSONL with automatic error logging. Use for non-critical writes.
pub fn append_jsonl_logged(path: &Path, value: &Value, context: &str) {
    if let Err(e) = append_jsonl(path, value) {
        crate::log_warn!("JSONL write failed [{}]: {} (path: {:?})", context, e, path);
    }
}

/// Append two JSON values to the same file (for user+assistant message pairs).
pub fn append_jsonl_pair(path: &Path, a: &Value, b: &Value, context: &str) {
    if let Err(e) = append_jsonl(path, a).and_then(|_| append_jsonl(path, b)) {
        crate::log_warn!("JSONL pair write failed [{}]: {}", context, e);
    }
}
