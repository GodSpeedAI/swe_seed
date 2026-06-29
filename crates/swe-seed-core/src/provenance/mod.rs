//! Provenance + clean-room boundary records (spec 0009). Fail-closed:
//! every active capability record needs `source_hash` + `license_tag`, and
//! `code_copied` must be `false`.

pub mod hash;
pub mod record;
pub mod verify;

pub use hash::content_hash;
pub use record::ProvenanceRecord;
pub use verify::{verify_dir, verify_manifest_records, verify_record, ProvenanceProblem};
