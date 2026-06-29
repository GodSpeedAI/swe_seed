use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use swe_seed_core::eval::SourceRef;
use swe_seed_core::learning::{
    can_promote_candidate, AdaptationDecision, LearningCandidate, LearningRecord, PromotionOutcome,
    SkillProposal,
};

#[derive(Subcommand)]
pub enum LearnAction {
    /// Promote a LearningRecord into a SkillProposal or RegressionCase
    Promote {
        record: String,
        /// Path to a candidate SkillProposal JSON (required for SkillProposal dispositions)
        #[arg(long)]
        proposal: Option<String>,
        /// Path to a candidate RegressionCase JSON (required for RegressionCase dispositions)
        #[arg(long)]
        regression: Option<String>,
    },
}

fn learning_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".agent-harness").join("learning")
}

pub fn run_reflect(root: &std::path::Path, trace: &str) -> Result<ExitCode> {
    use swe_seed_core::learning::{default_template, reflect};
    use swe_seed_core::trace::lifecycle::resolve_trace_path;
    use swe_seed_core::trace::TraceRecord;

    let trace_path = resolve_trace_path(root, trace);
    let record = TraceRecord::load(&trace_path)?;
    let learned = reflect(
        &default_template(),
        &record,
        &trace_path.display().to_string(),
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    let out_dir = learning_dir(root).join("records");
    std::fs::create_dir_all(&out_dir)?;
    let out = out_dir.join(format!("{}.json", learned.id));
    std::fs::write(
        &out,
        format!("{}\n", serde_json::to_string_pretty(&learned)?),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "learning_record": learned.id,
            "disposition": format!("{:?}", learned.disposition),
            "path": out.strip_prefix(root).unwrap_or(&out).display().to_string(),
        }))?
    );
    Ok(ExitCode::SUCCESS)
}

pub fn run_learn(action: LearnAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::learning::{
        promotion_gate, validate_learning_record, validate_proposal, validate_regression,
        LearningDisposition, RegressionCase,
    };
    let LearnAction::Promote {
        record,
        proposal,
        regression,
    } = action;
    let record_path = root.join(&record);
    let loaded: LearningRecord = serde_json::from_slice(&std::fs::read(&record_path)?)
        .map_err(|e| anyhow::anyhow!("parse {}: {e}", record_path.display()))?;

    // Validate the record itself before any candidate is considered: a corrupt
    // or evidence-less record must not unlock a promotion write.
    validate_learning_record(&loaded)
        .map_err(|e| anyhow::anyhow!("learning record {} failed validation: {e}", loaded.id))?;

    match loaded.disposition {
        LearningDisposition::ApprovedLesson => {
            // A published lesson needs no candidate artifact beyond the record.
            println!("published lesson '{}' ({})", loaded.id, loaded.summary);
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::NoChange | LearningDisposition::RejectedLesson => {
            println!(
                "nothing to promote: disposition is {:?}",
                loaded.disposition
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::SkillProposal => {
            let Some(p) = proposal else {
                eprintln!("SkillProposal disposition requires --proposal <path>");
                return Ok(ExitCode::from(1));
            };
            let decision = load_adaptation_decision(root, &loaded.id)?;
            if let Err(e) = promotion_gate(&decision) {
                eprintln!("{e}");
                return Ok(ExitCode::from(1));
            }
            let proposal_path = root.join(&p);
            let cand: SkillProposal = serde_json::from_slice(&std::fs::read(&proposal_path)?)
                .map_err(|e| anyhow::anyhow!("parse {}: {e}", proposal_path.display()))?;
            validate_proposal(&cand).map_err(|e| anyhow::anyhow!(e))?;
            validate_skill_candidate_ownership(&loaded, &cand)?;
            let out_dir = learning_dir(root).join("proposals");
            std::fs::create_dir_all(&out_dir)?;
            let out = out_dir.join(format!("{}.json", cand.id));
            std::fs::write(&out, format!("{}\n", serde_json::to_string_pretty(&cand)?))?;
            println!(
                "promoted SkillProposal '{}' -> {}",
                cand.id,
                out.strip_prefix(root).unwrap_or(&out).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::RegressionCase => {
            let Some(r) = regression else {
                eprintln!("RegressionCase disposition requires --regression <path>");
                return Ok(ExitCode::from(1));
            };
            let decision = load_adaptation_decision(root, &loaded.id)?;
            if !decision.regression_registration_allowed() {
                eprintln!(
                    "regression registration blocked by AdaptationDecision '{}': {}",
                    decision.decision_id, decision.reason
                );
                return Ok(ExitCode::from(1));
            }
            let reg_path = root.join(&r);
            let cand: RegressionCase = serde_json::from_slice(&std::fs::read(&reg_path)?)
                .map_err(|e| anyhow::anyhow!("parse {}: {e}", reg_path.display()))?;
            validate_regression(&cand).map_err(|e| anyhow::anyhow!(e))?;
            if cand.source_run_id != loaded.id {
                eprintln!(
                    "regression case '{}' source_run_id '{}' does not match LearningRecord '{}'",
                    cand.id, cand.source_run_id, loaded.id
                );
                return Ok(ExitCode::from(1));
            }
            let out_dir = learning_dir(root).join("regressions");
            std::fs::create_dir_all(&out_dir)?;
            let out = out_dir.join(format!("{}.json", cand.id));
            std::fs::write(&out, format!("{}\n", serde_json::to_string_pretty(&cand)?))?;
            println!(
                "promoted RegressionCase '{}' (linked_eval_check={}) -> {}",
                cand.id,
                cand.linked_eval_check,
                out.strip_prefix(root).unwrap_or(&out).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::HarnessADR => {
            // HarnessADR promotion is a doctrine write, out of scope for v0.1's
            // learning CLI (no doctrine store yet). Surface honestly.
            eprintln!("HarnessADR promotion is not implemented in v0.1; record only");
            Ok(ExitCode::from(1))
        }
    }
}

fn validate_skill_candidate_ownership(
    record: &LearningRecord,
    proposal: &SkillProposal,
) -> Result<()> {
    if !has_shared_evidence(&record.evidence, &proposal.evidence) {
        return Err(anyhow::anyhow!(
            "SkillProposal '{}' is not backed by LearningRecord '{}' evidence",
            proposal.id,
            record.id
        ));
    }
    let candidate = LearningCandidate {
        id: proposal.id.clone(),
        source_run_id: record.id.clone(),
        candidate_type: "SkillProposal".into(),
        claim: proposal.observed_problem.clone(),
        evidence: proposal.evidence.clone(),
        scope: proposal.proposed_skill_id.clone(),
        confidence: "reviewed".into(),
        promotion_status: "candidate".into(),
        required_regression_case: None,
    };
    match can_promote_candidate(&candidate) {
        PromotionOutcome::Allow { .. } => Ok(()),
        PromotionOutcome::Block { reason } => Err(anyhow::anyhow!(
            "SkillProposal '{}' failed LearningCandidate promotion gate: {reason}",
            proposal.id
        )),
    }
}

fn has_shared_evidence(record: &[SourceRef], proposal: &[SourceRef]) -> bool {
    proposal.iter().any(|proposal_ref| {
        record.iter().any(|record_ref| {
            !proposal_ref.path.trim().is_empty() && proposal_ref.path == record_ref.path
        })
    })
}

fn load_adaptation_decision(
    root: &std::path::Path,
    learning_record_id: &str,
) -> Result<AdaptationDecision> {
    let path = adaptation_decision_path(root, learning_record_id);
    let bytes = std::fs::read(&path).map_err(|e| {
        anyhow::anyhow!(
            "read AdaptationDecision for learning record '{}': {}: {e}",
            learning_record_id,
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| anyhow::anyhow!("parse AdaptationDecision {}: {e}", path.display()))
}

fn adaptation_decision_path(root: &std::path::Path, decision_key: &str) -> std::path::PathBuf {
    learning_dir(root)
        .join("decisions")
        .join(format!("{}.json", adaptation_decision_id(decision_key)))
}

fn adaptation_decision_id(decision_key: &str) -> String {
    format!("adapt-{decision_key}")
}

pub fn run_adapt(root: &std::path::Path, run_id: &str) -> Result<ExitCode> {
    use swe_seed_core::eval::EvalResult;
    use swe_seed_core::learning::build_adaptation_decision;

    // Collect this run's eval results (written by `eval run --output` or by the
    // caller under `.agent-harness/learning/runs/<run-id>/`). Missing results →
    // every class is Inconclusive and the decision fails closed (blocks).
    let run_dir = learning_dir(root).join("runs").join(run_id);
    let mut results: Vec<EvalResult> = Vec::new();
    if run_dir.is_dir() {
        for entry in std::fs::read_dir(&run_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            match serde_json::from_slice::<EvalResult>(&std::fs::read(&path)?) {
                Ok(r) => results.push(r),
                Err(e) => {
                    eprintln!("adapt: skipping unparseable {}: {e}", path.display());
                }
            }
        }
    }

    let decision = build_adaptation_decision(run_id, &results);
    let out_dir = learning_dir(root).join("decisions");
    std::fs::create_dir_all(&out_dir)?;
    let out = adaptation_decision_path(root, run_id);
    std::fs::write(
        &out,
        format!("{}\n", serde_json::to_string_pretty(&decision)?),
    )?;
    println!("{}", serde_json::to_string_pretty(&decision)?);
    Ok(ExitCode::SUCCESS)
}
