//! Fabricator layer (spec 0017): the bounded product→prototype pipeline.
//! Semantic chain `ProductSeed → JobStory → ProductHypothesis → PRD →
//! ProductADR → SDS → TDDPlan → AgentTask → FabricatorEvalSpec →
//! FabricatorProofRecord`, with EARS/Gherkin shape validation, chain-integrity
//! gating, and proof-gated (frozen-eval-spec) handoff. Contracts-as-data, no
//! LLM runtime (0019).

pub mod artifacts;
pub mod chain;
pub mod render;
pub mod validate;

pub use artifacts::{
    AgentTask, EARSPattern, EARSRequirement, FabricatorArtifactStatus, FabricatorEvalCheck,
    FabricatorEvalClass, FabricatorEvalSpec, FabricatorProofRecord, FabricatorSourceRef,
    GherkinScenario, JobStory, ProductADR, ProductHypothesis, ProductSeed, SDSComponent,
    SemanticChainValidationReport, TDDPlan, TraceabilityLink, YStatement, PRD, SDS,
};
pub use chain::{validate_semantic_chain, SemanticChain};
pub use render::{
    build_chain, fabricate, handoff, load_chain, render_chain, run_dir, HandoffManifest,
};
pub use validate::{
    validate_ears_requirement, validate_ears_requirements, validate_gherkin_scenario,
    validate_gherkin_scenarios,
};
