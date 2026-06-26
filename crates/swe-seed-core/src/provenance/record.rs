//! `ProvenanceRecord` — the runtime provenance carrier (spec 0009).
//! Stored at `.swe-seed/provenance/<capability-id>.json`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceRecord {
    pub capability_id: String,
    pub source_id: String,
    /// `git` | `local` | other.
    pub source_kind: String,
    pub source_location: String,
    pub source_ref: String,
    /// `sha256:<hex>`.
    pub source_hash: String,
    pub license_tag: String,
    /// `clear` | `ambiguous` | `restricted`.
    pub license_status: String,
    /// ISO date, e.g. `2026-06-26`.
    pub inspected_at: String,
    pub scan_ref: String,
    pub code_copied: bool,
    pub notes: String,
}
