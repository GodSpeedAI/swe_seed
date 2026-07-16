# Implementation Plan — SWE_SEED: Envelope Convergence, Parity, and Ownership Boundary

**Created:** 2026-07-01

**Source of truth:** `SEA/.agents/reports/syntelligent_infrastructure.md` —
implements LE-01 (SWE_SEED's Python-adapter half), LE-05, LE-09, §4.5
("Federation is optional and default-off"), §13 WP-1, WP-4, WP-8. Quote it,
don't paraphrase from memory.

**Originating context:** SWE_SEED is a **sovereign node** — its inner loop
(`ingest → normalize → register → project → verify`) works standalone with
federation off by default (`[federation].enabled = false`), and this must
never change: "any code path that can break the inner loop when the
federation is unreachable is a defect" (spec 0011, quoted directly). This
plan's tasks all operate *inside* the federation boundary — they change what
gets emitted when federation is on, never the standalone path's behavior.

**Status of the work today:** The Rust federation core
(`crates/swe-seed-core/src/federation/`) is the most rigorous contract in the
whole ecosystem — mutual Ed25519 signing, a committed cross-language parity
vector, real hash resolution with drift detection. What's missing: (a) the
Python `.agent-harness` side still builds the non-conformant family-A
envelope shape when it talks to SEA, (b) two parallel implementations
(Python harness, Rust core) exist with no declared canonical owner or parity
test for routing/proof logic specifically (signing already has parity
tests — routing doesn't), (c) no documented boundary between "SWE_SEED
proves" and "godspeed_agent settles."

---

## 0. How to use this plan (agent operating instructions)

- Task 1 (envelope conformance) and Task 3 (ownership doc) are independent.
  Task 2 (Python/Rust parity) is independent but larger — do it after Task 1
  so the envelope-shape change doesn't have to be re-applied to two code
  paths mid-parity-work.
- **Every task ends with a verification gate.** Do not mark a task done
  until its gate command exits 0.
- **Core principle: the sovereign inner loop must never require federation.**
  Every change in this plan happens inside `[federation].enabled = true`
  code paths. If a task's steps would touch `ingest`/`normalize`/`register`/
  `project`/`verify` in a way that adds a hard dependency on SEA/NATS
  reachability, stop — that violates spec 0011's hard invariant and is out
  of scope for every task below.
- Match surrounding code style: `crates/swe-seed-core/src/federation/` for
  Rust (existing `envelope.rs`, `signing.rs`, `flags.rs`, `consume.rs` are
  the idiom to mirror); `.agent-harness/` for the Python side.
- `crates/swe-seed-core/src/federation/federation_parity.rs` (the existing
  Rust↔Python signing parity vector) is the pattern to replicate for Task 2
  — it is the proof style this repo already trusts.

### Global verification gates (must stay green after EVERY task)

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --workspace
just ci-test-python     # or the equivalent Python test target in justfile
just lint
```

⟨No codegen/rebuild step is required unless a task touches
`crates/swe-seed-core/`'s public API, in which case:⟩

```bash
cargo build --workspace
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| Sovereign inner loop (must never require federation) | `.agent-harness/`, `scripts/harness.py` — `ingest → normalize → register → project → verify` |
| Federation config gates | `crates/swe-seed-core/src/federation/flags.rs` — `authority_mode`/`context_mode`/`settlement_mode`, `emits()`, `consumes()`, `is_standalone()`, `authority_gate()` |
| Federation default | `[federation].enabled = false` (default-off, per spec 0011) |
| Ed25519 signing (already rigorous — do not modify signing logic) | `crates/swe-seed-core/src/federation/signing.rs`; canonical string `{namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}` |
| Existing Rust↔Python parity test (the pattern Task 2 replicates) | `crates/swe-seed-core/src/federation/federation_parity.rs`, `federation_signing.rs`, `federation_verify_sea.rs` |
| Hash resolution (Rust side, already correct) | `crates/swe-seed-core/src/federation/envelope.rs::resolve_domain_model_hash()` |
| Drift check (already correct) | `crates/swe-seed-core/src/federation/consume.rs::check_drift()` |
| Python harness routing/proof implementation (needs parity coverage) | `scripts/harness.py`, `.agent-harness/` |
| Rust core routing/proof implementation (the likely canonical target, pending decision) | `crates/swe-seed-core/` |
| Target v1 envelope schema (external, do not modify — conform to it) | `hassos-addon-agent-memory-ledger/.../contracts/sea.agent.event.v1.json` — required `schema_version`(`"v1"`), `event_id`, `source_agent`, `occurred_at`, `payload`; optional `idempotency_key`, `source`, `event_type`, `correlation_id`, `causation_id`, `trace_id`, `provenance{origin,chain}`, `metadata`; `additionalProperties:false` |
| Today's non-conformant Python envelope shape (mirrors SEA's `adapters.py`) | look for the equivalent `_event()`-style builder in `.agent-harness/` — same family-A shape `{event_id, event_type, namespace, occurred_at, payload}` |
| Federation is optional — spec quote to preserve verbatim in any docs this plan touches | spec 0011: "not a re-design of the SEA event contract" — SWE_SEED *conforms to* family A/C, it doesn't own the schema |

---

## Task 1 — Python `.agent-harness` federation-path envelopes conform to v1 (LE-01, SWE_SEED half)

**Goal:** When SWE_SEED's Python harness emits events toward SEA/agent-memory-ledger
with federation enabled, the envelope shape matches
`sea.agent.event.v1.json`, mirroring SEA repo's plan task 1's convergence on
the Rust side already being correct.

**Why this shape:** LE-01 lists SWE_SEED among the repos needing envelope
convergence. The Rust federation path (`envelope.rs`) already resolves the
hash correctly and signs correctly — this task only needs to add the
missing v1 top-level fields to the *Python* harness's event construction,
matching the same convergence SEA's plan applies to its own adapters.

### Steps

1. Find the Python harness's event-envelope builder (grep `.agent-harness/`
   and `scripts/harness.py` for `event_id`/`event_type`/`namespace`/
   `occurred_at` together, following the same shape as SEA's
   `adapters.py:19-26`).
2. Add `schema_version: "v1"`, `source_agent` (SWE_SEED's own agent
   identifier), `idempotency_key` (default to `event_id`), and thread
   `correlation_id`/`causation_id` from the harness's existing work-request
   context if available.
3. Keep the Ed25519 signature as a `provenance`/payload sub-object per the
   audit's recommendation #2, not a new top-level field.
4. Only apply this inside the `federation.enabled` branch — confirm via a
   test that the standalone (`federation.enabled = false`) path is
   byte-for-byte unaffected by this change (its local events never need v1
   conformance since they never leave the node).

### Gate

```bash
cd /home/sprime01/projects/SWE_SEED
just ci-test-python -k "envelope or v1_conformance"
cargo test --workspace federation   # confirm Rust side untouched/still green
```

**Done when:** the v1-conformant event builds correctly with a test proving
it against the required-field set, AND a separate test proves the standalone
path's behavior is byte-identical before/after this change (run the
standalone inner loop, diff output).

**Redesign trigger:** none plausible — this is additive field construction
inside an already-isolated federation code branch.

---

## Task 2 — Python/Rust routing-and-proof parity (LE-05)

**Goal:** One implementation (Rust
`swe-seed-core`) is declared canonical for routing/proof logic; the other has
a parity test suite analogous to the existing `federation_parity.rs` signing
vector, proving the two stay behaviorally identical (or the non-canonical
one is scheduled for retirement, whichever the owner decides).

**Why this shape:** LE-05: "two implementations of routing/proof... behavioral
drift, hard bugs." This repo already has the *proof pattern* for this exact
problem (signing parity) — reuse it rather than inventing a new
verification style.

### Steps

1. Enumerate the routing/proof logic present in both `scripts/harness.py`/`.agent-harness/`
   (Python) and `crates/swe-seed-core/` (Rust) — list each decision point
   (route selection, proof command construction/execution, proof result
   interpretation).
2. Get (or make, if this is a pre-decided architectural direction already
   implied elsewhere in the repo, e.g. by which one is actively developed)
   an explicit canonical-implementation decision. If genuinely undecided,
   default to Rust as canonical (matches the pattern already set for
   signing/hash-resolution, both of which are Rust-canonical with Python as
   the older/legacy path per the audit's framing).
3. Build a parity test suite mirroring `federation_parity.rs`'s structure: a
   committed set of routing scenarios (input work requests) with expected
   route/proof-command output, run through both implementations, asserting
   identical results.
4. If full parity isn't achievable within this task's reach (e.g. one
   implementation has legitimately diverged in scope), document the
   divergence explicitly rather than forcing a false-positive parity test.

### Gate

```bash
cd /home/sprime01/projects/SWE_SEED
cargo test --workspace routing_parity
just ci-test-python -k "routing_parity"
```

**Done when:** the parity suite passes for the enumerated scenarios, and
deliberately diverging one implementation's routing logic for one scenario
makes the parity test fail — proving it's a real behavioral comparison, not
a schema-shape check.

**Redesign trigger:** if the two implementations have diverged so far that
true parity is a multi-week rewrite, narrow this task's deliverable to (a)
the canonical-implementation decision, documented, and (b) parity tests for
only the subset of scenarios both implementations still agree on today —
flag the remainder as a follow-up plan rather than blocking this task
indefinitely.

---

## Task 3 — Document the SWE_SEED-proves / godspeed_agent-settles ownership boundary (LE-09)

**Goal:** A short doc in this repo's `AGENTS.md` (or equivalent) states
explicitly: SWE_SEED owns `ProofStarted`/`ProofCompleted` (proof execution
and result), godspeed_agent owns `SettlementRecorded` (judging the proof's
consequence) — closing the ambiguous-ownership gap the audit found.

**Why this shape:** LE-09: "both touch proof/settlement... ambiguous
ownership... duplicated/conflicting logic." WP-8's stated acceptance: "one
owner per event type documented in AGENTS.md of each repo."

### Steps

1. Read this repo's current `AGENTS.md` (or `justfile`/`README.md` if that's
   where such conventions live) for any existing ownership language.
2. Add an explicit "Event ownership" section: "SWE_SEED emits and owns
   `WorkRequested`, `ContextRequired`, `RouteSelected`, `ActionProposed`,
   `ProofStarted`, `ProofCompleted`. It does not classify settlement —
   that's godspeed_agent's `SettlementRecorded`, consumed via
   `EvidenceRecorded`."
3. Grep this repo's source for any code that constructs a
   `SettlementRecorded`-shaped payload directly (rather than emitting
   `EvidenceRecorded` and letting godspeed_agent classify it) — if found,
   that's the "duplicated logic" LE-09 warns about; either remove it or
   confirm it's dead code and delete it.

### Gate

```bash
grep -n "Event ownership" /home/sprime01/projects/SWE_SEED/AGENTS.md
grep -rn "SettlementRecorded" /home/sprime01/projects/SWE_SEED --include="*.py" --include="*.rs"
```

**Done when:** the ownership section is present and greppable, and the
second grep either returns nothing (SWE_SEED never constructs
`SettlementRecorded` directly) or every hit is a documented pass-through/type
reference, not classification logic.

**Redesign trigger:** if SWE_SEED does have legitimate settlement-adjacent
logic that isn't actually duplicating godspeed_agent's classification (e.g.
it's proof-result interpretation that merely *feeds* settlement, not
settlement itself), document that distinction explicitly in the
"Event ownership" section rather than deleting working code.

---

## Final acceptance checklist (whole plan)

- [ ] Python harness's federation-path envelopes are v1-conformant; the
      standalone path is proven byte-identical before/after. *(Task 1)*
- [ ] A canonical routing/proof implementation is declared; a parity test
      suite (mirroring `federation_parity.rs`) proves the two implementations
      agree on the enumerated scenarios, with a teeth-check. *(Task 2)*
- [ ] `AGENTS.md` documents the SWE_SEED-proves / godspeed_agent-settles
      boundary; no dead/duplicated settlement-classification code remains
      undocumented. *(Task 3)*
- [ ] All global gates green (`cargo test --workspace`, Python tests, lint).
- [ ] `federation_parity.rs` and sibling signing-parity tests still pass
      unmodified — this plan didn't touch signing logic.

## Guardrails (do not violate)

- **Never make the standalone inner loop depend on federation being
  reachable.** Every task's changes live inside `federation.enabled = true`
  branches or are pure documentation.
- Do not modify `signing.rs`'s canonical-string construction or the existing
  Ed25519 parity vectors — those are already correct per the audit and out
  of this plan's scope.
- Do not unilaterally declare Python or Rust canonical in Task 2 without
  checking for an existing project-level signal first (recent commit
  activity, `AGENTS.md` hints) — Rust-as-default is this plan's fallback,
  not an override of an existing decision.
- Commit hygiene: keep Task 1 (envelope), Task 2 (parity), and Task 3 (docs)
  as separate commits/PRs — Task 2 is the highest-risk and most likely to
  need its own review cycle.
