//! `EvalCheckResult` / `EvalResult` + `run_eval` orchestration.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::check::evaluate_check;
use super::spec::{check_frozen, EvalSpec};
use super::{EvalClass, EvalStatus};
use crate::contracts::parity::{BamlParity, BamlShape};
use crate::util::utc_now;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCheckResult {
    pub id: String,
    pub eval_class: EvalClass,
    pub status: EvalStatus,
    pub evidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

impl BamlParity for EvalCheckResult {
    fn baml_name() -> &'static str {
        "EvalCheckResult"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["id", "eval_class", "status", "evidence", "failure_reason"],
            field_types: vec!["string", "EvalClass", "EvalStatus", "string", "string?"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub eval_id: String,
    pub run_id: String,
    pub status: EvalStatus,
    pub checks: Vec<EvalCheckResult>,
    pub summary: String,
    pub created_at: String,
}

impl BamlParity for EvalResult {
    fn baml_name() -> &'static str {
        "EvalResult"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "eval_id",
                "run_id",
                "status",
                "checks",
                "summary",
                "created_at",
            ],
            field_types: vec![
                "string",
                "string",
                "EvalStatus",
                "EvalCheckResult[]",
                "string",
                "string",
            ],
        }
    }
}

/// A waived check must carry a reason (waiver rationale / failure detail).
pub fn waived_requires_reason(r: &EvalCheckResult) -> Result<(), String> {
    if r.status == EvalStatus::Waived && r.failure_reason.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err(format!(
            "waived check '{}' requires a reason (failure_reason)",
            r.id
        ));
    }
    Ok(())
}

/// Run an eval spec deterministically. Enforces frozen-after-handoff first,
/// then evaluates each check. Status is `Pass` only when no required check
/// failed AND at least one check is a live pass (live-pass-only promotion).
///
/// `trusted` gates `command_check` (spec-controlled shell execution); default
/// off — pass `true` only from an explicit CLI opt-in.
pub fn run_eval(
    root: &Path,
    spec_path: &Path,
    spec: &EvalSpec,
    trusted: bool,
) -> Result<EvalResult> {
    check_frozen(root, spec_path, spec)?;

    let checks: Vec<EvalCheckResult> = spec
        .checks
        .iter()
        .map(|c| {
            let (status, evidence, failure_reason) = evaluate_check(root, c, trusted);
            EvalCheckResult {
                id: c.id.clone(),
                eval_class: c.eval_class,
                status,
                evidence,
                failure_reason,
            }
        })
        .collect();

    let passed = checks
        .iter()
        .filter(|r| r.status == EvalStatus::Pass)
        .count();
    let required_failed = spec
        .checks
        .iter()
        .zip(checks.iter())
        .any(|(c, r)| c.required && r.status != EvalStatus::Pass);
    // Live-pass-only: a fully waived/failed eval never activates.
    let any_live_pass = checks.iter().any(|r| r.status == EvalStatus::Pass);
    let status = if !required_failed && any_live_pass {
        EvalStatus::Pass
    } else {
        EvalStatus::Fail
    };

    Ok(EvalResult {
        eval_id: spec.id.clone(),
        run_id: spec.run_id.clone(),
        status,
        checks,
        summary: format!("{passed}/{} checks passed", spec.checks.len()),
        created_at: utc_now(),
    })
}
