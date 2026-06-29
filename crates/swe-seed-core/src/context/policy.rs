//! Build a `ContextBudget` from `budget-policy.yaml` + a route, and the
//! `context-plan` entry point (spec 0015).

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use super::budget::ContextBudget;
use super::pack::{build_pack, parse_line_cap, DEFAULT_STALE_THRESHOLD};
use crate::config::{load_yaml, BudgetPolicyConfig};
use crate::route::build_route_result;

pub const BUDGET_POLICY_PATH: &str = ".agent-harness/context/budget-policy.yaml";
const DEFAULT_LINE_CAP: usize = 200;

/// Construct the default `ContextBudget` for a route from `budget-policy.yaml`:
/// required = route context + the policy/README anchors; cap = the policy's
/// `max_default_lines`; summarization rules = the policy's containment strings.
pub fn build_budget_from_policy(
    route_required: &[String],
    policy: &BudgetPolicyConfig,
) -> ContextBudget {
    let cap = policy.raw_output_policy.max_default_lines.to_string();
    let mut required: Vec<String> = vec![
        ".agent-harness/context/README.md".into(),
        BUDGET_POLICY_PATH.into(),
    ];
    for r in route_required {
        if !required.contains(r) {
            required.push(r.clone());
        }
    }

    let summarization_rules = vec![
        format!(
            "raw output cap: {} lines (summarize, keep full output in a file/trace)",
            policy.raw_output_policy.max_default_lines
        ),
        "tool output containment: prefer counts, paths, JSON fields, focused excerpts over broad dumps".into(),
        "think in code: for bulk analysis, run a small script and bring back only the result".into(),
    ];

    ContextBudget {
        id: "default-route-budget".into(),
        max_context_size: cap,
        required_files: required,
        optional_files: Vec::new(),
        excluded_files: Vec::new(),
        freshness_requirements: vec![format!(
            "context older than ~{} days may be stale",
            DEFAULT_STALE_THRESHOLD.as_secs() / 86400
        )],
        relevance_rules: vec!["keep only context that changes the next action".into()],
        summarization_rules,
        validation: Vec::new(),
    }
}

/// `swe-seed context-plan <task>`: route the task, build the budget from the
/// policy, assemble the pack (excludes/cap/stale), and emit the plan.
pub fn context_plan(root: &Path, task: &str) -> Result<Value> {
    let route = build_route_result(root, task)?;
    let policy: BudgetPolicyConfig = load_yaml(&root.join(BUDGET_POLICY_PATH))?;
    let budget = build_budget_from_policy(&route.required_context, &policy);
    let pack = build_pack(
        root,
        &budget,
        &route.route_card,
        DEFAULT_STALE_THRESHOLD,
        std::time::SystemTime::now(),
    );
    let cap = parse_line_cap(&budget, DEFAULT_LINE_CAP);

    Ok(json!({
        "task": task,
        "route": {
            "job_type": route.job_type,
            "route_card": route.route_card,
            "confidence": route.confidence,
        },
        "context_budget": {
            "id": budget.id,
            "read_order": pack.included_files,
            "excluded": pack.excluded_files,
            "raw_output_cap_lines": cap,
            "raw_output_policy": budget.summarization_rules.first().cloned().unwrap_or_default(),
            "tool_output_containment": "Prefer counts, paths, JSON fields, focused excerpts over broad file dumps.",
            "think_in_code": "For bulk analysis, run a small script and bring back only the result.",
            "session_continuity": "Use trace records for durable decisions, proof, unresolved risks.",
            "stale_context_warnings": pack.stale_context_warnings,
        },
        "salient_facts": pack.salient_facts,
        "next_action": "Read context in order, then execute the selected route work loop.",
    }))
}

#[cfg(test)]
mod tests {
    use crate::context::pack::cap_lines;

    #[test]
    fn cap_lines_truncates_and_notes() {
        let big: String = (0..300)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let capped = cap_lines(&big, 200);
        let l = capped.lines().count();
        assert_eq!(l, 201, "200 lines + 1 truncation note");
        assert!(capped.contains("truncated"));
    }
}
