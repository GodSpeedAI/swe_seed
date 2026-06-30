//! `LearningCandidate` + promotion gate (spec 0016). A candidate promotes to a
//! `SkillProposal` only with evidence and, when required, a linked regression.

use serde::{Deserialize, Serialize};

use crate::contracts::harness::SourceRef;
use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `LearningCandidate`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LearningCandidate {
    pub id: String,
    pub source_run_id: String,
    pub candidate_type: String,
    pub claim: String,
    pub evidence: Vec<SourceRef>,
    pub scope: String,
    pub confidence: String,
    pub promotion_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_regression_case: Option<String>,
}

impl BamlParity for LearningCandidate {
    fn baml_name() -> &'static str {
        "LearningCandidate"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "source_run_id",
                "candidate_type",
                "claim",
                "evidence",
                "scope",
                "confidence",
                "promotion_status",
                "required_regression_case",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "SourceRef[]",
                "string",
                "string",
                "string",
                "string?",
            ],
        }
    }
}

/// Promotion outcome: either allowed (with the artifact kind to promote into)
/// or blocked with the gate that stopped it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionOutcome {
    Allow { into: String },
    Block { reason: String },
}

/// Gate a candidate for promotion. Requires a non-empty claim, evidence, and —
/// if `required_regression_case` is set — a non-empty regression case id. The
/// returned `into` is the candidate's own `candidate_type`.
pub fn can_promote_candidate(candidate: &LearningCandidate) -> PromotionOutcome {
    if candidate.claim.trim().is_empty() {
        return PromotionOutcome::Block {
            reason: "candidate has an empty claim".into(),
        };
    }
    if candidate.candidate_type.trim().is_empty() {
        return PromotionOutcome::Block {
            reason: "candidate has an empty candidate_type (no promotion target)".into(),
        };
    }
    if candidate.evidence.is_empty() {
        return PromotionOutcome::Block {
            reason: "candidate has no evidence".into(),
        };
    }
    if let Some(req) = candidate.required_regression_case.as_deref() {
        if req.trim().is_empty() {
            return PromotionOutcome::Block {
                reason: "candidate requires a regression case but none is linked".into(),
            };
        }
    }
    PromotionOutcome::Allow {
        into: candidate.candidate_type.clone(),
    }
}
