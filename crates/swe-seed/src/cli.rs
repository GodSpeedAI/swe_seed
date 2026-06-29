//! `swe-seed` CLI. Phase 0: `--help`. Phase 1: `seed {assemble,
//! validate-boundaries, regenerate}` + `provenance {verify, list, show}`.
//!
//! All repo-relative defaults resolve under a discovered project root
//! (env `SWE_SEED_ROOT`, else the workspace root by walking up from cwd),
//! so launching the binary from outside the repo still anchors paths correctly.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use crate::host_cli::{self, HostSelection};

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
        /// Include host projection drift checks for one host or all hosts
        #[arg(long, value_enum)]
        host: Option<HostSelection>,
    },
    /// Project SWE_SEED policy/hooks into host runtime files (spec 0004)
    Sync {
        #[arg(long, value_enum)]
        host: HostSelection,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        prune: bool,
    },
    /// Restore host runtime files from the last SWE_SEED snapshot
    Rollback {
        #[arg(long, value_enum)]
        host: HostSelection,
    },
    /// List supported host adapters and capability matrices
    Hosts,
    /// Plan context intake for a task (spec 0015)
    ContextPlan { task: String },
    /// Normalized hook runtime (spec 0005)
    AgentHooks {
        #[command(subcommand)]
        action: HooksAction,
    },
    /// Render SkillIR to host files (spec 0007)
    RenderSkills,
    /// Skill ingestion + scan gate (spec 0007)
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Reflect a finished trace into a proposed LearningRecord (spec 0016)
    Reflect { trace: String },
    /// Promote a learning record into a SkillProposal / RegressionCase (spec 0016)
    Learn {
        #[command(subcommand)]
        action: LearnAction,
    },
    /// Build an AdaptationDecision from a run's eval results (spec 0016)
    Adapt { run_id: String },
    /// Bounded product→prototype fabricate run (spec 0017)
    Fabricate {
        /// Product need (positional; required unless using a subcommand)
        need: Option<String>,
        #[command(subcommand)]
        action: Option<FabricateAction>,
    },
    /// Bounded inner-stack run; `--federation off` is fully standalone (spec 0011)
    Run {
        task: Option<String>,
        /// Federation master switch override (`off` | `on`); default `off`
        #[arg(long, value_enum)]
        federation: Option<FederationFlag>,
    },
    /// Federation flags + envelope I/O (spec 0011)
    Federation {
        #[command(subcommand)]
        action: FederationAction,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// Add (validate/normalize) a skill by path
    Add { path: String },
    /// Scan a skill with the external SkillSpector gate
    Scan { id: String },
    /// Approve (waive) a scan finding so it no longer blocks
    Approve { finding_id: String },
    /// List discovered skills with scan status
    List,
}

#[derive(Subcommand)]
enum LearnAction {
    /// Promote a LearningRecord into a SkillProposal or RegressionCase
    Promote {
        record: String,
        /// Path to a candidate SkillProposal JSON (required for SkillProposal dispositions)
        #[arg(long)]
        proposal: Option<String>,
        /// Path to a candidate RegressionCase JSON (required for RegressionCase dispositions)
        #[arg(long)]
        regression: Option<String>,
    },
}

#[derive(Subcommand)]
enum FabricateAction {
    /// Validate a rendered chain's integrity → SemanticChainValidationReport
    ValidateChain { run: String },
}

#[derive(Subcommand)]
enum FederationAction {
    /// Show resolved federation flags + domain_model_hash
    Status,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FederationFlag {
    Off,
    On,
}

#[derive(Subcommand)]
enum HooksAction {
    /// Append a hook event to the JSONL log (payload from stdin)
    Capture {
        event: String,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        trace_id: Option<String>,
        #[arg(long)]
        hook_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Export the event log
    Export {
        #[command(subcommand)]
        format: ExportFormat,
    },
    /// Compact oversized logs (gzip)
    CompactLogs,
    /// Rebuild the SQLite index from the JSONL logs
    Index,
}

#[derive(Subcommand)]
enum ExportFormat {
    /// OpenTelemetry-compatible JSON
    Otel {
        #[arg(long)]
        output: Option<String>,
    },
    /// JUnit XML
    Junit {
        #[arg(long)]
        output: Option<String>,
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
        Command::Doctor { json, host } => run_doctor_cmd(&root, json, host),
        Command::Sync {
            host,
            dry_run,
            prune,
        } => host_cli::run_sync(&root, host, dry_run, prune),
        Command::Rollback { host } => host_cli::run_rollback(&root, host),
        Command::Hosts => host_cli::run_hosts(),
        Command::ContextPlan { task } => run_context_plan(&root, &task),
        Command::AgentHooks { action } => run_agent_hooks(action, &root),
        Command::RenderSkills => run_render_skills(&root),
        Command::Skill { action } => run_skill(action, &root),
        Command::Reflect { trace } => run_reflect(&root, &trace),
        Command::Learn { action } => run_learn(action, &root),
        Command::Adapt { run_id } => run_adapt(&root, &run_id),
        Command::Fabricate { need, action } => run_fabricate(&root, need, action),
        Command::Run { task, federation } => run_run(&root, task, federation),
        Command::Federation { action } => run_federation(&root, action),
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
                    ".agents/plans/0001-swe-seed-v0-1-implementation.md".into()
                ],
                approved_seed_package_manifests: vec![seed::manifest::DEFAULT_MANIFEST_PATH.into()],
                approved_layer_capability_maps: vec![
                    ".agents/specs/0003-capability-registry.md".into()
                ],
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

fn run_doctor_cmd(
    root: &std::path::Path,
    json: bool,
    host: Option<HostSelection>,
) -> Result<ExitCode> {
    use swe_seed_core::adapters::DriftStatus;
    use swe_seed_core::doctor::{run_doctor, DoctorStatus};
    let report = run_doctor(root);
    let mut host_reports = Vec::new();
    let mut host_failed = false;
    if let Some(selection) = host {
        for drift in host_cli::host_drift_reports(root, selection)? {
            host_failed |= drift.status == DriftStatus::Drifted;
            host_reports.push(drift);
        }
    }
    let combined_overall = if report.overall == DoctorStatus::Fail || host_failed {
        DoctorStatus::Fail
    } else {
        report.overall.clone()
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "core": report,
                "hosts": host_reports,
                "overall": combined_overall,
            }))?
        );
    } else {
        for c in &report.checks {
            println!("{:?}\t{}\t{}", c.status, c.name, c.detail);
        }
        for host_report in &host_reports {
            println!(
                "{:?}\thost:{}\t{}",
                host_report.status,
                host_report.host_id,
                if host_report.drifted_files.is_empty() {
                    "clean".to_string()
                } else {
                    host_report.drifted_files.join(",")
                }
            );
        }
        println!("{:?}\toverall", combined_overall);
    }
    Ok(if combined_overall == DoctorStatus::Fail {
        // Plan 0008: exit non-zero iff any check fails. Warn is surfaced in the
        // report but does not fail the run.
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_context_plan(root: &std::path::Path, task: &str) -> Result<ExitCode> {
    let plan = swe_seed_core::context::context_plan(root, task)?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(ExitCode::SUCCESS)
}

fn run_agent_hooks(action: HooksAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::config::{load_yaml, HooksConfig};
    use swe_seed_core::hooks;

    let cfg_path = root.join(".agent-hooks/config.yaml");
    let hooks_cfg: HooksConfig = load_yaml(&cfg_path)?;
    let log_dir = root.join(&hooks_cfg.paths.logs);
    let index_db = root.join(&hooks_cfg.paths.index_db);
    let redact_cfg = hooks::RedactionConfig {
        key_substrings: hooks_cfg.redaction.key_substrings.clone(),
        value_patterns: hooks_cfg.redaction.value_patterns.clone(),
    };

    match action {
        HooksAction::Capture {
            event,
            session_id,
            trace_id,
            hook_id,
            status,
        } => {
            let mut payload: serde_json::Value = serde_json::Value::Object(Default::default());
            if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                let mut buf = String::new();
                if std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).is_ok()
                    && !buf.trim().is_empty()
                {
                    payload = serde_json::from_str(&buf).unwrap_or(payload);
                }
            }
            let mut envelope = serde_json::json!({
                "event": event,
                "hook_id": hook_id.unwrap_or_else(|| event.clone()),
                "status": status.unwrap_or_else(|| "ok".into()),
                "trace_id": trace_id,
                "session_id": session_id,
                "attributes": payload,
            });
            let (log, event_id) = hooks::append_event(&log_dir, &mut envelope, &redact_cfg)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "event_id": event_id,
                    "log": log.strip_prefix(root).unwrap_or(&log).display().to_string(),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        HooksAction::Export { format } => match format {
            ExportFormat::Otel { output } => {
                let payload = hooks::export::export_otel(&log_dir, root)?;
                emit_export(&serde_json::to_string_pretty(&payload)?, root, output)
            }
            ExportFormat::Junit { output } => {
                let payload = hooks::export::export_junit(&log_dir)?;
                emit_export(&payload, root, output)
            }
        },
        HooksAction::CompactLogs => {
            let min = hooks_cfg.logging.compact_min_size_bytes;
            let compacted = hooks::compact_logs(&log_dir, min)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "compacted": compacted.iter().map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string()).collect::<Vec<_>>(),
                    "count": compacted.len(),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        HooksAction::Index => {
            let n = hooks::index::rebuild_index(&index_db, &log_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "index_db": index_db.strip_prefix(root).unwrap_or(&index_db).display().to_string(),
                    "indexed_events": n,
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn emit_export(content: &str, root: &std::path::Path, output: Option<String>) -> Result<ExitCode> {
    if let Some(out) = output {
        let p = root.join(&out);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&p, content)?;
    }
    print!("{content}");
    Ok(ExitCode::SUCCESS)
}

fn run_render_skills(root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::security::{exceptions_store_path, Exceptions};
    use swe_seed_core::skill::render_all;
    let exceptions = Exceptions::load(&exceptions_store_path(root));
    let (rendered, blocked) = render_all(root, root, &exceptions)?;
    for b in &blocked {
        eprintln!("blocked (scan): {b}");
    }
    println!(
        "rendered {rendered} target file(s); {} blocked",
        blocked.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn run_skill(action: SkillAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::security::{exceptions_store_path, run_skillspector, Exceptions};
    use swe_seed_core::skill::{discover, fetch, ingest_one, normalize, SkillRecord};
    match action {
        SkillAction::Add { path } => {
            let p = root.join(&path);
            let rec = ingest_one(&p)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": rec.ir.id,
                    "version": rec.ir.version,
                    "status": format!("{:?}", rec.ir.status),
                    "source_hash": rec.source_hash,
                    "scan_status": format!("{:?}", rec.scan.status),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::Scan { id } => {
            // Match by SkillIR.id, not the filename stem.
            let mut found = None;
            for p in discover(root)? {
                let raw = fetch(&p)?;
                let ir = normalize(&raw)?;
                if ir.id == id {
                    found = Some(p);
                    break;
                }
            }
            let Some(path) = found else {
                eprintln!("skill not found: {id}");
                return Ok(ExitCode::from(1));
            };
            let scan = run_skillspector(&path);
            println!("{}", serde_json::to_string_pretty(&scan)?);
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::Approve { finding_id } => {
            // Persist the waiver so it no longer blocks projection/render.
            let store = exceptions_store_path(root);
            let mut exceptions = Exceptions::load(&store);
            exceptions.waive(&finding_id);
            exceptions.save(&store)?;
            println!(
                "approved (waived) finding '{}' -> {}",
                finding_id,
                store.strip_prefix(root).unwrap_or(&store).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::List => {
            let mut rows = Vec::new();
            for p in discover(root)? {
                let rec: SkillRecord = match ingest_one(&p) {
                    Ok(r) => r,
                    Err(e) => {
                        rows.push(format!("?\t{}\tINVALID: {e}", p.display()));
                        continue;
                    }
                };
                rows.push(format!(
                    "{:?}\t{}\t{}",
                    rec.scan.status, rec.ir.id, rec.source_hash
                ));
            }
            println!("scan\tid\tsource_hash");
            for r in rows {
                println!("{r}");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// `.agent-harness/learning` — durable store for records/proposals/regressions
/// and per-run eval results consumed by `adapt`.
fn learning_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".agent-harness").join("learning")
}

fn run_reflect(root: &std::path::Path, trace: &str) -> Result<ExitCode> {
    use swe_seed_core::learning::{default_template, reflect};
    use swe_seed_core::trace::lifecycle::resolve_trace_path;
    use swe_seed_core::trace::TraceRecord;

    let trace_path = resolve_trace_path(root, trace);
    let record = TraceRecord::load(&trace_path)?;
    let learned = reflect(
        &default_template(),
        &record,
        &trace_path.display().to_string(),
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    let out_dir = learning_dir(root).join("records");
    std::fs::create_dir_all(&out_dir)?;
    let out = out_dir.join(format!("{}.json", learned.id));
    std::fs::write(
        &out,
        format!("{}\n", serde_json::to_string_pretty(&learned)?),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "learning_record": learned.id,
            "disposition": format!("{:?}", learned.disposition),
            "path": out.strip_prefix(root).unwrap_or(&out).display().to_string(),
        }))?
    );
    Ok(ExitCode::SUCCESS)
}

fn run_learn(action: LearnAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::learning::{
        validate_learning_record, validate_proposal, validate_regression, LearningDisposition,
        LearningRecord, RegressionCase, SkillProposal,
    };
    let LearnAction::Promote {
        record,
        proposal,
        regression,
    } = action;
    let record_path = root.join(&record);
    let loaded: LearningRecord = serde_json::from_slice(&std::fs::read(&record_path)?)
        .map_err(|e| anyhow::anyhow!("parse {}: {e}", record_path.display()))?;

    // Validate the record itself before any candidate is considered: a corrupt
    // or evidence-less record must not unlock a promotion write.
    validate_learning_record(&loaded)
        .map_err(|e| anyhow::anyhow!("learning record {} failed validation: {e}", loaded.id))?;

    match loaded.disposition {
        LearningDisposition::ApprovedLesson => {
            // A published lesson needs no candidate artifact beyond the record.
            println!("published lesson '{}' ({})", loaded.id, loaded.summary);
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::NoChange | LearningDisposition::RejectedLesson => {
            println!(
                "nothing to promote: disposition is {:?}",
                loaded.disposition
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::SkillProposal => {
            let Some(p) = proposal else {
                eprintln!("SkillProposal disposition requires --proposal <path>");
                return Ok(ExitCode::from(1));
            };
            let proposal_path = root.join(&p);
            let cand: SkillProposal = serde_json::from_slice(&std::fs::read(&proposal_path)?)
                .map_err(|e| anyhow::anyhow!("parse {}: {e}", proposal_path.display()))?;
            validate_proposal(&cand).map_err(|e| anyhow::anyhow!(e))?;
            let out_dir = learning_dir(root).join("proposals");
            std::fs::create_dir_all(&out_dir)?;
            let out = out_dir.join(format!("{}.json", cand.id));
            std::fs::write(&out, format!("{}\n", serde_json::to_string_pretty(&cand)?))?;
            println!(
                "promoted SkillProposal '{}' -> {}",
                cand.id,
                out.strip_prefix(root).unwrap_or(&out).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::RegressionCase => {
            let Some(r) = regression else {
                eprintln!("RegressionCase disposition requires --regression <path>");
                return Ok(ExitCode::from(1));
            };
            let reg_path = root.join(&r);
            let cand: RegressionCase = serde_json::from_slice(&std::fs::read(&reg_path)?)
                .map_err(|e| anyhow::anyhow!("parse {}: {e}", reg_path.display()))?;
            validate_regression(&cand).map_err(|e| anyhow::anyhow!(e))?;
            let out_dir = learning_dir(root).join("regressions");
            std::fs::create_dir_all(&out_dir)?;
            let out = out_dir.join(format!("{}.json", cand.id));
            std::fs::write(&out, format!("{}\n", serde_json::to_string_pretty(&cand)?))?;
            println!(
                "promoted RegressionCase '{}' (linked_eval_check={}) -> {}",
                cand.id,
                cand.linked_eval_check,
                out.strip_prefix(root).unwrap_or(&out).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        LearningDisposition::HarnessADR => {
            // HarnessADR promotion is a doctrine write, out of scope for v0.1's
            // learning CLI (no doctrine store yet). Surface honestly.
            eprintln!("HarnessADR promotion is not implemented in v0.1; record only");
            Ok(ExitCode::from(1))
        }
    }
}

fn run_adapt(root: &std::path::Path, run_id: &str) -> Result<ExitCode> {
    use swe_seed_core::eval::EvalResult;
    use swe_seed_core::learning::build_adaptation_decision;

    // Collect this run's eval results (written by `eval run --output` or by the
    // caller under `.agent-harness/learning/runs/<run-id>/`). Missing results →
    // every class is Inconclusive and the decision fails closed (blocks).
    let run_dir = learning_dir(root).join("runs").join(run_id);
    let mut results: Vec<EvalResult> = Vec::new();
    if run_dir.is_dir() {
        for entry in std::fs::read_dir(&run_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            match serde_json::from_slice::<EvalResult>(&std::fs::read(&path)?) {
                Ok(r) => results.push(r),
                Err(e) => {
                    eprintln!("adapt: skipping unparseable {}: {e}", path.display());
                }
            }
        }
    }

    let decision = build_adaptation_decision(run_id, &results);
    let out_dir = learning_dir(root).join("decisions");
    std::fs::create_dir_all(&out_dir)?;
    let out = out_dir.join(format!("{}.json", decision.decision_id));
    std::fs::write(
        &out,
        format!("{}\n", serde_json::to_string_pretty(&decision)?),
    )?;
    println!("{}", serde_json::to_string_pretty(&decision)?);
    Ok(ExitCode::SUCCESS)
}

fn run_fabricate(
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

/// Resolve the federation config from the `--federation` flag (off=standalone,
/// on=emit+consume enabled). Absent flag = standalone default.
fn federation_config_from(
    flag: Option<FederationFlag>,
) -> swe_seed_core::federation::FederationConfig {
    use swe_seed_core::federation::{FederationConfig, SettlementMode};
    let mut cfg = FederationConfig::default();
    match flag {
        Some(FederationFlag::On) => {
            cfg.enabled = true;
            cfg.emit_envelope = true;
            cfg.consume_envelope = true;
            // Planes stay local/off by default; `on` only enables envelope I/O.
            cfg.settlement.mode = SettlementMode::Emit;
        }
        Some(FederationFlag::Off) | None => {}
    }
    cfg
}

fn run_run(
    root: &std::path::Path,
    task: Option<String>,
    federation: Option<FederationFlag>,
) -> Result<ExitCode> {
    use swe_seed_core::federation::{
        dispatch, emit_work_requested, resolve_domain_model_hash, Dispatch,
    };

    let cfg = federation_config_from(federation);
    let resolved = resolve_domain_model_hash();
    let task = task.as_deref().unwrap_or("inner-stack smoke");

    // Build a WorkRequested envelope in memory (never dispatched when off).
    let envelope = emit_work_requested(
        &resolved.hash,
        "run-smoke",
        "swe-seed",
        "run",
        task,
        "low",
        None,
    );
    let dispatched = dispatch(&cfg, &envelope, None);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "task": task,
            "root": root.display().to_string(),
            "federation": {
                "enabled": cfg.enabled,
                "emits": cfg.emits(),
                "consumes": cfg.consumes(),
                "standalone": cfg.is_standalone(),
            },
            "domain_model_hash": resolved.hash,
            "hash_source": format!("{:?}", resolved.source),
            "hash_warned": resolved.warned,
            "dispatched": match dispatched { Dispatch::Written(_) => "written", Dispatch::Suppressed => "suppressed" },
        }))?
    );
    Ok(ExitCode::SUCCESS)
}

fn run_federation(root: &std::path::Path, action: FederationAction) -> Result<ExitCode> {
    use swe_seed_core::federation::{fallback_hash, resolve_domain_model_hash};
    let FederationAction::Status = action;
    let resolved = resolve_domain_model_hash();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "config_root": root.join(".swe-seed").display().to_string(),
            "domain_model_hash": resolved.hash,
            "fallback_hash": fallback_hash(),
            "hash_source": format!("{:?}", resolved.source),
            "hash_warned": resolved.warned,
            "namespace": swe_seed_core::federation::NAMESPACE,
        }))?
    );
    Ok(ExitCode::SUCCESS)
}
