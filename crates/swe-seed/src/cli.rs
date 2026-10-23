//! `swe-seed` CLI command schema and dispatch. Command-family implementations
//! live in focused sibling modules so this file stays small enough to review.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::context_cli::run_context_plan;
use crate::doctor_cli::run_doctor_cmd;
use crate::eval_cli::{run_eval_cmd, run_validate, EvalAction};
use crate::fabricate_cli::{run_fabricate, FabricateAction};
use crate::federation_cli::{run_federation, run_run, FederationAction, FederationFlag};
use crate::gateway_cli::{run_gateway, GatewayAction};
use crate::hooks_cli::{run_agent_hooks, HooksAction};
use crate::host_cli::{self, HostSelection};
use crate::learning_cli::{run_adapt, run_learn, run_reflect, LearnAction};
use crate::provenance_cli::{run_provenance, ProvenanceAction};
use crate::seed_cli::{run_seed, SeedAction};
use crate::skill_cli::{run_render_skills, run_skill, SkillAction};
use crate::trace_cli::{run_trace, TraceAction};

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
    /// Harness structure validation (CI integrity; Rust port of harness validate)
    Harness,
    /// MCPGate runtime: capability/tool gateway plane (spec 0020)
    Gateway {
        #[command(subcommand)]
        action: GatewayAction,
    },
    /// Routing enforcement gate: exits 0 iff <trace> has a RouteSelected genesis
    /// (and the chain verifies, with --verify)
    Gate {
        trace: String,
        /// Also verify the full chain's tamper-evidence (recompute hashes)
        #[arg(long)]
        verify: bool,
    },
}

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
        Command::Harness => crate::harness_cli::run_harness_validate(&root),
        Command::Gateway { action } => run_gateway(&root, action),
        Command::Gate { trace, verify } => crate::gate_cli::run_gate(&root, &trace, verify),
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
