# swe-seed-core library root (src)

## Overview

Source root of the `swe-seed-core` governance library (the SWE_Seed Rust
rewrite). `lib.rs` declares the module tree: Phase 0 `.baml`
contracts-as-data + config parsing (spec 0019), Phase 1 SweSeed layer
(specs 0003/0009/0018: capability registry, seed-package assembly, boundary
validation, idempotent regeneration, fail-closed provenance verification), plus
the harness plane (routing, traces, eval, context, hooks, skills, learning,
doctor), the Fabricator layer (spec 0017), the gateway (spec 0020), and the
optional SEA federation boundary (spec 0011). Five files live directly here:
`lib.rs` (the module tree), the routing enforcement gate, the tamper-evident
trace ledger, the harness structure validation, and shared
format-compatibility helpers.

## Interface

- `pub mod` declarations for all 17 subsystem modules plus the 4 root-level
  modules (`harness_validate`, `routing_gate`, `trace_ledger`, `util`; the
  parenthetical below enumerates all 21 `pub mod` names) (`adapters`, `config`,
  `context`, `contracts`, `doctor`, `eval`,
  `fabricator`, `federation`, `gateway`, `harness_validate`, `hooks`,
  `learning`, `provenance`, `route`, `routing_gate`, `security`, `seed`,
  `skill`, `trace`, `trace_ledger`, `util`) [lib.rs]
- `pub enum RouteGate { Allow, Block(String) }` and `route_gate(root,
trace_id)` / `route_gate_with(root, trace_id, policy)`: blocks work on a
  trace that has no `RouteSelected` genesis in the tamper-evident chain;
  ledger-absent is `Block` (fail-closed) [routing_gate.rs:RouteGate],
  [routing_gate.rs:route_gate], [routing_gate.rs:route_gate_with]
- `pub const GENESIS_PREV: &str` (zero hash), `default_db_path(root)` =
  `.agent-harness/traces/ledger.sqlite3`, `open(path)`, `LedgerEntry`,
  `Appended`, `pub enum AnchorPolicy { Required, AllowUnanchored }`,
  `HeadAnchor { length, head }`, `append(conn, entry)`, `has_genesis(conn,
trace_id)`, `chain_root(conn, trace_id)`, `verify_chain(conn, trace_id)`,
  `verify_chain_with(conn, trace_id, policy)`: the hash-chained SQLite ledger
  (`entry.hash = SHA256(event_type || payload_json || prev_hash)`) with
  per-trace head anchors [trace_ledger.rs:GENESIS_PREV],
  [trace_ledger.rs:default_db_path], [trace_ledger.rs:open],
  [trace_ledger.rs:LedgerEntry], [trace_ledger.rs:Appended],
  [trace_ledger.rs:AnchorPolicy], [trace_ledger.rs:HeadAnchor],
  [trace_ledger.rs:append], [trace_ledger.rs:has_genesis],
  [trace_ledger.rs:chain_root], [trace_ledger.rs:verify_chain],
  [trace_ledger.rs:verify_chain_with]
- `validate(root) -> Vec<String>`: harness structure validation (root specs,
  `.baml` contracts, required dirs, route cards per job type with resolvable
  skills, skill well-formedness, render-target notice + duplicate content,
  incomplete-work markers); `REQUIRED_JOB_TYPES` (11 types)
  [harness_validate.rs:validate], [harness_validate.rs:REQUIRED_JOB_TYPES]
- `secure_write(root, components, name, bytes, mode)`: symlink-refusing,
  openat-based atomic harness file write (Unix; plain fallback elsewhere)
  [util.rs:secure_write]
- `utc_now()` (`2026-06-17T00:23:16+00:00`), `utc_stamp()`
  (`20260617T002316Z`), `slugify(text)` (capped at 48 chars, default `trace`),
  `redact_secrets(text)` (sk-/ghp*/gho*/xox/AKIA/PRIVATE-KEY patterns ->
  `[REDACTED]`) [util.rs:utc_now], [util.rs:utc_stamp], [util.rs:slugify],
  [util.rs:redact_secrets]

### Subsystem contracts

- [adapters](./adapters/README.md) - host adapter projection for 6 agent hosts
  with snapshot/rollback/drift
- [config](./config/README.md) - typed YAML config structs + generic
  `load_yaml`
- [context](./context/README.md) - context budget plane (spec 0015) and the
  `context-plan` entry point
- [contracts](./contracts/README.md) - `.baml` contracts-as-data + `BamlParity`
  (spec 0019) + shared harness types
- [doctor](./doctor/README.md) - doctor + drift detection aggregation
  (spec 0008)
- [eval](./eval/README.md) - deterministic eval + proof records (spec 0013)
- [fabricator](./fabricator/README.md) - product-to-prototype semantic chain
  (spec 0017)
- [federation](./federation/README.md) - SEA federation boundary (spec 0011)
- [gateway](./gateway/README.md) - MCPGate runtime, gates, and server
  (spec 0020)
- [hooks](./hooks/README.md) - normalized hook runtime + permission gate
  (spec 0005)
- [learning](./learning/README.md) - learning and adaptation loop (spec 0016)
- [provenance](./provenance/README.md) - license/provenance boundary records
  (spec 0009)
- [route](./route/README.md) - route-card loading, scoring, and selection
  (specs 0004/0012)
- [security](./security/README.md) - skill scan gate + SkillSpector
  (spec 0007)
- [seed](./seed/README.md) - 3-layer governance + seed package assembly
  (specs 0003/0018)
- [skill](./skill/README.md) - skill ingestion + render (spec 0007)
- [trace](./trace/README.md) - trace lifecycle + durable decisions
  (spec 0014)

## Invariants

- Routing is mandatory: `route_gate` fails closed when the ledger is absent,
  when chain verification fails, or when the trace has no `RouteSelected`
  genesis [routing_gate.rs:route_gate]
- Ledger entries are hash-chained: altering any entry's payload breaks the hash
  of every later entry, and `verify_chain_with` recomputes the whole chain
  [trace_ledger.rs:verify_chain_with]
- Truncation is detected via per-trace head anchors: an append must extend
  exactly the chain the anchor describes (`length == seq` and `head == prev`)
  or it is refused before the anchor is overwritten; an unanchored trace fails
  verification under `AnchorPolicy::Required` [trace_ledger.rs:update_anchor],
  [trace_ledger.rs:verify_chain_with]
- The genesis entry (seq 0) is the route decision and must carry
  `prev_hash == GENESIS_PREV`; a fresh append mints a ULID `event_id`
  [trace_ledger.rs:append], [trace_ledger.rs:GENESIS_PREV]
- `secure_write` never follows or replaces symlinked path components and
  commits via temp-file + `renameat` + directory fsync
  [util.rs:secure_write_unix]
- All persisted route/trace text is redacted through `redact_secrets` before
  storage [util.rs:redact_secrets]
- Harness validation fails on any missing root spec, missing `.baml` contract,
  missing required directory, job type without a route card, route referencing
  a removed proof command surface (`python scripts/harness.py`,
  `bash tests/validate-harness.sh`), or incomplete-work marker outside policy
  lines [harness_validate.rs:validate],
  [harness_validate.rs:REMOVED_ROUTE_PROOF_COMMANDS],
  [harness_validate.rs:INCOMPLETE_MARKERS]

## Side Effects

- `route_gate` / `route_gate_with` open the SQLite ledger at
  `.agent-harness/traces/ledger.sqlite3` (read-only intent; `open` creates the
  schema when missing) [routing_gate.rs:route_gate_with],
  [trace_ledger.rs:open]
- `append` inserts chain rows inside an `BEGIN IMMEDIATE` transaction and
  writes the sidecar anchor file `ledger.anchors.json` (temp + fsync + rename)
  [trace_ledger.rs:append], [trace_ledger.rs:write_anchors]
- `validate` reads the repository tree: root specs, `.agent-harness/**`
  (skills, routes, render-targets), `tests/`, `docs/specs`
  [harness_validate.rs:validate]
- `secure_write` creates directories (`mkdirat`) and writes + fsyncs files
  under `root` [util.rs:secure_write_unix]

## Verification

- Run: `cargo test -p swe-seed-core`
