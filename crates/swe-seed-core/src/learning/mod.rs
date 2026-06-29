//! Learning + adaptation loop (spec 0016). The self-improvement loop:
//! `finished trace → ReflectionTemplate → LearningRecord → (LearningCandidate)
//!    → AdaptationDecision (gated by 4 eval results)
//!    → SkillProposal | RegressionCase | HarnessADR | NoChange`.
//!
//! Depends on `eval` (0013) and `trace` (0014). No proposal promotes without
//! proof + rollback; regression cases make a lesson enforceable.

pub mod adaptation;
pub mod candidate;
pub mod proposal;
pub mod record;
pub mod reflection;
pub mod regression;

pub use adaptation::{build_adaptation_decision, promotion_gate, AdaptationDecision};
pub use candidate::{can_promote_candidate, LearningCandidate, PromotionOutcome};
pub use proposal::{validate_proposal, SkillProposal};
pub use record::{validate_learning_record, LearningDisposition, LearningRecord};
pub use reflection::{default_template, reflect, validate_reflection, ReflectionTemplate};
pub use regression::{eval_check_for_regression, validate_regression, RegressionCase};

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `ValidationRequirement` (learning-local copy; leaf type, not
/// registered for parity — matches the convention used by route/hooks/context).
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ValidationRequirement {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

impl BamlParity for ValidationRequirement {
    fn baml_name() -> &'static str {
        "ValidationRequirement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["check", "blocking", "evidence"],
            field_types: vec!["string", "bool", "string"],
        }
    }
}
