# doctor

## Overview

Doctor and drift detection (spec 0008). Aggregates four checks into one
`DoctorReport`: boundary validation of the assembled manifest, the core eval
run, frozen-integrity of the core eval spec, and manifest drift against a
recorded baseline. Exits non-zero iff any check is `Fail`.

## Interface

- `run_doctor(root: &Path) -> DoctorReport`: runs the full check set and
  computes the overall status [report.rs:run_doctor]
- `pub enum DoctorStatus { Pass, Fail, Warn }` with
  `#[serde(rename_all = "PascalCase")]` [report.rs:DoctorStatus]
- `pub struct DoctorCheck { name, status, detail }` [report.rs:DoctorCheck]
- `pub struct DoctorReport { checks, overall }` [report.rs:DoctorReport]
- `pub const CORE_EVAL_SPEC: &str = "tests/fixtures/eval.toml"`: repo-relative
  core eval spec path doctor runs against [report.rs:CORE_EVAL_SPEC]
- `boundary_check(root: &Path) -> DoctorCheck`: boundary validation on the
  assembled manifest via `assemble_default` + `validate_boundaries`
  [check.rs:boundary_check]
- `eval_check(root, spec_path) -> DoctorCheck`: runs the core eval spec; Pass
  iff `EvalStatus::Pass` [check.rs:eval_check]
- `frozen_check(root, spec_path) -> DoctorCheck`: frozen-after-handoff integrity
  via `check_frozen` [check.rs:frozen_check]
- `manifest_drift_check(root: &Path) -> DoctorCheck`: records or compares the
  manifest hash in `<root>/.swe-seed/manifest.sha256` [drift.rs:manifest_drift_check]

## Invariants

- Overall status precedence is `Fail` > `Warn` > `Pass`: any `Fail` fails the
  run, otherwise any `Warn` makes overall `Warn` (non-clean runs are not
  hidden), and only an all-`Pass` run is `Pass` [report.rs:run_doctor]
- The check set is consistent even when the core spec is absent: missing
  `tests/fixtures/eval.toml` yields `Warn` checks named `eval` and
  `frozen-integrity` rather than dropping the checks [report.rs:run_doctor]
- Manifest drift is warn-level and non-fatal: a changed manifest is surfaced but
  never fails doctor [drift.rs:manifest_drift_check]
- On drift, the baseline sidecar is updated to the current hash so the next run
  compares against the latest state while the change is still surfaced
  [drift.rs:manifest_drift_check]

## Side Effects

- Writes `<root>/.swe-seed/manifest.sha256`: creates the `.swe-seed` directory
  and baseline file on first run, rewrites it when drift is detected
  [drift.rs:manifest_drift_check]
- `eval_check` executes the eval run against the spec file (delegates to
  `crate::eval::run_eval` with `use_llm = false`) [check.rs:eval_check]

## Verification

- Run: `cargo test -p swe-seed-core`
