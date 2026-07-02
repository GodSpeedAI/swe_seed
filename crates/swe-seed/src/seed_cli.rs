use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SeedAction {
    /// Assemble the SeedPackageManifest and write it to .swe-seed/
    Assemble,
    /// Validate layer boundaries; exit non-zero iff any finding
    ValidateBoundaries,
    /// Produce a deterministic (idempotent) regeneration plan
    Regenerate,
}

pub fn run_seed(action: SeedAction, root: &std::path::Path) -> Result<ExitCode> {
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
            let manifest = seed::assemble_default();
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
                swe_seed_spec_path: "docs/specs/0018-layer-boundary-governance.md".into(),
                approved_project_seeds: vec![
                    ".agents/plans/0001-swe-seed-v0-1-implementation.md".into()
                ],
                approved_seed_package_manifests: vec![seed::manifest::DEFAULT_MANIFEST_PATH.into()],
                approved_layer_capability_maps: vec![
                    "docs/specs/0003-capability-registry.md".into()
                ],
                approved_lower_layer_artifact_refs: vec![
                    "docs/specs/0012-existing-harness-reconciliation.md".into(),
                ],
            };
            let plan = seed::regenerate::regenerate(root, &input);
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}
