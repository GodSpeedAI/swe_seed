//! Fail-closed provenance verification (spec 0009).

use std::path::Path;

use anyhow::{Context, Result};

use super::record::ProvenanceRecord;

/// A single verification failure for one record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceProblem {
    MissingSourceHash,
    MalformedSourceHash,
    MissingLicenseTag,
    CodeCopied,
    LicenseNotClear { status: String },
}

impl ProvenanceProblem {
    pub fn message(&self) -> String {
        match self {
            ProvenanceProblem::MissingSourceHash => "missing source_hash".into(),
            ProvenanceProblem::MalformedSourceHash => {
                "malformed source_hash (expected 'sha256:<hex>')".into()
            }
            ProvenanceProblem::MissingLicenseTag => "missing license_tag".into(),
            ProvenanceProblem::CodeCopied => "code_copied is true".into(),
            ProvenanceProblem::LicenseNotClear { status } => {
                format!("license_status '{status}' is not clear (human approval required)")
            }
        }
    }
}

/// `source_hash` must be `sha256:` followed by one or more hex digits, matching
/// the contract produced by [`super::content_hash`] (spec 0009).
fn is_valid_source_hash(h: &str) -> bool {
    let Some(hex) = h.strip_prefix("sha256:") else {
        return false;
    };
    !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Verify one record. Fails closed on missing/malformed hash, missing license,
/// copied code, or a non-`clear` license status (human-gated per spec 0009).
pub fn verify_record(r: &ProvenanceRecord) -> Result<(), ProvenanceProblem> {
    if r.source_hash.trim().is_empty() {
        return Err(ProvenanceProblem::MissingSourceHash);
    }
    if !is_valid_source_hash(r.source_hash.trim()) {
        return Err(ProvenanceProblem::MalformedSourceHash);
    }
    if r.license_tag.trim().is_empty() {
        return Err(ProvenanceProblem::MissingLicenseTag);
    }
    if r.code_copied {
        return Err(ProvenanceProblem::CodeCopied);
    }
    if r.license_status != "clear" {
        return Err(ProvenanceProblem::LicenseNotClear {
            status: r.license_status.clone(),
        });
    }
    Ok(())
}

/// Read and verify every `*.json` record under `dir`. Returns `(records, problems)`.
///
/// Fail-closed on filesystem errors: a genuinely unreadable directory or entry
/// is propagated, not masked as an empty record set. A *missing* directory is
/// the one intentional exception (zero records → vacuously ok), matching the
/// documented behavior that an empty provenance set is not an error.
pub fn verify_dir(dir: &Path) -> Result<(Vec<ProvenanceRecord>, Vec<(String, ProvenanceProblem)>)> {
    let mut records = Vec::new();
    let mut problems = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok((records, problems));
        }
        Err(e) => return Err(anyhow::Error::from(e).context(format!("read {}", dir.display()))),
    };

    let mut files: Vec<_> = Vec::new();
    for entry in entries {
        let p = entry.context("read dir entry")?.path();
        if p.extension().is_some_and(|x| x == "json") {
            files.push(p);
        }
    }
    files.sort();
    for f in files {
        let bytes = std::fs::read(&f).with_context(|| format!("read {}", f.display()))?;
        let rec: ProvenanceRecord =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", f.display()))?;
        if let Err(p) = verify_record(&rec) {
            problems.push((rec.capability_id.clone(), p));
        }
        records.push(rec);
    }
    Ok((records, problems))
}
