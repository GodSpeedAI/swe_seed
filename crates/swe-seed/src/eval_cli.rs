use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum EvalAction {
    /// Run an eval spec; exit non-zero iff the eval fails
    Run {
        /// Path to the eval spec (TOML or JSON)
        #[arg(long)]
        spec: String,
        /// Allow `command_check` (spec-controlled shell execution). Default off.
        #[arg(long)]
        trusted: bool,
        /// Optional output path for the result JSON
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn run_eval_cmd(action: EvalAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::eval::{load_eval_spec, run_eval, EvalStatus};
    let EvalAction::Run {
        spec,
        trusted,
        output,
    } = action;
    let spec_path = root.join(&spec);
    let loaded = load_eval_spec(&spec_path)?;
    let result = run_eval(root, &spec_path, &loaded, trusted)?;
    let rendered = format!("{}\n", serde_json::to_string_pretty(&result)?);
    if let Some(out) = output {
        let out_path = root.join(&out);
        if let Some(p) = out_path.parent() {
            std::fs::create_dir_all(p).ok();
        }
        std::fs::write(&out_path, &rendered)?;
    }
    print!("{rendered}");
    Ok(if result.status == EvalStatus::Pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

pub fn run_validate(root: &std::path::Path, spec: Option<String>) -> Result<ExitCode> {
    use swe_seed_core::doctor::CORE_EVAL_SPEC;
    use swe_seed_core::eval::{load_eval_spec, spec::check_frozen};
    let rel = spec.unwrap_or_else(|| CORE_EVAL_SPEC.to_string());
    let spec_path = root.join(&rel);
    let loaded = match load_eval_spec(&spec_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("validate: {rel}: {e:#}");
            return Ok(ExitCode::from(1));
        }
    };
    if let Err(e) = check_frozen(root, &spec_path, &loaded) {
        eprintln!("validate: {rel}: {e:#}");
        return Ok(ExitCode::from(1));
    }
    println!("validate: {rel} ok ({} checks)", loaded.checks.len());
    Ok(ExitCode::SUCCESS)
}
