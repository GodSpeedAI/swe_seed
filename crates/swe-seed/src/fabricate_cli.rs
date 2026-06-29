use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum FabricateAction {
    /// Validate a rendered chain's integrity → SemanticChainValidationReport
    ValidateChain { run: String },
}

pub fn run_fabricate(
    root: &std::path::Path,
    need: Option<String>,
    action: Option<FabricateAction>,
) -> Result<ExitCode> {
    use swe_seed_core::fabricator::{fabricate, load_chain, run_dir, validate_semantic_chain};
    use swe_seed_core::util::{slugify, utc_stamp};

    match action {
        Some(FabricateAction::ValidateChain { run }) => {
            // Reload a rendered chain and report its integrity.
            let dir = run_dir(root, &run);
            let chain = match load_chain(&dir) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("fabricate validate-chain: {run}: {e:#}");
                    return Ok(ExitCode::from(1));
                }
            };
            let report = validate_semantic_chain(&chain);
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(if report.passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        None => {
            let Some(need) = need else {
                eprintln!("fabricate: <product-need> is required (or use a subcommand)");
                return Ok(ExitCode::from(2));
            };
            // Bounded run: build → validate → render → proof-gated handoff.
            let run_id = format!("{}-{}", utc_stamp(), slugify(&need));
            match fabricate(root, &need, &run_id) {
                Ok((dir, report)) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "run_id": run_id,
                            "run_dir": dir.strip_prefix(root).unwrap_or(&dir).display().to_string(),
                            "chain_passed": report.passed,
                            "handoff": format!(".fabricator/runs/{run_id}/handoff/MANIFEST.json"),
                        }))?
                    );
                    Ok(ExitCode::SUCCESS)
                }
                Err(report) => {
                    // Chain integrity failed: the run rendered, but handoff was
                    // blocked. Surface the report and exit non-zero.
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "run_id": run_id,
                            "chain_passed": false,
                            "blocked": true,
                            "missing_links": report.missing_links,
                            "blocked_claims": report.blocked_claims,
                        }))?
                    );
                    Ok(ExitCode::from(1))
                }
            }
        }
    }
}
