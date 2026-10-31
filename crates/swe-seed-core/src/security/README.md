# security

## Overview

Security gate for skill ingestion (spec 0007): scan results with a five-state
status vocabulary, the external SkillSpector gate (never a faked pass), the
blocking projection gate, the activation gate (`source_hash` + terminal,
non-blocking scan), and persisted scan-finding waivers.

## Interface

- `pub enum ScanStatus { Pending, Clean, Warning, Critical, Error }` with
  `is_terminal()` (Clean/Warning/Critical) and `is_blocking()`
  (Critical/Error); PascalCase serde [scan_result.rs:ScanStatus],
  [scan_result.rs:is_terminal], [scan_result.rs:is_blocking]
- `ScanFinding { id, severity, message }` and `ScanResult { skill_id, status,
findings, detail }` with `ScanResult::pending(skill_id, detail)`
  [scan_result.rs:ScanFinding], [scan_result.rs:ScanResult],
  [scan_result.rs:pending]
- `scan_blocks_projection(scan, exceptions) -> bool`: a blocking status prevents
  projection unless every finding is explicitly waived
  [gate.rs:scan_blocks_projection]
- `can_activate(record: &SkillRecord, exceptions) -> bool`: activation requires
  provenance (`source_hash`) AND a terminal, non-blocking scan
  [gate.rs:can_activate]
- `Exceptions` with `waive(id)`, `waives(id)`, `len()`, `is_empty()`,
  `load(path)`, `save(path)`; `exceptions_store_path(root)` =
  `.swe-seed/skill-exceptions.json` [exceptions.rs:Exceptions],
  [exceptions.rs:load], [exceptions.rs:save], [exceptions.rs:store_path]
- `pub const SKILLSPECTOR_BIN: &str = "skillspector"`,
  `SKILLSPECTOR_TIMEOUT_SECS: u64 = 30`
  [skillspector.rs:SKILLSPECTOR_BIN], [skillspector.rs:SKILLSPECTOR_TIMEOUT_SECS]
- `skillspector_available() -> bool` [skillspector.rs:skillspector_available]
- `run_skillspector(skill_path)` and
  `run_skillspector_with(skill_path, available)`: run the external scanner with
  a hard timeout [skillspector.rs:run_skillspector],
  [skillspector.rs:run_skillspector_with]
- `parse_skillspector_output(id, output) -> ScanResult`: parses the zero-exit
  JSON contract `{"status": "clean"|"warning"|"critical", "findings": [...]}`
  [skillspector.rs:parse_skillspector_output]

## Invariants

- SkillSpector is never faked: binary not installed or timed out gives
  `Pending`; non-zero exit gives blocking `Error`; unrecognized output (including
  empty stdout or an unknown status string) stays `Pending` rather than being
  promoted to `Clean` [skillspector.rs:run_skillspector_with],
  [skillspector.rs:parse_skillspector_output]
- A scanner hang is killed after `SKILLSPECTOR_TIMEOUT_SECS` (30s) and reported
  as `Pending` ("timed out"), never as a pass [skillspector.rs:run_with_timeout]
- Fail closed on the status: a blocking status blocks even when `findings` is
  empty; an empty findings list never bypasses the projection gate
  [gate.rs:scan_blocks_projection]
- The block is lifted only if EVERY finding is waived; a single unwaived
  finding keeps the block [gate.rs:scan_blocks_projection]
- `Pending` is never treated as a pass and never activates: activation requires
  `is_terminal()` AND `!is_blocking()` AND non-empty `source_hash`
  [scan_result.rs:Pending], [gate.rs:can_activate]
- Waived finding ids persist as a sorted JSON array; a missing store loads as
  empty [exceptions.rs:save], [exceptions.rs:load]

## Side Effects

- `run_skillspector` spawns the external `skillspector` binary with piped
  stdout/stderr and kills it on timeout [skillspector.rs:run_skillspector_with],
  [skillspector.rs:run_with_timeout]
- `Exceptions::load` reads `.swe-seed/skill-exceptions.json`;
  `Exceptions::save` creates the parent directory and writes the JSON array
  [exceptions.rs:load], [exceptions.rs:save]
- `skillspector_available` walks `PATH` looking for the executable (read-only
  lookup) [skillspector.rs:skillspector_available]

## Verification

- Run: `cargo test -p swe-seed-core`
