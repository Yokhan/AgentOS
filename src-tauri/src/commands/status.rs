//! Status enums — compile-time safe replacement for hardcoded status strings.
//! serde(rename_all = "snake_case") ensures JSON compatibility with existing frontend.

use serde::{Deserialize, Serialize};
use std::fmt;

// === Delegation Status ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    Pending,
    Scheduled,
    Running,
    Escalated,
    Deciding,
    Verifying,
    Done,
    Failed,
    Rejected,
    Cancelled,
    NeedsPermission,
}

impl fmt::Display for DelegationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Running => write!(f, "running"),
            Self::Escalated => write!(f, "escalated"),
            Self::Deciding => write!(f, "deciding"),
            Self::Verifying => write!(f, "verifying"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
            Self::Rejected => write!(f, "rejected"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::NeedsPermission => write!(f, "needs_permission"),
        }
    }
}

impl DelegationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed | Self::Rejected | Self::Cancelled)
    }
}

// === Strategy Status ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStatus {
    Draft,
    Approved,
    Executing,
    Done,
}

impl fmt::Display for StrategyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Approved => write!(f, "approved"),
            Self::Executing => write!(f, "executing"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl StrategyStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Draft | Self::Approved | Self::Executing)
    }
}

// === Strategy Step Status ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Approved,
    Running,
    Queued,
    Done,
    Failed,
    Skipped,
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Running => write!(f, "running"),
            Self::Queued => write!(f, "queued"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

impl StepStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed | Self::Skipped)
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Done => "✓",
            Self::Failed => "✗",
            Self::Running | Self::Queued => "⏳",
            Self::Skipped => "⊘",
            _ => "○",
        }
    }
}

// === Plan Status ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Active,
    Completed,
    Paused,
}

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Paused => write!(f, "paused"),
        }
    }
}

// === Plan Step Status ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    Pending,
    Running,
    Done,
    Failed,
}

impl fmt::Display for PlanStepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl PlanStepStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Done => "✓",
            Self::Failed => "✗",
            Self::Running => "⏳",
            Self::Pending => "○",
        }
    }
}
