//! `ReflectionTemplate` + `reflect` (spec 0016). Reflection runs after a
//! finished trace; `no_change_allowed=false` forces an explicit disposition
//! with rationale rather than a silent skip.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::eval::SourceRef;
use crate::trace::TraceRecord;
use super::record::{LearningDisposition, LearningRecord};
use super::ValidationRequirement;

/// harness.baml `ReflectionTemplate`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReflectionTemplate {
    pub id: String,
    pub prompts: Vec<String>,
    pub evidence_required: Vec<String>,
    pub no_change_allowed: bool,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
}

impl BamlParity for ReflectionTemplate {
    fn baml_name() -> &'static str {
        "ReflectionTemplate"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["id", "prompts", "evidence_required", "no_change_allowed", "validation"],
            field_types: vec![
                "string",
                "string[]",
                "string[]",
                "bool",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// Default reflection template: a finished run with a completion claim becomes
/// an `ApprovedLesson`; otherwise an explicit `NoChange`. `no_change_allowed`
/// is false so a skip must carry rationale (enforced by `validate_reflection`).
pub fn default_template() -> ReflectionTemplate {
    ReflectionTemplate {
        id: "default-reflection".into(),
        prompts: vec![
            "What was the outcome of this run?".into(),
            "Is there a reusable lesson, a skill to propose, or a regression to record?".into(),
        ],
        evidence_required: vec!["finished trace".into()],
        no_change_allowed: false,
        validation: Vec::new(),
    }
}

/// Reflect a finished trace into a *proposed* LearningRecord. Deterministic
/// (no LLM runtime): a completion claim yields `ApprovedLesson`; its absence
/// yields an explicit `NoChange`. The trace is always cited as evidence.
pub fn reflect(
    template: &ReflectionTemplate,
    trace: &TraceRecord,
    trace_path: &str,
) -> Result<LearningRecord, String> {
    let evidence = vec![SourceRef {
        path: trace_path.to_string(),
        summary: format!("trace for: {}", trace.task),
    }];
    let (disposition, summary, reason) = match &trace.completion_claim {
        Some(claim) => (
            LearningDisposition::ApprovedLesson,
            claim.claim.clone(),
            format!(
                "run completed ({}) — proposing an approved lesson",
                claim.claim
            ),
        ),
        None => (
            LearningDisposition::NoChange,
            format!("no completion claim for: {}", trace.task),
            "run has no completion claim; no lesson extracted".into(),
        ),
    };
    let record = LearningRecord {
        id: format!("learn-{}", trace.trace_id),
        disposition,
        summary,
        evidence,
        decision_reason: reason,
        follow_up_artifact: None,
        validation: Vec::new(),
    };
    validate_reflection(template, &record)?;
    Ok(record)
}

/// Validate a reflection against its template. Evidence is always required; if
/// the template forbids a silent no-change, a `NoChange` record must carry a
/// non-empty rationale (an explicit disposition, not a skip).
pub fn validate_reflection(
    template: &ReflectionTemplate,
    record: &LearningRecord,
) -> Result<(), String> {
    if record.evidence.is_empty() {
        return Err(format!(
            "reflection '{}' requires evidence (template '{}')",
            record.id, template.id
        ));
    }
    if !template.no_change_allowed
        && record.disposition == LearningDisposition::NoChange
        && record.decision_reason.trim().is_empty()
    {
        return Err(format!(
            "reflection '{}': template '{}' has no_change_allowed=false; a NoChange needs explicit rationale",
            record.id, template.id
        ));
    }
    // Reuse the record-level invariant (summary/reason/evidence-for-lessons).
    super::record::validate_learning_record(record)
}
