//! JSONL append utility — single shared function replacing 19 duplicate sites.
//! All chat saves, delegation logs, audit entries go through here.

use serde_json::Value;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static JSONL_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
pub const RECENT_READ_MAX_BYTES: u64 = 8 * 1024 * 1024;

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

/// Archive machine-generated telemetry without deleting user chat history.
pub fn archive_if_larger(
    path: &Path,
    max_bytes: u64,
) -> std::io::Result<Option<std::path::PathBuf>> {
    let size = match std::fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if size <= max_bytes {
        return Ok(None);
    }
    let archive_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("archive");
    std::fs::create_dir_all(&archive_dir)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("telemetry.jsonl");
    let target = archive_dir.join(format!("{name}.{stamp}.archive"));
    std::fs::rename(path, &target)?;
    Ok(Some(target))
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

/// Read newest lines without loading an unbounded append-only file into memory.
pub fn read_recent_lines(
    path: &Path,
    max_lines: usize,
    max_bytes: u64,
) -> std::io::Result<Vec<String>> {
    if max_lines == 0 {
        return Ok(Vec::new());
    }
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::with_capacity((len - start).min(max_bytes) as usize);
    file.read_to_end(&mut bytes)?;
    let mut content = String::from_utf8_lossy(&bytes).into_owned();
    if start > 0 {
        if let Some(first_newline) = content.find('\n') {
            content.drain(..=first_newline);
        } else {
            return Ok(Vec::new());
        }
    }
    Ok(content
        .lines()
        .rev()
        .take(max_lines)
        .map(str::to_string)
        .collect())
}

/// Visit every line newest-first without loading the whole append-only file.
pub fn for_each_line_reverse<F>(path: &Path, mut visitor: F) -> std::io::Result<()>
where
    F: FnMut(&str) -> bool,
{
    const CHUNK_SIZE: usize = 64 * 1024;
    let mut file = std::fs::File::open(path)?;
    let mut position = file.metadata()?.len();
    let mut reversed_line = Vec::new();
    while position > 0 {
        let read_len = position.min(CHUNK_SIZE as u64) as usize;
        position -= read_len as u64;
        file.seek(SeekFrom::Start(position))?;
        let mut chunk = vec![0_u8; read_len];
        file.read_exact(&mut chunk)?;
        for byte in chunk.into_iter().rev() {
            if byte == b'\n' {
                if !reversed_line.is_empty() {
                    reversed_line.reverse();
                    if reversed_line.last() == Some(&b'\r') {
                        reversed_line.pop();
                    }
                    let line = String::from_utf8_lossy(&reversed_line);
                    if !visitor(&line) {
                        return Ok(());
                    }
                    reversed_line.clear();
                }
            } else {
                reversed_line.push(byte);
            }
        }
    }
    if !reversed_line.is_empty() {
        reversed_line.reverse();
        if reversed_line.last() == Some(&b'\r') {
            reversed_line.pop();
        }
        let line = String::from_utf8_lossy(&reversed_line);
        visitor(&line);
    }
    Ok(())
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

    #[test]
    fn recent_lines_reads_bounded_tail_in_newest_first_order() {
        let path = std::env::temp_dir().join(format!(
            "agentos-jsonl-tail-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::write(&path, "one\ntwo\nthree\nfour\n").expect("seed tail file");
        let lines = read_recent_lines(&path, 2, RECENT_READ_MAX_BYTES).expect("read tail");
        assert_eq!(lines, vec!["four", "three"]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reverse_reader_visits_all_lines_across_chunks() {
        let path = std::env::temp_dir().join(format!(
            "agentos-jsonl-reverse-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let content = (0..20_000)
            .map(|index| format!("line-{index:05}"))
            .collect::<Vec<_>>()
            .join("\r\n");
        std::fs::write(&path, content).expect("seed reverse file");
        let mut lines = Vec::new();
        for_each_line_reverse(&path, |line| {
            lines.push(line.to_string());
            true
        })
        .expect("read reverse");
        assert_eq!(lines.len(), 20_000);
        assert_eq!(lines.first().map(String::as_str), Some("line-19999"));
        assert_eq!(lines.last().map(String::as_str), Some("line-00000"));
        let _ = std::fs::remove_file(path);
    }
}
