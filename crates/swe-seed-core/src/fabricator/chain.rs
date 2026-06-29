//! The semantic chain + integrity validation (spec 0017 §1–2). The chain is
//! the spine `ProductSeed → JobStory → ProductHypothesis → PRD → ProductADR →
//! SDS → TDDPlan → AgentTask → FabricatorEvalSpec → FabricatorProofRecord`;
//! `validate_semantic_chain` walks every edge and reports broken traceability.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::artifacts::{
    AgentTask, FabricatorEvalSpec, FabricatorProofRecord, JobStory, PRD, ProductADR, ProductHypothesis,
    ProductSeed, SDS, SemanticChainValidationReport, TDDPlan, TraceabilityLink,
};

/// The assembled semantic chain. Every artifact is required; integrity is
/// proven by cross-referencing each artifact's upstream id fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChain {
    pub run_id: String,
    pub product_seed: ProductSeed,
    pub job_story: JobStory,
    pub hypothesis: ProductHypothesis,
    pub prd: PRD,
    pub adr: ProductADR,
    pub sds: SDS,
    pub tdd_plan: TDDPlan,
    pub agent_task: AgentTask,
    pub eval_spec: FabricatorEvalSpec,
    pub proof_record: FabricatorProofRecord,
    /// Documented chain edges (may carry a `waiver_id` for an acknowledged gap).
    pub links: Vec<TraceabilityLink>,
}

fn nonempty(s: &str) -> bool {
    !s.trim().is_empty()
}

/// Validate the chain's traceability. Returns a report whose `passed` is true
/// iff no edge is broken and no proof claim is unresolvable. Waived edges (a
/// `TraceabilityLink` with a `waiver_id`) are acknowledged in `waived_links`,
/// not counted as failures.
pub fn validate_semantic_chain(chain: &SemanticChain) -> SemanticChainValidationReport {
    let mut missing: Vec<String> = Vec::new();
    let mut blocked: Vec<String> = Vec::new();

    let seed_id = &chain.product_seed.id;
    let jtbd_id = &chain.job_story.id;
    let prd_id = &chain.prd.id;
    let sds_id = &chain.sds.id;
    let eval_id = &chain.eval_spec.id;

    // Spine edges (upstream -> downstream via the downstream's id field).
    if chain.hypothesis.product_seed_id != *seed_id {
        missing.push(format!(
            "ProductSeed -> ProductHypothesis: hypothesis.product_seed_id='{}' != seed.id='{}'",
            chain.hypothesis.product_seed_id, seed_id
        ));
    }
    if chain.prd.product_seed_id != *seed_id {
        missing.push(format!(
            "ProductSeed -> PRD: prd.product_seed_id='{}' != seed.id='{}'",
            chain.prd.product_seed_id, seed_id
        ));
    }
    if !chain.prd.job_story_ids.iter().any(|id| id == jtbd_id) {
        missing.push(format!(
            "JobStory -> PRD: prd.job_story_ids={:?} does not contain job_story.id='{}'",
            chain.prd.job_story_ids, jtbd_id
        ));
    }
    if chain.adr.product_seed_id != *seed_id {
        missing.push(format!(
            "ProductSeed -> ProductADR: adr.product_seed_id='{}' != seed.id='{}'",
            chain.adr.product_seed_id, seed_id
        ));
    }
    if !chain.adr.linked_job_ids.iter().any(|id| id == jtbd_id) {
        missing.push(format!(
            "JobStory -> ProductADR: adr.linked_job_ids={:?} does not contain job_story.id='{}'",
            chain.adr.linked_job_ids, jtbd_id
        ));
    }
    if chain.sds.prd_id != *prd_id {
        missing.push(format!(
            "PRD -> SDS: sds.prd_id='{}' != prd.id='{}'",
            chain.sds.prd_id, prd_id
        ));
    }
    if chain.tdd_plan.sds_id != *sds_id {
        missing.push(format!(
            "SDS -> TDDPlan: tdd.sds_id='{}' != sds.id='{}'",
            chain.tdd_plan.sds_id, sds_id
        ));
    }
    if !chain
        .agent_task
        .eval_spec_ids
        .iter()
        .any(|id| id == eval_id)
    {
        missing.push(format!(
            "FabricatorEvalSpec -> AgentTask: task.eval_spec_ids={:?} does not contain eval_spec.id='{}'",
            chain.agent_task.eval_spec_ids, eval_id
        ));
    }
    if chain.proof_record.eval_id != *eval_id {
        missing.push(format!(
            "FabricatorEvalSpec -> FabricatorProofRecord: proof.eval_id='{}' != eval_spec.id='{}'",
            chain.proof_record.eval_id, eval_id
        ));
    }

    // Requirement traceability: every SDS component and scenario must link to a
    // requirement id that actually exists in the PRD.
    let req_ids: HashSet<&str> = chain.prd.requirements.iter().map(|r| r.id.as_str()).collect();
    for comp in &chain.sds.components {
        if comp.linked_requirement_ids.is_empty() {
            missing.push(format!(
                "PRD -> SDSComponent: component '{}' links no requirement",
                comp.id
            ));
        } else if !comp.linked_requirement_ids.iter().any(|id| req_ids.contains(id.as_str())) {
            missing.push(format!(
                "PRD -> SDSComponent: component '{}' linked_requirement_ids={:?} not in prd requirements",
                comp.id, comp.linked_requirement_ids
            ));
        }
    }
    for scn in &chain.sds.scenarios {
        if !scn.linked_requirement_ids.iter().any(|id| req_ids.contains(id.as_str())) {
            missing.push(format!(
                "PRD -> GherkinScenario: scenario '{}' linked_requirement_ids={:?} not in prd requirements",
                scn.id, scn.linked_requirement_ids
            ));
        }
    }

    // Proof claims: every eval check that links a requirement/scenario/component
    // /y-statement must reference one that exists, else the proof is blocked.
    let scenario_ids: HashSet<&str> = chain.sds.scenarios.iter().map(|s| s.id.as_str()).collect();
    let component_ids: HashSet<&str> = chain.sds.components.iter().map(|c| c.id.as_str()).collect();
    let y_ids: HashSet<&str> = chain.adr.y_statements.iter().map(|y| y.id.as_str()).collect();
    for chk in &chain.eval_spec.checks {
        for id in &chk.linked_requirement_ids {
            if nonempty(id) && !req_ids.contains(id.as_str()) {
                blocked.push(format!(
                    "eval check '{}' links unknown requirement '{}'",
                    chk.id, id
                ));
            }
        }
        for id in &chk.linked_scenario_ids {
            if nonempty(id) && !scenario_ids.contains(id.as_str()) {
                blocked.push(format!(
                    "eval check '{}' links unknown scenario '{}'",
                    chk.id, id
                ));
            }
        }
        for id in &chk.linked_component_ids {
            if nonempty(id) && !component_ids.contains(id.as_str()) {
                blocked.push(format!(
                    "eval check '{}' links unknown component '{}'",
                    chk.id, id
                ));
            }
        }
        for id in &chk.linked_y_statement_ids {
            if nonempty(id) && !y_ids.contains(id.as_str()) {
                blocked.push(format!(
                    "eval check '{}' links unknown y-statement '{}'",
                    chk.id, id
                ));
            }
        }
    }

    let waived: Vec<String> = chain
        .links
        .iter()
        .filter(|l| l.waiver_id.is_some())
        .map(|l| format!("{} -> {} ({}) waived: {}", l.upstream_id, l.downstream_id, l.relation, l.waiver_id.as_deref().unwrap_or("")))
        .collect();

    let passed = missing.is_empty() && blocked.is_empty();
    SemanticChainValidationReport {
        id: format!("chain-report-{}", chain.run_id),
        passed,
        missing_links: missing,
        waived_links: waived,
        blocked_claims: blocked,
    }
}
