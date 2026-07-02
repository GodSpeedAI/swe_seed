//! `RegressionCase` + linkage (spec 0016). A regression case links an
//! `EvalCheck` so a captured failure mode is caught on recurrence — that is
//! how a lesson becomes a permanent, enforceable check.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::eval::{EvalCheck, EvalClass};

/// harness.baml `RegressionCase`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegressionCase {
    pub id: String,
    pub source_run_id: String,
    pub failure_mode: String,
    pub detection: String,
    pub future_rule: String,
    pub status: String,
    pub linked_eval_check: String,
}

impl BamlParity for RegressionCase {
    fn baml_name() -> &'static str {
        "RegressionCase"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "source_run_id",
                "failure_mode",
                "detection",
                "future_rule",
                "status",
                "linked_eval_check",
            ],
            field_types: vec![
                "string", "string", "string", "string", "string", "string", "string",
            ],
        }
    }
}

/// Validate a regression case. The failure mode, how to detect it, the rule to
/// apply going forward, and a linked eval check id are all required — a
/// regression without a linked check can never fire and is rejected.
pub fn validate_regression(case: &RegressionCase) -> Result<(), String> {
    for (field, val) in [
        ("failure_mode", &case.failure_mode),
        ("detection", &case.detection),
        ("future_rule", &case.future_rule),
        ("linked_eval_check", &case.linked_eval_check),
    ] {
        if val.trim().is_empty() {
            return Err(format!(
                "regression case '{}' has an empty {field}",
                case.id
            ));
        }
    }
    // A `require:` future_rule must carry rule content, else the linked
    // EvalCheck would be built with an empty pattern (kept aligned with the
    // prefix-trim in `eval_check_for_regression`).
    if let Some(rest) = case.future_rule.strip_prefix("require:") {
        if rest.trim().is_empty() {
            return Err(format!(
                "regression case '{}' future_rule uses 'require:' with no rule content",
                case.id
            ));
        }
    }
    Ok(())
}

/// Build the `EvalCheck` a regression case links, so the captured failure mode
/// is detected on recurrence. The failure mode is encoded as a forbidden
/// pattern (or a required pattern when `future_rule` starts with `require:`);
/// the check id is the case's `linked_eval_check`. Running this check against a
/// target that re-exhibits the failure mode returns `Fail`.
pub fn eval_check_for_regression(case: &RegressionCase) -> Result<EvalCheck, String> {
    validate_regression(case)?;
    let (check_type, rule) = if let Some(req) = case.future_rule.strip_prefix("require:") {
        ("static_required_patterns", req.trim().to_string())
    } else {
        ("static_forbidden_patterns", case.failure_mode.clone())
    };
    Ok(EvalCheck {
        id: case.linked_eval_check.clone(),
        eval_class: EvalClass::LearningQuality,
        check_type: check_type.into(),
        target: case.detection.clone(),
        required: true,
        rule,
        evidence_required: format!("regression {}: must not recur", case.id),
    })
}
