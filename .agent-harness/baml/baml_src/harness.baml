// BAML source contract for typed harness artifact generation.
// Root specs remain the normative source of truth; this file is generated or reviewed from them.

enum ArtifactStatus {
  Draft
  Candidate
  Active
  Deprecated
  Rejected
}

enum ProofDisposition {
  Required
  Optional
  SkippedWithReason
}

enum LearningDisposition {
  ApprovedLesson
  RejectedLesson
  SkillProposal
  RegressionCase
  HarnessADR
  NoChange
}

enum EvalClass {
  ProductOutcome
  ProcessCompliance
  LearningQuality
  AdaptationEligibility
}

enum EvalStatus {
  Pass
  Fail
  Waived
  Inconclusive
}

class SourceRef {
  path string
  summary string
}

class ValidationRequirement {
  check string
  blocking bool
  evidence string
}

class HarnessNeed {
  request string
  desired_outcome string
  constraints string[]
  non_goals string[]
  proof_requirements string[]
}

class HarnessADR {
  id string
  title string
  status ArtifactStatus
  context string
  decision string
  alternatives string[]
  consequences string[]
  rollback_plan string
  validation ValidationRequirement[]
  sources SourceRef[]
}

class RouteCard {
  id string
  job_type string
  purpose string
  semantic_triggers string[]
  positive_examples string[]
  negative_examples string[]
  required_context string[]
  required_skills string[]
  work_loop string[]
  required_artifacts string[]
  proof string[]
  done_when string[]
  failure_modes string[]
  fallback_policy string
  validation ValidationRequirement[]
  sources SourceRef[]
}

class SkillIR {
  id string
  version string
  category string
  jtbd string
  description string
  triggers string[]
  inputs string[]
  procedure string[]
  evidence_required string[]
  forbidden_behaviors string[]
  outputs string[]
  success_criteria string[]
  failure_modes string[]
  render_targets string[]
  status ArtifactStatus
  validation ValidationRequirement[]
  sources SourceRef[]
}

class EvalCheck {
  id string
  eval_class EvalClass
  check_type string
  target string
  required bool
  rule string
  evidence_required string
}

class EvalSpec {
  id string
  version string
  run_id string
  target_type string
  target_path string
  purpose string
  eval_classes EvalClass[]
  checks EvalCheck[]
  pass_condition string
  outputs string[]
  frozen_after_handoff bool
  sources SourceRef[]
}

class EvalCheckResult {
  id string
  eval_class EvalClass
  status EvalStatus
  evidence string
  failure_reason string?
}

class EvalResult {
  eval_id string
  run_id string
  status EvalStatus
  checks EvalCheckResult[]
  summary string
  created_at string
}

class ProofRecord {
  id string
  run_id string
  claims string[]
  evidence SourceRef[]
  skipped_checks string[]
  unresolved_risks string[]
}

class AdaptationDecision {
  run_id string
  decision_id string
  product_result EvalStatus
  process_result EvalStatus
  learning_result EvalStatus
  adaptation_result EvalStatus
  allowed_adaptations string[]
  blocked_adaptations string[]
  reason string
  required_next_actions string[]
  created_artifacts string[]
}

class LearningCandidate {
  id string
  source_run_id string
  candidate_type string
  claim string
  evidence SourceRef[]
  scope string
  confidence string
  promotion_status string
  required_regression_case string?
}

class ContextBudget {
  id string
  max_context_size string
  required_files string[]
  optional_files string[]
  excluded_files string[]
  freshness_requirements string[]
  relevance_rules string[]
  summarization_rules string[]
  validation ValidationRequirement[]
}

class ContextPack {
  id string
  route_card_id string
  budget_id string
  included_files string[]
  excluded_files string[]
  salient_facts string[]
  stale_context_warnings string[]
  validation ValidationRequirement[]
}

class HookPolicy {
  id string
  events string[]
  blocking_rules string[]
  advisory_rules string[]
  payload_fields string[]
  failure_behavior string
  validation ValidationRequirement[]
}

class PermissionPolicy {
  id string
  allowed_actions string[]
  approval_gated_actions string[]
  forbidden_actions string[]
  validation ValidationRequirement[]
}

class TraceSchema {
  id string
  required_fields string[]
  optional_fields string[]
  identifier_fields string[]
  proof_fields string[]
  validation ValidationRequirement[]
}

class ReflectionTemplate {
  id string
  prompts string[]
  evidence_required string[]
  no_change_allowed bool
  validation ValidationRequirement[]
}

class LearningRecord {
  id string
  disposition LearningDisposition
  summary string
  evidence SourceRef[]
  decision_reason string
  follow_up_artifact string?
  validation ValidationRequirement[]
}

class SkillProposal {
  id string
  proposed_skill_id string
  observed_problem string
  evidence SourceRef[]
  proposed_behavior string[]
  evals EvalCheck[]
  rollout_plan string
  rollback_plan string
  validation ValidationRequirement[]
}

class RegressionCase {
  id string
  source_run_id string
  failure_mode string
  detection string
  future_rule string
  status string
  linked_eval_check string
}

class RegenerationInput {
  harness_spec_path string
  approved_route_cards string[]
  approved_skill_ir_files string[]
  eval_specs string[]
  hook_policies string[]
  approved_learning_records string[]
  regression_cases string[]
}

class RegenerationPlan {
  source_artifacts SourceRef[]
  render_targets string[]
  preserved_decisions string[]
  blocking_diffs string[]
  validation ValidationRequirement[]
}

function GenerateRouteCard(need: HarnessNeed, adr: HarnessADR?, existing_routes: RouteCard[]) -> RouteCard {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate one executable harness RouteCard from the need.
    Preserve intent, constraints, non-goals, proof requirements, and done conditions.
    Return only the typed RouteCard.

    Need:
    {{ need }}

    ADR:
    {{ adr }}

    Existing routes:
    {{ existing_routes }}

    {{ ctx.output_format }}
  "#
}

function GenerateSkillIR(route_card: RouteCard, traces: SourceRef[], reflection: ReflectionTemplate?) -> SkillIR {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a reusable SkillIR that supports the route card without expanding scope.
    Include trigger conditions, procedure, evidence, forbidden behavior, outputs, eval hooks, and status.

    Route card:
    {{ route_card }}

    Trace evidence:
    {{ traces }}

    Reflection template:
    {{ reflection }}

    {{ ctx.output_format }}
  "#
}

function GenerateEvalSpec(route_card: RouteCard, target: SourceRef, constraints: string[], regression_cases: RegressionCase[]) -> EvalSpec {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a local EvalSpec draft for the routed behavior.
    Include product outcome, process compliance, learning quality, and adaptation eligibility classes.
    Use only local EvalSpec fields. The active eval freezes after handoff.

    Route card:
    {{ route_card }}

    Target:
    {{ target }}

    Constraints:
    {{ constraints }}

    Regression cases:
    {{ regression_cases }}

    {{ ctx.output_format }}
  "#
}

function GenerateEvalChecklist(eval_spec: EvalSpec) -> SourceRef {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Render a concise manual checklist from the EvalSpec.
    Every required check must map to required evidence.

    EvalSpec:
    {{ eval_spec }}

    {{ ctx.output_format }}
  "#
}

function GenerateProofRecord(eval_result: EvalResult, observations: SourceRef[]) -> ProofRecord {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a ProofRecord from EvalResult and observations.
    Map claims to eval evidence or explicit skipped-check justifications.

    EvalResult:
    {{ eval_result }}

    Observations:
    {{ observations }}

    {{ ctx.output_format }}
  "#
}

function GenerateContextBudget(route_card: RouteCard, specs: SourceRef[], memory_map: SourceRef[]) -> ContextBudget {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a bounded ContextBudget for this route.
    Specify required, optional, excluded, stale, relevance, and summarization rules.

    Route card:
    {{ route_card }}

    Specs:
    {{ specs }}

    Memory:
    {{ memory_map }}

    {{ ctx.output_format }}
  "#
}

function GenerateContextPack(route_card: RouteCard, budget: ContextBudget, available_context: SourceRef[]) -> ContextPack {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a route-specific ContextPack within the ContextBudget.
    Keep only context that changes the next action.

    Route card:
    {{ route_card }}

    Budget:
    {{ budget }}

    Available context:
    {{ available_context }}

    {{ ctx.output_format }}
  "#
}

function GenerateHookPolicy(route_card: RouteCard, trace_schema: TraceSchema, safety_rules: PermissionPolicy) -> HookPolicy {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a HookPolicy that binds lifecycle events to route, proof, trace, and safety behavior.

    Route card:
    {{ route_card }}

    Trace schema:
    {{ trace_schema }}

    Permission policy:
    {{ safety_rules }}

    {{ ctx.output_format }}
  "#
}

function GeneratePermissionPolicy(safety_rules: string[], route_card: RouteCard) -> PermissionPolicy {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a PermissionPolicy for this route.
    Split actions into allowed, approval-gated, and forbidden lists.

    Safety rules:
    {{ safety_rules }}

    Route card:
    {{ route_card }}

    {{ ctx.output_format }}
  "#
}

function GenerateTraceSchema(route_cards: RouteCard[], proof_model: string[]) -> TraceSchema {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a TraceSchema that connects request, route, files, commands, proof, risks, and claim.

    Route cards:
    {{ route_cards }}

    Proof model:
    {{ proof_model }}

    {{ ctx.output_format }}
  "#
}

function GenerateReflectionTemplate(route_card: RouteCard, trace_schema: TraceSchema, eval_result: EvalResult?) -> ReflectionTemplate {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate an evidence-bearing ReflectionTemplate for this route.
    Include a no-change option and require eval/proof citations.

    Route card:
    {{ route_card }}

    Trace schema:
    {{ trace_schema }}

    Eval result:
    {{ eval_result }}

    {{ ctx.output_format }}
  "#
}

function GenerateAdaptationDecision(eval_result: EvalResult, reflection: SourceRef, proof_record: ProofRecord) -> AdaptationDecision {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate an AdaptationDecision.
    Apply the eval-to-adaptation map. No adaptation without evaluation.

    EvalResult:
    {{ eval_result }}

    Reflection:
    {{ reflection }}

    ProofRecord:
    {{ proof_record }}

    {{ ctx.output_format }}
  "#
}

function GenerateLearningRecord(trace: SourceRef, reflection: SourceRef, proof: SourceRef[]) -> LearningRecord {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Distill the run into one LearningRecord.
    Choose approved lesson, rejected lesson, skill proposal, regression case, HarnessADR, or no-change.

    Trace:
    {{ trace }}

    Reflection:
    {{ reflection }}

    Proof:
    {{ proof }}

    {{ ctx.output_format }}
  "#
}

function ProposeLearningCandidate(reflection: SourceRef, eval_result: EvalResult, proof_record: ProofRecord) -> LearningCandidate {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Propose one LearningCandidate only when the reflection is grounded in eval and proof evidence.
    Include claim, evidence, scope, confidence, and promotion status.

    Reflection:
    {{ reflection }}

    EvalResult:
    {{ eval_result }}

    ProofRecord:
    {{ proof_record }}

    {{ ctx.output_format }}
  "#
}

function ProposeSkillFromEvaluation(candidate: LearningCandidate, eval_spec: EvalSpec, eval_result: EvalResult) -> SkillProposal {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Propose a SkillIR change from an evaluated LearningCandidate.
    Include trigger, guardrails, rollout, rollback, and proposed eval checks.

    LearningCandidate:
    {{ candidate }}

    EvalSpec:
    {{ eval_spec }}

    EvalResult:
    {{ eval_result }}

    {{ ctx.output_format }}
  "#
}

function ProposeRegressionCase(failed_eval: EvalResult, trace: SourceRef, proof_record: ProofRecord) -> RegressionCase {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Convert the failed eval into a regression case that future validation can catch.
    Link the failure to an eval check and future rule.

    Failed eval:
    {{ failed_eval }}

    ProofRecord:
    {{ proof_record }}

    Trace:
    {{ trace }}

    {{ ctx.output_format }}
  "#
}

function GenerateHarnessADR(need: HarnessNeed, alternatives: string[], constraints: string[]) -> HarnessADR {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a HarnessADR for a major harness decision.
    Include decision, alternatives, consequences, validation, and rollback.

    Need:
    {{ need }}

    Alternatives:
    {{ alternatives }}

    Constraints:
    {{ constraints }}

    {{ ctx.output_format }}
  "#
}

function RegenerateHarnessArtifacts(inputs: RegenerationInput) -> RegenerationPlan {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a regeneration plan for harness artifacts.
    Preserve explicit human decisions, active and deprecated skills, regression cases, and traceability.
    Report blocking diffs instead of overwriting reviewed changes.

    Inputs:
    {{ inputs }}

    {{ ctx.output_format }}
  "#
}
