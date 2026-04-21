//! Graph file collection, import resolution, and layer detection.
//! Extracted from graph_scan.rs for file size.

use std::collections::HashMap;
use std::path::Path;

pub const MAX_FILES: usize = 500;

pub const EXCLUDES: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    ".git",
    "__pycache__",
    ".next",
    "vendor",
    ".venv",
    "venv",
    "env",
    ".cache",
    ".tox",
    "Python",
    "pythoncore",
    "_archive",
    "_reference",
    "docs",
    "Doc",
    ".mypy_cache",
    ".pytest_cache",
    "coverage",
    ".nyc_output",
    "site-packages",
    ".gradle",
    ".idea",
    ".vs",
    "Pods",
    "DerivedData",
    ".godot",
    "addons",
    "storybook-static",
    ".parcel-cache",
    ".turbo",
    ".svelte-kit",
    "ia-memory",
];

pub const EXTENSIONS: &[&str] = &["rs", "ts", "tsx", "js", "jsx", "py", "gd"];

pub fn collect_files(dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    // Load .graphignore patterns
    let extra_excludes = load_ignore(dir, ".graphignore");
    let git_excludes = load_ignore(dir, ".gitignore");
    let all_excludes: Vec<&str> = EXCLUDES
        .iter()
        .copied()
        .chain(extra_excludes.iter().map(|s| s.as_str()))
        .chain(
            git_excludes
                .iter()
                .filter(|s| !s.contains('*') && !s.contains('/'))
                .map(|s| s.as_str()),
        )
        .collect();

    let mut files = Vec::new();
    collect_recursive(dir, dir, &all_excludes, &mut files, 0);
    files
}

fn load_ignore(dir: &Path, filename: &str) -> Vec<String> {
    let path = dir.join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.trim().trim_end_matches('/').to_string())
        .collect()
}

fn collect_recursive(
    base: &Path,
    dir: &Path,
    excludes: &[&str],
    out: &mut Vec<(String, std::path::PathBuf)>,
    depth: u32,
) {
    if depth > 10 || out.len() >= MAX_FILES {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != ".gitignore" && name != ".graphignore" {
            continue;
        }
        if excludes.iter().any(|e| name == *e) {
            continue;
        }
        if path.is_dir() {
            collect_recursive(base, &path, excludes, out, depth + 1);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if EXTENSIONS.contains(&ext) {
                let rel = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/");
                out.push((rel, path));
            }
        }
    }
}

pub fn detect_layer(path: &str) -> u32 {
    let lower = path.to_lowercase();
    if lower.contains("component")
        || lower.contains("view")
        || lower.contains("page")
        || lower.contains("ui")
    {
        return 0;
    }
    if lower.contains("api")
        || lower.contains("route")
        || lower.contains("handler")
        || lower.contains("controller")
    {
        return 1;
    }
    if lower.contains("service") || lower.contains("domain") || lower.contains("model") {
        return 2;
    }
    if lower.contains("infra")
        || lower.contains("repo")
        || lower.contains("db")
        || lower.contains("storage")
    {
        return 3;
    }
    if lower.contains("util")
        || lower.contains("helper")
        || lower.contains("lib")
        || lower.contains("common")
    {
        return 4;
    }
    2
}

pub fn layer_name(layer: u32) -> String {
    match layer {
        0 => "ui",
        1 => "api",
        2 => "domain",
        3 => "infra",
        4 => "util",
        _ => "unknown",
    }
    .to_string()
}

pub fn file_label(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

pub fn display_label(path: &str, counts: &HashMap<String, u32>) -> String {
    let label = file_label(path);
    if counts.get(&label).copied().unwrap_or(0) <= 1 {
        return label;
    }
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 1 {
        return label;
    }
    parts[1..].join("/")
}

pub fn resolve_import(
    source_path: &str,
    import: &str,
    file_map: &HashMap<String, String>,
    ext: &str,
) -> Option<String> {
    if !import.starts_with('.')
        && !import.starts_with("crate::")
        && !import.starts_with("super::")
        && !import.starts_with("res://")
    {
        return None;
    }
    let source_dir = source_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    match ext {
        "ts" | "tsx" | "js" | "jsx" => {
            let resolved = if import.starts_with("./") || import.starts_with("../") {
                normalize_path(&format!("{}/{}", source_dir, import))
            } else {
                import.to_string()
            };
            for try_ext in &["", ".ts", ".tsx", ".js", ".jsx", "/index.ts", "/index.js"] {
                let candidate = format!("file:{}{}", resolved, try_ext);
                if file_map.values().any(|id| *id == candidate) {
                    return Some(candidate);
                }
            }
            None
        }
        "rs" => {
            let module = import.replace("crate::", "src/").replace("::", "/");
            for try_ext in &[".rs", "/mod.rs"] {
                let candidate = format!("file:{}{}", module, try_ext);
                if file_map.values().any(|id| *id == candidate) {
                    return Some(candidate);
                }
            }
            None
        }
        "py" => {
            let module = import.replace('.', "/");
            for try_ext in &[".py", "/__init__.py"] {
                let candidate = format!("file:{}{}", module, try_ext);
                if file_map.values().any(|id| *id == candidate) {
                    return Some(candidate);
                }
            }
            None
        }
        "gd" => {
            let clean = import.trim_start_matches("res://");
            let candidate = format!("file:{}", clean);
            if file_map.values().any(|id| *id == candidate) {
                Some(candidate)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_label_keeps_unique_basename() {
        let mut counts = HashMap::new();
        counts.insert("main.rs".to_string(), 1);
        assert_eq!(display_label("src/main.rs", &counts), "main.rs");
    }

    #[test]
    fn display_label_disambiguates_duplicates() {
        let mut counts = HashMap::new();
        counts.insert("index.ts".to_string(), 2);
        assert_eq!(display_label("src/web/index.ts", &counts), "web/index.ts");
    }
}
