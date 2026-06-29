//! `SkillProposal` + validation (spec 0016). A proposal must carry the evals
//! that verify the new behavior, a rollout plan, **and a rollback plan** — no
//! proposal without a rollback.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::eval::{EvalCheck, SourceRef};
use super::ValidationRequirement;

/// harness.baml `SkillProposal`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillProposal {
    pub id: String,
    pub proposed_skill_id: String,
    pub observed_problem: String,
    pub evidence: Vec<SourceRef>,
    pub proposed_behavior: Vec<String>,
    pub evals: Vec<EvalCheck>,
    pub rollout_plan: String,
    pub rollback_plan: String,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
}

impl BamlParity for SkillProposal {
    fn baml_name() -> &'static str {
        "SkillProposal"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "proposed_skill_id",
                "observed_problem",
                "evidence",
                "proposed_behavior",
                "evals",
                "rollout_plan",
                "rollback_plan",
                "validation",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "SourceRef[]",
                "string[]",
                "EvalCheck[]",
                "string",
                "string",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// Validate a skill proposal. Observed problem, proposed behavior, the evals
/// that verify it, a rollout plan, and a rollback plan are all required. This
/// is the "no proposal without a rollback" gate (spec 0016 §5).
pub fn validate_proposal(proposal: &SkillProposal) -> Result<(), String> {
    if proposal.observed_problem.trim().is_empty() {
        return Err(format!(
            "skill proposal '{}' has an empty observed_problem",
            proposal.id
        ));
    }
    if proposal
        .proposed_behavior
        .iter()
        .all(|s| s.trim().is_empty())
    {
        return Err(format!(
            "skill proposal '{}' has no proposed_behavior",
            proposal.id
        ));
    }
    if proposal.evals.is_empty() {
        return Err(format!(
            "skill proposal '{}' has no evals (no way to verify the new behavior)",
            proposal.id
        ));
    }
    if proposal.rollout_plan.trim().is_empty() {
        return Err(format!(
            "skill proposal '{}' has an empty rollout_plan",
            proposal.id
        ));
    }
    // ponytail: the rollback gate is the load-bearing rule of this module.
    if proposal.rollback_plan.trim().is_empty() {
        return Err(format!(
            "skill proposal '{}' has an empty rollback_plan (no proposal without a rollback)",
            proposal.id
        ));
    }
    Ok(())
}
