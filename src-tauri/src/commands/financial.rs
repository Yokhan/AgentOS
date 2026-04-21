//! Financial tracking: income recording and dashboard.

use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct IncomeEntry {
    pub ts: String,
    pub amount: f64,
    pub category: String,
    pub description: String,
}

fn income_path(state: &AppState) -> std::path::PathBuf {
    state.root.join("tasks").join("income.json")
}

fn load_income(state: &AppState) -> Vec<IncomeEntry> {
    std::fs::read_to_string(&income_path(state))
        .ok().and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_income(state: &AppState, entries: &[IncomeEntry]) {
    let _ = std::fs::write(&income_path(state), serde_json::to_string_pretty(entries).unwrap_or_default());
}

/// INCOME_RECORD
pub fn income_record(state: &AppState, amount: f64, category: &str, description: &str) -> Option<String> {
    let mut entries = load_income(state);
    entries.push(IncomeEntry {
        ts: state.now_iso(),
        amount,
        category: category.to_string(),
        description: description.to_string(),
    });
    save_income(state, &entries);
    crate::log_info!("[financial] recorded {} RUB ({})", amount, category);
    Some(format!("**Income recorded:** {} RUB [{}] {}", amount, category, description))
}

/// FINANCIAL_DASHBOARD
pub fn financial_dashboard(state: &AppState) -> Option<String> {
    let entries = load_income(state);
    if entries.is_empty() { return Some("**Financial:** No income recorded yet.".to_string()); }

    let total: f64 = entries.iter().map(|e| e.amount).sum();
    let mut by_category: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for e in &entries {
        *by_category.entry(e.category.clone()).or_insert(0.0) += e.amount;
    }

    // Monthly breakdown
    let now = chrono::Utc::now();
    let this_month = now.format("%Y-%m").to_string();
    let month_total: f64 = entries.iter()
        .filter(|e| e.ts.starts_with(&this_month))
        .map(|e| e.amount).sum();

    let target = 450_000.0_f64;
    let pct = (total / target * 100.0).min(100.0);
    let deadline = chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap_or(chrono::Utc::now().date_naive());
    let days_left = (deadline - chrono::Utc::now().date_naive()).num_days().max(1);

    let mut lines = vec![
        format!("**Financial Dashboard**"),
        format!("Total: {:.0} / {:.0} RUB ({:.1}%)", total, target, pct),
        format!("This month: {:.0} RUB", month_total),
        format!("Need: {:.0} RUB/day ({} days left)", (target - total).max(0.0) / days_left as f64, days_left),
    ];

    lines.push("By category:".to_string());
    for (cat, amount) in &by_category {
        lines.push(format!("  {}: {:.0} RUB", cat, amount));
    }

    // Last 5 entries
    lines.push("Recent:".to_string());
    for e in entries.iter().rev().take(5) {
        lines.push(format!("  {} {:.0} [{}] {}", e.ts.chars().take(10).collect::<String>(), e.amount, e.category, e.description.chars().take(40).collect::<String>()));
    }

    Some(lines.join("\n"))
}
