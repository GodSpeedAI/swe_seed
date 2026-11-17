# context

## Overview

Context plane (spec 0015). `ContextBudget` bounds what a route may load and
`build_pack` enforces that budget (drops excluded files, caps raw output, warns
on stale context) to produce a `ContextPack`. The module exposes the
`swe-seed context-plan <task>` entry point, which routes the task, builds the
budget from `budget-policy.yaml`, and emits the plan as JSON. Both data structs
declare BAML parity with `harness.baml` shapes.

## Interface

- `pub struct ContextBudget` (harness.baml `ContextBudget`: `id`,
  `max_context_size`, `required_files`, `optional_files`, `excluded_files`,
  `freshness_requirements`, `relevance_rules`, `summarization_rules`,
  `validation`) implementing `BamlParity` with `baml_name() == "ContextBudget"`
  [budget.rs:ContextBudget], [budget.rs:BamlParity]
- `pub struct ContextPack` (harness.baml `ContextPack`: `id`, `route_card_id`,
  `budget_id`, `included_files`, `excluded_files`, `salient_facts`,
  `stale_context_warnings`, `validation`) implementing `BamlParity`
  [pack.rs:ContextPack], [pack.rs:BamlParity]
- `pub const DEFAULT_STALE_THRESHOLD: Duration` = 30 days [pack.rs:DEFAULT_STALE_THRESHOLD]
- `cap_lines(text: &str, max: usize) -> String`: truncates to `max` lines and
  appends a `... [truncated: ...]` note line when cut [pack.rs:cap_lines]
- `parse_line_cap(budget: &ContextBudget, default: usize) -> usize`: parses
  `max_context_size` as a numeric line cap, falling back to `default`
  [pack.rs:parse_line_cap]
- `stale_files(root, files, threshold, now) -> Vec<String>`: files whose mtime
  age exceeds the threshold [pack.rs:stale_files]
- `build_pack(root, budget, route_card_id, stale_threshold, now) -> ContextPack`:
  order-preserving dedupe of required + optional minus excluded, stale warnings,
  salient facts from summarization + relevance rules [pack.rs:build_pack]
- `pub const BUDGET_POLICY_PATH: &str = ".agent-harness/context/budget-policy.yaml"`
  [policy.rs:BUDGET_POLICY_PATH]
- `build_budget_from_policy(route_required: &[String], policy: &BudgetPolicyConfig) -> ContextBudget`:
  default budget for a route; required files are the context README + budget
  policy + route context, cap comes from `raw_output_policy.max_default_lines`
  [policy.rs:build_budget_from_policy]
- `context_plan(root: &Path, task: &str) -> Result<Value>`: the
  `swe-seed context-plan <task>` entry point; routes via `build_route_result`,
  loads the policy, builds the budget and pack, emits the plan JSON
  [policy.rs:context_plan]

## Invariants

- `build_pack` never includes an excluded file: entries in
  `budget.excluded_files` are dropped from the included list, and
  `excluded_files` in the pack only records exclusions that were actually
  required or optional [pack.rs:build_pack]
- Included files are deduplicated in first-seen order (required files first,
  then optional) [pack.rs:build_pack]
- `cap_lines` retains exactly `max` lines plus one truncation-note line, so a
  reader can always tell output was bounded [pack.rs:cap_lines]
- `context_plan` computes staleness against `SystemTime::now()` and the module
  constant `DEFAULT_STALE_THRESHOLD`, not a caller-supplied clock
  [policy.rs:context_plan]

## Side Effects

- `context_plan` reads `.agent-harness/context/budget-policy.yaml` from disk via
  `load_yaml` and `std::fs::metadata` on each included file (mtime staleness
  check) [policy.rs:context_plan], [pack.rs:stale_files]
- `stale_files` performs `std::fs::metadata` on every candidate file
  [pack.rs:stale_files]

## Verification

- Run: `cargo test -p swe-seed-core`
