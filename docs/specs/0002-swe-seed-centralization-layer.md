# 0002 — SWE_Seed Centralization Layer (Overview)

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** SWE_Seed
> is the **outer layer of a 3-layer stack** `SweSeed → Harness → Fabricator` (`LayerName`),
> not a standalone installer. Doctrine = the AGENTS.md Agent Operating Contract; routing is
> mandatory. Where this spec's invented terms shadow proven ones, the proven vocabulary wins.
> Layer governance is detailed in [0018](0018-layer-boundary-governance.md).

## Purpose

Define SWE_Seed as the **centralization and governance layer** that installs, normalizes,
routes, projects, and governs capabilities (skills, MCPs, hooks, agents, slash commands,
doctrine) across many coding-agent tools. This is the umbrella spec; 0003–0010 detail each
subsystem.

## Non-goals

- SWE_Seed is **not** a skill pack. It bundles no specific skills.
- Not a chat/LLM gateway (cf. gateway prior-art's own disclaimer that it is not an LLM gateway,
  `ARCHITECTURE.md` Routing Boundary).
- Not a replacement for the host tools; it projects *into* them.
- v0.1 does not implement the four seed integrations — only the layer that can host them.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `ARCHITECTURE.md` | System Overview | One endpoint fronting many backends; discover-then-invoke | SWE_Seed owns the capability/tool plane via gateway integration (0006) |
| gateway prior-art | `src/projection/` | dir | Projection engine w/ modes & roles | SWE_Seed `Projection` = render normalized capabilities into host files (0004) |
| multi-host runtime prior-art | `(prior-art path redacted)*-loader` | dir | Per-format loaders for agents/commands/mcp/plugins | SWE_Seed host adapters normalize many tool formats (0004) |
| multi-host runtime prior-art | `(prior-art path redacted)`, `(prior-art path redacted)` | dir | AGENTS.md discovery/injection + rules engine | AGENTS.md = doctrine; `Rule` is a first-class capability (0003) |
| multi-host runtime prior-art | `(prior-art path redacted)` vs `(prior-art)` | dir | Separate host runtime packages | Host adapters are separated per host, degrade honestly (0004) |

## SWE_Seed requirements

The layer is composed of seven planes:

```
AGENTS.md             = doctrine (human + agent rules, hierarchical)
Capability registry   = inventory + normalized metadata (spec 0003)
Hook runtime          = lifecycle/event coordination (spec 0005)
MCP gateway integ.    = capability/tool plane (spec 0006)
Host adapters         = projection into Claude/Codex/OpenCode/Copilot/Antigravity/CLI/CI (0004)
Doctor                = drift detection + verification (spec 0008)
Provenance/security   = scan gate + license boundaries (specs 0007, 0009)
```

Core flow (the **sovereign inner loop**): **ingest → normalize → register → project →
verify**. A capability enters from a `CapabilitySource`, is normalized into registry
metadata, optionally scanned by a `SecurityGate`, then projected into each enabled
`HostAdapter`, and verified by `doctor`.

SWE_Seed is **single-binary, file-first, deterministic**. State lives in a project-local
directory (proposed `.swe-seed/`) plus generated host files. No daemon required for v0.1.

### Federation boundary (SWE_Seed is one node, but self-contained)

SWE_Seed is a node in the larger SEA agentic capability loop (Context Kernel, SEA-Forge,
GodSpeed-Agent) **but never depends on it**. Integration happens only at the **semantic
event envelope**, which SWE_Seed can optionally consume as input and produce as output. All
external delegation (authority→SEA-Forge, context→Context Kernel, settlement→GodSpeed-Agent)
is **behind config flags, off by default**. With federation off, the inner loop runs fully
standalone. See spec 0011 — it governs how 0005/0006/0007/0008 degrade to local-only and
defines the Rust port of the existing `agentic_capability_loop/` adapter. **Hard invariant:
the inner loop always works regardless of federation availability.**

## Data model

Canonical concepts (full definitions in spec 0003): `SeedPackageManifest`,
`LayerCapability`, `SkillIR`, `MCPServer`, `Command`, `Agent`, `Hook`, `Rule`,
`HostAdapter`, `SecurityGate`, `PermissionPolicy`, `Projection`, `EvalCheck`, and
`ArtifactMetadata`. Legacy names such as `SkillPack`, `CapabilityProfile`, and
`ProvenanceRecord` are historical aliases only.

Top-level registry artifact: `SeedPackageManifest` under `.swe-seed/`, with
`LayerCapability` entries and `ArtifactMetadata` provenance.

```toml
[meta]
version = "0.1.0"

[[source]]
id = "addyosmani-agent-skills"
kind = "git"
url  = "https://github.com/addyosmani/agent-skills"

[profile.coding]
allow = ["skill", "command", "mcp:readonly"]
require_scan = true
```

## CLI behavior, if applicable

Top-level verbs (detailed per spec): `swe-seed seed assemble`,
`swe-seed seed validate-boundaries`, `swe-seed doctor` (0008), `swe-seed scan` (0007),
`swe-seed gateway` (0006), and `swe-seed provenance` (0009).

## Generated files, if applicable

Per-host files only (spec 0004). SWE_Seed never edits host files outside the regions/files
it owns; generated files carry a managed-by header + content hash.

## Rust module boundaries

```
swe_seed_core::seed        # ProjectSeed, LayerCapability, SeedPackageManifest, boundary reports
swe_seed_core::skill       # SkillIR ingestion/render model
swe_seed_core::hooks       # normalized lifecycle runtime and PermissionPolicy (0005)
swe_seed_core::gateway     # MCP gateway integration (0006)
swe_seed_core::provenance  # ArtifactMetadata/provenance hashing and traceability (0009)
swe_seed_core::eval        # EvalCheck/EvalSpec/EvalResult proof contracts (0008/0013)
swe_seed_core::federation  # OPTIONAL SEA envelope emit/consume, off by default (0011)
swe_seed                  # CLI argument parsing → subsystem calls
```

`federation` is the only module touching the SEA contract; all others run without it.

## Security and provenance considerations

Every registered capability carries `ArtifactMetadata` provenance (source + hash + license tag).
Risky capability groups require explicit approval flags (profiles, 0006). All security
concepts are derived from public specs — **never** from gateway prior-art EE files (0009).

## Tests

- Round-trip: a `CapabilitySource` ingested → registry → projected → doctor passes.
- Determinism: same inputs produce byte-identical generated files (hash-stable).
- Honest degradation: projecting an unsupported capability type yields a recorded
  partial-support warning, not a silent drop.

## Decisions

- **State directory**: `.swe-seed/`. Matches host conventions (`.claude/`, `.vscode/`);
  load-bearing for every path, fixed now.
- **Doctrine**: v0.1 **reads/validates** AGENTS.md only (authoritative source of truth);
  registry holds references. Authoring is a later feature.
- **Packaging**: **single binary**, modules as listed. Split into a crate workspace only
  if compile time becomes a problem (YAGNI).

## Acceptance criteria

- [ ] The seven planes map 1:1 onto specs 0003–0010.
- [ ] No plane requires copying reference code.
- [ ] v0.1 scope (0010) is a strict subset that still produces a working ingest→project→
      verify loop for at least one host (Claude).
