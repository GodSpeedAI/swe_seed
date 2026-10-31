# fabricator

## Overview

Fabricator layer (spec 0017): the bounded product-to-prototype pipeline. The
semantic chain spine is `ProductSeed -> JobStory -> ProductHypothesis -> PRD ->
ProductADR -> SDS -> TDDPlan -> AgentTask -> FabricatorEvalSpec ->
FabricatorProofRecord`, with EARS/Gherkin shape validation, chain-integrity
gating, and proof-gated (frozen-eval-spec) handoff. Contracts-as-data with no
LLM runtime (spec 0019); every artifact type is baml-faithful to
`fabricator.baml` and asserts `BamlParity`.

## Interface

- `build_chain(need: &str, run_id: &str) -> SemanticChain`: deterministic
  no-LLM chain constructor with stable ids derived from `run_id`; the
  constructed chain passes `validate_semantic_chain` [render.rs:build_chain]
- `render_chain(chain: &mut SemanticChain, root: &Path, created_at: &str) -> Result<PathBuf>`:
  writes each artifact as JSON plus `CHAIN.json` under
  `.fabricator/runs/<run>/generated/` [render.rs:render_chain]
- `load_chain(run_dir: &Path) -> Result<SemanticChain>`: reloads a rendered
  chain from `generated/CHAIN.json` [render.rs:load_chain]
- `handoff(chain: &SemanticChain, root: &Path) -> Result<SemanticChainValidationReport, SemanticChainValidationReport>`:
  proof-gated handoff; writes `handoff/MANIFEST.json` with the frozen eval-spec
  hash on pass, writes nothing on failure [render.rs:handoff]
- `fabricate(root, need, run_id) -> Result<(PathBuf, SemanticChainValidationReport), SemanticChainValidationReport>`:
  one-shot bounded run (the `fabricate <need>` CLI path): build, validate,
  render, hand off [render.rs:fabricate]
- `run_dir(root, run_id) -> PathBuf` = `.fabricator/runs/<run_id>`
  [render.rs:run_dir]
- `HandoffManifest { run_id, eval_spec_id, eval_spec_sha256, frozen, artifacts }`
  [render.rs:HandoffManifest]
- `pub struct SemanticChain { run_id, product_seed, job_story, hypothesis, prd,
adr, sds, tdd_plan, agent_task, eval_spec, proof_record, links }`
  [chain.rs:SemanticChain]
- `validate_semantic_chain(chain: &SemanticChain) -> SemanticChainValidationReport`:
  walks every spine edge, requirement traceability, and proof-claim linkage
  [chain.rs:validate_semantic_chain]
- `validate_ears_requirement(req: &EARSRequirement) -> Result<(), String>` and
  `validate_ears_requirements(reqs) -> Result<(), Vec<String>>`: EARS shape
  validation per declared pattern [validate.rs:validate_ears_requirement],
  [validate.rs:validate_ears_requirements]
- `validate_gherkin_scenario(scn: &GherkinScenario) -> Result<(), String>` and
  `validate_gherkin_scenarios(scns) -> Result<(), Vec<String>>`: non-empty name
  plus at least one Given/When/Then step [validate.rs:validate_gherkin_scenario],
  [validate.rs:validate_gherkin_scenarios]
- Artifact types (all `BamlParity`, PascalCase serde): `ProductSeed`,
  `JobStory`, `ProductHypothesis`, `PRD`, `ProductADR`, `YStatement`, `SDS`,
  `SDSComponent`, `GherkinScenario`, `TDDPlan`, `AgentTask`,
  `FabricatorEvalSpec`, `FabricatorEvalCheck` (field `type` via
  `#[serde(rename = "type")]`), `FabricatorProofRecord`, `FabricatorSourceRef`,
  `TraceabilityLink`, `SemanticChainValidationReport`
  [artifacts.rs:ProductSeed], [artifacts.rs:JobStory],
  [artifacts.rs:ProductHypothesis], [artifacts.rs:PRD], [artifacts.rs:ProductADR],
  [artifacts.rs:YStatement], [artifacts.rs:SDS], [artifacts.rs:SDSComponent],
  [artifacts.rs:GherkinScenario], [artifacts.rs:TDDPlan], [artifacts.rs:AgentTask],
  [artifacts.rs:FabricatorEvalSpec], [artifacts.rs:FabricatorEvalCheck],
  [artifacts.rs:FabricatorProofRecord], [artifacts.rs:FabricatorSourceRef],
  [artifacts.rs:TraceabilityLink], [artifacts.rs:SemanticChainValidationReport]
- Enums: `FabricatorArtifactStatus { Draft, Proposed, Approved, Active,
Rejected, Deprecated }`, `FabricatorEvalClass { ProductOutcome,
ProcessCompliance, LearningQuality, AdaptationEligibility }`,
  `EARSPattern { Ubiquitous, EventDriven, StateDriven, OptionalFeature,
UnwantedBehavior }` [artifacts.rs:FabricatorArtifactStatus],
  [artifacts.rs:FabricatorEvalClass], [artifacts.rs:EARSPattern]

## Invariants

- Chain integrity is proven by cross-referencing each artifact's upstream id
  fields (for example `hypothesis.product_seed_id == seed.id`,
  `sds.prd_id == prd.id`, `task.eval_spec_ids` contains `eval_spec.id`), not by
  the presence of `links` entries [chain.rs:validate_semantic_chain]
- Every SDS component must link at least one requirement id that exists in the
  PRD; eval checks that link a requirement/scenario/component/y-statement must
  reference an existing id or the proof is blocked
  [chain.rs:validate_semantic_chain]
- Waived edges: a `TraceabilityLink` with a `waiver_id` is acknowledged in
  `waived_links`, not counted as a failure [chain.rs:validate_semantic_chain]
- Handoff is proof-gated: `handoff` returns `Err(report)` and writes nothing
  when chain integrity fails; on pass the eval spec is frozen via
  `eval_spec_sha256` with `frozen: true` [render.rs:handoff]
- `build_chain` is deterministic: stable ids from `run_id`, no clock; the proof
  record's `created_at` is empty until stamped at render time
  [render.rs:build_chain], [render.rs:render_chain]
- EARS shape: every pattern needs non-empty `response`, `ears_text`, and
  `system_name`; each non-Ubiquitous pattern needs its named clause; a
  requirement marked not `testable` must give a `non_testable_reason`
  [validate.rs:validate_ears_requirement]
- `FabricatorProofRecord.evidence_refs` is skipped when empty
  (`skip_serializing_if = "Vec::is_empty"`) so empty-by-default records
  serialize identically to before; chain validation does not inspect this field
  [artifacts.rs:FabricatorProofRecord]

## Side Effects

- `render_chain` creates `.fabricator/runs/<run>/generated/` and writes
  `PRODUCT_SEED.json`, `JOB_STORY.json`, `HYPOTHESIS.json`, `PRD.json`,
  `ADR.json`, `SDS.json`, `TDD_PLAN.json`, `AGENT_TASK.json`, `EVAL_SPEC.json`,
  `PROOF_RECORD.json`, `CHAIN.json` [render.rs:render_chain]
- `handoff` creates `.fabricator/runs/<run>/handoff/` and writes
  `MANIFEST.json` [render.rs:handoff]
- `load_chain` reads `generated/CHAIN.json` from disk [render.rs:load_chain]

## Verification

- Run: `cargo test -p swe-seed-core`
