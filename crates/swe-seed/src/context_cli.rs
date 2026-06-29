use std::process::ExitCode;

use anyhow::Result;

pub fn run_context_plan(root: &std::path::Path, task: &str) -> Result<ExitCode> {
    let plan = swe_seed_core::context::context_plan(root, task)?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(ExitCode::SUCCESS)
}
