//! `EvalCheck` + `SourceRef` + the deterministic evaluator.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use super::{EvalClass, EvalStatus};

/// harness.baml `SourceRef`.
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct SourceRef {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub summary: String,
}

impl BamlParity for SourceRef {
    fn baml_name() -> &'static str {
        "SourceRef"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["path", "summary"],
            field_types: vec!["string", "string"],
        }
    }
}

/// harness.baml `EvalCheck`. `rule` is a single string interpreted per
/// `check_type` (keeps the type 1:1 with `.baml`).
#[derive(Debug, Clone, Deserialize)]
pub struct EvalCheck {
    pub id: String,
    pub eval_class: EvalClass,
    pub check_type: String,
    #[serde(default)]
    pub target: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub evidence_required: String,
}

fn default_true() -> bool {
    true
}

impl BamlParity for EvalCheck {
    fn baml_name() -> &'static str {
        "EvalCheck"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "eval_class",
                "check_type",
                "target",
                "required",
                "rule",
                "evidence_required",
            ],
            field_types: vec![
                "string",
                "EvalClass",
                "string",
                "string",
                "bool",
                "string",
                "string",
            ],
        }
    }
}

/// Patterns carried in `rule`, one per line (blank lines dropped).
fn rule_patterns(rule: &str) -> Vec<String> {
    rule.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn read_target<'a>(root: &Path, target: &str, err_out: &'a mut Option<String>) -> Option<String> {
    match std::fs::read_to_string(root.join(target)) {
        Ok(t) => Some(t),
        Err(e) => {
            *err_out = Some(format!("cannot read target '{target}': {e}"));
            None
        }
    }
}

/// Evaluate one check deterministically. Mirrors the Python check types:
/// `file_exists`, `static_required_patterns`, `static_forbidden_patterns`,
/// `command_check`, `json_schema_check`, and the manual family. A non-required
/// check that fails is downgraded to `Waived`.
///
/// `trusted` gates `command_check`: spec-controlled shell execution is rejected
/// unless the caller explicitly opted in (default off — fail closed).
pub fn evaluate_check(
    root: &Path,
    check: &EvalCheck,
    trusted: bool,
) -> (EvalStatus, String, Option<String>) {
    let mut status = EvalStatus::Fail;
    let mut evidence = String::new();
    let mut failure_reason: Option<String> = None;
    let mut err: Option<String> = None;

    match check.check_type.as_str() {
        "file_exists" | "artifact_consistency" => {
            let exists = root.join(&check.target).exists();
            status = if exists { EvalStatus::Pass } else { EvalStatus::Fail };
            evidence = check.target.clone();
            if !exists {
                failure_reason = Some(format!("target does not exist: {}", check.target));
            }
        }
        "static_required_patterns" => {
            if let Some(text) = read_target(root, &check.target, &mut err) {
                let patterns = rule_patterns(&check.rule);
                let missing: Vec<&str> = patterns
                    .iter()
                    .filter(|p| !text.contains(p.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                status = if missing.is_empty() { EvalStatus::Pass } else { EvalStatus::Fail };
                evidence = format!("required_patterns_checked={}", patterns.len());
                if !missing.is_empty() {
                    failure_reason = Some(format!("missing required patterns: {}", missing.join(", ")));
                }
            }
        }
        "static_forbidden_patterns" => {
            if let Some(text) = read_target(root, &check.target, &mut err) {
                let patterns = rule_patterns(&check.rule);
                let found: Vec<&str> = patterns
                    .iter()
                    .filter(|p| text.contains(p.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                status = if found.is_empty() { EvalStatus::Pass } else { EvalStatus::Fail };
                evidence = format!("forbidden_patterns_checked={}", patterns.len());
                if !found.is_empty() {
                    failure_reason = Some(format!("found forbidden patterns: {}", found.join(", ")));
                }
            }
        }
        "command_check" => {
            let cmd = check.rule.trim();
            if cmd.is_empty() {
                failure_reason = Some("command_check requires a rule command".into());
            } else if !trusted {
                // Security: spec-controlled shell execution is opt-in only.
                status = EvalStatus::Fail;
                failure_reason = Some(
                    "command_check disabled: requires --trusted mode (spec-controlled shell execution)".into(),
                );
                evidence = format!("command={cmd} (not run; trusted mode off)");
            } else {
                evidence = format!("command={cmd}");
                match std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(root).output() {
                    Ok(out) => {
                        status = if out.status.success() { EvalStatus::Pass } else { EvalStatus::Fail };
                        evidence = format!("command={cmd} exit={}", out.status.code().unwrap_or(-1));
                        if status != EvalStatus::Pass {
                            failure_reason = Some(
                                String::from_utf8_lossy(if out.stderr.is_empty() { &out.stdout } else { &out.stderr })
                                    .trim()
                                    .to_string(),
                            );
                        }
                    }
                    Err(e) => failure_reason = Some(format!("command failed to spawn: {e}")),
                }
            }
        }
        "json_schema_check" => {
            match std::fs::read(root.join(&check.target)) {
                Ok(bytes) => {
                    let data: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
                    let required = rule_patterns(&check.rule);
                    let missing: Vec<&str> = required
                        .iter()
                        .filter(|f| !data.get(f.as_str()).is_some_and(|v| !v.is_null()))
                        .map(|s| s.as_str())
                        .collect();
                    status = if missing.is_empty() { EvalStatus::Pass } else { EvalStatus::Fail };
                    evidence = format!("required_fields_checked={}", required.len());
                    if !missing.is_empty() {
                        failure_reason = Some(format!("missing required fields: {}", missing.join(", ")));
                    }
                }
                Err(e) => err = Some(format!("cannot read target '{}': {e}", check.target)),
            }
        }
        "manual_check" | "reflection_check" | "promotion_policy" => {
            // These checks need runtime proof (a ProofRecord's attached evidence),
            // not the static `evidence_required` metadata. A deterministic run has
            // no attached proof, so the check stays Inconclusive — never auto-pass.
            status = EvalStatus::Inconclusive;
            failure_reason = Some(format!(
                "{} requires runtime proof (ProofRecord evidence); none attached",
                check.check_type
            ));
        }
        other => {
            failure_reason = Some(format!("unsupported check type: {other}"));
        }
    }

    if let Some(e) = err.take() {
        failure_reason = Some(e);
    }

    // Live-pass-only: a non-required failure is a waiver, never a pass.
    if !check.required && status == EvalStatus::Fail {
        status = EvalStatus::Waived;
    }

    (status, evidence, failure_reason)
}
