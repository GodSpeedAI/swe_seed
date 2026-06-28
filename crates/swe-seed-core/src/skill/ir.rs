//! `SkillIR` (harness.baml). The runtime `skills/*.json` is richer than the
//! baml class (`inputs` is an object, `status` lowercase, `version` numeric,
//! plus extra fields). The struct stays 1:1 with `.baml` and loads tolerantly:
//! `inputs.required` is coerced to the `string[]`, `version` to a string, and
//! `status` lowercase-maps to `ArtifactStatus`. Extra JSON fields are ignored.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `ArtifactStatus` (lowercase variant naming matches the runtime
/// JSON, e.g. `"active"`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactStatus {
    Draft,
    Candidate,
    Active,
    Deprecated,
    Rejected,
}

/// harness.baml `ValidationRequirement` (skill-local).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillValidation {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

/// harness.baml `SourceRef` (skill-local; reused from eval would cross modules).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillSourceRef {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub summary: String,
}

/// harness.baml `SkillIR`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIR {
    pub id: String,
    #[serde(deserialize_with = "flex_string", default)]
    pub version: String,
    pub category: String,
    pub jtbd: String,
    pub description: String,
    pub triggers: Vec<String>,
    #[serde(default, deserialize_with = "flex_string_list")]
    pub inputs: Vec<String>,
    pub procedure: Vec<String>,
    pub evidence_required: Vec<String>,
    pub forbidden_behaviors: Vec<String>,
    pub outputs: Vec<String>,
    pub success_criteria: Vec<String>,
    pub failure_modes: Vec<String>,
    pub render_targets: Vec<String>,
    pub status: ArtifactStatus,
    #[serde(default)]
    pub validation: Vec<SkillValidation>,
    #[serde(default)]
    pub sources: Vec<SkillSourceRef>,
}

impl BamlParity for SkillIR {
    fn baml_name() -> &'static str {
        "SkillIR"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "category",
                "jtbd",
                "description",
                "triggers",
                "inputs",
                "procedure",
                "evidence_required",
                "forbidden_behaviors",
                "outputs",
                "success_criteria",
                "failure_modes",
                "render_targets",
                "status",
                "validation",
                "sources",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "ArtifactStatus",
                "ValidationRequirement[]",
                "SourceRef[]",
            ],
        }
    }
}

/// Accept a JSON string or number (the two documented `version` shapes), and
/// reject anything else so malformed metadata fails fast.
fn flex_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Error;
    match Value::deserialize(d)? {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        other => Err(Error::custom(format!(
            "version must be a string or number, got {}",
            json_type(&other)
        ))),
    }
}

/// Accept the documented `inputs` shapes only: an array of strings, or an
/// object `{ "required": [strings] }`. Anything else is a deserialization
/// error; non-string entries are rejected too.
fn flex_string_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    use serde::de::Error;
    let v = Value::deserialize(d)?;
    match v {
        Value::Array(a) => a
            .into_iter()
            .map(|x| match x {
                Value::String(s) => Ok(s),
                other => Err(Error::custom(format!(
                    "inputs list entry must be a string, got {}",
                    json_type(&other)
                ))),
            })
            .collect(),
        Value::Object(o) => match o.get("required") {
            Some(Value::Array(a)) => a
                .iter()
                .map(|x| match x {
                    Value::String(s) => Ok(s.clone()),
                    other => Err(Error::custom(format!(
                        "inputs.required entry must be a string, got {}",
                        json_type(other)
                    ))),
                })
                .collect(),
            Some(other) => Err(Error::custom(format!(
                "inputs.required must be an array of strings, got {}",
                json_type(other)
            ))),
            None => Err(Error::custom(
                "inputs object must have a 'required' array of strings",
            )),
        },
        other => Err(Error::custom(format!(
            "inputs must be an array of strings or {{required: [...]}}, got {}",
            json_type(&other)
        ))),
    }
}

fn json_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
