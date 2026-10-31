# hooks

## Overview

Normalized hook runtime (spec 0005): the canonical hook event vocabulary, the
JSONL event log + SQLite index, secret redaction, OTEL/JUnit export, log
compaction, and the `HookPolicy`/`PermissionPolicy` permission gate (including
the federated authority gate). The adapters module projects these canonical
events into host hook configs, and the gateway reuses `PermissionPolicy` and
`RedactionConfig` for its own policy and audit paths.

## Interface

- `pub enum CanonicalHookEvent` (16 variants: `SessionStart`, `SessionEnd`,
  `UserPromptSubmit`, `PreToolUse`, `PermissionRequest`, `PostToolUse`,
  `ToolError`, `SubagentStart`, `SubagentStop`, `Stop`, `StopFailure`,
  `PreCompact`, `PostCompact`, `Notification`, `FileChanged`, `CwdChanged`)
  with `as_str()`; `ALL_CANONICAL_HOOK_EVENTS`; `event_names(events)`
  [events.rs:CanonicalHookEvent], [events.rs:as_str],
  [events.rs:ALL_CANONICAL_HOOK_EVENTS], [events.rs:event_names]
- `pub const REQUIRED_LIFECYCLE_EVENTS: &[&str]` = `SessionStart`,
  `ContextBuild`, `PreToolUse`, `PostToolUse`, `TaskEnd` (the five lifecycle
  events v0.1 supports end-to-end) [policy.rs:REQUIRED_LIFECYCLE_EVENTS]
- `HookPolicy { id, events, blocking_rules, advisory_rules, payload_fields,
failure_behavior, validation }` (harness.baml `HookPolicy`, `BamlParity`)
  [policy.rs:HookPolicy]
- `PermissionPolicy { id, allowed_actions, approval_gated_actions,
forbidden_actions, validation }` (harness.baml `PermissionPolicy`,
  `BamlParity`, `Default`) [policy.rs:PermissionPolicy]
- `pub enum ActionGate { Allowed, ApprovalGated, Forbidden }` and
  `gate_action(policy, action)`: forbidden takes precedence, then
  approval-gated, then allowed (default for unlisted actions)
  [policy.rs:ActionGate], [policy.rs:gate_action]
- `gate_action_with_authority(policy, action, cfg, risk, incoming)`: local gate
  first; federation consulted only when `authority_mode()` is Delegate/Hybrid
  [policy.rs:gate_action_with_authority]
- `pub enum HookDecision { Allow, Deny { reason }, Advisory { reason } }`
  [decision.rs:HookDecision]
- `HookPortMapping { canonical_event, native_event, command }`
  [port.rs:HookPortMapping]
- `RuntimeHookMap { runtime, events }` [runtime_map.rs:RuntimeHookMap]
- `RedactionConfig { key_substrings, value_patterns }` and
  `redact_value(value, cfg)`: recursive in-place JSON redaction
  [redact.rs:RedactionConfig], [redact.rs:redact_value]
- `append_event(log_dir, envelope, cfg) -> Result<(PathBuf, String)>`: redacts,
  assigns `event_id`/`timestamp` if missing, appends to today's
  `events-<date>.jsonl` [runtime.rs:append_event]
- `iter_events(log_dir)`: all envelopes across `events-*.jsonl` logs (sorted by
  filename) [runtime.rs:iter_events]
- `compact_logs(log_dir, min_bytes)`: gzips oversized non-today logs and removes
  the originals [runtime.rs:compact_logs]
- `rebuild_index(index_db, log_dir) -> Result<usize>` and
  `count_rows(index_db)`: SQLite index over the event log
  [index.rs:rebuild_index], [index.rs:count_rows]
- `export_otel(log_dir, root) -> Result<Value>` (OpenTelemetry logs JSON,
  `service.name = "swe-seed-dev-harness"`) and `export_junit(log_dir) ->
Result<String>` (JUnit XML; non-`ok` events become failures)
  [export.rs:export_otel], [export.rs:export_junit]

## Invariants

- Every appended event is redacted BEFORE the log write: `append_event` calls
  `redact_value` on the envelope first [runtime.rs:append_event]
- Redaction never produces partial secrets: key-substring matches replace the
  whole value with `"[REDACTED]"`, and value-pattern regexes rewrite matched
  spans in place [redact.rs:redact_value], [redact.rs:redact_one]
- Forbidden takes precedence over approval-gated, which takes precedence over
  the default-allow for unlisted actions [policy.rs:gate_action]
- A local `Forbidden` is never overridden by an external `allow` under
  `gate_action_with_authority`; in `Local` authority mode the federation
  boundary is not consulted at all (standalone invariant)
  [policy.rs:gate_action_with_authority]
- `compact_logs` never compacts today's log [runtime.rs:compact_logs]
- `rebuild_index` drops and recreates the `events` table so the index is a full
  rebuild keyed by `event_id` (PK), not an incremental merge
  [index.rs:rebuild_index]
- `export_junit` maps every non-`ok` status to a `<failure>` testcase; XML text
  is escaped [export.rs:export_junit], [export.rs:xml_escape]

## Side Effects

- `append_event` creates the log directory and appends one JSON line to
  `events-<date>.jsonl` [runtime.rs:append_event]
- `compact_logs` writes `.jsonl.gz` files (flate2 gzip) and removes the
  original logs [runtime.rs:compact_logs]
- `rebuild_index` opens/creates the SQLite database (rusqlite bundled), drops
  and recreates the `events` table, and inserts one row per event
  [index.rs:rebuild_index]
- `iter_events` reads every `events-*.jsonl` file under `log_dir`
  [runtime.rs:iter_events]

## Verification

- Run: `cargo test -p swe-seed-core`
