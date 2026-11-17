# route

## Overview

Routing (specs 0004/0012). Loads the frozen route cards from
`.agent-harness/routes/*.json`, scores a task against each via deterministic
token overlap, and selects a card. Route-decision records are written
byte-compatible with the captured Python output (`harness.score_route`,
`harness.build_route_result`, `harness.infer_bootstrap_route`, `harness.rel`).

## Interface

- `pub const REQUIRED_JOB_TYPES: &[&str]`: the 11 required job types
  (`research`, `spec`, `implementation`, `bugfix`, `refactor`, `test`,
  `review`, `release`, `documentation`, `harness_improvement`,
  `skill_authoring`), each of which must resolve to a route card
  [mod.rs:REQUIRED_JOB_TYPES]
- `pub const DECISION_BASIS: &str = "deterministic token overlap against
route-card triggers, examples, and job type"` (frozen record label)
  [mod.rs:DECISION_BASIS]
- `pub struct RouteCard` (mirrors `class RouteCard` in `harness.baml`, 16
  fields; runtime cards omit `validation`/`sources`, which default to empty)
  with `BamlParity`; `RouteSourceRef` [mod.rs:RouteCard], [mod.rs:BamlParity],
  [mod.rs:RouteSourceRef]
- `pub struct RouteResult { job_type, route_card, confidence, assumption,
required_context, required_skills, work_loop, required_artifacts, proof,
done_when, next_action }` (field order matches the captured Python output)
  [mod.rs:RouteResult]
- `harness_dir(root)` = `<root>/.agent-harness`, `routes_dir(root)` =
  `.agent-harness/routes`, `route_decisions_dir(root)` =
  `.agent-harness/traces/route-decisions` [mod.rs:harness_dir],
  [mod.rs:routes_dir], [mod.rs:route_decisions_dir]
- `load_routes(root) -> Result<Vec<RouteCard>>`: loads all `*.json` cards,
  sorted by filename [mod.rs:load_routes]
- `score_route(task, card) -> i64`: deterministic token-overlap scoring
  (triggers +8 phrase / +2 per token overlap; positive examples +3 or +1/token;
  negative examples -4 or -2/token; job-type hit +5) [mod.rs:score_route]
- `build_route_result(root, task) -> Result<RouteResult>`: ranks cards by
  (score desc, id desc), falls back to the bootstrap heuristic, and sets
  confidence `medium`/`low` [mod.rs:build_route_result],
  [mod.rs:infer_bootstrap_route]
- `write_route_decision(root, task, result) -> Result<String>`: writes a
  route-decision record with a redacted task and a trace id, returning the
  repo-relative path [mod.rs:write_route_decision]

## Invariants

- Scoring is deterministic and frozen: token overlap over `[a-z0-9_]+` tokens
  of length > 1, with fixed weights, so the same task and cards always select
  the same card [mod.rs:score_route], [mod.rs:tokenize]
- Ranking ties are broken by (score desc, id desc), matching Python
  `sorted(..., reverse=True)` for format parity [mod.rs:build_route_result]
- A non-positive score falls back to `infer_bootstrap_route`, which returns the
  `spec` card only when the task is a build verb with no clarifying nouns, no
  direct-change terms, and no file extension; otherwise confidence is `low`
  with the assumption "No strong semantic match; selected safest default
  route." [mod.rs:infer_bootstrap_route], [mod.rs:build_route_result]
- Routing without route cards fails: `build_route_result` bails with "no route
  cards found" when the routes directory is empty [mod.rs:build_route_result]
- Recorded tasks are redacted before persistence: both the trace-id slug and
  the stored `task` field use `redact_secrets(task)`
  [mod.rs:write_route_decision]
- Decision records always carry `decision_basis` = `DECISION_BASIS` so how a
  route was chosen is recorded [mod.rs:write_route_decision]

## Side Effects

- `load_routes` reads every `*.json` route card under
  `.agent-harness/routes/` [mod.rs:load_routes]
- `write_route_decision` creates `.agent-harness/traces/route-decisions/` and
  writes `<trace_id>.json` (mode 0o644) via `secure_write`
  [mod.rs:write_route_decision]

## Verification

- Run: `cargo test -p swe-seed-core`
