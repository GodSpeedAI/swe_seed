use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TraceAction {
    /// Start a trace (writes route decision + trace record)
    Start { task: String },
    /// Append a note event
    Append { trace: String, note: String },
    /// Append a checkpoint event
    Checkpoint {
        trace: String,
        #[arg(long)]
        stage: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        next_action: Option<String>,
        #[arg(long = "artifact")]
        artifacts: Vec<String>,
        #[arg(long = "risk")]
        risks: Vec<String>,
    },
    /// Print the resume summary
    Resume { trace: String },
    /// Distill a learning review
    Distill { trace: String },
    /// Finish a trace — claim is required
    Finish {
        trace: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        result: Option<String>,
    },
}

pub fn run_trace(action: TraceAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::trace::lifecycle;
    let value = match action {
        TraceAction::Start { task } => lifecycle::start(root, &task)?,
        TraceAction::Append { trace, note } => lifecycle::append(root, &trace, &note)?,
        TraceAction::Checkpoint {
            trace,
            stage,
            summary,
            next_action,
            artifacts,
            risks,
        } => lifecycle::checkpoint(
            root,
            &trace,
            &stage,
            &summary,
            next_action.as_deref(),
            &artifacts,
            &risks,
        )?,
        TraceAction::Resume { trace } => lifecycle::resume(root, &trace)?,
        TraceAction::Distill { trace } => lifecycle::distill(root, &trace)?,
        TraceAction::Finish {
            trace,
            claim,
            command,
            result,
        } => lifecycle::finish(root, &trace, &claim, command.as_deref(), result.as_deref())?,
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(ExitCode::SUCCESS)
}
