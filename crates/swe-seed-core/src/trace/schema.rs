//! `TraceSchema` (harness.baml). Defines identifier/required fields and rejects
//! trace records missing the fields that make them addressable.

use serde::Deserialize;

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::trace::record::TraceRecord;

#[derive(Debug, Clone, Deserialize)]
pub struct TraceSchema {
    pub id: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub identifier_fields: Vec<String>,
    pub proof_fields: Vec<String>,
    #[serde(default)]
    pub validation: Vec<TraceSchemaValidation>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TraceSchemaValidation {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

impl BamlParity for TraceSchema {
    fn baml_name() -> &'static str {
        "TraceSchema"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "required_fields",
                "optional_fields",
                "identifier_fields",
                "proof_fields",
                "validation",
            ],
            field_types: vec![
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// The canonical trace schema: a record is addressable by `trace_id` + `task`.
pub fn trace_schema() -> TraceSchema {
    TraceSchema {
        id: "trace".into(),
        required_fields: vec![
            "trace_id".into(),
            "task".into(),
            "route".into(),
            "events".into(),
        ],
        optional_fields: vec!["verification".into(), "unresolved_risks".into()],
        identifier_fields: vec!["trace_id".into(), "task".into()],
        proof_fields: vec!["verification".into(), "completion_claim".into()],
        validation: Vec::new(),
    }
}

/// Reject a record missing any identifier field (empty or absent). Returns the
/// list of missing identifiers (empty = valid).
pub fn missing_identifiers(schema: &TraceSchema, record: &TraceRecord) -> Vec<String> {
    let mut missing = Vec::new();
    for id in &schema.identifier_fields {
        let empty = match id.as_str() {
            "trace_id" => record.trace_id.trim().is_empty(),
            "task" => record.task.trim().is_empty(),
            "route_decision_record" => record.route_decision_record.trim().is_empty(),
            other => {
                // Unknown identifier field → treat as missing (fail closed).
                let _ = other;
                true
            }
        };
        if empty {
            missing.push(id.clone());
        }
    }
    missing
}
