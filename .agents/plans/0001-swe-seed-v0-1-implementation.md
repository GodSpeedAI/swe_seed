# SWE_Seed v0.1 — Rust Rewrite Implementation Plan

**Status:** Approved
**Evidence basis:** Specs `.agents/specs/0002`–`0019` (0012 reconciliation is authoritative); first-party Python harness (`scripts/`, `.strategy/`, `agentic_capability_loop/`) + `.baml` contracts as the proven behavior/schema to reproduce.
**Proof level target:** local-confidence → live-dev-proof per phase; **golden-file parity** against current Python output where formats are frozen; full end-to-end live-dev-proof at Phase 10.

> **Scope:** full Rust rewrite of all three layers (SweSeed → Harness → Fabricator),
> build-to-specs, superseding the Python. `.baml` is canonical schema, **contracts-as-data,
> no LLM runtime** (0019).
---

## Design Decisions Applied

| Decision | Resolution |
|---|---|
| Architecture | 3-layer: `SweSeed → Harness → Fabricator` (`LayerName`, spec 0012/0018). |
| Vocabulary | Proven `.baml`/CLI terms are authoritative: RouteCard, SkillIR, EvalSpec/ProofRecord, ContextBudget/Pack, HookPolicy/PermissionPolicy, TraceSchema, LearningRecord, LayerCapability, SeedPackageManifest. |
| LLM/BAML | Contracts-as-data; no LLM client. Hand-written Rust structs + `baml_parity` CI test (0019). |
| v0.1 scope | All three layers, full parity with current Python (maintainer decision). |
| Packaging | Cargo **workspace**: one crate per layer module group + a `swe-seed` binary crate. |
| YAML | `serde_yaml` + parity test against existing config files (replaces hand-rolled parser). |
| Determinism / supersession | Golden-file parity vs captured current-Python output for every superseded CLI; Python removed at Phase 10. |
| Deps | `serde, serde_json, serde_yaml, toml, clap, sha2, uuid, chrono, rusqlite, flate2, regex, anyhow, thiserror`. No reference repo, no `baml-py`. |

---

## Dependency Order

```
Phase 0: Scaffold + BAML parity      ← schema is the foundation; every type checks against .baml (0019)
Phase 1: SweSeed layer + provenance  ← outer governance + registry; everything is a LayerCapability (0018/0003/0009)
Phase 2: Routing + Traces            ← RouteCard drives work; traces give continuity (0004/0014)
Phase 3: Eval + Proof + Doctor       ← proof gate verifies everything Phases 1–2 produce (0013/0008)
Phase 4: Context plane + Hook runtime← budgets feed routing; hooks/permissions gate actions (0015/0005)
Phase 5: Skill ingestion + render    ← SkillIR + scan gate; render-skills projects (0007)
Phase 6: Learning + adaptation loop  ← consumes traces+evals to propose skills/regressions (0016)
Phase 7: Fabricator layer            ← product→prototype semantic chain, proof-gated (0017)
Phase 8: Federation boundary         ← OPTIONAL SEA envelope; inner stack already stands alone (0011)
Phase 9: Host adapters               ← project the assembled manifest into hosts (0004)
Phase 10: E2E parity + Python removal ← golden-file parity across all CLIs; delete Python
```

Each phase is self-contained once its prerequisites are met. Phases 1–7 are the sovereign
inner stack and must pass with **zero federation code present** (the standalone invariant,
0011). Every phase that supersedes a Python CLI carries a golden-file parity test.

---

## Phase 0 — Scaffold + BAML Contracts-as-Data

**Repo:** `SWE_SEED` (`/home/sprime01/projects/SWE_SEED`)
**Prerequisite:** None.

### Outcome (falsifiable)

1. Cargo workspace builds; `swe-seed --help` runs.
2. `swe_seed::contracts` parses every `class`/`enum` in `.baml` into `BamlType`.
3. `cargo test baml_parity` passes: each registered Rust struct/enum matches its `.baml`
   counterpart (field + variant names).
4. `serde_yaml` parses every existing config (`budget-policy.yaml`, `.agent-hooks/config.yaml`,
   `.fabricator/config.yaml`, `.strategy/config.yaml`) into typed structs (`tests/yaml_parity.rs`).

**Falsification:** Rename a field in a Rust struct without updating `.baml` → `baml_parity`
fails. Feed a real config file → if `serde_yaml` can't parse it, `yaml_parity` fails.

### Files to Modify

#### [NEW] `Cargo.toml` (workspace) + `crates/swe-seed/` (bin) + `crates/swe-seed-core/` (lib)

Workspace with a binary crate and a core lib; deps pinned per Design Decisions.

#### [NEW] `crates/swe-seed-core/src/contracts/{baml_parse.rs,parity.rs}`

Minimal `.baml` reader + parity assertion (0019).

**Note:** No domain logic yet — this phase only proves the schema spine and config parsing.

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo build
cargo test baml_parity yaml_parity
cargo run -p swe-seed -- --help
```

**Expected:** Build + both tests pass; help prints.

### Done Conditions (Phase 0)

- [ ] Workspace builds; binary runs.
- [ ] `baml_parity` green for all registered types.
- [ ] All existing YAML configs parse into typed structs.

---

## Phase 1 — SweSeed Layer + Provenance

**Repo:** `SWE_SEED`
**Prerequisite:** Phase 0 (types validated against `.baml`).

### Outcome (falsifiable)

1. `LayerCapability`, `ProjectSeed`, `SeedPackageManifest`, `BoundaryReport`, `ArtifactMetadata`
   implemented per 0018; `swe-seed seed assemble` produces a manifest.
2. `swe-seed seed validate-boundaries` returns `BoundaryReport.passed=false` on a misplaced/
   upward-referencing artifact (`tests/boundary.rs`).
3. `swe-seed seed regenerate` is idempotent: unchanged inputs → empty diff; preserves approved
   decisions (`tests/regenerate_idempotent.rs`).
4. `swe-seed provenance verify` fails on missing hash/license or `code_copied=true` (0009).

**Falsification:** Mark a Harness artifact `owner=Fabricator` → boundary fails; if it passes,
outcome 2 false. Run `regenerate` twice unchanged → non-empty diff fails outcome 3.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/seed/{project_seed.rs,capability.rs,manifest.rs,boundary.rs,regenerate.rs}`

#### [NEW] `crates/swe-seed-core/src/provenance/{record.rs,hash.rs,verify.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `seed {assemble,validate-boundaries,regenerate}`, `provenance {verify,list,show}`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test boundary --test regenerate_idempotent
cargo run -p swe-seed -- seed assemble
cargo run -p swe-seed -- seed validate-boundaries; echo "exit=$?"
cargo run -p swe-seed -- provenance verify; echo "exit=$?"
```

**Expected:** Tests pass; assemble writes a manifest; boundary + provenance fail closed on bad fixtures.

### Done Conditions (Phase 1)

- [ ] 3-layer ownership enforced; manifest assembled.
- [ ] Boundary validation + idempotent regeneration work.
- [ ] Provenance fails closed.

---

## Phase 2 — Routing + Traces

**Repo:** `SWE_SEED`
**Prerequisite:** Phase 1.

### Outcome (falsifiable)

1. `RouteCard` (0004/0012) + `swe-seed route <task> [--record]` selects a card for the routed
   job type and writes a route-decision record matching the frozen JSON format
   (`tests/route_golden.rs` vs a captured Python record).
2. Trace lifecycle `start/append/checkpoint/resume/distill/finish` works; `finish` requires
   `--claim` (`tests/trace_lifecycle.rs`).
3. `TraceSchema` validation rejects records missing identifier fields.
4. All 11 required job types (AGENTS.md) have a resolvable RouteCard.

**Falsification:** `route --record` output that differs byte-wise from the golden record →
`route_golden` fails. `trace finish` without `--claim` must error.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/route/mod.rs`, `src/trace/{schema.rs,record.rs,lifecycle.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `route`, `trace {...}`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test route_golden --test trace_lifecycle
cargo run -p swe-seed -- route "checkpoint smoke" --record
cargo run -p swe-seed -- trace start "x" && cargo run -p swe-seed -- trace finish <id> --claim "done"
```

**Expected:** Tests pass; route record matches golden; finish requires claim.

### Done Conditions (Phase 2)

- [ ] RouteCard routing + frozen-format route records.
- [ ] Full trace lifecycle; TraceSchema validation.
- [ ] 11 job-type cards resolve.

---

## Phase 3 — Eval + Proof + Doctor

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 1–2 (doctor verifies seed + routes; proof references traces).

### Outcome (falsifiable)

1. `EvalSpec`/`EvalCheck`/`EvalResult` + `swe-seed eval run --spec` produce deterministic
   results across the 4 eval classes (0013).
2. `frozen_after_handoff` enforced: editing a handed-off spec → validation fail.
3. `ProofRecord` rejects claims without evidence; `Waived` requires a reason.
4. `swe-seed doctor` = `validate` + `eval run` + `validate-boundaries`; exit non-zero iff any
   `Fail` (0008); `--json` schema stable.
5. Live-pass-only promotion wired (a `Waived`/simulation never activates a capability).

**Falsification:** Edit a frozen spec post-handoff → must fail. A claim with no evidence → ProofRecord rejected. Doctor with a failing required eval exits 0 → bug.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/eval/{spec.rs,check.rs,result.rs,proof.rs}`, `src/doctor/{check.rs,drift.rs,report.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `eval run`, `validate`, `doctor`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test eval_frozen --test proof_evidence --test doctor_json
cargo run -p swe-seed -- eval run --spec tests/fixtures/eval.toml
cargo run -p swe-seed -- doctor; echo "exit=$?"
```

**Expected:** Tests pass; doctor fails closed on a non-compliant fixture.

### Done Conditions (Phase 3)

- [ ] Eval (4 classes) deterministic; frozen-after-handoff enforced.
- [ ] ProofRecord requires evidence; Waived requires reason.
- [ ] Doctor aggregates validate+eval+boundary; live-pass-only promotion.

---

## Phase 4 — Context Plane + Hook Runtime

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 1–3 (context ties to routes; hooks gate actions, log to runtime).

### Outcome (falsifiable)

1. `ContextBudget`/`ContextPack` + `swe-seed context-plan <task>` respects `budget-policy.yaml`
   (excludes excluded, caps raw output, warns stale) — 0015.
2. Hook runtime ports `agent_hooks`: JSONL event log + SQLite index + redaction + OTEL/JUnit
   export + compaction (`tests/hooks_runtime.rs`); redaction removes secret patterns.
3. `HookPolicy`/`PermissionPolicy` gate: a forbidden action is blocked; an approval-gated
   action requires approval (`tests/permission_gate.rs`).
4. The 5 required lifecycle events (0005) project to host hook config (Phase 9 consumes).

**Falsification:** A secret-pattern value survives in a written log → redaction test fails. A
forbidden action executes → permission gate false.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/context/{budget.rs,pack.rs,policy.rs}`, `src/hooks/{runtime.rs,policy.rs,redact.rs,export.rs,index.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `context-plan`, `agent-hooks {...}`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test hooks_runtime --test permission_gate --test context_plan
cargo run -p swe-seed -- context-plan "checkpoint smoke"
```

**Expected:** Tests pass; redaction + permission gates hold; context-plan honors the budget.

### Done Conditions (Phase 4)

- [ ] Context budget/pack + policy enforced.
- [ ] Hook runtime (log/index/redaction/export/compaction) ported.
- [ ] HookPolicy/PermissionPolicy gate actions.

---

## Phase 5 — Skill Ingestion + Render

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 1–3.

### Outcome (falsifiable)

1. Ingestion `discover→fetch→scan→normalize→project→verify` yields `SkillIR`; a blocking scan
   prevents projection (0007).
2. `swe-seed render-skills` renders `SkillIR` to `render_targets` (host files) deterministically
   (`tests/render_skills_golden.rs`).
3. SkillSpector is an external `EvalCheck`/gate returning `Pending` until verified — never a faked pass.
4. Activation requires `source_hash` + terminal scan status.

**Falsification:** A critical-finding skill gets rendered → outcome 1 false. render-skills output differs from golden → fails.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/skill/{ir.rs,ingest.rs,render.rs}`, `src/security/{gate.rs,scan_result.rs,skillspector.rs,exceptions.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `render-skills`, `skill {add,scan,approve,list}`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test ingest_pipeline --test render_skills_golden --test scan_gate
cargo run -p swe-seed -- render-skills
```

**Expected:** Tests pass; blocked skills not rendered; render matches golden.

### Done Conditions (Phase 5)

- [ ] Pipeline ordered; blocking scan prevents render.
- [ ] SkillIR → host render deterministic.
- [ ] SkillSpector external `Pending` gate; activation gated.

---

## Phase 6 — Learning + Adaptation Loop

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 2–3 (consumes traces + eval results).

### Outcome (falsifiable)

1. `reflect → LearningRecord → AdaptationDecision → SkillProposal|RegressionCase` loop (0016).
2. A `SkillProposal` without a rollback plan is rejected.
3. A blocked `AdaptationEligibility` eval blocks promotion (`tests/adaptation_gate.rs`).
4. A promoted `RegressionCase` links an `EvalCheck` that fails on recurrence (`tests/regression_link.rs`).

**Falsification:** Proposal with no rollback accepted → outcome 2 false. Recurrence not caught by the linked regression check → outcome 4 false.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/learning/{reflection.rs,record.rs,candidate.rs,adaptation.rs,proposal.rs,regression.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `reflect`, `learn promote`, `adapt`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test adaptation_gate --test regression_link
cargo run -p swe-seed -- reflect <trace> && cargo run -p swe-seed -- adapt <run-id>
```

**Expected:** Tests pass; gates + rollback enforced; regressions become enforceable checks.

### Done Conditions (Phase 6)

- [ ] Full learning loop; adaptation proof-gated.
- [ ] Proposals require rollback; regressions link eval checks.

---

## Phase 7 — Fabricator Layer

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 3, 6 (reuses eval + learning).

### Outcome (falsifiable)

1. Semantic chain `ProductSeed→…→AgentTask→FabricatorEvalSpec→ProofRecord` rendered from
   `.fabricator/templates` (0017).
2. `ValidateSemanticChain` → `SemanticChainValidationReport.passed=false` on a broken
   traceability link → handoff blocked (`tests/chain_integrity.rs`).
3. EARS/Gherkin shape validation rejects malformed requirements/scenarios.
4. `swe-seed fabricate` produces a complete, traceable chain matching golden output.

**Falsification:** Drop a PRD→SDS link → chain report must fail; if it passes, outcome 2 false.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/fabricator/{chain.rs,artifacts.rs,validate.rs,render.rs}`

#### [MODIFY] `crates/swe-seed/src/cli.rs` — add `fabricate [validate-chain]`

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test chain_integrity --test fabricate_golden
cargo run -p swe-seed -- fabricate "sample need"
```

**Expected:** Tests pass; broken chain blocks handoff; fabricate matches golden.

### Done Conditions (Phase 7)

- [ ] Semantic chain + integrity gate.
- [ ] EARS/Gherkin validation; proof-gated frozen eval at handoff.

---

## Phase 8 — Federation Boundary (SEA Envelope Port)

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 1–7 (inner stack must pass standalone first).

### Outcome (falsifiable)

1. `swe_seed::federation` ports `adapters.py`: `Envelope`, `domain_model_hash` resolution,
   emit/consume the SEA events (0011); `tests/federation_parity.rs` vs captured Python output.
2. **Standalone invariant**: `federation.enabled=false` + SEA unreachable → full inner stack
   passes, **zero external calls** (`tests/standalone_invariant.rs`).
3. Authority/context/settlement delegation behind default-off flags; `local` ignores envelopes.
4. Python `agentic_capability_loop/` superseded; contract tests ported.

**Falsification:** With federation off + dead `SEA_ROOT`, any external call attempted → outcome 2 false. Payload key drift → parity fails.

### Files to Modify

#### [NEW] `crates/swe-seed-core/src/federation/{envelope.rs,emit.rs,consume.rs,flags.rs}`

#### [MODIFY] hook/permission/doctor call sites — flag-guarded delegation

#### [REMOVE] `agentic_capability_loop/` (Python) — superseded

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --test federation_parity --test standalone_invariant
SEA_ROOT=/nonexistent cargo run -p swe-seed -- run --federation off
```

**Expected:** Tests pass; standalone run makes no external calls.

### Done Conditions (Phase 8)

- [ ] Envelope parity; standalone invariant proven.
- [ ] Delegation behind default-off flags; Python loop superseded.

---

## Phase 9 — Agent Runtime Host Adapters

**Repo:** `SWE_SEED`
**Prerequisite:** Phases 1–5 project the assembled manifest, skills, routes, and canonical hook intent.

### Outcome (falsifiable)

1. `HostAdapter` + `HookPortAdapter` traits project the SWE_SEED manifest into runtime-specific host files deterministically for:

   - `claude`
   - `codex`
   - `opencode`
   - `github-copilot`
   - `antigravity`
   - `ci`

2. Each adapter maps SWE_SEED’s canonical hook intent into the strongest native mechanism available for that runtime:

   - Claude CLI: JSON command hooks in `.claude/settings*.json`
   - Codex CLI: command hooks in `.codex/hooks.json`
   - OpenCode CLI: TypeScript/JavaScript plugin projection under `.opencode/plugins/`
   - GitHub Copilot CLI: hook JSON under `.github/hooks/*.json`
   - Antigravity CLI: minimal `.agents/hooks.json` projection, initially focused on `PreToolUse`
   - CI: generated policy/check scripts for non-interactive enforcement

3. Projection uses managed markers and byte-stable formatting. Re-running `sync` without manifest changes produces no byte changes.

4. Rollback is byte-exact for all managed files. Files created entirely by SWE_SEED are removed. Existing files modified by SWE_SEED are restored to their prior bytes using snapshots.

5. Unsupported host capabilities surface as `PartialSupport`, not silent success. Each host exposes an explicit capability matrix:

   - `can_block_pre_tool`
   - `can_rewrite_tool_input`
   - `can_modify_tool_result`
   - `can_inject_prompt_context`
   - `can_continue_on_stop`
   - `has_subagent_hooks`
   - `has_file_watch_hooks`
   - `pre_tool_failure_mode`

6. `swe-seed sync --host <host> [--dry-run --prune]`, `swe-seed rollback --host <host>`, and `swe-seed hosts` work for every supported host.

7. `doctor` detects drift between the current host files and the expected deterministic projection from the assembled manifest.

**Falsification:**

- Re-sync changes bytes without manifest changes → determinism fails.
- Rollback leaves a SWE_SEED-managed file behind → rollback fails.
- Rollback does not restore a pre-existing host file byte-for-byte → rollback fails.
- Adapter claims support for a canonical hook that the runtime cannot enforce → capability honesty fails.
- `doctor` misses a manually edited managed region → drift detection fails.

---

### Canonical Hook Port

SWE_SEED should not model Claude, Codex, OpenCode, Copilot, or Antigravity as the source of truth.

The source of truth is a canonical hook port:

```rust
pub enum CanonicalHookEvent {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    ToolError,
    SubagentStart,
    SubagentStop,
    Stop,
    StopFailure,
    PreCompact,
    PostCompact,
    Notification,
    FileChanged,
    CwdChanged,
}
```

Each runtime adapter is responsible for projecting these events into the closest native runtime mechanism.

Unsupported or partially supported events must return `PartialSupport` with a reason.

Example:

```rust
pub enum AdapterSupport {
    Full,
    Partial { reason: String },
    Unsupported { reason: String },
}
```

---

### Host Capability Model

Add an explicit capability model so the abstraction does not lie.

```rust
pub struct HostCapabilities {
    pub can_block_pre_tool: bool,
    pub can_rewrite_tool_input: bool,
    pub can_modify_tool_result: bool,
    pub can_inject_prompt_context: bool,
    pub can_continue_on_stop: bool,
    pub has_subagent_hooks: bool,
    pub has_file_watch_hooks: bool,
    pub pre_tool_failure_mode: FailureMode,
}

pub enum FailureMode {
    FailOpen,
    FailClosed,
    RuntimeSpecific,
    Unknown,
}
```

Every adapter must expose capabilities through:

```rust
pub trait HostAdapter {
    fn host_id(&self) -> HostId;
    fn capabilities(&self) -> HostCapabilities;
    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport;
    fn project(&self, manifest: &AssembledManifest) -> Result<ProjectionPlan>;
    fn rollback(&self, snapshot: &ProjectionSnapshot) -> Result<RollbackPlan>;
}
```

---

### Runtime Adapter Expectations

#### Claude adapter

Projects canonical hooks into Claude Code settings files.

Expected managed files:

```text
.claude/settings.json
.claude/settings.local.json
```

Minimum supported events:

```text
SessionStart
UserPromptSubmit
PreToolUse
PermissionRequest
PostToolUse
PostToolUseFailure
SubagentStart
SubagentStop
Stop
StopFailure
PreCompact
PostCompact
Notification
FileChanged
CwdChanged
```

Claude should be used as the richest adapter, but not as the canonical data model.

---

#### Codex adapter

Projects canonical hooks into Codex hook configuration.

Expected managed files:

```text
.codex/hooks.json
```

Minimum supported events:

```text
SessionStart
UserPromptSubmit
PreToolUse
PermissionRequest
PostToolUse
SubagentStart
SubagentStop
Stop
PreCompact
PostCompact
```

Known constraint:

Codex primarily supports command hooks. Prompt-style, agent-style, or async hook behavior must not be claimed unless verified by tests.

Unsupported behavior must become `PartialSupport`.

---

#### OpenCode adapter

Projects canonical hooks into an OpenCode plugin.

Expected managed files:

```text
.opencode/plugins/swe_seed.ts
```

Minimum supported mappings:

```text
PreToolUse       -> tool.execute.before
PostToolUse      -> tool.execute.after
PermissionRequest -> permission.asked / permission.replied
Stop             -> session.idle
StopFailure      -> session.error
PostCompact      -> session.compacted
FileChanged      -> file.edited / file.watcher.updated
```

Known constraint:

OpenCode is plugin/event based, not JSON-command-hook based. The adapter should generate a thin plugin that calls the shared SWE_SEED hook port.

---

#### GitHub Copilot CLI adapter

Projects canonical hooks into GitHub Copilot hook files.

Expected managed files:

```text
.github/hooks/swe-seed.json
```

Minimum supported events:

```text
SessionStart
UserPromptSubmit
PreToolUse
PermissionRequest
PostToolUse
PostToolUseFailure
SubagentStart
SubagentStop
Stop
StopFailure
PreCompact
Notification
```

Known constraint:

Copilot supports both camelCase and PascalCase style names. Prefer PascalCase internally for alignment with the canonical model.

Cloud-agent constraints must be represented as `PartialSupport`, especially where interactive permissions or local filesystem assumptions do not hold.

---

#### Antigravity adapter

Projects only the verified minimum hook surface at first.

Expected managed files:

```text
.agents/hooks.json
```

Minimum supported event:

```text
PreToolUse
```

Initial matcher focus:

```text
run_command
shell
bash
write
edit
```

Known constraint:

Antigravity hook schemas and behavior appear less stable than Claude, Codex, OpenCode, and Copilot. This adapter must be conservative.

The adapter may block through exit-code behavior first. Structured allow/deny JSON should only be emitted after schema tests prove it is accepted by the installed CLI version.

Most non-`PreToolUse` events should start as `PartialSupport` or `Unsupported`.

---

#### CI adapter

Projects SWE_SEED policies into non-interactive enforcement scripts.

Expected managed files:

```text
.agents/ci/swe-seed-policy-check.{sh,ps1}
.github/workflows/swe-seed-policy.yml
```

Purpose:

CI is not an agent runtime, but it is the enforcement backstop. Any runtime hook that is advisory or partial should have a corresponding CI check where possible.

---

### Files to Modify

#### [NEW]

```text
crates/swe-seed-core/src/adapters/mod.rs
crates/swe-seed-core/src/adapters/host.rs
crates/swe-seed-core/src/adapters/capabilities.rs
crates/swe-seed-core/src/adapters/marker.rs
crates/swe-seed-core/src/adapters/snapshot.rs
crates/swe-seed-core/src/adapters/projection.rs
crates/swe-seed-core/src/adapters/claude.rs
crates/swe-seed-core/src/adapters/codex.rs
crates/swe-seed-core/src/adapters/opencode.rs
crates/swe-seed-core/src/adapters/github_copilot.rs
crates/swe-seed-core/src/adapters/antigravity.rs
crates/swe-seed-core/src/adapters/ci.rs
```

#### [NEW]

```text
crates/swe-seed-core/src/hooks/port.rs
crates/swe-seed-core/src/hooks/events.rs
crates/swe-seed-core/src/hooks/decision.rs
crates/swe-seed-core/src/hooks/runtime_map.rs
```

#### [MODIFY]

```text
crates/swe-seed/src/cli.rs
```

Add:

```text
swe-seed sync --host <host> [--dry-run --prune]
swe-seed rollback --host <host>
swe-seed hosts
swe-seed doctor --host <host>
```

Supported host values:

```text
claude
codex
opencode
github-copilot
antigravity
ci
all
```

---

### Projection Rules

1. All generated content must be deterministic.

2. All managed regions must use SWE_SEED markers.

Example:

```text
# BEGIN SWE_SEED MANAGED: <stable-id>
...
# END SWE_SEED MANAGED: <stable-id>
```

For JSON files, use stable managed object keys where comments are not legal.

Example:

```json
{
  "swe_seed_managed": {
    "version": 1,
    "stable_id": "..."
  }
}
```

1. Existing unmanaged user content must be preserved.

2. If a host file already exists, snapshot it before writing.

3. If a host file does not exist, create it and record that SWE_SEED owns the whole file.

4. `--prune` removes stale SWE_SEED-managed regions no longer present in the manifest.

5. `--dry-run` prints the projection plan without writing files.

6. Rollback restores pre-sync state using snapshots.

7. Drift detection compares current managed regions against deterministic expected output.

---

### Tests

#### [NEW]

```text
tests/host_adapter_capabilities.rs
tests/host_projection_determinism.rs
tests/host_projection_rollback.rs
tests/host_partial_support.rs
tests/host_drift_detection.rs
tests/claude_projection.rs
tests/codex_projection.rs
tests/opencode_projection.rs
tests/github_copilot_projection.rs
tests/antigravity_projection.rs
tests/ci_projection.rs
```

### Verification Commands

```bash
cd /home/sprime01/projects/SWE_SEED

cargo test \
  --test host_adapter_capabilities \
  --test host_projection_determinism \
  --test host_projection_rollback \
  --test host_partial_support \
  --test host_drift_detection

cargo test \
  --test claude_projection \
  --test codex_projection \
  --test opencode_projection \
  --test github_copilot_projection \
  --test antigravity_projection \
  --test ci_projection
```

Manual smoke commands:

```bash
cargo run -p swe-seed -- hosts

cargo run -p swe-seed -- sync --host claude --dry-run
cargo run -p swe-seed -- sync --host codex --dry-run
cargo run -p swe-seed -- sync --host opencode --dry-run
cargo run -p swe-seed -- sync --host github-copilot --dry-run
cargo run -p swe-seed -- sync --host antigravity --dry-run
cargo run -p swe-seed -- sync --host ci --dry-run

cargo run -p swe-seed -- sync --host all
cargo run -p swe-seed -- doctor --host all
cargo run -p swe-seed -- rollback --host all
```

**Expected:**

- `hosts` lists every adapter and its capability matrix.
- `sync --dry-run` shows deterministic projection plans.
- `sync --host all` writes only managed regions/files.
- Re-running `sync --host all` produces no byte changes.
- `doctor --host all` reports clean after sync.
- Manual edits inside managed regions are detected as drift.
- `rollback --host all` restores the repo to the pre-sync byte state.

---

### Done Conditions — Phase 9

- [ ] Runtime-neutral `HostAdapter` trait exists.
- [ ] Runtime-neutral canonical hook event model exists.
- [ ] Runtime capability matrix exists and is exposed through `swe-seed hosts`.
- [ ] Claude adapter projects deterministic managed config.
- [ ] Codex adapter projects deterministic managed config.
- [ ] OpenCode adapter projects deterministic managed plugin.
- [ ] GitHub Copilot adapter projects deterministic managed hook config.
- [ ] Antigravity adapter projects conservative minimal managed hook config.
- [ ] CI adapter projects deterministic enforcement backstop.
- [ ] Unsupported and partially supported hook kinds surface as `PartialSupport` or `Unsupported`.
- [ ] `sync`, `rollback`, `hosts`, and `doctor` work for every host.
- [ ] Re-sync is byte-stable.
- [ ] Rollback is byte-exact.
- [ ] Drift detection catches managed-region edits.
- [ ] No adapter claims stronger enforcement than the runtime actually supports.

---

## Phase 10 — End-to-End Parity + Python Removal

Run only after Phases 0–9.

### Command Sequence

```bash
cd /home/sprime01/projects/SWE_SEED
cargo build --release
cargo test                                  # full suite incl. all golden-file parity tests

# Standalone inner stack, SEA unreachable
SEA_ROOT=/nonexistent target/release/swe-seed seed assemble
SEA_ROOT=/nonexistent target/release/swe-seed route "checkpoint smoke" --record
SEA_ROOT=/nonexistent target/release/swe-seed eval run --spec tests/fixtures/eval.toml
SEA_ROOT=/nonexistent target/release/swe-seed doctor; echo "exit=$?"   # expect 0
SEA_ROOT=/nonexistent target/release/swe-seed fabricate "sample need"

# Parity vs current Python CLIs (must match golden captures)
just ci                                     # existing proof command still green via Rust

# Supersede Python
git rm -r scripts/harness.py scripts/agent_hooks.py scripts/fabricate.py .strategy/strategy.py agentic_capability_loop/
cargo test                                  # still green with Python removed
```

### Final Proof Artifact

> The `swe-seed` release binary reproduces all three layers (SweSeed/Harness/Fabricator) and
> every superseded Python CLI (`validate, doctor, render-skills, route, inspect, context-plan,
> eval run, trace*, fabricate, agent-hooks*`) with golden-file parity; the full inner stack
> runs with `federation.enabled=false` and a dead `SEA_ROOT` making zero external calls; the
> `baml_parity` test proves every Rust type matches its `.baml` contract; and the test suite
> stays green after the Python implementation is deleted.

---

## What Remains After This Plan

| Gap | Boundary | Path Forward |
|---|---|---|
| Additional host adapters | v0.1 = Claude live, others honest stubs | Implement per 0004 matrix; OpenCode next. |
| `.baml`→Rust generator | v0.1 = hand-written structs + parity test | Add a generator only if `.baml` churns often (0019). |
| Real SkillSpector wiring | v0.1 = external gate, `Pending` stub | Verify its CLI/SARIF contract, then implement (0007). |
| Embedded MCP router | v0.1 = host MCP config generation | Add a minimal router behind a flag (0006). |
| Remaining 9 lifecycle events | v0.1 = 5 required | Promote reserved events when a consumer needs them (0005). |
| Human-gated provenance items | Specs flag them | Commercial-distribution posture, upstream SHAs, seed-source license tags (0009). |
