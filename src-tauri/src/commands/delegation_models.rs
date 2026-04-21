//! Delegation extended models: priority, templates, chain metadata.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationPriority {
    High,
    Med,
    Low,
}

impl DelegationPriority {
    pub fn ord(&self) -> u8 {
        match self {
            Self::High => 0,
            Self::Med => 1,
            Self::Low => 2,
        }
    }
}

impl std::fmt::Display for DelegationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "HIGH"),
            Self::Med => write!(f, "MED"),
            Self::Low => write!(f, "LOW"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DelegationTemplate {
    pub name: String,
    pub task: String,
    pub created: String,
    #[serde(default)]
    pub used_count: u32,
}

fn templates_path(root: &Path) -> std::path::PathBuf {
    root.join("tasks").join(".delegation-templates.json")
}

pub fn load_templates(root: &Path) -> Vec<DelegationTemplate> {
    let path = templates_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_templates(root: &Path, templates: &[DelegationTemplate]) {
    let path = templates_path(root);
    let _ = std::fs::write(&path, serde_json::to_string_pretty(templates).unwrap_or_default());
}
