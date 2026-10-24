# Workflow: Task Routing to Proof

This document traces the complete execution lifecycle of a coding task under SWE_SEED governance, from initial intent routing to proof verification and gate closure.

---

## 1. Summary

Every material engineering change follows a closed verification loop: the task is routed to an explicit RouteCard, bounded context is computed to constrain intake, a cryptographic trace is initialized, work is performed according to the card's work loop, pre-bound proof commands are executed, the trace is finalized with command evidence, and the routing gate confirms ledger integrity before merging.

---

## 2. Sequence

1. **Routing**: Developer or agent runs `swe-seed route "<task>" --record`. The semantic router selects a RouteCard and writes a route-decision record to `.agent-harness/traces/route-decisions/`. No ledger entry is created at this step.
2. **Context Planning**: `swe-seed context-plan "<task>"` outputs the bounded context files required for the task.
3. **Trace Initialization**: `swe-seed trace start "<task>"` routes the task again, writes `.agent-harness/traces/records/<trace_id>.json`, and appends the `RouteSelected` genesis entry to `.agent-harness/traces/ledger.sqlite3`.
4. **Execution Iterations**: The agent reads the context files and performs edits. Key milestones are recorded via `swe-seed trace checkpoint <trace_id> --stage <stage> --summary <summary>`.
5. **Proof Execution**: The agent executes the pre-bound proof command (`just ci`).
6. **Trace Closure**: `swe-seed trace finish <trace_id> --claim "<summary>" --command "just ci" --result "pass"` appends the proof record and seals the trace.
7. **Gate Verification**: `swe-seed gate <trace_id> --verify` recomputes the SHA-256 hash chain and confirms the routing genesis. If valid, exits 0.

---

## 3. Detailed Path

- `crates/swe-seed-core/src/route/mod.rs`: `score_route()`, `infer_bootstrap_route()`, `build_route_result()`, `write_route_decision()`.
- `crates/swe-seed-core/src/context/pack.rs`: `plan_context()`.
- `crates/swe-seed-core/src/trace/lifecycle.rs`: `start()`, `finish()`.
- `crates/swe-seed-core/src/trace_ledger.rs`: `append()`, `verify_chain()`.
- `crates/swe-seed-core/src/routing_gate.rs`: `route_gate()`.

---

## 4. State Changes

- **Trace Record**: Creates `.agent-harness/traces/records/<trace_id>.json` containing task description, checkpoints, and proof record.
- **Trace Ledger**: Inserts rows into `.agent-harness/traces/ledger.sqlite3`, chaining SHA-256 hashes from genesis to completion.
- **Repository Source**: Target code files are modified by the agent to satisfy the task.

---

## 5. Failure Branches

- **Failing Proof**: `just ci` exits non-zero. The agent cannot claim completion; the trace remains open or is finished with `result = "fail"`.
- **Bypassed Routing**: A trace that was never opened with `swe-seed trace start` has no `RouteSelected` genesis in the ledger. `swe-seed gate` fails closed. Running `swe-seed route` alone, with or without `--record`, cannot satisfy the gate.
- **Hash Chain Mismatch**: Tampering with trace files breaks the SHA-256 chain; `verify_chain()` returns an error.

---

## 6. Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Dev as Developer / Host Agent
    participant Route as swe-seed route
    participant Context as swe-seed context-plan
    participant Trace as swe-seed trace
    participant CI as Proof Runner (just ci)
    participant Gate as swe-seed gate
    participant Ledger as ledger.sqlite3

    Dev->>Route: swe-seed route "task" --record
    Route->>Dev: Work Contract JSON + route-decisions/<id>.json

    Dev->>Context: swe-seed context-plan "task"
    Context-->>Dev: Bounded ContextPack

    Dev->>Trace: trace start "task"
    Trace->>Ledger: Insert RouteSelected genesis (seq 0)
    Trace-->>Dev: Initialized trace_id

    Dev->>Dev: Execute work loop stages

    Dev->>CI: Run pre-bound proof (just ci)
    CI-->>Dev: Exit 0 (Proof verified)

    Dev->>Trace: trace finish <id> --claim "done" --command "just ci"
    Trace->>Ledger: Insert TraceFinished event (claim + evidence)

    Dev->>Gate: swe-seed gate <id> --verify
    Gate->>Ledger: verify_chain(id)
    Ledger-->>Gate: Valid hash chain
    Gate-->>Dev: Exit 0 (Merge permitted)
```

---

## 7. Source Trail

- `crates/swe-seed-core/src/route/mod.rs`: Routing logic.
- `crates/swe-seed-core/src/trace/lifecycle.rs`: Trace state transitions.
- `crates/swe-seed-core/src/trace_ledger.rs`: Ledger persistence and hash chaining.
- `crates/swe-seed-core/src/routing_gate.rs`: `route_gate()` verification.
- `justfile`: `harness-route`, `harness-trace-*`, `gate-merge` recipes.
