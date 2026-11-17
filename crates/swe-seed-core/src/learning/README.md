# learning

## Overview

Learning and adaptation loop (spec 0016): `finished trace ->
ReflectionTemplate -> LearningRecord -> (LearningCandidate) ->
AdaptationDecision (gated by the four eval classes) -> SkillProposal |
RegressionCase | HarnessADR | NoChange`. Depends on `eval` (0013) and `trace`
(0014). No proposal promotes without proof and a rollback plan; regression
cases make a lesson enforceable as a permanent eval check.

## Interface

- `pub enum LearningDisposition { ApprovedLesson, RejectedLesson,
SkillProposal, RegressionCase, HarnessADR, NoChange }` (harness.baml
  `LearningDisposition`, `BamlParity`, PascalCase serde)
  [record.rs:LearningDisposition]
- `LearningRecord { id, disposition, summary, evidence, decision_reason,
follow_up_artifact, validation }` (harness.baml `LearningRecord`,
  `BamlParity`) and `validate_learning_record(record)`
  [record.rs:LearningRecord], [record.rs:validate_learning_record]
- `LearningCandidate { id, source_run_id, candidate_type, claim, evidence,
scope, confidence, promotion_status, required_regression_case }` (harness.baml
  `LearningCandidate`, `BamlParity`) with `can_promote_candidate(candidate) ->
PromotionOutcome { Allow { into }, Block { reason } }`
  [candidate.rs:LearningCandidate], [candidate.rs:can_promote_candidate],
  [candidate.rs:PromotionOutcome]
- `ReflectionTemplate { id, prompts, evidence_required, no_change_allowed,
validation }` (harness.baml `ReflectionTemplate`, `BamlParity`);
  `default_template()` [reflection.rs:ReflectionTemplate],
  [reflection.rs:default_template]
- `reflect(template, trace, trace_path) -> Result<LearningRecord, String>`:
  deterministic (no LLM) reflection over a finished `TraceRecord`; a completion
  claim yields `ApprovedLesson`, its absence an explicit `NoChange`
  [reflection.rs:reflect]
- `validate_reflection(template, record)`: evidence always required; a
  `NoChange` under `no_change_allowed=false` needs explicit rationale
  [reflection.rs:validate_reflection]
- `AdaptationDecision { run_id, decision_id, product_result, process_result,
learning_result, adaptation_result, allowed_adaptations, blocked_adaptations,
reason, required_next_actions, created_artifacts }` (harness.baml
  `AdaptationDecision`, `BamlParity`) with `promotion_allowed()` and
  `regression_registration_allowed()` [adaptation.rs:AdaptationDecision],
  [adaptation.rs:promotion_allowed],
  [adaptation.rs:regression_registration_allowed]
- `build_adaptation_decision(run_id, results) -> AdaptationDecision`: aggregates
  eval results per `EvalClass`; the four adaptation kinds are `skill_promotion`,
  `regression_registration`, `adr_update`, `lesson_publish`
  [adaptation.rs:build_adaptation_decision], [adaptation.rs:ADAPTATIONS]
- `promotion_gate(decision)`: a proposal may promote only when the decision
  grants `skill_promotion` [adaptation.rs:promotion_gate]
- `SkillProposal { id, proposed_skill_id, observed_problem, evidence,
proposed_behavior, evals, rollout_plan, rollback_plan, validation }`
  (harness.baml `SkillProposal`, `BamlParity`) with
  `validate_proposal(proposal)` [proposal.rs:SkillProposal],
  [proposal.rs:validate_proposal]
- `RegressionCase { id, source_run_id, failure_mode, detection, future_rule,
status, linked_eval_check }` (harness.baml `RegressionCase`, `BamlParity`)
  with `validate_regression(case)` and
  `eval_check_for_regression(case) -> Result<EvalCheck, String>`
  [regression.rs:RegressionCase], [regression.rs:validate_regression],
  [regression.rs:eval_check_for_regression]

## Invariants

- Adaptation eligibility is proof-gated: a non-`Pass` `AdaptationEligibility`
  eval blocks EVERY adaptation; promotion never overrides an eligibility
  failure [adaptation.rs:build_adaptation_decision]
- `skill_promotion` (and `regression_registration`) additionally require
  `ProductOutcome` and `LearningQuality` to `Pass`; `adr_update` requires
  `ProcessCompliance`; `lesson_publish` requires `LearningQuality`
  [adaptation.rs:build_adaptation_decision]
- Missing eval classes default to `Inconclusive` (fail-closed: an unknown gate
  never grants trust), and any `Fail` dominates the per-class aggregate
  [adaptation.rs:per_class], [adaptation.rs:combine]
- Skill promotion requires regression registration to be a distinct grant:
  `promotion_allowed()` checks exactly `skill_promotion` and is not satisfied by
  `regression_registration` [adaptation.rs:promotion_allowed]
- No proposal without a rollback: `validate_proposal` requires non-empty
  observed_problem, proposed_behavior, evals, rollout_plan, and rollback_plan
  [proposal.rs:validate_proposal]
- No silent learning: every record needs a summary and a `decision_reason`, and
  any disposition other than `NoChange` requires evidence
  [record.rs:validate_learning_record]
- A candidate promotes only with a non-empty claim, non-empty `candidate_type`,
  evidence, and (when required) a linked regression case id
  [candidate.rs:can_promote_candidate]
- A regression without a linked eval check can never fire and is rejected;
  a `require:`-prefixed `future_rule` must carry rule content
  [regression.rs:validate_regression]
- A regression encodes its failure mode as a `static_forbidden_patterns` check
  (or `static_required_patterns` when `future_rule` starts with `require:`),
  always `required`, in class `LearningQuality`
  [regression.rs:eval_check_for_regression]

## Side Effects

- None: this module is pure data construction and validation; reflection reads
  the caller-supplied `TraceRecord` and cites the caller-supplied trace path,
  and no file, network, or process effect exists in any submodule
  [reflection.rs:reflect], [record.rs:validate_learning_record]

## Verification

- Run: `cargo test -p swe-seed-core`
