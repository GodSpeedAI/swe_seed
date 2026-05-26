enum FabricatorArtifactStatus {
  Draft
  Proposed
  Approved
  Active
  Rejected
  Deprecated
}

enum FabricatorEvalClass {
  ProductOutcome
  ProcessCompliance
  LearningQuality
  AdaptationEligibility
}

class FabricatorSourceRef {
  artifact_type string
  artifact_id string
  path string
  version string?
}

class FabricatorValidationRequirement {
  id string
  command string?
  rule string
  evidence_required string[]
}

class TraceabilityLink {
  upstream_id string
  downstream_id string
  relation string
  waiver_id string?
}

class JobStory {
  id string
  when_clause string
  action_clause string
  outcome_clause string
  source_refs string[]
  waived bool?
  waiver_reason string?
}

class ProductSeed {
  id string
  version string
  status FabricatorArtifactStatus
  target_user string
  product_idea string
  constraints string[]
  non_goals string[]
  desired_prototype_outcome string
  proof_required string[]
  linked_artifacts FabricatorSourceRef[]
}

class ProductHypothesis {
  id string
  version string
  product_seed_id string
  hypothesis string?
  hypothesis_statement string
  target_job string
  target_user_segment string
  situation_context string
  pains string[]
  desired_gains string[]
  value_proposition string
  alternatives_today string[]
  channels string[]
  adoption_triggers string[]
  adoption_barriers string[]
  measurable_success string[]
  riskiest_assumptions string[]
  source_refs string[]
}

enum EARSPattern {
  Ubiquitous
  EventDriven
  StateDriven
  OptionalFeature
  UnwantedBehavior
}

class EARSRequirement {
  id string
  pattern EARSPattern
  system_name string
  condition string?
  trigger string?
  state string?
  feature string?
  unwanted_condition string?
  response string
  rationale string?
  source_refs string[]
  ears_text string
  testable bool
  non_testable_reason string?
}

class YStatement {
  id string
  context string
  constraint string
  chosen_option string
  neglected_alternatives string[]
  benefit string
  accepted_cost string
  linked_requirement_ids string[]
  y_text string
}

class ProductADR {
  id string
  version string
  product_seed_id string
  decision string
  alternatives string[]
  constraints string[]
  consequences string[]
  rollback_path string
  y_statements YStatement[]
  linked_job_ids string[]
  linked_requirement_ids string[]
}

class SDSComponent {
  id string
  responsibility string
  inputs string[]
  outputs string[]
  state_owned string[]
  failure_modes string[]
  linked_requirement_ids string[]
  linked_scenario_ids string[]
}

class GherkinScenario {
  id string
  name string
  given_steps string[]
  when_steps string[]
  then_steps string[]
  linked_requirement_ids string[]
  waiver_reason string?
}

class PRD {
  id string
  version string
  product_seed_id string
  user string
  job_story_ids string[]
  requirements EARSRequirement[]
  acceptance_criteria string[]
  non_goals string[]
}

class SDS {
  id string
  version string
  prd_id string
  c4_level string[]
  mermaid_diagrams string[]
  architecture string[]
  files string[]
  interfaces string[]
  constraints string[]
  components SDSComponent[]
  scenarios GherkinScenario[]
}

class TDDPlan {
  id string
  version string
  sds_id string
  tests string[]
  proof_commands string[]
  manual_checks string[]
  scenarios GherkinScenario[]
}

class AgentTask {
  id string
  version string
  target_path string
  instructions string[]
  required_context string[]
  excluded_context string[]
  proof_required string[]
  eval_spec_ids string[]
  linked_traceability_ids string[]
}

class FabricatorEvalCheck {
  id string
  eval_class FabricatorEvalClass
  type string
  target string
  required bool
  rule string
  evidence_required string[]
  linked_requirement_ids string[]
  linked_scenario_ids string[]
  linked_component_ids string[]
  linked_y_statement_ids string[]
}

class FabricatorEvalSpec {
  id string
  version string
  run_id string
  target_type string
  target_path string
  purpose string
  eval_classes FabricatorEvalClass[]
  checks FabricatorEvalCheck[]
  pass_condition string
  outputs string[]
}

class FabricatorProofRecord {
  id string
  run_id string
  eval_id string
  status string
  evidence string[]
  created_at string
  satisfied_check_ids string[]
  linked_eval_result_id string?
}

class FabricatorReflectionTemplate {
  id string
  run_id string
  prompts string[]
  evidence_required string[]
  forbidden_claims string[]
}

class FabricatorAdaptationDecision {
  run_id string
  decision_id string
  product_result string
  process_result string
  learning_result string
  adaptation_result string
  allowed_adaptations string[]
  blocked_adaptations string[]
  reason string
  required_next_actions string[]
  created_artifacts string[]
}

class FabricatorRegressionCase {
  id string
  source_run_id string
  failure_mode string
  detection string
  future_rule string
  status string
  linked_eval_check string
}

class FabricatorLearningCandidate {
  id string
  source_run_id string
  type string
  claim string
  evidence string[]
  scope string
  confidence string
  promotion_status string
  required_regression_case string?
}

class SemanticChainValidationReport {
  id string
  passed bool
  missing_links string[]
  waived_links string[]
  blocked_claims string[]
}

function GenerateProductSeed(seed_text: string, constraints: string[], non_goals: string[]) -> ProductSeed {
  client "openai/gpt-4o"
  prompt #"
    Convert the bounded product input into a ProductSeed.
    Preserve the target user, constraints, non-goals, desired prototype outcome, and proof requirements.
    Do not add requirements that are not in the input.

    Input:
    {{ seed_text }}

    Constraints:
    {{ constraints }}

    Non-goals:
    {{ non_goals }}
  "#
}

function GenerateJobStory(product_seed: ProductSeed) -> JobStory {
  client "openai/gpt-4o"
  prompt #"
    Generate one Job Story from the ProductSeed.
    Use this exact syntax:
    When [situation], I want to [motivation or action], so I can [desired outcome].
    Preserve source intent and do not invent missing context.

    ProductSeed: {{ product_seed }}
  "#
}

function GeneratePRD(product_seed: ProductSeed, hypothesis: ProductHypothesis, adr: ProductADR) -> PRD {
  client "openai/gpt-4o"
  prompt #"
    Generate a PRD from the ProductSeed, ProductHypothesis, and ADR.
    Requirements must be testable and traceable to the seed.
    Every requirement MUST use EARS format and MUST include a stable requirement ID.
    Use one of these EARS patterns for each requirement: Ubiquitous, EventDriven,
    StateDriven, OptionalFeature, UnwantedBehavior.
    Return each requirement as a structured EARSRequirement object.

    ProductSeed: {{ product_seed }}
    ProductHypothesis: {{ hypothesis }}
    ADR: {{ adr }}
  "#
}

function GenerateEARSRequirements(job_story: JobStory, product_seed: ProductSeed, hypothesis: ProductHypothesis) -> EARSRequirement[] {
  client "openai/gpt-4o"
  prompt #"
    Generate structured EARS requirements from the JobStory, ProductSeed, and ProductHypothesis.
    Preserve stable IDs and upstream links.
    If source intent is ambiguous, mark the ambiguity in rationale instead of filling gaps silently.

    JobStory: {{ job_story }}
    ProductSeed: {{ product_seed }}
    ProductHypothesis: {{ hypothesis }}
  "#
}

function GenerateProductHypothesis(product_seed: ProductSeed, jtbd: string) -> ProductHypothesis {
  client "openai/gpt-4o"
  prompt #"
    Generate a ProductHypothesis as a job-hypothesis business canvas.
    Preserve measurable success and riskiest assumptions.

    The canvas must include:
    - target job
    - target user segment
    - situation context
    - pains
    - desired gains
    - value proposition
    - alternatives today
    - channels
    - adoption triggers
    - adoption barriers

    ProductSeed: {{ product_seed }}
    JTBD: {{ jtbd }}
  "#
}

function GenerateYStatementADR(product_seed: ProductSeed, requirements: EARSRequirement[], hypothesis: ProductHypothesis) -> ProductADR {
  client "openai/gpt-4o"
  prompt #"
    Generate an ADR with at least one Y-Statement.
    Each Y-Statement must name the chosen option, neglected alternatives, benefit, and accepted cost.
    Do not invent unlinked design choices.

    ProductSeed: {{ product_seed }}
    Requirements: {{ requirements }}
    ProductHypothesis: {{ hypothesis }}
  "#
}

function GenerateSDS(prd: PRD, constraints: string[]) -> SDS {
  client "openai/gpt-4o"
  prompt #"
    Generate an SDS that implements the PRD with the supplied constraints.
    Keep architecture bounded and local.

    PRD: {{ prd }}
    Constraints: {{ constraints }}
  "#
}

function GenerateSDSComponents(prd: PRD, adr: ProductADR, constraints: string[]) -> SDSComponent[] {
  client "openai/gpt-4o"
  prompt #"
    Generate SDS component contracts from the PRD and ADR.
    Each component must link to requirement IDs and, when possible, scenario IDs.

    PRD: {{ prd }}
    ADR: {{ adr }}
    Constraints: {{ constraints }}
  "#
}

function GenerateGherkinScenarios(prd: PRD, sds_components: SDSComponent[]) -> GherkinScenario[] {
  client "openai/gpt-4o"
  prompt #"
    Generate Gherkin scenarios for must-have requirements.
    Each scenario must link to one or more requirement IDs.
    Avoid implementation-detail assertions unless they are required by the SDS or ADR.

    PRD: {{ prd }}
    SDSComponents: {{ sds_components }}
  "#
}

function GenerateTDDPlan(sds: SDS, prd: PRD) -> TDDPlan {
  client "openai/gpt-4o"
  prompt #"
    Generate a TDDPlan from the SDS and PRD.
    Include deterministic checks and manual checks where automation is not enough.

    SDS: {{ sds }}
    PRD: {{ prd }}
  "#
}

function GenerateAgentTask(tdd_plan: TDDPlan, context: FabricatorSourceRef[]) -> AgentTask {
  client "openai/gpt-4o"
  prompt #"
    Generate a bounded AgentTask from the TDDPlan.
    Include required context, excluded context, and proof requirements.

    TDDPlan: {{ tdd_plan }}
    Context: {{ context }}
  "#
}

function GenerateAgentTaskFromSemanticChain(job_story: JobStory, prd: PRD, adr: ProductADR, sds: SDS, eval_spec: FabricatorEvalSpec) -> AgentTask {
  client "openai/gpt-4o"
  prompt #"
    Generate a bounded AgentTask from the semantic specification chain.
    Preserve upstream IDs, allowed actions, forbidden actions, and proof requirements.

    JobStory: {{ job_story }}
    PRD: {{ prd }}
    ADR: {{ adr }}
    SDS: {{ sds }}
    EvalSpec: {{ eval_spec }}
  "#
}

function GenerateFabricatorEvalSpec(task: AgentTask, regression_cases: FabricatorRegressionCase[]) -> FabricatorEvalSpec {
  client "openai/gpt-4o"
  prompt #"
    Generate a file-first EvalSpec for the AgentTask.
    Include product outcome, process compliance, learning quality, and adaptation eligibility checks.
    Include relevant regression cases.

    AgentTask: {{ task }}
    RegressionCases: {{ regression_cases }}
  "#
}

function GenerateEvalSpecFromSemanticChain(prd: PRD, adr: ProductADR, sds: SDS, scenarios: GherkinScenario[], task: AgentTask, regression_cases: FabricatorRegressionCase[]) -> FabricatorEvalSpec {
  client "openai/gpt-4o"
  prompt #"
    Generate a file-first EvalSpec from the semantic specification chain.
    Include checks derived from EARS requirements, Y-Statements, SDS component contracts, Gherkin scenarios, AgentTask permissions, and RegressionCases.
    Preserve upstream IDs and do not invent unlinked checks.

    PRD: {{ prd }}
    ADR: {{ adr }}
    SDS: {{ sds }}
    Scenarios: {{ scenarios }}
    AgentTask: {{ task }}
    RegressionCases: {{ regression_cases }}
  "#
}

function GenerateFabricatorProofRecord(eval_result: string, evidence: string[]) -> FabricatorProofRecord {
  client "openai/gpt-4o"
  prompt #"
    Generate a ProofRecord from the EvalResult summary and evidence.
    Only record observed evidence.

    EvalResult: {{ eval_result }}
    Evidence: {{ evidence }}
  "#
}

function GenerateFabricatorAdaptationDecision(eval_result: string, reflection: string, proof_record: FabricatorProofRecord) -> FabricatorAdaptationDecision {
  client "openai/gpt-4o"
  prompt #"
    Generate an AdaptationDecision using the eval-to-adaptation policy.
    Block learning when proof or eval evidence is missing.

    EvalResult: {{ eval_result }}
    Reflection: {{ reflection }}
    ProofRecord: {{ proof_record }}
  "#
}

function ProposeFabricatorRegressionCase(eval_result: string, proof_record: FabricatorProofRecord) -> FabricatorRegressionCase {
  client "openai/gpt-4o"
  prompt #"
    Propose a RegressionCase only when the EvalResult shows a reusable failure.
    Include detection and a future rule.

    EvalResult: {{ eval_result }}
    ProofRecord: {{ proof_record }}
  "#
}

function ValidateSemanticChain(job_story: JobStory, prd: PRD, adr: ProductADR, sds: SDS, eval_spec: FabricatorEvalSpec, task: AgentTask, proof_record: FabricatorProofRecord) -> SemanticChainValidationReport {
  client "openai/gpt-4o"
  prompt #"
    Validate the semantic specification chain.
    Report missing links, waivers, and blocked proof claims.
    Do not invent links that are not present.

    JobStory: {{ job_story }}
    PRD: {{ prd }}
    ADR: {{ adr }}
    SDS: {{ sds }}
    EvalSpec: {{ eval_spec }}
    AgentTask: {{ task }}
    ProofRecord: {{ proof_record }}
  "#
}

function ProposeTraceabilityRepair(report: SemanticChainValidationReport, links: TraceabilityLink[]) -> string[] {
  client "openai/gpt-4o"
  prompt #"
    Propose the smallest valid repair for a broken semantic chain.
    Preserve upstream IDs, source intent, and proof history.
    If source intent is ambiguous, mark ambiguity instead of silently filling the gap.

    ValidationReport: {{ report }}
    ExistingLinks: {{ links }}
  "#
}
