# eval

## Overview

Eval and proof (spec 0013). Deterministic rule-based evaluation across the four
eval classes (`ProductOutcome`, `ProcessCompliance`, `LearningQuality`,
`AdaptationEligibility`), frozen-after-handoff enforcement for eval specs,
proof records that require per-claim evidence, declared-proof export for an
independent verifier, and live-pass-only promotion. All data types declare BAML
parity with their `harness.baml` counterparts.

## Interface

- `pub enum EvalClass { ProductOutcome, ProcessCompliance, LearningQuality,
AdaptationEligibility }` (`BamlParity`, PascalCase serde)
  [mod.rs:EvalClass]
- `pub enum EvalStatus { Pass, Fail, Waived, Inconclusive }` (`BamlParity`,
  PascalCase serde) [mod.rs:EvalStatus]
- `pub struct EvalSpec` (harness.baml `EvalSpec`, `BamlParity`) with
  `frozen_after_handoff` flag [spec.rs:EvalSpec]
- `load_eval_spec(path: &Path) -> Result<EvalSpec>`: reads the file, dispatching
  on extension [spec.rs:load_eval_spec]
- `parse_eval_spec(bytes: &[u8], path: &Path) -> Result<EvalSpec>`: parses JSON
  (`.json`) or TOML (default) and structurally validates
  [spec.rs:parse_eval_spec]
- `check_frozen(root, spec_path, spec) -> Result<()>`: pins a frozen spec's file
  bytes at first run under `.swe-seed/eval-handoff/<hash-of-id>.sha256` and
  fails on any later edit [spec.rs:check_frozen]
- `pub struct EvalCheck` (harness.baml `EvalCheck`: `id`, `eval_class`,
  `check_type`, `target`, `required` default true, `rule`, `evidence_required`)
  [check.rs:EvalCheck]
- `evaluate_check(root, check, trusted) -> (EvalStatus, String, Option<String>)`:
  deterministic evaluator for `artifact_consistency`, `file_exists`,
  `static_required_patterns`, `static_forbidden_patterns`, `command_check`,
  `json_schema_check`, and the manual family [check.rs:evaluate_check]
- `pub struct EvalCheckResult` and `pub struct EvalResult` (`BamlParity`)
  [result.rs:EvalCheckResult], [result.rs:EvalResult]
- `run_eval(root, spec_path, spec, trusted) -> Result<EvalResult>`: enforces
  frozen-after-handoff, evaluates each check, computes the overall status
  [result.rs:run_eval]
- `waived_requires_reason(r: &EvalCheckResult) -> Result<(), String>`: a waived
  check must carry a non-empty `failure_reason` [result.rs:waived_requires_reason]
- `pub struct ProofRecord` (harness.baml `ProofRecord`: `id`, `run_id`,
  `claims`, `evidence: Vec<SourceRef>`, `skipped_checks`, `unresolved_risks`)
  [proof.rs:ProofRecord]
- `validate_proof(record: &ProofRecord) -> Result<(), String>`: per-claim
  evidence linkage; claims map positionally to evidence [proof.rs:validate_proof]
- `pub const DECLARED_PROOF_SCHEMA: &str = "swe_seed.eval_proof.v1"` and
  `pub const PROOF_SHELL: &str = "sh"` [declared_proof.rs:DECLARED_PROOF_SCHEMA],
  [declared_proof.rs:PROOF_SHELL]
- `pub struct DeclaredProof { schema, spec_id, spec_version, spec_sha256,
check_id, required, argv, cwd }` [declared_proof.rs:DeclaredProof]
- `pub enum DeclaredProofError` with stable machine codes via `code()`
  (`invalid_pin`, `spec_unreadable`, `spec_digest_mismatch`, `spec_invalid`,
  `unknown_check`, `not_command_check`, `empty_rule`)
  [declared_proof.rs:DeclaredProofError], [declared_proof.rs:code]
- `declared_proof(path, pin, check_id) -> Result<DeclaredProof, DeclaredProofError>`:
  exports the proof a `command_check` declares, only if the spec bytes hash to
  the caller-supplied pin [declared_proof.rs:declared_proof]

## Invariants

- Frozen-after-handoff: a spec with `frozen_after_handoff = true` has its file
  bytes pinned on first run; any later byte change fails `check_frozen`
  [spec.rs:check_frozen]
- The frozen sidecar key is `content_hash(spec.id)`, chosen because a lossy
  char-map would collapse distinct ids like `a/b` and `a-b` onto one sidecar
  [spec.rs:check_frozen]
- `command_check` is opt-in only: without `trusted = true` the check fails
  closed with "command_check disabled: requires --trusted mode"; the shell is
  never spawned [check.rs:evaluate_check]
- `artifact_consistency` fails closed when `rule` does not record
  `sha256:<64 hex>`: without a recorded digest the check never degrades to an
  existence check [check.rs:evaluate_check]
- Manual-family checks (`manual_check`, `reflection_check`, `promotion_policy`)
  are always `Inconclusive` in a deterministic run; they never auto-pass
  [check.rs:evaluate_check]
- Live-pass-only promotion: a non-required failure is downgraded to `Waived`,
  never a pass, and `run_eval` is `Pass` only when no required check failed AND
  at least one check is a live pass [check.rs:evaluate_check],
  [result.rs:run_eval]
- Proof claims require positional per-claim evidence: `evidence.len()` must be
  at least `claims.len()` and each backing `SourceRef.path` must be non-empty
  [proof.rs:validate_proof]
- `declared_proof` executes nothing and writes nothing; the digest is not
  trusted on first use (the consumer pins the spec sha256 itself)
  [declared_proof.rs:declared_proof]
- An `EvalSpec` with an empty `checks` list is rejected
  [spec.rs:validate_structure]

## Side Effects

- `check_frozen` creates `.swe-seed/eval-handoff/` and writes/reads the
  `<key>.sha256` sidecar [spec.rs:check_frozen]
- `evaluate_check` reads target files from disk (`file_exists`,
  `static_*_patterns`, `json_schema_check`, `artifact_consistency`)
  [check.rs:evaluate_check]
- `command_check` with `trusted = true` spawns `sh -c <rule>` in the repository
  root via `std::process::Command` [check.rs:evaluate_check]
- `declared_proof` reads the spec file at `path` [declared_proof.rs:declared_proof]

## Verification

- Run: `cargo test -p swe-seed-core`
