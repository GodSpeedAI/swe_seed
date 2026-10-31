# swe-seed CLI Source

## Overview

Source of the `swe-seed` binary crate (see `../Cargo.toml`, `[[bin]] name = "swe-seed"`).
This is the command-line surface of the SWE Seed governance CLI
("SWE Seed governance CLI (SweSeed → Harness → Fabricator)", `cli.rs:Cli`). It contains
only the clap command schema, argument enums, and thin dispatch handlers; all domain work
is delegated to the sibling `swe-seed-core` crate (`use swe_seed_core::...` in every
module). `main.rs` declares 16 private modules (`cli` plus one module per command family)
and delegates to `cli::run()`.

## Interface

- [main.rs:main] binary entry point; calls `cli::run()` and maps any error to exit code 2.
- [cli.rs:Cli] clap derive `Parser` root; `#[command(name = "swe-seed")]`, optional subcommand.
- [cli.rs:Command] private clap `Subcommand` enum with 23 top-level commands: `Seed`,
  `Provenance`, `Route`, `Trace`, `Eval`, `Validate`, `Doctor`, `Sync`, `Rollback`,
  `Hosts`, `ContextPlan`, `AgentHooks`, `RenderSkills`, `Skill`, `Reflect`, `Learn`,
  `Adapt`, `Fabricate`, `Run`, `Federation`, `Harness`, `Gateway`, `Gate`.
- [cli.rs:run] parses `Cli`, resolves the repo root once, and dispatches each command to
  its family handler; prints JSON to stdout.
- [cli.rs:discover_root] root resolution: `SWE_SEED_ROOT` env var, else nearest ancestor
  containing a `.agent-harness` directory, else nearest ancestor `Cargo.toml` containing
  `[workspace]`, else the current directory.
- [cli.rs:run_route] implements `swe-seed route <task>` via
  `swe_seed_core::route::build_route_result`; `--record` also appends a route-decision
  ledger entry via `route::write_route_decision`.
- [seed_cli.rs:SeedAction] / [seed_cli.rs:run_seed]: `seed assemble`,
  `seed validate-boundaries`, `seed regenerate`.
- [provenance_cli.rs:ProvenanceAction] / [provenance_cli.rs:run_provenance]:
  `provenance verify`, `provenance list`, `provenance show <id>`.
- [trace_cli.rs:TraceAction] / [trace_cli.rs:run_trace]: `trace start`, `trace append`,
  `trace checkpoint`, `trace resume`, `trace distill`, `trace finish --claim`.
- [eval_cli.rs:EvalAction] / [eval_cli.rs:run_eval_cmd]: `eval run --spec [--trusted]
[--output]`, `eval proof --spec --check --sha256`; [eval_cli.rs:run_validate]:
  `validate [spec]` (defaults to the core fixture, checks structure + frozen integrity).
- [doctor_cli.rs:run_doctor_cmd]: `doctor [--json] [--host <selection>]`; aggregates core
  checks with optional host projection drift reports.
- [host_cli.rs:HostSelection] value enum (`claude`, `codex`, `opencode`,
  `github-copilot`, `antigravity`, `ci`, `all`); [host_cli.rs:host_ids],
  [host_cli.rs:host_drift_reports]; [host_cli.rs:run_sync] (`sync --host [--dry-run]
[--prune]`), [host_cli.rs:run_rollback] (`rollback --host`), [host_cli.rs:run_hosts]
  (`hosts`: adapter capability matrices).
- [context_cli.rs:run_context_plan]: `context-plan <task>` via
  `swe_seed_core::context::context_plan`.
- [hooks_cli.rs:HooksAction] / [hooks_cli.rs:run_agent_hooks]: `agent-hooks capture`,
  `agent-hooks export` ([hooks_cli.rs:ExportFormat] `otel` / `junit`),
  `agent-hooks compact-logs`, `agent-hooks index`, `agent-hooks route-gate`.
- [skill_cli.rs:SkillAction] / [skill_cli.rs:run_skill]: `skill add`, `skill scan`,
  `skill approve <finding_id>`, `skill list`; [skill_cli.rs:run_render_skills]:
  `render-skills`.
- [learning_cli.rs:LearnAction] / [learning_cli.rs:run_learn]: `learn promote`;
  [learning_cli.rs:run_reflect]: `reflect <trace>`; [learning_cli.rs:run_adapt]:
  `adapt <run_id>`.
- [fabricate_cli.rs:FabricateAction] / [fabricate_cli.rs:run_fabricate]:
  `fabricate <need>` (bounded build → validate → render → proof-gated handoff) and
  `fabricate validate-chain <run>`.
- [federation_cli.rs:FederationAction] / [federation_cli.rs:run_federation]:
  `federation status`, `federation keygen`, `federation sign`, `federation verify`;
  [federation_cli.rs:FederationFlag] (`off` / `on`) and [federation_cli.rs:run_run]:
  `run [task] [--federation]`.
- [gateway_cli.rs:GatewayAction] / [gateway_cli.rs:run_gateway]: `gateway doctor`,
  `gateway list`, `gateway proof` (routing / concurrency / reload),
  `gateway import` (OpenAPI), `gateway serve`.
- [harness_cli.rs:run_harness_validate]: `harness` (CI structure validation via
  `swe_seed_core::harness_validate::validate`).
- [gate_cli.rs:run_gate]: `gate <trace> [--verify] [--allow-unanchored]` routing
  enforcement gate.
- Subdirectories: none (flat module directory; no child contracts).

## Invariants

- [main.rs:main] any error escaping `cli::run()` prints `error: {e:#}` to stderr and
  exits with code 2 (`ExitCode::from(2)`).
- [cli.rs:run] invoking the binary without a subcommand prints help and exits 0
  (`Cli::parse_from(["swe-seed", "--help"])`).
- [cli.rs:discover_root] root discovery is env-first (`SWE_SEED_ROOT`), then walks up for
  a `.agent-harness` directory or a workspace `Cargo.toml`; falls back to the current
  directory.
- [gate_cli.rs:run_gate] fails closed by default: an unanchored trace blocks unless
  `--allow-unanchored` is passed, and that flag requires `--verify`
  (`#[arg(long, requires = "verify")]` in `cli.rs:Command::Gate`).
- [doctor_cli.rs:run_doctor_cmd] exits non-zero iff any check fails; `Warn` is surfaced
  in the report but does not fail the run (Plan 0008 comment).
- [eval_cli.rs:run_eval_cmd] `eval run` exits 0 iff `EvalStatus::Pass`; spec-controlled
  shell execution (`command_check`) requires `--trusted` and is off by default.
  [eval_cli.rs:run_validate] fails when `check_frozen` rejects the spec.
- [learning_cli.rs:run_learn] validates the LearningRecord before any promotion write;
  SkillProposal promotion passes `promotion_gate` on the stored AdaptationDecision and
  [learning_cli.rs:validate_skill_candidate_ownership] requires shared evidence paths
  ([learning_cli.rs:has_shared_evidence]); RegressionCase registration requires
  `decision.regression_registration_allowed()` and `source_run_id == record id`;
  HarnessADR promotion exits 1 (unimplemented in v0.1).
- [provenance_cli.rs:ensure_safe_capability_id] `provenance show` rejects capability ids
  that are not a single path component (no `/`, `\`, `.`, `..`, empty).
- [federation_cli.rs:federation_config_from] federation defaults to off/standalone;
  `--federation on` only enables envelope emit/consume, planes stay local
  (`SettlementMode::Emit`).
- [federation_cli.rs:run_federation] `Sign` refuses to emit unless the trace chain
  verifies (`trace_ledger::verify_chain`) and the trace record carries a passing
  verification entry with no unresolved risks ([federation_cli.rs:verified_trace_proof],
  [federation_cli.rs:is_passing_result]).
- [gateway_cli.rs:ServeArgs] serve binds loopback by default; non-loopback binds require
  TLS/tunnel (doc comment on `bind`).
- [hooks_cli.rs:run_agent_hooks] `capture`: if the audit log append fails, the hook
  blocks (exit 1). `route-gate`: no active trace (`--trace-id` unset and
  `SWE_SEED_TRACE` unset) allows; a gate error blocks.

## Side Effects

- [cli.rs:run_route] `--record` appends a route-decision ledger entry
  (`swe_seed_core::route::write_route_decision`).
- [seed_cli.rs:run_seed] `assemble` writes the SeedPackageManifest to
  `seed::manifest::DEFAULT_MANIFEST_PATH` under the resolved root.
- [trace_cli.rs:run_trace] every action delegates to
  `swe_seed_core::trace::lifecycle` (`start` writes a route decision + trace record;
  `append` / `checkpoint` / `finish` mutate the trace record).
- [learning_cli.rs:run_reflect] writes `.agent-harness/learning/records/<id>.json`;
  [learning_cli.rs:run_learn] writes promoted candidates under
  `.agent-harness/learning/proposals/` or `.agent-harness/learning/regressions/`;
  [learning_cli.rs:run_adapt] reads `.agent-harness/learning/runs/<run_id>/*.json` and
  writes `.agent-harness/learning/decisions/adapt-<run_id>.json`.
- [eval_cli.rs:run_eval_cmd] `eval run` writes the result JSON to the `--output` path
  when given (creates parent dirs); `eval proof` is read-only (runs nothing, writes
  nothing).
- [host_cli.rs:run_sync] projects SWE_SEED policy/hooks into host runtime files via
  `swe_seed_core::adapters::sync_host` (honors `--dry-run` / `--prune`);
  [host_cli.rs:run_rollback] restores host runtime files from the last snapshot via
  `rollback_host`.
- [hooks_cli.rs:run_agent_hooks] loads `.agent-hooks/config.yaml`, appends redacted JSONL
  hook events under `hooks_cfg.paths.logs` (`hooks::append_event`), rebuilds a SQLite
  index at `hooks_cfg.paths.index_db` (`index`), gzips oversized logs (`compact-logs`),
  and writes export files when `--output` is given; `capture` reads the hook payload
  JSON from stdin.
- [skill_cli.rs:run_render_skills] renders SkillIR into host target files
  (`skill::render_all`); [skill_cli.rs:run_skill] `approve` persists scan waivers to the
  exceptions store (`exceptions_store_path`), `scan` runs the external SkillSpector gate
  (`run_skillspector`).
- [fabricate_cli.rs:run_fabricate] `fabricate <need>` writes a run under
  `.fabricator/runs/<run_id>/` including `handoff/MANIFEST.json`; handoff is blocked
  (exit 1) when chain integrity fails.
- [federation_cli.rs:run_federation] `keygen` writes an Ed25519 keypair (private key
  into gitignored `.swe-seed/federation/keys/`, public key into
  `.agent-harness/federation/keys/` for commit);
  `sign` opens the SQLite trace ledger (`trace_ledger::open(default_db_path)`) and
  prints a signed envelope to stdout; `verify` reads the envelope file and a public key
  file from disk.
- [gateway_cli.rs:run_serve] binds an HTTP JSON-RPC listener (`GatewayServer::bind`,
  `serve_once` / `serve_loop`) and prints the selected address as JSON before serving.
- [gateway_cli.rs:run_routing_proof] spawns a `sh -c` stdio responder subprocess
  (`StdioTransport::new("sh", ...)`).
- [gateway_cli.rs:run_concurrency_proof] creates a scratch journal directory under
  `std::env::temp_dir()` and removes it after the proof (`remove_dir_all`).
- [gateway_cli.rs:run_import] reads an OpenAPI JSON document from disk.

## Verification

- Run: `cargo test -p swe-seed`
