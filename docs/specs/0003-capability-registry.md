# 0003 — Capability Registry

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** Adopt the
> proven vocabulary: `SkillPack` → **SkillIR** (0007), `CapabilityProfile` → **PermissionPolicy**
> (0005/0006), `ProvenanceRecord` carrier → **ArtifactMetadata** (0009/0018), generic
> `Capability` → **LayerCapability** (0018). The assembled registry is a **SeedPackageManifest**
> (0018). Per-concept proven types: RouteCard (0004/0014), EvalSpec/ProofRecord (0013),
> ContextBudget/ContextPack (0015), HookPolicy (0005), TraceSchema (0014), LearningRecord (0016).
> All struct/enum shapes are canonical in `.baml` (contracts-as-data, [0019](0019-baml-contracts-as-data.md)).

## Purpose

Define the inventory and normalized metadata model for every capability SWE_Seed manages,
and the canonical SWE_Seed concepts. The registry is the single source of truth that host
adapters project from and doctor verifies against.

## Non-goals

- Not a host-specific format. The registry is host-neutral; adapters translate it (0004).
- Not the gateway runtime (0006) — the registry *describes* MCP servers; the gateway
  *routes* to them.
- v0.1: no hot reload of the registry (restart to apply). Watch concept noted below.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `src/capability/definition/mod.rs`, `parser.rs`, `loader.rs` | files | Declarative capability definitions loaded + parsed | `CapabilitySource` + normalized registry entries |
| gateway prior-art | `src/capability/hash.rs` | file | Content hashing of capabilities | `ArtifactMetadata.source_hash` |
| gateway prior-art | `src/capability/validator/`, `schema_validator/` | dirs | Schema + semantic validation | Per-concept validation rules |
| gateway prior-art | `src/capability/watcher.rs`, `src/config_reload/mod.rs` | files | File-watch hot reload | Deferred post-v0.1 reload concept |
| multi-host runtime prior-art | `(prior-art path redacted)` | `SkillDefinition`, `SkillSource`, `SkillsConfig`, `RuntimeSkillConfig` | Typed skill model w/ source union + runtime variant | `SkillIR` fields + runtime-vs-portable flag |
| multi-host runtime prior-art | `(prior-art path redacted)`, `engine/` | files | Rules sourced from AGENTS.md + frontmatter | `Rule` concept |

## SWE_Seed requirements

The registry is the `SeedPackageManifest`. It stores stable `LayerCapability` entries,
normalized metadata, `ArtifactMetadata` provenance, and projection state. Legacy terms such
as `CapabilitySource`, `SkillPack`, `CapabilityProfile`, `DoctorCheck`, and
`ProvenanceRecord` are historical aliases for the reconciled types named below.

### Canonical concepts

Each concept: **purpose / required / optional / example / Rust location / validation /
tests**.

#### CapabilitySource
- **Purpose**: where capabilities come from (git repo, local path, registry URL).
- **Required**: `id`, `kind` (`git|path|url`), `location`.
- **Optional**: `ref` (tag/branch/commit), `subpath`, `license_hint`.
- **Example**:
  ```toml
  [[source]]
  id = "addyosmani-agent-skills"
  kind = "git"
  location = "https://github.com/addyosmani/agent-skills"
  ref = "main"
  ```
- **Rust**: `swe_seed::capability::source`.
- **Validation**: `id` unique kebab-case; `location` resolvable; git refs pinned for
  reproducibility (warn if floating like `main`).
- **Tests**: parse/resolve each kind; reject duplicate ids.

#### SkillIR
- **Purpose**: a skill or workflow bundle (e.g. from addyosmani/agent-skills).
- **Required**: `id`, `source_id`, `name`, `entrypoint`, `runtime` (`portable|full`).
- **Optional**: `description`, `tags`, `commands[]`, `mcp[]`, `scan_status`.
- **Example**:
  ```toml
  [[skill]]
  id = "pdf-extractor"
  source_id = "addyosmani-agent-skills"
  name = "pdf-extractor"
  entrypoint = "SKILL.md"
  runtime = "portable"
  ```
- **Rust**: `swe_seed_core::skill`.
- **Validation**: `entrypoint` exists in source; `runtime=portable` may not declare
  host-only deps; scan required before activation if profile.require_scan (0007).
- **Tests**: portable vs full classification; missing entrypoint rejected.

#### MCPServer
- **Purpose**: a backend MCP server SWE_Seed can register with the gateway (0006).
- **Required**: `id`, `transport` (`stdio|http`), `command_or_url`, `profile_tags[]`.
- **Optional**: `env[]`, `args[]`, `approval` (`none|prompt|explicit`), `health_url`.
- **Example**:
  ```toml
  [[mcp]]
  id = "serena"
  transport = "stdio"
  command_or_url = "serena-mcp"
  profile_tags = ["coding", "readonly"]
  ```
- **Rust**: `swe_seed::capability::mcp`.
- **Validation**: stdio binary on PATH or http url well-formed; `dangerous` tag forces
  `approval=explicit`.
- **Tests**: env var presence check; approval escalation rule.

#### Command
- **Purpose**: a slash command projected into hosts that support them.
- **Required**: `id`, `source_id`, `name`, `body_ref`.
- **Optional**: `args_schema`, `host_support[]`.
- **Rust**: `swe_seed::capability::command`. **Validation**: unique name per host scope.
- **Tests**: name collision detection across sources.

#### Agent
- **Purpose**: a subagent definition projected where supported.
- **Required**: `id`, `name`, `description`, `prompt_ref`.
- **Optional**: `tools[]`, `model_hint`, `host_support[]`.
- **Rust**: `swe_seed::capability::agent`. **Validation**: description non-empty; model
  hints are advisory, never host-binding.
- **Tests**: agents projected only into hosts that support agents (0004).

#### Hook
- **Purpose**: a lifecycle event handler (0005).
- **Required**: `id`, `event` (normalized lifecycle name), `action_ref`, `blocking` bool.
- **Optional**: `host_support[]`, `timeout_ms`, `on_failure` (`block|warn|ignore`).
- **Rust**: `swe_seed::hooks`. **Validation**: `event` ∈ normalized lifecycle set (0005);
  v0.1 only the five required events.
- **Tests**: unknown event rejected; blocking/non-blocking honored.

#### Rule
- **Purpose**: doctrine fragment / policy (sourced from AGENTS.md or explicit rule files).
- **Required**: `id`, `scope` (`global|project|path-glob`), `text_ref`.
- **Optional**: `priority`, `tags`.
- **Rust**: `swe_seed::capability::rule`. **Validation**: conflicting rules at same scope
  flagged by doctor (0008).
- **Tests**: precedence resolution; conflict detection.

#### HostAdapter
- **Purpose**: projects registry → a specific host's files (0004).
- **Required**: `host_id`, `supported_kinds[]`, `generate()`, `rollback()`.
- **Rust**: `swe_seed::adapters`. **Validation**: must declare unsupported kinds.
- **Tests**: see 0004.

#### SecurityGate
- **Purpose**: scanner that gates activation (0007); SkillSpector is the first impl.
- **Required**: `id`, `applies_to[]` (kinds), `run()` → `ScanResult`.
- **Rust**: `swe_seed::security`. **Validation**: failed/over-threshold scan blocks
  activation unless an approved exception exists.
- **Tests**: see 0007. **No EE-derived logic.**

#### PermissionPolicy
- **Purpose**: named allow/deny + approval policy (`readonly`, `coding`, `dangerous`).
- **Required**: `name`, `allow[]`, `require_scan` bool.
- **Optional**: `deny[]`, `approvals[]`.
- **Rust**: `swe_seed_core::hooks::PermissionPolicy`. **Validation**: `dangerous` requires explicit
  approvals for risky tool groups.
- **Tests**: profile resolution; deny overrides allow.

#### Projection
- **Purpose**: record of what was generated into which host (for drift + rollback).
- **Required**: `capability_id`, `host_id`, `generated_files[]`, `content_hash`, `ts`.
- **Rust**: `swe_seed::adapters::projection`. **Validation**: hash matches on disk or
  doctor reports drift.
- **Tests**: drift detection (0008).

#### EvalCheck
- **Purpose**: a verification probe (0008/0013).
- **Required**: `name`, `severity`, `run()` → `pass|warn|fail`, `remediation`,
  `auto_fixable` bool. **Rust**: `swe_seed_core::eval`. **Tests**: see 0008/0013.

#### ArtifactMetadata
- **Purpose**: trace a capability to its source + integrity (0009).
- **Required**: `capability_id`, `source_id`, `source_hash`, `inspected_at`,
  `license_tag`, `code_copied=false`.
- **Optional**: `scan_ref`, `notes`.
- **Example (JSON)**:
  ```json
  {"capability_id":"serena","source_id":"oraios-serena",
   "source_hash":"sha256:…","inspected_at":"2026-06-26",
   "license_tag":"MIT","code_copied":false}
  ```
- **Rust**: `swe_seed_core::provenance`. **Validation**: `code_copied` must be false;
  `source_hash` required before activation.
- **Tests**: hash stability; missing provenance blocks activation.

## CLI behavior, if applicable

`swe-seed add <source>`, `swe-seed list [--kind]`, `swe-seed remove <id>`,
`swe-seed registry validate`.

## Generated files, if applicable

`SeedPackageManifest` under `.swe-seed/`, plus `.swe-seed/provenance/*.json`. Host files
via adapters (0004). All generated files include a managed-by header + hash.

## Rust module boundaries

`swe_seed_core::seed` owns `SeedPackageManifest` and `LayerCapability`;
`swe_seed_core::skill` owns `SkillIR`; `swe_seed_core::hooks` owns `PermissionPolicy`;
`swe_seed_core::provenance` owns provenance verification. Adapters and gateway consume the
manifest read-only.

## Security and provenance considerations

Provenance is mandatory per entry. Risky kinds (`mcp` with `dangerous`) cannot be
projected without explicit approval. Source hashes detect upstream tampering.

## Tests

- Registry round-trips TOML losslessly and deterministically.
- Each concept's validation rules have a failing + passing case.
- Activation blocked when provenance/scan missing under a `require_scan` profile.

## Decisions

- **One `SeedPackageManifest`** registry artifact, not per-kind files. Easier diffs and
  atomic writes; split only if it becomes unwieldy.
- **AGENTS.md is authoritative** for `Rule`/doctrine; the registry stores references, never
  a duplicate copy. Avoids two sources of truth.

## Acceptance criteria

- [ ] All 13 concepts have required fields, an example, Rust location, validation, tests.
- [ ] No field/format copied from reference repos (independently derived).
- [ ] Registry can describe all four v0.1 seed sources without code changes.
