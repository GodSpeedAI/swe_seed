# swe-seed-core

## Overview

The governance library crate of the SWE_Seed Rust rewrite (workspace member,
Cargo lib). Phase 0 provides `.baml` contracts-as-data + typed config parsing
(spec 0019); Phase 1 provides the SweSeed layer (specs 0003/0009/0018):
capability registry, seed-package assembly, boundary validation, idempotent
regeneration, and fail-closed provenance verification. The crate also carries
the harness plane (routing, traces, eval, context, hooks, skills, learning,
doctor), the Fabricator layer (spec 0017), the gateway (spec 0020), and the
optional SEA federation boundary (spec 0011, default off). It is a pure
library: the `swe-seed` binary crate (not in this subtree) consumes it.

## Subdirectories & Boundaries

- [src](./src/README.md) - library source root: module tree, routing
  enforcement gate, tamper-evident trace ledger, harness validation, shared
  helpers; indexes all 17 subsystem contracts

## Global Invariants

- Contracts-as-data: `.baml` files are read-only inputs parsed into runtime
  types; hand-written types assert `BamlParity` against their `.baml` shapes
  and there is no LLM runtime anywhere in the crate
  [src/contracts/README.md], [src/seed/README.md]
- 3-layer ownership is downward-only (`SweSeed -> Harness -> Fabricator`);
  `validate_boundaries` fails the report on upward references, unclassified
  paths, duplicate capability ids, or missing required capabilities, and
  `passed=false` blocks release [src/seed/README.md]
- Routing is mandatory before work: a trace without a `RouteSelected` genesis
  in the tamper-evident SQLite chain is blocked by `route_gate`, and a missing
  ledger fails closed [src/README.md]
- Federation is standalone by default: with `enabled=false` (the default) no
  external call is ever made and every gate resolves locally; the inner stack
  must pass with federation disabled [src/federation/README.md]
- Claims require proof and redaction: proof records need per-claim evidence,
  eval promotion is live-pass-only, `command_check` is opt-in (`--trusted`
  only), and all persisted task/trace text passes `redact_secrets`
  [src/eval/README.md], [src/README.md]
- Durable writes are atomic and symlink-refusing: harness files commit via
  temp + rename (`secure_write`), trace records refuse symlinked paths, and
  audit/governance/ledger appends are line-atomic or fsynced
  [src/README.md], [src/trace/README.md], [src/gateway/README.md]

## Global Side Effects

- SQLite databases: the tamper-evident trace ledger
  (`.agent-harness/traces/ledger.sqlite3` + `ledger.anchors.json` sidecar) and the
  hook-event index (SQLite via rusqlite bundled); the gateway governance journal
  (`.swe-seed/gateway/governance.journal`) and audit log are fsynced/line-atomic
  JSONL appends, not SQLite
  [src/README.md], [src/hooks/README.md], [src/gateway/README.md]
- Host projection files: `.claude/`, `.codex/`, `.opencode/`, `.github/`,
  `.agents/` hook configs written by the adapters; rendered skill targets
  under `.agent-harness/render-targets/` [src/adapters/README.md],
  [src/skill/README.md]
- Harness state directories: `.agent-harness/traces/records/`,
  `.agent-harness/traces/route-decisions/`, `.agent-harness/skills/` (read),
  `.swe-seed/` provenance + seed manifest + eval-handoff pins, `.fabricator/`
  run artifacts, and hook logs under `.agent-hooks/`-adjacent config
  [src/trace/README.md], [src/route/README.md], [src/seed/README.md],
  [src/fabricator/README.md]
- External processes: SkillSpector scans, eval `command_check` shell runs
  (trusted mode only), stdio MCP backends and the Context Kernel client
  spawned/connected by the gateway and federation modules
  [src/security/README.md], [src/eval/README.md], [src/gateway/README.md],
  [src/federation/README.md]

## System Verification

- Run: `cargo test -p swe-seed-core`
