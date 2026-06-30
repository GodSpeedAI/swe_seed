//! `ContextPack` + `build_pack` (excludes excluded, caps raw output, warns
//! stale) and the `cap_lines` raw-output policy helper.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::budget::ContextBudget;
use crate::contracts::harness::ValidationRequirement;
use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `ContextPack`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub id: String,
    pub route_card_id: String,
    pub budget_id: String,
    pub included_files: Vec<String>,
    pub excluded_files: Vec<String>,
    pub salient_facts: Vec<String>,
    pub stale_context_warnings: Vec<String>,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
}

impl BamlParity for ContextPack {
    fn baml_name() -> &'static str {
        "ContextPack"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "route_card_id",
                "budget_id",
                "included_files",
                "excluded_files",
                "salient_facts",
                "stale_context_warnings",
                "validation",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// Default staleness threshold for context files (30 days).
pub const DEFAULT_STALE_THRESHOLD: Duration = Duration::from_secs(30 * 24 * 3600);

/// Cap raw output to `max` lines (the budget's raw-output policy). Returns the
/// retained lines; if truncation occurred, the final line notes the cut so a
/// reader can tell the output was bounded.
pub fn cap_lines(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max {
        return text.to_string();
    }
    let mut out: Vec<String> = lines.iter().take(max).map(|s| s.to_string()).collect();
    out.push(format!(
        "... [truncated: {} lines shown of {} per context budget]",
        max,
        lines.len()
    ));
    out.join("\n")
}

/// Parse the budget's raw-output line cap from `max_context_size` (a numeric
/// string). Falls back to `default` when unset/invalid.
pub fn parse_line_cap(budget: &ContextBudget, default: usize) -> usize {
    budget
        .max_context_size
        .trim()
        .parse::<usize>()
        .unwrap_or(default)
}

/// Files in `files` whose mtime is older than `threshold` (relative to `now`).
pub fn stale_files(
    root: &Path,
    files: &[String],
    threshold: Duration,
    now: SystemTime,
) -> Vec<String> {
    let mut stale = Vec::new();
    for f in files {
        let path = root.join(f);
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(age) = now.duration_since(mtime) {
                    if age > threshold {
                        stale.push(f.clone());
                    }
                }
            }
        }
    }
    stale
}

/// Build a `ContextPack` from a budget: included = (required + optional) minus
/// excluded (deduped, order-preserving); excluded recorded; stale files warned;
/// salient facts = the budget's summarization/relevance rules.
pub fn build_pack(
    root: &Path,
    budget: &ContextBudget,
    route_card_id: &str,
    stale_threshold: Duration,
    now: SystemTime,
) -> ContextPack {
    let excluded: std::collections::HashSet<&str> =
        budget.excluded_files.iter().map(|s| s.as_str()).collect();

    // Order-preserving dedupe of required + optional, dropping excluded entries.
    let mut included: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for f in budget
        .required_files
        .iter()
        .chain(budget.optional_files.iter())
    {
        if excluded.contains(f.as_str()) {
            continue;
        }
        if seen.insert(f.clone()) {
            included.push(f.clone());
        }
    }

    let actually_excluded: Vec<String> = budget
        .excluded_files
        .iter()
        .filter(|e| {
            budget
                .required_files
                .iter()
                .chain(budget.optional_files.iter())
                .any(|r| r == *e)
        })
        .cloned()
        .collect();

    let stale = stale_files(root, &included, stale_threshold, now);
    let stale_context_warnings: Vec<String> = stale
        .iter()
        .map(|f| format!("stale context: {f} older than threshold"))
        .collect();

    let mut salient_facts = budget.summarization_rules.clone();
    salient_facts.extend(budget.relevance_rules.iter().cloned());

    ContextPack {
        id: format!("pack-{}", budget.id),
        route_card_id: route_card_id.to_string(),
        budget_id: budget.id.clone(),
        included_files: included,
        excluded_files: actually_excluded,
        salient_facts,
        stale_context_warnings,
        validation: Vec::new(),
    }
}
