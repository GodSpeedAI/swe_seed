use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProvenanceAction {
    /// Verify every provenance record; exit non-zero iff any fails
    Verify,
    /// List provenance records (exit non-zero if any are corrupt)
    List,
    /// Show one provenance record by capability id
    Show { id: String },
}

pub fn run_provenance(action: ProvenanceAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::{provenance, seed};
    let dir = root.join(".swe-seed/provenance");
    match action {
        ProvenanceAction::Verify => {
            let manifest = seed::assemble_default();
            let (records, problems) = provenance::verify_manifest_records(&dir, &manifest)?;
            if records.is_empty() {
                println!("no provenance records in {}", dir.display());
            }
            for (id, p) in &problems {
                println!("FAIL {id}: {}", p.message());
            }
            if problems.is_empty() {
                println!("ok: {} record(s) verified", records.len());
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        ProvenanceAction::List => {
            // List records AND surface verification problems so a corrupted
            // provenance set is clearly distinguished from a healthy one.
            let (records, problems) = provenance::verify_dir(&dir)?;
            if records.is_empty() {
                println!("no provenance records in {}", dir.display());
            }
            for r in &records {
                println!(
                    "{}\t{}\t{}",
                    r.capability_id, r.license_tag, r.license_status
                );
            }
            for (id, p) in &problems {
                println!("FAIL {id}: {}", p.message());
            }
            if problems.is_empty() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        ProvenanceAction::Show { id } => {
            ensure_safe_capability_id(&id)?;
            let path = dir.join(format!("{id}.json"));
            let bytes = std::fs::read(&path)?;
            // Re-emit pretty to normalize formatting.
            let val: serde_json::Value = serde_json::from_slice(&bytes)?;
            println!("{}", serde_json::to_string_pretty(&val)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn ensure_safe_capability_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || std::path::Path::new(id).components().count() != 1
        || id == "."
        || id == ".."
    {
        return Err(anyhow::anyhow!("invalid capability id: {id}"));
    }
    Ok(())
}
