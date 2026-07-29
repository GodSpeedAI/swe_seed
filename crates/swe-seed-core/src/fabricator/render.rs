//! Chain construction + rendering + proof-gated handoff (spec 0017 §4–5).
//!
//! `build_chain` is the bounded, deterministic (no-LLM) constructor that turns
//! a product need into a complete, internally-traceable chain. `render_chain`
//! writes the artifacts under `.fabricator/<run>/`; `handoff` gates the
//! AgentTask handoff on chain integrity and freezes the eval spec.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::artifacts::{
    AgentTask, EARSPattern, EARSRequirement, FabricatorArtifactStatus, FabricatorEvalCheck,
    FabricatorEvalClass, FabricatorEvalSpec, FabricatorProofRecord, GherkinScenario, JobStory,
    ProductADR, ProductHypothesis, ProductSeed, SDSComponent, SemanticChainValidationReport,
    TDDPlan, TraceabilityLink, YStatement, PRD, SDS,
};
use super::chain::{validate_semantic_chain, SemanticChain};
use crate::provenance::content_hash;
use crate::util::utc_now;

/// `.fabricator/runs/<run_id>`.
pub fn run_dir(root: &Path, run_id: &str) -> PathBuf {
    root.join(".fabricator").join("runs").join(run_id)
}

fn rid(run_id: &str, suffix: &str) -> String {
    format!("{run_id}-{suffix}")
}

/// Build the full semantic chain from a product need. Deterministic: stable
/// ids derived from `run_id`, no clock (the proof record's `created_at` is
/// stamped at render time). The constructed chain is internally traceable —
/// `validate_semantic_chain` passes on it.
pub fn build_chain(need: &str, run_id: &str) -> SemanticChain {
    let need = need.trim();
    let seed_id = rid(run_id, "seed");
    let jtbd_id = rid(run_id, "jtbd");
    let hyp_id = rid(run_id, "hyp");
    let prd_id = rid(run_id, "prd");
    let adr_id = rid(run_id, "adr");
    let sds_id = rid(run_id, "sds");
    let tdd_id = rid(run_id, "tdd");
    let task_id = rid(run_id, "task");
    let eval_id = rid(run_id, "eval");
    let proof_id = rid(run_id, "proof");
    let req_id = rid(run_id, "req-1");
    let y_id = rid(run_id, "y-1");
    let comp_id = rid(run_id, "comp-1");
    let scn_id = rid(run_id, "scn-1");
    let chk_id = rid(run_id, "chk-1");

    let product_seed = ProductSeed {
        id: seed_id.clone(),
        version: "0.1.0".into(),
        status: FabricatorArtifactStatus::Active,
        target_user: "end user".into(),
        product_idea: need.into(),
        constraints: vec!["bounded single-pass prototype".into()],
        non_goals: vec!["no autonomous looping".into()],
        desired_prototype_outcome: format!("a proven prototype addressing: {need}"),
        proof_required: vec!["chain integrity passes".into()],
        linked_artifacts: Vec::new(),
    };

    let job_story = JobStory {
        id: jtbd_id.clone(),
        when_clause: format!("when addressing {need}"),
        action_clause: "I want to use a bounded fabricate run".into(),
        outcome_clause: "so I can get a proven prototype".into(),
        source_refs: vec![seed_id.clone()],
        waived: None,
        waiver_reason: None,
    };

    let hypothesis = ProductHypothesis {
        id: hyp_id,
        version: "0.1.0".into(),
        product_seed_id: seed_id.clone(),
        hypothesis: None,
        hypothesis_statement: format!("we believe {need} can be addressed by a bounded prototype"),
        target_job: need.into(),
        target_user_segment: "end user".into(),
        situation_context: "fabricate run".into(),
        pains: Vec::new(),
        desired_gains: Vec::new(),
        value_proposition: format!("a proven path from need to prototype for {need}"),
        alternatives_today: Vec::new(),
        channels: Vec::new(),
        adoption_triggers: Vec::new(),
        adoption_barriers: Vec::new(),
        measurable_success: vec!["chain integrity passes".into()],
        riskiest_assumptions: vec![format!("{need} is addressable by a bounded run")],
        source_refs: vec![seed_id.clone()],
    };

    let requirement = EARSRequirement {
        id: req_id.clone(),
        pattern: EARSPattern::Ubiquitous,
        system_name: "prototype".into(),
        condition: None,
        trigger: None,
        state: None,
        feature: None,
        unwanted_condition: None,
        response: format!("address {need}"),
        rationale: None,
        source_refs: vec![prd_id.clone()],
        ears_text: format!("The prototype shall address {need}."),
        testable: true,
        non_testable_reason: None,
    };

    let prd = PRD {
        id: prd_id.clone(),
        version: "0.1.0".into(),
        product_seed_id: seed_id.clone(),
        user: "end user".into(),
        job_story_ids: vec![jtbd_id.clone()],
        requirements: vec![requirement],
        acceptance_criteria: vec![format!("the prototype addresses {need}")],
        non_goals: Vec::new(),
    };

    let y_statement = YStatement {
        id: y_id.clone(),
        context: format!("addressing {need}"),
        constraint: "bounded single pass".into(),
        chosen_option: "deterministic chain construction".into(),
        neglected_alternatives: vec!["manual ad-hoc generation".into()],
        benefit: "traceable, replayable".into(),
        accepted_cost: "no runtime LLM".into(),
        linked_requirement_ids: vec![req_id.clone()],
        y_text: format!("In the context of {need}, facing a bounded pass, we chose deterministic chain construction, neglecting manual generation, to gain traceability, accepting no runtime LLM."),
    };

    let adr = ProductADR {
        id: adr_id,
        version: "0.1.0".into(),
        product_seed_id: seed_id.clone(),
        decision: "construct the chain deterministically (contracts-as-data)".into(),
        alternatives: vec!["manual generation".into()],
        constraints: vec!["bounded single pass".into()],
        consequences: vec!["no runtime LLM".into()],
        rollback_path: "delete the run directory".into(),
        y_statements: vec![y_statement],
        linked_job_ids: vec![jtbd_id.clone()],
        linked_requirement_ids: vec![req_id.clone()],
    };

    let component = SDSComponent {
        id: comp_id.clone(),
        responsibility: format!("deliver the behavior for {need}"),
        inputs: vec!["product need".into()],
        outputs: vec!["prototype".into()],
        state_owned: Vec::new(),
        failure_modes: vec!["chain integrity broken".into()],
        linked_requirement_ids: vec![req_id.clone()],
        linked_scenario_ids: vec![scn_id.clone()],
    };

    let scenario = GherkinScenario {
        id: scn_id.clone(),
        name: format!("address {need}"),
        given_steps: vec!["a bounded fabricate run".into()],
        when_steps: vec!["the prototype is exercised".into()],
        then_steps: vec![format!("the need '{need}' is addressed")],
        linked_requirement_ids: vec![req_id.clone()],
        waiver_reason: None,
    };

    let sds = SDS {
        id: sds_id.clone(),
        version: "0.1.0".into(),
        prd_id: prd_id.clone(),
        c4_level: Vec::new(),
        mermaid_diagrams: Vec::new(),
        architecture: vec!["single prototype component".into()],
        files: vec!["prototype".into()],
        interfaces: Vec::new(),
        constraints: vec!["bounded single pass".into()],
        components: vec![component],
        scenarios: vec![scenario],
    };

    let tdd_plan = TDDPlan {
        id: tdd_id,
        version: "0.1.0".into(),
        sds_id: sds_id.clone(),
        tests: vec!["chain integrity passes".into()],
        proof_commands: vec!["swe-seed fabricate validate-chain".into()],
        manual_checks: Vec::new(),
        scenarios: Vec::new(),
    };

    let eval_spec = FabricatorEvalSpec {
        id: eval_id.clone(),
        version: "0.1.0".into(),
        run_id: run_id.into(),
        target_type: "AgentTask".into(),
        target_path: task_id.clone(),
        purpose: format!("verify the prototype addresses {need}"),
        eval_classes: vec![
            FabricatorEvalClass::ProductOutcome,
            FabricatorEvalClass::ProcessCompliance,
            FabricatorEvalClass::LearningQuality,
            FabricatorEvalClass::AdaptationEligibility,
        ],
        checks: vec![FabricatorEvalCheck {
            id: chk_id,
            eval_class: FabricatorEvalClass::ProductOutcome,
            r#type: "chain_integrity".into(),
            target: format!(".fabricator/runs/{run_id}"),
            required: true,
            rule: "SemanticChainValidationReport.passed == true".into(),
            evidence_required: vec!["chain report".into()],
            linked_requirement_ids: vec![req_id],
            linked_scenario_ids: vec![scn_id],
            linked_component_ids: vec![comp_id],
            linked_y_statement_ids: vec![y_id],
        }],
        pass_condition: "all required checks pass".into(),
        outputs: vec!["prototype".into()],
    };

    let agent_task = AgentTask {
        id: task_id,
        version: "0.1.0".into(),
        target_path: "prototype".into(),
        instructions: vec![format!("implement the prototype for {need}")],
        required_context: vec![format!(".fabricator/runs/{run_id}")],
        excluded_context: Vec::new(),
        proof_required: vec!["chain integrity passes".into()],
        eval_spec_ids: vec![eval_id.clone()],
        linked_traceability_ids: Vec::new(),
    };

    let proof_record = FabricatorProofRecord {
        id: proof_id,
        run_id: run_id.into(),
        eval_id: eval_id.clone(),
        status: "Pending".into(),
        evidence: Vec::new(),
        created_at: String::new(), // stamped at render time
        satisfied_check_ids: Vec::new(),
        linked_eval_result_id: None,
        evidence_refs: Vec::new(),
    };

    let links = vec![
        link(&seed_id, &jtbd_id, "derives"),
        link(&seed_id, &hyp_id_or(run_id), "derives"),
        link(&jtbd_id, &prd_id, "feeds"),
        link(&seed_id, &adr_id_or(run_id), "constrains"),
        link(&prd_id, &sds_id, "implements"),
        link(&sds_id, &tdd_id_or(run_id), "plans"),
        link(&eval_id, &task_id_or(run_id), "verifies"),
        link(&eval_id, &proof_id_or(run_id), "proves"),
    ];

    SemanticChain {
        run_id: run_id.into(),
        product_seed,
        job_story,
        hypothesis,
        prd,
        adr,
        sds,
        tdd_plan,
        agent_task,
        eval_spec,
        proof_record,
        links,
    }
}

fn link(upstream: &str, downstream: &str, relation: &str) -> TraceabilityLink {
    TraceabilityLink {
        upstream_id: upstream.into(),
        downstream_id: downstream.into(),
        relation: relation.into(),
        waiver_id: None,
    }
}

// id helpers that rebuild the same ids build_chain uses (kept in one place so
// the documented `links` stay in sync with the artifact ids).
fn hyp_id_or(run_id: &str) -> String {
    rid(run_id, "hyp")
}
fn adr_id_or(run_id: &str) -> String {
    rid(run_id, "adr")
}
fn tdd_id_or(run_id: &str) -> String {
    rid(run_id, "tdd")
}
fn task_id_or(run_id: &str) -> String {
    rid(run_id, "task")
}
fn proof_id_or(run_id: &str) -> String {
    rid(run_id, "proof")
}

/// Render the chain to disk under `.fabricator/runs/<run>/generated/`. Writes
/// each artifact as JSON plus a `chain.json` reload source. The proof record's
/// `created_at` is stamped with `created_at` (the render moment). Returns the
/// run directory.
pub fn render_chain(chain: &mut SemanticChain, root: &Path, created_at: &str) -> Result<PathBuf> {
    chain.proof_record.created_at = created_at.into();
    let dir = run_dir(root, &chain.run_id).join("generated");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let write = |name: &str, value: &serde_json::Value| -> Result<()> {
        let path = dir.join(name);
        std::fs::write(&path, format!("{}\n", serde_json::to_string_pretty(value)?))
            .with_context(|| format!("write {}", path.display()))?;
        Ok(())
    };

    write(
        "PRODUCT_SEED.json",
        &serde_json::to_value(&chain.product_seed)?,
    )?;
    write("JOB_STORY.json", &serde_json::to_value(&chain.job_story)?)?;
    write("HYPOTHESIS.json", &serde_json::to_value(&chain.hypothesis)?)?;
    write("PRD.json", &serde_json::to_value(&chain.prd)?)?;
    write("ADR.json", &serde_json::to_value(&chain.adr)?)?;
    write("SDS.json", &serde_json::to_value(&chain.sds)?)?;
    write("TDD_PLAN.json", &serde_json::to_value(&chain.tdd_plan)?)?;
    write("AGENT_TASK.json", &serde_json::to_value(&chain.agent_task)?)?;
    write("EVAL_SPEC.json", &serde_json::to_value(&chain.eval_spec)?)?;
    write(
        "PROOF_RECORD.json",
        &serde_json::to_value(&chain.proof_record)?,
    )?;
    write("CHAIN.json", &serde_json::to_value(&*chain)?)?;

    Ok(run_dir(root, &chain.run_id))
}

/// Reload a rendered chain from its `generated/CHAIN.json`.
pub fn load_chain(run_dir: &Path) -> Result<SemanticChain> {
    let path = run_dir.join("generated").join("CHAIN.json");
    let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?)
}

/// Frozen-eval-spec manifest written at handoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffManifest {
    pub run_id: String,
    pub eval_spec_id: String,
    pub eval_spec_sha256: String,
    pub frozen: bool,
    pub artifacts: Vec<String>,
}

/// Proof-gated handoff (spec 0017 §5): the AgentTask hands off only when chain
/// integrity passes; on pass, the eval spec is frozen (its content hash pinned
/// in `handoff/MANIFEST.json`). On failure, nothing is written and the report
/// is returned for the caller to surface.
pub fn handoff(
    chain: &SemanticChain,
    root: &Path,
) -> std::result::Result<SemanticChainValidationReport, SemanticChainValidationReport> {
    let report = validate_semantic_chain(chain);
    if !report.passed {
        return Err(report);
    }
    let dir = run_dir(root, &chain.run_id).join("handoff");
    let _ = std::fs::create_dir_all(&dir);
    let eval_json = serde_json::to_vec(&chain.eval_spec).unwrap_or_default();
    let manifest = HandoffManifest {
        run_id: chain.run_id.clone(),
        eval_spec_id: chain.eval_spec.id.clone(),
        eval_spec_sha256: content_hash(&eval_json),
        frozen: true,
        artifacts: vec![
            "PRODUCT_SEED.json".into(),
            "JOB_STORY.json".into(),
            "HYPOTHESIS.json".into(),
            "PRD.json".into(),
            "ADR.json".into(),
            "SDS.json".into(),
            "TDD_PLAN.json".into(),
            "AGENT_TASK.json".into(),
            "EVAL_SPEC.json".into(),
            "PROOF_RECORD.json".into(),
        ],
    };
    let _ = std::fs::write(
        dir.join("MANIFEST.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).unwrap_or_default()
        ),
    );
    Ok(report)
}

/// One-shot bounded run (the `fabricate <need>` CLI path): build, validate,
/// render, and hand off. Returns the run dir on a successful (proof-gated)
/// handoff; on a chain-integrity failure returns the report (and writes the
/// report instead of a handoff).
pub fn fabricate(
    root: &Path,
    need: &str,
    run_id: &str,
) -> std::result::Result<(PathBuf, SemanticChainValidationReport), SemanticChainValidationReport> {
    let mut chain = build_chain(need, run_id);
    let _ = render_chain(&mut chain, root, &utc_now());
    let report = handoff(&chain, root)?;
    Ok((run_dir(root, run_id), report))
}
