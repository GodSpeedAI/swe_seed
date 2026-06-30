# 0016 — Learning and Adaptation Loop

## Purpose

Define the self-improvement loop: how a finished run produces a reflection, becomes a
learning record, and is governed into a skill proposal, regression case, or harness ADR —
under proof-gated adaptation eligibility. This is the **agentic capability loop's inner
mechanism**, entirely missing from specs 0002–0011.

## Non-goals

- Not auto-applying changes: the loop *proposes*; promotion is gated by eval + human/policy.
- Not the SEA federation (0011) — this loop is local and always works.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `harness.baml` | `ReflectionTemplate{prompts,evidence_required,no_change_allowed}`, `LearningRecord{disposition,...}`, `LearningDisposition{ApprovedLesson,RejectedLesson,SkillProposal,RegressionCase,HarnessADR,NoChange}`, `LearningCandidate{candidate_type,claim,evidence,scope,confidence,promotion_status,required_regression_case}`, `SkillProposal{observed_problem,proposed_behavior,evals,rollout_plan,rollback_plan}`, `RegressionCase{failure_mode,detection,future_rule,linked_eval_check}`, `AdaptationDecision{product/process/learning/adaptation_result,allowed_adaptations,blocked_adaptations,required_next_actions}` |

## SWE_Seed requirements

1. **Reflection** (`ReflectionTemplate`) runs after a finished trace (0014); `no_change_allowed`
   means an explicit `NoChange` disposition is allowed, but it must still be recorded with
   evidence rather than silently skipped.
2. **LearningRecord** classifies the reflection into a `LearningDisposition`
   (ApprovedLesson | RejectedLesson | SkillProposal | RegressionCase | HarnessADR | NoChange)
   with a `decision_reason` and optional `follow_up_artifact`.
3. **AdaptationDecision** computes, from the four eval results (0013), which adaptations are
   `allowed` vs `blocked`. **Adaptation eligibility is proof-gated:** a failing
   `AdaptationEligibility` eval blocks promotion regardless of other results.
4. **LearningCandidate → promotion**: a candidate may promote to a `SkillProposal` only with
   evidence and (if required) a `RegressionCase`. `promotion_status` tracks the gate.
5. **SkillProposal** must include `evals` (how to verify the new behavior), a `rollout_plan`
   and a `rollback_plan` — no proposal without a rollback.
6. **RegressionCase** links to an `EvalCheck` (`linked_eval_check`) so the failure is caught
   in future runs (`future_rule`). This is how lessons become permanent.

## Data model

All types canonical from `harness.baml` (0019). Loop order:

```
finished trace → ReflectionTemplate → LearningRecord → (LearningCandidate)
   → AdaptationDecision (gated by 4 eval results) → SkillProposal | RegressionCase | HarnessADR | NoChange
```

- **Rust**: `swe_seed::learning` (`reflection.rs`, `record.rs`, `candidate.rs`,
  `adaptation.rs`, `proposal.rs`, `regression.rs`).
- **Validation**: SkillProposal requires evals + rollback; RegressionCase requires a linked
  eval check; AdaptationDecision must cite the four eval results.

## CLI behavior

```
swe-seed reflect <trace>            # → LearningRecord (proposed)
swe-seed learn promote <record>     # → SkillProposal | RegressionCase (gated)
swe-seed adapt <run-id>             # → AdaptationDecision from eval results
```

(Exact verbs may map onto existing `trace distill`/`eval`; align during the rewrite.)

## Generated files

`LearningRecord`/`SkillProposal`/`RegressionCase`/`AdaptationDecision` JSON under
`.agent-harness/learning/` (or configured); regression cases feed back into eval specs (0013).

## Rust module boundaries

`swe_seed::learning`; depends on `eval` (0013) and `trace` (0014); emits skills into the
ingestion path (0007) and ADRs into doctrine.

## Security and provenance considerations

No proposal promotes without proof + rollback. Regression cases make a lesson enforceable.
`no_change_allowed=false` means reflection must produce a non-`NoChange` disposition or a
blocked decision with a reason; silent learning skips are never valid.

## Tests

- A SkillProposal without a rollback plan → rejected.
- A blocked `AdaptationEligibility` eval → AdaptationDecision blocks promotion.
- A promoted RegressionCase creates/links an `EvalCheck` that fails on recurrence.
- Reflection with `no_change_allowed=false` requires an explicit disposition.

## Open questions

- Where do promoted skills land — the registry (0003) directly, or a review queue first?
  (Recommend review queue → eval pass → registry.)

## Acceptance criteria

- [ ] Reflection→Learning→Adaptation→Proposal/Regression loop implemented.
- [ ] Adaptation proof-gated; proposals require rollback; regressions link eval checks.
- [ ] Lessons become enforceable regression checks.
