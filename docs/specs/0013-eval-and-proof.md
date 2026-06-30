# 0013 — Eval and Proof Subsystem

## Purpose

Define SWE_Seed's **proof gate**: the evaluation system (`EvalSpec`/`EvalCheck`/`EvalResult`)
and the `ProofRecord` that determines whether work is complete and whether a capability may
be activated/promoted. This is the proven Harness-layer mechanism the rewrite must
reproduce (`eval run`, `validate`, `doctor`).

## Non-goals

- Not an LLM judge at runtime — eval checks are deterministic commands/rules over artifacts.
- Not the trace system (0014) — proof references traces but is separate.
- Not the learning loop (0016) — adaptation eligibility is computed here, applied there.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `harness.baml` | `EvalClass{ProductOutcome,ProcessCompliance,LearningQuality,AdaptationEligibility}`, `EvalStatus{Pass,Fail,Waived,Inconclusive}`, `EvalCheck`, `EvalSpec{frozen_after_handoff}`, `EvalCheckResult`, `EvalResult`, `ProofRecord{claims,evidence,skipped_checks,unresolved_risks}`, `ProofDisposition{Required,Optional,SkippedWithReason}` |
| `scripts/harness.py:863` `evaluate_check`, `:1167` `validate`, `:1499` `doctor`, `eval run` dispatch | Deterministic check evaluation + proof |

## SWE_Seed requirements

1. **Four eval classes** gate four concerns: `ProductOutcome` (does the work do the thing),
   `ProcessCompliance` (was the route/loop followed), `LearningQuality`, `AdaptationEligibility`.
2. **EvalSpec is frozen after handoff** (`frozen_after_handoff`) — once an agent receives an
   eval spec, its checks cannot be edited to make work pass. Enforced.
3. **`eval run`** executes each `EvalCheck` against `target_path`, producing `EvalCheckResult`s
   and an `EvalResult` with overall `EvalStatus` per `pass_condition`.
4. **ProofRecord** records `claims`, `evidence` (SourceRefs), `skipped_checks`, and
   `unresolved_risks`. A claim without evidence is invalid.
5. **Live-proof rule (unifies with 0007/0011):** a capability is activated/promoted only on
   a `Pass` from a **live** proof run; `Waived`/`Inconclusive`/simulation never promote.
6. `Waived` requires a recorded reason (`ProofDisposition::SkippedWithReason`).

## Data model

`EvalSpec`, `EvalCheck`, `EvalResult`, `EvalCheckResult`, `ProofRecord`, `ProofDisposition`,
`EvalClass`, `EvalStatus` — fields exactly as `harness.baml` (canonical schema, 0019).

```toml
# example eval spec (rendered from EvalSpec)
id = "eval-route-test"
target_type = "route"
target_path = ".agent-harness/routes/test.json"
eval_classes = ["ProcessCompliance", "ProductOutcome"]
pass_condition = "all required checks Pass"
frozen_after_handoff = true
[[checks]]
id = "proof-command-present"
eval_class = "ProcessCompliance"
check_type = "file_contains"
target = "proof"
required = true
rule = "route.proof is non-empty"
evidence_required = "proof command list"
```

- **Rust**: `swe_seed::eval` (`spec.rs`, `check.rs`, `result.rs`, `proof.rs`).
- **Validation**: every `required` check must have a result; `frozen_after_handoff` spec
  hash must match the handed-off hash; claims must cite evidence.

## CLI behavior

```
swe-seed eval run --spec <path> [--output <path>]   # → EvalResult JSON
swe-seed validate                                    # static checks (skill/route/markers)
swe-seed doctor                                      # validate + eval + boundary checks (0008)
```

## Generated files

`EvalResult` JSON under `.agent-harness/eval/` (or configured); `ProofRecord` under traces
(0014). Both content-hashed.

## Rust module boundaries

`swe_seed::eval` consumed by `swe_seed::doctor` (0008) and the learning loop (0016).

## Security and provenance considerations

Frozen eval specs prevent "edit the test to pass." `Waived` always needs a reason.
ProofRecord `unresolved_risks` must be surfaced, never silently dropped.

## Tests

- A `required` check with no result → `EvalResult` cannot be `Pass`.
- Editing a frozen spec after handoff → validation fails (hash mismatch).
- A claim with no evidence → ProofRecord rejected.
- `Waived` without reason → rejected.

## Open questions

- Golden-file parity target for `eval run` output vs current Python? (Recommend yes.)

## Acceptance criteria

- [ ] 4 eval classes + 4 statuses implemented; `eval run` deterministic.
- [ ] Frozen-after-handoff enforced; claims require evidence.
- [ ] Live-pass-only promotion rule wired to activation (0007/0011).
