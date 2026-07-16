# Implementation Plan — SWE_SEED: Envelope Convergence, Canonical-Implementation Confirmation, and Ownership Boundary

**Created:** 2026-07-01

**Source of truth:** `SEA/.agents/reports/syntelligent_infrastructure.md` —
implements LE-01 (SWE_SEED's Rust federation-envelope half), LE-05, LE-09,
§4.5 ("Federation is optional and default-off"), §13 WP-1, WP-4, WP-8. Quote
it, don't paraphrase from memory.

**Originating context:** SWE_SEED is a **sovereign node** — its inner loop
(`ingest → normalize → register → project → verify`) works standalone with
federation off by default (`[federation].enabled = false`), and this must
never change: "any code path that can break the inner loop when the
federation is unreachable is a defect" (spec 0011, quoted directly). This
plan's tasks all operate *inside* the federation boundary — they change what
gets emitted when federation is on, never the standalone path's behavior.

**Status of the work today (re-verified against the code, 2026-07-01):**
SWE_SEED is a **single Rust implementation**. The Python harness this audit
originally flagged (`scripts/harness.py`) was removed in the "Phase 10
cutover" — `scripts/ci.sh::run_test()` documents this directly: "Replaces the
former `python scripts/harness.py validate` ... after the Phase 10 cutover."
`Cargo.toml` is headed "SWE_Seed Rust rewrite"; every `justfile` target runs
through `cargo run -p swe-seed`; `.agent-harness/` is config/assets
(`baml/baml_client/` generated LLM client, `config.yaml`, `routes/`,
`skills/`, `evals/`, `memory/`, `traces/`) consumed *by* the Rust binary, not
a second implementation. The only other `.py` lives under `tests/` and
exercises the Rust binary over its CLI — it does not implement routing/proof.

Consequence: **LE-05's "two implementations of routing/proof" premise is
already resolved by the rewrite** — Rust is the sole, canonical
implementation. There is no Python implementation to choose between, to keep
in parity, or to retire. What remains is (a) the envelope is still emitted in
the non-conformant family-A *shape* from Rust, and (b) the canonical decision
is not yet recorded in-repo and the codebase still carries misleading
"Python parity" naming that implies a second implementation exists.

The Rust federation core (`crates/swe-seed-core/src/federation/`) is the most
rigorous contract in the whole ecosystem — mutual Ed25519 signing, a
committed behavioral vector (`federation_parity.rs`), real hash resolution
with drift detection. What's missing: (a) the federation *emit* path still
serializes the non-conformant family-A envelope shape when it talks to SEA,
(b) no documented boundary between "SWE_SEED proves" and "godspeed_agent
settles", (c) the canonical-Rust decision is undocumented and stale
"Python parity" naming lingers in Rust doc-comments/test names.

---

## 0. How to use this plan (agent operating instructions)

- Task 1 (envelope conformance) and Task 3 (ownership doc) are independent.
  Task 2 (canonical-implementation confirmation) is independent and small —
  do it after Task 1 so the envelope-shape change and the doc/naming cleanup
  land in a coherent order.
- **Every task ends with a verification gate.** Do not mark a task done
  until its gate command exits 0.
- **Core principle: the sovereign inner loop must never require federation.**
  Every change in this plan happens inside `[federation].enabled = true`
  code paths. If a task's steps would touch `ingest`/`normalize`/`register`/
  `project`/`verify` in a way that adds a hard dependency on SEA/NATS
  reachability, stop — that violates spec 0011's hard invariant and is out
  of scope for every task below.
- Match surrounding code style: `crates/swe-seed-core/src/federation/` is the
  idiom to mirror (existing `envelope.rs`, `signing.rs`, `flags.rs`,
  `consume.rs`, `emit.rs`).
- `crates/swe-seed-core/src/federation/signing.rs` + `tests/federation_signing.rs`
  - `tests/federation_verify_sea.rs` (the existing Ed25519 signing/verify
  vectors) are the proof style this repo already trusts — do not weaken them.

### Global verification gates (must stay green after EVERY task)

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --workspace          # full Rust suite (core lib + bin + CLI goldens)
just lint                       # ruff over the .py test/baml-client files
just ci                         # doctor + format + lint + test (the default proof command)
```

`just test`/`just ci` run via `scripts/ci.sh`; the targeted gates below reuse
`cargo test` substring filters. There is **no** `just ci-test-python` target
— that command in earlier drafts of this plan was stale and does not exist.

⟨No codegen/rebuild step is required unless a task touches
`crates/swe-seed-core/`'s public API, in which case:⟩

```bash
cargo build --workspace
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| Sovereign inner loop (must never require federation) | `crates/swe-seed/src/harness_cli.rs` + `crates/swe-seed-core/src/contracts/harness.rs` — `ingest → normalize → register → project → verify`, driven by the `swe-seed` binary (`just harness-*`) |
| Federation config gates | `crates/swe-seed-core/src/federation/flags.rs` — `authority_mode`/`context_mode`/`settlement_mode`, `emits()`, `consumes()`, `is_standalone()`, `authority_gate()` |
| Federation default | `[federation].enabled = false` (default-off, per spec 0011) |
| Ed25519 signing (already rigorous — do not modify signing logic) | `crates/swe-seed-core/src/federation/signing.rs`; canonical string `{namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}` |
| Existing behavioral vector for the federation module (the pattern Task 2 reconciles) | `crates/swe-seed-core/tests/federation_parity.rs`, `federation_signing.rs`, `federation_verify_sea.rs` |
| Hash resolution (Rust side, already correct) | `crates/swe-seed-core/src/federation/envelope.rs::resolve_domain_model_hash()` |
| Drift check (already correct) | `crates/swe-seed-core/src/federation/consume.rs::check_drift()` |
| **Envelope construction — the family-A shape LE-01 wants converged** | `crates/swe-seed-core/src/federation/envelope.rs::make_event()` + `Envelope` struct (`event_id, event_type, namespace, occurred_at, payload`); builders in `emit.rs::emit_*` |
| Wire serialization boundary (where v1 conformance is actually enforced) | `crates/swe-seed-core/src/federation/emit.rs::dispatch()` — writes pretty JSON to sink iff `cfg.emits()`, else no filesystem touch (the standalone invariant) |
| Routing/proof implementation (the canonical Rust path) | `crates/swe-seed-core/src/route/mod.rs`, `crates/swe-seed-core/src/routing_gate.rs`; behavioral coverage in `tests/route_golden.rs`, `tests/routing_gate.rs` |
| Target v1 envelope schema (external, do not modify — conform to it) | `hassos-addon-agent-memory-ledger/.../contracts/sea.agent.event.v1.json` — required `schema_version`(`"v1"`), `event_id`, `source_agent`, `occurred_at`, `payload`; optional `idempotency_key`, `source`, `event_type`, `correlation_id`, `causation_id`, `trace_id`, `provenance{origin,chain}`, `metadata`; `additionalProperties:false` |
| Today's non-conformant envelope shape (family A) — now lives in Rust, ported from the old Python adapter | `crates/swe-seed-core/src/federation/envelope.rs` — struct + `make_event()` build `{event_id, event_type, namespace, occurred_at, payload}`; no `schema_version`, uses `namespace` not `source`/`source_agent`, no `idempotency_key`/`correlation_id`/`causation_id`/`trace_id` |
| Federation is optional — spec quote to preserve verbatim in any docs this plan touches | spec 0011: "not a re-design of the SEA event contract" — SWE_SEED *conforms to* family A/C, it doesn't own the schema |

---

## Task 1 — Rust federation-path envelopes conform to v1 (LE-01, SWE_SEED half)

**Goal:** When SWE_SEED's federation path emits events toward SEA/agent-memory-ledger
with federation enabled, the serialized envelope matches
`sea.agent.event.v1.json`. This is now a purely Rust change — the old Python
adapter half no longer exists (Phase 10 cutover).

**Why this shape:** LE-01 lists SWE_SEED among the repos needing envelope
convergence. The Rust federation path already resolves the domain-model hash
correctly and signs correctly — but the *envelope shape it serializes* is
still family A, because `envelope.rs` is explicitly "Pure port of
`agentic_capability_loop/adapters.py` `_event`" (see its top doc-comment and
the `emit.rs` "mirrors the Python `emit_*` payload keys 1:1" comment). This
task converges that ported shape onto v1.

### Steps

1. In `crates/swe-seed-core/src/federation/envelope.rs`, change the `Envelope`
   struct + `make_event()` so the federation-path wire output carries the v1
   fields: `schema_version: "v1"`, `source_agent` (SWE_SEED's own agent
   identifier), `idempotency_key` (default to `event_id`), and thread
   `correlation_id`/`causation_id`/`trace_id` from the harness's existing
   work-request/trace context where available. `additionalProperties:false`
   means the serialized body must carry exactly the v1 keys — the top-level
   `namespace` field does **not** exist in v1 (it is replaced by
   `source`/`source_agent`).
2. Keep the Ed25519 signature under a `provenance`/payload sub-object per the
   audit's recommendation #2, not a new top-level field outside the v1
   allow-list. The `Envelope::with_trace_chain_root()` helper already injects
   into the payload — reuse that seam.
3. **Do not touch `signing.rs`'s canonical-string construction.** It is built
   from `{namespace}\n{event_type}\n{occurred_at}\n{sorted_payload}` and is
   covered by committed parity vectors. The v1 *wire* shape (what `dispatch()`
   serializes) and the signing *canonical string* are separate concerns:
   `namespace` can remain available as a signing input even though the v1
   wire envelope exposes `source`/`source_agent` instead. If the two cannot
   be reconciled without changing the canonical string, stop — that is out of
   scope and must be raised, not silently changed.
4. Only apply the v1 shape inside the `federation.enabled` emit path. Add a
   test proving the standalone (`federation.enabled = false`) path is
   byte-for-byte unaffected (its local events never leave the node and never
   need v1 conformance).
5. Update the now-stale "Pure port of ... adapters.py" / "mirrors the Python"
   doc-comments in `envelope.rs` and `emit.rs` to describe what the code
   actually does (a v1-conformant Rust emit path), not a port of a removed
   Python file.

### Gate

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test -p swe-seed-core federation       # federation module + signing/drift vectors stay green
cargo test -p swe-seed-core -- v1_conform    # new conformance test (add it)
just ci                                       # full suite
```

**Done when:** the v1-conformant event serializes correctly with a test
proving it against the required-field set from
`sea.agent.event.v1.json`, AND a separate test proves the standalone path's
behavior is byte-identical before/after this change (run the standalone inner
loop, diff output), AND the signing parity vectors still pass unmodified.

**Redesign trigger:** if conforming the wire shape requires changing the
signing canonical string (because `namespace` can't be retained as a signing
input), stop and raise it — that crosses into signing-contract territory this
plan explicitly excludes.

---

## Task 2 — Confirm Rust as the canonical routing/proof implementation (LE-05)

**Goal:** Rust (`crates/swe-seed-core`) is declared — not chosen, not defaulted —
the sole canonical implementation for routing/proof. The duplication LE-05
warned about is already gone (the Python harness was removed in the Phase 10
cutover). This task records that decision so it is not re-litigated or
re-introduced, confirms the canonical path is protected by committed
behavioral vectors, and removes the misleading "Python parity" framing that
still implies a second implementation exists.

**Why this shape:** LE-05: "two implementations of routing/proof... behavioral
drift, hard bugs." The repo already collapsed to one implementation (Rust).
The residual risk is no longer cross-language drift — it is that (a) the
decision is undocumented and (b) the codebase still *says* it has a Python
counterpart (stale doc-comments and the `federation_parity.rs` name), which
misleads the next reader into thinking a parity obligation exists.

### Steps

1. Confirm the canonical status from the repo's own evidence (do not re-derive
   from the audit, which predates the cutover): `Cargo.toml` ("SWE_Seed Rust
   rewrite"), `scripts/ci.sh::run_test()` ("Replaces the former
   `python scripts/harness.py validate` ... after the Phase 10 cutover"),
   `justfile` (all targets → `cargo run -p swe-seed`), and the absence of any
   routing/proof logic under `.agent-harness/` or `tests/*.py` (the `.py`
   there drives the Rust CLI; the only other `.py` is the generated
   `baml/baml_client/`).
2. Enumerate the routing/proof decision points that live *only* in Rust now:
   `crates/swe-seed-core/src/route/mod.rs` and `routing_gate.rs` (route
   selection, proof-command construction/execution, proof-result
   interpretation). Confirm each has a committed behavioral vector that would
   fail if the logic drifted: `tests/route_golden.rs`, `tests/routing_gate.rs`,
   `tests/proof_evidence.rs`. Add a teeth-check (mutate one routing decision,
   confirm the relevant test fails) — this is the real "parity" guarantee now
   that there is no second implementation to compare against.
3. Record the decision in SWE_SEED's `AGENTS.md` (a short "Canonical
   implementation" note): SWE_SEED is Rust-only; the Python harness was
   removed in the Phase 10 cutover; do not re-introduce a second
   routing/proof implementation — extend the Rust one instead.
4. Reconcile the stale naming: `tests/federation_parity.rs` and the
   "matches the Python adapter contract" / "Python parity" doc-comments in
   `envelope.rs`, `emit.rs`, and the test file describe a Python counterpart
   that no longer exists. That test is a pure-Rust behavioral self-test (it
   asserts the Rust fallback hash equals a hardcoded `PYTHON_FALLBACK`
   constant and exercises Rust emit/consume/drift/gate). Either rename it to
   reflect what it is (e.g. `federation_behavior.rs`) or, at minimum, correct
   its doc-comment so it no longer implies a live two-implementation parity
   obligation. Keep the hardcoded hash constant as a regression guard for
   the historical porting reference; just stop calling it "Python parity".

### Gate

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test -p swe-seed-core routing          # routing_gate + route_golden vectors
cargo test -p swe-seed-core federation       # federation self-test (renamed or recommented) still green
grep -n "Canonical implementation" /home/sprime01/projects/SWE_SEED/AGENTS.md
just ci
```

**Done when:** AGENTS.md carries the canonical-implementation note, the
routing/proof vectors pass, the teeth-check proves at least one routing test
fails when its decision is mutated, and no remaining Rust doc-comment or test
name asserts a Python implementation that the repo no longer contains.

**Redesign trigger:** if a routing decision point turns out to have *no*
committed behavioral vector (i.e. it is untested), do not fabricate a fake
"parity" test — add a real golden/gate test for that decision, or explicitly
flag the gap as a follow-up rather than claiming coverage that doesn't exist.

---

## Task 3 — Document the SWE_SEED-proves / godspeed_agent-settles ownership boundary (LE-09)

**Goal:** A short doc in SWE_SEED's `AGENTS.md` states explicitly: SWE_SEED owns
`ProofStarted`/`ProofCompleted` (proof execution and result), godspeed_agent
owns `SettlementRecorded` (judging the proof's consequence) — closing the
ambiguous-ownership gap the audit found.

**Why this shape:** LE-09: "both touch proof/settlement... ambiguous
ownership... duplicated/conflicting logic." WP-8's stated acceptance: "one
owner per event type documented in AGENTS.md of each repo."

### Steps

1. Read SWE_SEED's current `AGENTS.md` (Rust-focused; no "Event ownership"
   section exists today) for where such a convention should land.
2. Add an explicit "Event ownership" section: "SWE_SEED emits and owns
   `WorkRequested`, `ContextRequired`, `RouteSelected`, `ProofStarted`,
   `ProofCompleted`. It does not classify settlement — that's
   godspeed_agent's `SettlementRecorded`, consumed via `EvidenceRecorded`."
3. Grep SWE_SEED's Rust source for any code that constructs a
   `SettlementRecorded` payload *as a classification* (rather than emitting
   `EvidenceRecorded` and letting godspeed_agent classify it) — if found,
   that's the "duplicated logic" LE-09 warns about; either remove it or
   confirm it's a consume-side type reference and document it. Note:
   `consume_settlement_recorded()` in `consume.rs` is a *consumer* (reading
   godspeed's event), not classification logic — that is correct and stays.

### Gate

```bash
cd /home/sprime01/projects/SWE_SEED
grep -n "Event ownership" AGENTS.md
rg -n "SettlementRecorded" crates/ --type rust
```

**Done when:** the ownership section is present and greppable, and the second
grep's hits are all consume-side type references (e.g.
`consume_settlement_recorded`) — not SWE_SEED constructing/classifying a
settlement. If any hit is genuine classification logic, either remove it or
document why it isn't duplication.

**Redesign trigger:** if SWE_SEED does have legitimate settlement-adjacent
logic that isn't actually duplicating godspeed_agent's classification (e.g.
it's proof-result interpretation that merely *feeds* settlement, not
settlement itself), document that distinction explicitly in the
"Event ownership" section rather than deleting working code.

---

## Final acceptance checklist (whole plan)

- [ ] Rust federation-path envelopes serialize v1-conformant; the standalone
      path is proven byte-identical before/after. *(Task 1)*
- [ ] Rust is declared the sole canonical routing/proof implementation in
      `AGENTS.md`; routing/proof decision points have committed behavioral
      vectors with a teeth-check; no stale "Python parity" naming implies a
      second implementation exists. *(Task 2)*
- [ ] `AGENTS.md` documents the SWE_SEED-proves / godspeed_agent-settles
      boundary; no undocumented settlement-classification code remains.
      *(Task 3)*
- [ ] All global gates green (`cargo test --workspace`, `just lint`, `just ci`).
- [ ] `federation_signing.rs` and `federation_verify_sea.rs` still pass
      unmodified — this plan didn't touch signing logic or the canonical
      string.

## Guardrails (do not violate)

- **Never make the standalone inner loop depend on federation being
  reachable.** Every task's changes live inside `federation.enabled = true`
  branches or are pure documentation. `dispatch()` in `emit.rs` is the
  reference implementation of this invariant — it touches the sink only when
  `cfg.emits()`.
- Do not modify `signing.rs`'s canonical-string construction or the existing
  Ed25519 parity vectors (`federation_signing.rs`, `federation_verify_sea.rs`)
  — those are already correct per the audit and out of this plan's scope. If
  v1 envelope conformance appears to require a canonical-string change, stop
  and raise it.
- **Rust is canonical; do not re-introduce a Python implementation.** The
  Python harness was removed in the Phase 10 cutover; LE-05 is resolved by
  collapse-to-one, not by maintaining two impls. Any "Python" reference that
  remains in SWE_SEED doc-comments/test names is stale naming to be corrected
  (Task 2), not a signal to rebuild a second implementation.
- Commit hygiene: keep Task 1 (envelope), Task 2 (canonical confirmation +
  naming), and Task 3 (docs) as separate commits/PRs — Task 1 is the
  highest-risk because it touches the serialized contract.
