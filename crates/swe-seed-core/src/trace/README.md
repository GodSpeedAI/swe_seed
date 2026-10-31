# trace

## Overview

Trace and durable-decision records (spec 0014). Lifecycle ports
`scripts/harness.py` `trace_*` so Rust records stay compatible with captured
Python output: `start` (route + genesis), `append`, `checkpoint`, `resume`,
`distill`, `finish`. Records live under
`.agent-harness/traces/records/<trace_id>.json` and every mutation is mirrored
best-effort into the tamper-evident SQLite trace ledger.

## Interface

- `pub struct TraceRecord { trace_id, created_at, task, route_decision_record,
route, events, verification, unresolved_risks, completion_claim }` (field
  order matches the captured Python records; `events` and `verification` are
  heterogeneous `serde_json::Value`s) with `TraceRecord::load(path)`,
  `TraceRecord::save(path)`, `latest_checkpoint()`
  [record.rs:TraceRecord], [record.rs:load], [record.rs:save],
  [record.rs:latest_checkpoint]
- `CompletionClaim { at, claim }` [record.rs:CompletionClaim]
- `pub struct TraceSchema { id, required_fields, optional_fields,
identifier_fields, proof_fields, validation }` (harness.baml `TraceSchema`,
  `BamlParity`) with `trace_schema()` (identifiers: `trace_id`, `task`) and
  `missing_identifiers(schema, record) -> Vec<String>`
  [schema.rs:TraceSchema], [schema.rs:trace_schema],
  [schema.rs:missing_identifiers]
- `resolve_trace_path(root, trace) -> Result<PathBuf>`: resolves a generated
  timestamp-slug trace id to its record file [lifecycle.rs:resolve_trace_path]
- `start(root, task) -> Result<Value>`: routes the task, writes the route
  decision + trace record + genesis ledger entry; returns
  `{ trace_id, trace_record, route_decision_record }` [lifecycle.rs:start]
- `append(root, trace, note)`: appends a `trace.note` event
  [lifecycle.rs:append]
- `checkpoint(root, trace, stage, summary, next_action, artifacts,
unresolved_risks)`: appends a `trace.checkpoint` event and updates
  `unresolved_risks` when supplied [lifecycle.rs:checkpoint]
- `resume(root, trace)`: prints the resume summary (last checkpoint, risks,
  claim) [lifecycle.rs:resume]
- `finish(root, trace, claim, command, result)`: requires a claim; sets
  `completion_claim` (+ optional verification entry) [lifecycle.rs:finish]
- `distill(root, trace) -> Result<Value>`: produces the learning review from
  the record (memory/skill/harness update candidates with destinations
  `successful-patterns`, `failure-patterns`, `open-questions`)
  [lifecycle.rs:distill]

## Invariants

- Every update path re-derives the record from disk and rejects tampering:
  `load_for_update` refuses symlinks/non-files, requires the record's
  `trace_id` to match the requested trace, and requires a routing genesis in
  the ledger whose payload (task, job_type, route_card) matches the record
  [lifecycle.rs:load_for_update]
- The genesis is mandatory: `start` appends the `RouteSelected` genesis entry
  to the tamper-evident ledger and the trace record is created only after
  routing [lifecycle.rs:start], [lifecycle.rs:ledger_append_result]
- All persisted task/note/stage/summary/artifact/claim text is redacted via
  `redact_secrets` before storage [lifecycle.rs:start],
  [lifecycle.rs:append], [lifecycle.rs:checkpoint], [lifecycle.rs:finish]
- `save` is atomic and symlink-refusing: writes to a temp file then renames,
  and refuses to write through a symlink [record.rs:save]
- The records directory path is validated component-by-component: a symlinked
  component or non-directory bails instead of writing through it
  [lifecycle.rs:records_dir]
- Trace ids must be generated timestamp-slugs (16-char stamp with `T`/`Z`
  separators plus a lowercase/digit/dash slug); arbitrary paths are rejected
  [lifecycle.rs:resolve_trace_path]
- `finish` is claim-required; `distill` classifies verification as `missing`
  (no verification entries), `partial` (risks or no success marker), or
  `verified` (a success marker and no unresolved risks)
  [lifecycle.rs:finish], [lifecycle.rs:classify_verification_status]
- Ledger appends outside `start` are best-effort: failures warn to stderr and
  never break the trace command [lifecycle.rs:ledger_append]

## Side Effects

- `start` writes the route-decision record, the trace record JSON, and the
  genesis ledger entry (SQLite via `trace_ledger`) [lifecycle.rs:start]
- `append` / `checkpoint` / `finish` rewrite the trace record file atomically
  and append `TraceNote` / `TraceCheckpoint` / `TraceFinished` ledger entries
  [lifecycle.rs:append], [lifecycle.rs:checkpoint], [lifecycle.rs:finish]
- `records_dir` creates `.agent-harness/traces/records/` when absent
  [lifecycle.rs:records_dir]
- `TraceRecord::load` reads the record JSON; the ledger opens the SQLite
  database at the default path [record.rs:load], [lifecycle.rs:load_for_update]

## Verification

- Run: `cargo test -p swe-seed-core`
