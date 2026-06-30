# 0010 — v0.1 Implementation Plan

> **Superseded by [`.agents/plans/0001-swe-seed-v0-1-implementation.md`](../plans/0001-swe-seed-v0-1-implementation.md)**
> after the [0012](0012-existing-harness-reconciliation.md) reconciliation. The plan of record
> is the 3-layer Rust rewrite (`SweSeed → Harness → Fabricator`, contracts-as-data, golden-file
> parity, supersede the Python). This spec-level milestone sketch is retained for history only.

## Purpose

A small, boring, deterministic plan to build SWE_Seed v0.1: the centralization layer that
can ingest, normalize, register, project, and verify capabilities for at least one host,
with provenance and a scan-gate interface in place (scanner not yet implemented).

## Non-goals

- Not implementing the four seed integrations' specific logic (serena, context7,
  agent-skills, SkillSpector) — only the layer that hosts them.
- No embedded full MCP gateway in v0.1 (config generation + minimal router only).
- No hot reload, no daemon, no telemetry.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `src/commands/{add_remove,doctor,config_export}.rs` | files | CLI verb decomposition | v0.1 verbs: add/sync/doctor/scan/provenance |
| gateway prior-art | `src/skills/installer.rs` | file | install pipeline shape | ingest pipeline (0007) |
| multi-host runtime prior-art | `(prior-art path redacted)/.../claude-code-*-loader` | dirs | per-format loaders | Claude adapter first |

## SWE_Seed requirements (v0.1 scope)

Deliver an end-to-end loop: **add a source → normalize → register w/ provenance →
project into Claude → doctor verifies → rollback works.**

### Milestones (sequential, each independently shippable)

**M1 — Registry + provenance core** (`swe_seed::config`, `::capability`, `::provenance`)
- Parse `.swe-seed/registry.toml`; implement `CapabilitySource`, `SkillPack`, `MCPServer`,
  `Command`, `Agent`, `Hook`, `Rule` structs + validation (0003).
- `ProvenanceRecord` with source hashing; `swe-seed provenance verify` (0009).
- Tests: round-trip, validation pass/fail, provenance completeness.

**M2 — Claude host adapter + sync/rollback** (`swe_seed::adapters::claude`)
- `HostAdapter` trait; Claude adapter generating `.claude/skills`, `.claude/commands`,
  `.claude/agents`, settings hooks/mcp region, AGENTS.md doctrine ref (0004).
- Managed markers + content hashing; `swe-seed sync [--dry-run] [--prune]`,
  `swe-seed rollback`.
- Tests: determinism (golden files), own-only writes, rollback round-trip.

**M3 — Doctor + drift** (`swe_seed::doctor`)
- `DoctorCheck` registry; the 14 checks (0008); drift via projection hashes;
  `--fix`, `--json`, `--host`.
- Tests: per-check fixtures, drift tamper/restore, exit codes.

**M4 — Hook runtime (5 events) + gateway config generation**
- `swe_seed::hooks`: five required events, projection to Claude settings (0005).
- `swe_seed::gateway`: register `MCPServer`s, profiles `readonly/coding/dangerous`,
  generate host MCP config, `gateway import` (scan existing configs), backend health in
  doctor (0006). Minimal router optional/deferred within M4.
- Tests: event projection, profile gate, MCP config generation, import.

**M5 — Skill ingestion pipeline + scan-gate interface** (`swe_seed::security`,
`::capability::skill::ingest`)
- Pipeline discover→fetch→scan→normalize→project→verify (0007).
- `SecurityGate` trait + SkillSpector adapter **stub** (shells out; returns
  `Pending`/`Error` until wired); exceptions file; blocking semantics.
- Tests: pipeline ordering, blocking prevents projection, exception scoping.

**M6 — Federation boundary + Rust port of the loop adapter** (`swe_seed::federation`)
- Port `agentic_capability_loop/adapters.py` → Rust: `Envelope`, `domain_model_hash`
  resolution (env→manifest→fallback), `emit_*`/`consume_*` for the five emit + three
  consume events (0011).
- `FederationConfig` flags (off by default); wire optional delegation into authority
  (0006/0007), context (0005), and proof emission (0008) **behind flags only**.
- `swe-seed federation status|emit|consume`; `--federation on/off`.
- Tests: **standalone invariant** (federation off ⇒ zero external calls, full inner loop
  passes with SEA unreachable); envelope port-parity vs the Python adapter; authority-mode
  matrix; drift guard on `domain_model_hash`. Port `tests/test_agentic_capability_*.py`.
- On completion, the Python `agentic_capability_loop/` package is superseded.

### Order rationale
M1 underpins all; M2 proves projection on one host; M3 makes it verifiable; M4 adds the
tool plane; M5 adds governance; M6 adds the **optional** SEA federation boundary last —
because the sovereign inner loop (M1–M5) must work fully standalone before any federation
exists. Each milestone leaves a working binary. **Hard invariant across all milestones: the
inner loop never depends on SEA-Forge/Context Kernel/GodSpeed-Agent.**

## CLI behavior (v0.1 surface)

```
swe-seed add <source>
swe-seed list [--kind K] [--status S]
swe-seed sync [--host claude] [--dry-run] [--prune]
swe-seed rollback [--host claude]
swe-seed doctor [--host H] [--project .] [--fix] [--json]
swe-seed scan <skill-id> | --all
swe-seed gateway register|import|list|export
swe-seed provenance list|show|verify
swe-seed federation status|emit|consume   # optional SEA boundary, off by default (0011)
swe-seed run [--federation on|off]        # default off = standalone
```

## Generated files

`.swe-seed/{registry.toml, provenance/*.json, scans/**}`; Claude host files via adapter.
All carry managed markers + hashes.

## Rust module boundaries

Single binary, modules per spec 0002: `config, capability, hooks, gateway, adapters,
security, provenance, doctor, cli`. Suggested deps: `serde`, `toml`, `serde_json`, `sha2`,
`clap`, `anyhow/thiserror`. **No reference repo as a dependency.**

## Security and provenance considerations

Provenance + scan-gate land in v0.1 (interface-complete) so governance is structural from
the start, even before a real scanner is wired. Dangerous profile fails closed.

## Tests

- Integration: full add→sync→doctor→rollback loop on Claude (golden, deterministic).
- `provenance verify` and EE-exclusion guard run in CI.
- Each milestone has unit tests per its spec's Tests section.

## Decisions

- **State dir / registry**: `.swe-seed/` with a single `registry.toml` (per 0002/0003).
- **Second host adapter**: **OpenCode** after Claude (richer capability support than Codex
  → exercises more of the adapter contract). Codex third. Both post-v0.1.
- **Gateway**: **config-only** in v0.1 (per 0006); minimal router deferred.
- **Self-contained build**: the reference repos are deleted; M1–M6 are implementable from
  `docs/specs/` alone, with no access to any reference repo.

## Acceptance criteria

- [ ] M1–M5 each ship a working binary with passing tests.
- [ ] End-to-end Claude loop is deterministic and reversible.
- [ ] Provenance + scan-gate interfaces present; no scanner logic copied or implemented.
- [ ] No reference code copied; traceability matrix clean.
