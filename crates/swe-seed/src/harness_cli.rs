//! `swe-seed harness` — harness structure validation (CI integrity). Rust port
//! of the load-bearing subset of the former `scripts/harness.py validate`.

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

pub fn run_harness_validate(root: &Path) -> Result<ExitCode> {
    let errors = swe_seed_core::harness_validate::validate(root);
    if errors.is_empty() {
        println!("Harness validation passed");
        return Ok(ExitCode::SUCCESS);
    }
    for e in &errors {
        eprintln!("{e}");
    }
    eprintln!("Harness validation failed: {} error(s)", errors.len());
    Ok(ExitCode::from(1))
}
