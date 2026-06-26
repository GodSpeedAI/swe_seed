//! `swe-seed` CLI. Phase 0: `--help`. Phase 1: `seed {assemble,
//! validate-boundaries, regenerate}` + `provenance {verify, list, show}`.
//!
//! All repo-relative defaults resolve under a discovered project root
//! (env `SWE_SEED_ROOT`, else the workspace root by walking up from cwd),
//! so launching the binary from outside the repo still anchors paths correctly.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "swe-seed",
    version,
    about = "SWE Seed governance CLI (SweSeed → Harness → Fabricator)"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Assemble and validate the seed package (specs 0003/0018)
    Seed {
        #[command(subcommand)]
        action: SeedAction,
    },
    /// Capability provenance records (spec 0009)
    Provenance {
        #[command(subcommand)]
        action: ProvenanceAction,
    },
}

#[derive(Subcommand)]
enum SeedAction {
    /// Assemble the SeedPackageManifest and write it to .swe-seed/
    Assemble,
    /// Validate layer boundaries; exit non-zero iff any finding
    ValidateBoundaries,
    /// Produce a deterministic (idempotent) regeneration plan
    Regenerate,
}

#[derive(Subcommand)]
enum ProvenanceAction {
    /// Verify every provenance record; exit non-zero iff any fails
    Verify,
    /// List provenance records (exit non-zero if any are corrupt)
    List,
    /// Show one provenance record by capability id
    Show { id: String },
}

/// Discover the project root: `SWE_SEED_ROOT` if set, else walk up from cwd to
/// the first ancestor holding the repo marker (`.agent-harness`) or a workspace
/// `Cargo.toml`; falls back to cwd if none is found.
fn discover_root() -> Result<PathBuf> {
    if let Ok(r) = std::env::var("SWE_SEED_ROOT") {
        return Ok(PathBuf::from(r));
    }
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join(".agent-harness").is_dir() {
            return Ok(dir);
        }
        let cargo = dir.join("Cargo.toml");
        if cargo.is_file() {
            if let Ok(content) = std::fs::read_to_string(&cargo) {
                if content.contains("[workspace]") {
                    return Ok(dir);
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(std::env::current_dir()?)
}

pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let Some(cmd) = cli.command else {
        Cli::parse_from(["swe-seed", "--help"]);
        return Ok(ExitCode::SUCCESS);
    };
    let root = discover_root()?;
    match cmd {
        Command::Seed { action } => run_seed(action, &root),
        Command::Provenance { action } => run_provenance(action, &root),
    }
}

fn run_seed(action: SeedAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::seed;
    match action {
        SeedAction::Assemble => {
            let manifest = seed::assemble_default();
            let path = root.join(seed::manifest::DEFAULT_MANIFEST_PATH);
            seed::manifest::write_manifest(&manifest, &path)?;
            println!(
                "assembled {} capabilities -> {}",
                manifest.capabilities.len(),
                path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        SeedAction::ValidateBoundaries => {
            // Load the assembled manifest if present, else assemble in memory.
            let path = root.join(seed::manifest::DEFAULT_MANIFEST_PATH);
            let manifest = if path.exists() {
                seed::manifest::read_manifest(&path)?
            } else {
                seed::assemble_default()
            };
            let report = seed::validate_boundaries(&manifest);
            println!("{}", serde_json::to_string_pretty(&report)?);
            if report.passed {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        SeedAction::Regenerate => {
            // Default input: the self seed spec + this repo's approved artifacts.
            let input = seed::regenerate::SeedRegenerationInput {
                swe_seed_spec_path: ".agents/specs/0018-layer-boundary-governance.md".into(),
                approved_project_seeds: vec![
                    ".agents/plans/0001-swe-seed-v0-1-implementation.md".into(),
                ],
                approved_seed_package_manifests: vec![seed::manifest::DEFAULT_MANIFEST_PATH.into()],
                approved_layer_capability_maps: vec![".agents/specs/0003-capability-registry.md"
                    .into()],
                approved_lower_layer_artifact_refs: vec![
                    ".agents/specs/0012-existing-harness-reconciliation.md".into(),
                ],
            };
            let plan = seed::regenerate::regenerate(root, &input);
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn run_provenance(action: ProvenanceAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::provenance;
    let dir = root.join(".swe-seed/provenance");
    match action {
        ProvenanceAction::Verify => {
            let (records, problems) = provenance::verify_dir(&dir)?;
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
            let path = dir.join(format!("{id}.json"));
            let bytes = std::fs::read(&path)?;
            // Re-emit pretty to normalize formatting.
            let val: serde_json::Value = serde_json::from_slice(&bytes)?;
            println!("{}", serde_json::to_string_pretty(&val)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}
