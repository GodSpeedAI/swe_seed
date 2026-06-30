//! Eval + proof (spec 0013). Deterministic rule-based evaluation across the
//! four eval classes, frozen-after-handoff enforcement, proof records that
//! require evidence, and live-pass-only promotion.

pub mod check;
pub mod proof;
pub mod result;
pub mod spec;

pub use crate::contracts::harness::SourceRef;
pub use check::{evaluate_check, EvalCheck};
pub use proof::{validate_proof, ProofRecord};
pub use result::{run_eval, waived_requires_reason, EvalCheckResult, EvalResult};
pub use spec::{check_frozen, load_eval_spec, EvalSpec};

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// The four eval classes (spec 0013).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EvalClass {
    ProductOutcome,
    ProcessCompliance,
    LearningQuality,
    AdaptationEligibility,
}

/// Eval check / result status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EvalStatus {
    Pass,
    Fail,
    Waived,
    Inconclusive,
}

impl BamlParity for EvalClass {
    fn baml_name() -> &'static str {
        "EvalClass"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "ProductOutcome",
                "ProcessCompliance",
                "LearningQuality",
                "AdaptationEligibility",
            ],
        }
    }
}

impl BamlParity for EvalStatus {
    fn baml_name() -> &'static str {
        "EvalStatus"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec!["Pass", "Fail", "Waived", "Inconclusive"],
        }
    }
}
