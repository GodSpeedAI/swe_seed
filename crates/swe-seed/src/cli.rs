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
    /// Route a task to a job-type card (specs 0004/0012)
    Route {
        task: String,
        /// Write a route-decision ledger entry
        #[arg(long)]
        record: bool,
    },
    /// Trace lifecycle (spec 0014)
    Trace {
        #[command(subcommand)]
        action: TraceAction,
    },
    /// Run an eval spec (spec 0013)
    Eval {
        #[command(subcommand)]
        action: EvalAction,
    },
    /// Validate an eval spec (structure + frozen integrity)
    Validate {
        /// Path to the eval spec (defaults to the core fixture)
        spec: Option<String>,
    },
    /// Aggregate validate + eval + boundary (spec 0008)
    Doctor {
        /// Emit a stable JSON report
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EvalAction {
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

#[derive(Subcommand)]
enum TraceAction {
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
        Command::Route { task, record } => run_route(&root, &task, record),
        Command::Trace { action } => run_trace(action, &root),
        Command::Eval { action } => run_eval_cmd(action, &root),
        Command::Validate { spec } => run_validate(&root, spec),
        Command::Doctor { json } => run_doctor_cmd(&root, json),
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

fn run_route(root: &std::path::Path, task: &str, record: bool) -> Result<ExitCode> {
    use swe_seed_core::route;
    let result = route::build_route_result(root, task)?;
    let mut val = serde_json::to_value(&result)?;
    if record {
        let rd = route::write_route_decision(root, task, &result)?;
        val["route_decision_record"] = serde_json::Value::String(rd);
    }
    println!("{}", serde_json::to_string_pretty(&val)?);
    Ok(ExitCode::SUCCESS)
}

fn run_trace(action: TraceAction, root: &std::path::Path) -> Result<ExitCode> {
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

fn run_eval_cmd(action: EvalAction, root: &std::path::Path) -> Result<ExitCode> {
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

fn run_validate(root: &std::path::Path, spec: Option<String>) -> Result<ExitCode> {
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

fn run_doctor_cmd(root: &std::path::Path, json: bool) -> Result<ExitCode> {
    use swe_seed_core::doctor::{run_doctor, DoctorStatus};
    let report = run_doctor(root);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for c in &report.checks {
            println!("{:?}\t{}\t{}", c.status, c.name, c.detail);
        }
        println!("{:?}\toverall", report.overall);
    }
    Ok(if report.overall == DoctorStatus::Fail {
        // Plan 0008: exit non-zero iff any check fails. Warn is surfaced in the
        // report but does not fail the run.
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
