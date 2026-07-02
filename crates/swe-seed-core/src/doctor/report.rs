//! Doctor report + aggregation.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{check, drift};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DoctorStatus {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub overall: DoctorStatus,
}

/// The core eval spec path doctor runs against (repo-relative).
pub const CORE_EVAL_SPEC: &str = "tests/fixtures/eval.toml";

/// Run all doctor checks. Overall precedence is `Fail` > `Warn` > `Pass`: any
/// `Fail` fails the run, otherwise any `Warn` makes overall `Warn` (non-clean
/// runs are not hidden), and only an all-`Pass` run is `Pass`.
pub fn run_doctor(root: &Path) -> DoctorReport {
    let mut checks = Vec::new();
    checks.push(check::boundary_check(root));

    let core = root.join(CORE_EVAL_SPEC);
    if core.is_file() {
        checks.push(check::eval_check(root, &core));
        checks.push(check::frozen_check(root, &core));
    } else {
        // Keep the check set consistent (boundary + eval + frozen + drift)
        // even when the core spec is absent: report both as blocked/warn.
        checks.push(DoctorCheck {
            name: "eval".into(),
            status: DoctorStatus::Warn,
            detail: format!("no core eval spec at {CORE_EVAL_SPEC}"),
        });
        checks.push(DoctorCheck {
            name: "frozen-integrity".into(),
            status: DoctorStatus::Warn,
            detail: "no core eval spec to freeze-check".into(),
        });
    }
    checks.push(drift::manifest_drift_check(root));

    let overall = if checks.iter().any(|c| c.status == DoctorStatus::Fail) {
        DoctorStatus::Fail
    } else if checks.iter().any(|c| c.status == DoctorStatus::Warn) {
        DoctorStatus::Warn
    } else {
        DoctorStatus::Pass
    };
    DoctorReport { checks, overall }
}
