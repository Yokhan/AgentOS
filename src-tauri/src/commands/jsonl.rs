//! JSONL append utility — single shared function replacing 19 duplicate sites.
//! All chat saves, delegation logs, audit entries go through here.

use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static JSONL_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn write_line(file: &mut std::fs::File, value: &Value) -> std::io::Result<()> {
    use std::io::Write;
    let serialized = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    writeln!(file, "{}", serialized)
}

/// Append a JSON value as a line to a JSONL file. Returns io::Result for explicit error handling.
pub fn append_jsonl(path: &Path, value: &Value) -> std::io::Result<()> {
    let _guard = JSONL_WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    write_line(&mut f, value)
}

/// Append JSONL with automatic error logging. Use for non-critical writes.
pub fn append_jsonl_logged(path: &Path, value: &Value, context: &str) {
    if let Err(e) = append_jsonl(path, value) {
        crate::log_warn!("JSONL write failed [{}]: {} (path: {:?})", context, e, path);
    }
}

/// Append two JSON values to the same file (for user+assistant message pairs).
pub fn append_jsonl_pair(path: &Path, a: &Value, b: &Value, context: &str) {
    let result = (|| {
        let _guard = JSONL_WRITE_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        write_line(&mut file, a)?;
        write_line(&mut file, b)
    })();
    if let Err(e) = result {
        crate::log_warn!("JSONL pair write failed [{}]: {}", context, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn concurrent_appends_keep_every_line_valid() {
        let unique = format!(
            "agentos-jsonl-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let path = Arc::new(std::env::temp_dir().join(unique));
        let mut workers = Vec::new();
        for worker in 0..8 {
            let path = path.clone();
            workers.push(std::thread::spawn(move || {
                for sequence in 0..100 {
                    append_jsonl(&path, &json!({"worker": worker, "sequence": sequence}))
                        .expect("append must succeed");
                }
            }));
        }
        for worker in workers {
            worker.join().expect("writer thread must finish");
        }

        let content = std::fs::read_to_string(path.as_ref()).expect("jsonl must be readable");
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 800);
        assert!(lines
            .iter()
            .all(|line| serde_json::from_str::<Value>(line).is_ok()));
        let _ = std::fs::remove_file(path.as_ref());
    }
}
