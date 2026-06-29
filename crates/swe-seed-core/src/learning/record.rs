//! `LearningDisposition` + `LearningRecord` (spec 0016). A record classifies a
//! reflection into a disposition with a decision reason and evidence.

use serde::{Deserialize, Serialize};

use super::ValidationRequirement;
use crate::contracts::parity::{BamlParity, BamlShape};
use crate::eval::SourceRef;

/// harness.baml `LearningDisposition`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum LearningDisposition {
    ApprovedLesson,
    RejectedLesson,
    SkillProposal,
    RegressionCase,
    HarnessADR,
    NoChange,
}

impl BamlParity for LearningDisposition {
    fn baml_name() -> &'static str {
        "LearningDisposition"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "ApprovedLesson",
                "RejectedLesson",
                "SkillProposal",
                "RegressionCase",
                "HarnessADR",
                "NoChange",
            ],
        }
    }
}

/// harness.baml `LearningRecord`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LearningRecord {
    pub id: String,
    pub disposition: LearningDisposition,
    pub summary: String,
    pub evidence: Vec<SourceRef>,
    pub decision_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_artifact: Option<String>,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
}

impl BamlParity for LearningRecord {
    fn baml_name() -> &'static str {
        "LearningRecord"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "disposition",
                "summary",
                "evidence",
                "decision_reason",
                "follow_up_artifact",
                "validation",
            ],
            field_types: vec![
                "string",
                "LearningDisposition",
                "string",
                "SourceRef[]",
                "string",
                "string?",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// Validate a learning record. Every record needs a summary + rationale; an
/// actual lesson (anything but `NoChange`) must be backed by evidence.
pub fn validate_learning_record(record: &LearningRecord) -> Result<(), String> {
    if record.summary.trim().is_empty() {
        return Err(format!(
            "learning record '{}' has an empty summary",
            record.id
        ));
    }
    if record.decision_reason.trim().is_empty() {
        return Err(format!(
            "learning record '{}' has an empty decision_reason (no silent learning)",
            record.id
        ));
    }
    if record.disposition != LearningDisposition::NoChange && record.evidence.is_empty() {
        return Err(format!(
            "learning record '{}' disposition {:?} requires evidence",
            record.id, record.disposition
        ));
    }
    Ok(())
}
