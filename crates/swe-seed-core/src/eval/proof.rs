//! `ProofRecord` + validation (spec 0013). Claims require evidence.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use super::check::SourceRef;

/// harness.baml `ProofRecord`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRecord {
    pub id: String,
    pub run_id: String,
    pub claims: Vec<String>,
    pub evidence: Vec<SourceRef>,
    #[serde(default)]
    pub skipped_checks: Vec<String>,
    #[serde(default)]
    pub unresolved_risks: Vec<String>,
}

impl BamlParity for ProofRecord {
    fn baml_name() -> &'static str {
        "ProofRecord"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["id", "run_id", "claims", "evidence", "skipped_checks", "unresolved_risks"],
            field_types: vec![
                "string",
                "string",
                "string[]",
                "SourceRef[]",
                "string[]",
                "string[]",
            ],
        }
    }
}

/// Validate a proof record with per-claim evidence linkage. Claims map
/// positionally to evidence (`claim[i]` is backed by `evidence[i]`), so every
/// claim needs its own non-empty evidence ref — a single `SourceRef` cannot
/// cover multiple claims. (The `ProofRecord` shape stays 1:1 with `.baml`;
/// linkage is expressed via the existing parallel `claims`/`evidence` arrays.)
pub fn validate_proof(record: &ProofRecord) -> Result<(), String> {
    if record.claims.is_empty() {
        return Ok(());
    }
    if record.evidence.len() < record.claims.len() {
        return Err(format!(
            "proof record '{}' makes {} claim(s) but only carries {} evidence ref(s); each claim needs its own backing evidence",
            record.id,
            record.claims.len(),
            record.evidence.len()
        ));
    }
    for (i, ev) in record.evidence.iter().take(record.claims.len()).enumerate() {
        if ev.path.trim().is_empty() {
            return Err(format!(
                "proof record '{}': evidence[{i}] for claim has an empty path",
                record.id
            ));
        }
    }
    Ok(())
}
