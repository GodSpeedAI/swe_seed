//! Doctor checks. Each returns a `DoctorCheck` (Pass/Fail/Warn + detail).

use std::path::Path;

use super::report::{DoctorCheck, DoctorStatus};
use crate::eval::{load_eval_spec, run_eval, spec::check_frozen};
use crate::seed::{assemble_default, validate_boundaries};

/// Layer-boundary validation on the assembled manifest.
pub fn boundary_check(_root: &Path) -> DoctorCheck {
    let manifest = assemble_default();
    let report = validate_boundaries(&manifest);
    if report.passed {
        DoctorCheck {
            name: "boundary".into(),
            status: DoctorStatus::Pass,
            detail: format!("{} capabilities, no findings", manifest.capabilities.len()),
        }
    } else {
        DoctorCheck {
            name: "boundary".into(),
            status: DoctorStatus::Fail,
            detail: format!(
                "{} finding(s): {}",
                report.findings.len(),
                report.findings[0].issue
            ),
        }
    }
}

/// Run the core eval spec; Pass iff the eval status is Pass.
pub fn eval_check(root: &Path, spec_path: &Path) -> DoctorCheck {
    let spec = match load_eval_spec(spec_path) {
        Ok(s) => s,
        Err(e) => {
            return DoctorCheck {
                name: format!("eval:{}", spec_path.display()),
                status: DoctorStatus::Fail,
                detail: format!("invalid spec: {e:#}"),
            };
        }
    };
    let name = format!("eval:{}", spec.id);
    match run_eval(root, spec_path, &spec, false) {
        Ok(result) => {
            let status = if result.status == crate::eval::EvalStatus::Pass {
                DoctorStatus::Pass
            } else {
                DoctorStatus::Fail
            };
            DoctorCheck {
                name,
                status,
                detail: result.summary,
            }
        }
        Err(e) => DoctorCheck {
            name,
            status: DoctorStatus::Fail,
            detail: format!("eval failed: {e:#}"),
        },
    }
}

/// Frozen-after-handoff integrity for the core spec.
pub fn frozen_check(root: &Path, spec_path: &Path) -> DoctorCheck {
    let spec = match load_eval_spec(spec_path) {
        Ok(s) => s,
        Err(e) => {
            return DoctorCheck {
                name: "frozen-integrity".into(),
                status: DoctorStatus::Fail,
                detail: format!("invalid spec: {e:#}"),
            };
        }
    };
    match check_frozen(root, spec_path, &spec) {
        Ok(()) => DoctorCheck {
            name: "frozen-integrity".into(),
            status: DoctorStatus::Pass,
            detail: if spec.frozen_after_handoff {
                "frozen spec intact".into()
            } else {
                "spec not frozen".into()
            },
        },
        Err(e) => DoctorCheck {
            name: "frozen-integrity".into(),
            status: DoctorStatus::Fail,
            detail: format!("{e:#}"),
        },
    }
}
