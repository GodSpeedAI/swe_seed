# Explanation: Why File-First Tamper-Evident Traces?

This document explains why SWE_SEED stores task execution traces as local, human-readable JSON files backed by an SQLite ledger with cryptographic SHA-256 hash chains, rather than relying on remote SaaS dashboards or central databases.

---

## 1. The Core Question

Many modern developer tools rely on hosted cloud platforms (OpenTelemetry collectors, Datadog, SaaS dashboards) to record agent execution logs. Why does SWE_SEED mandate that trace records live directly in repository files (`.agent-harness/traces/records/`) and use local cryptographic hash chaining?

---

## 2. Confirmed Design Rationale

### A. Local-First and Zero-Infrastructure
Software engineers work in varied environments: laptops, offline flights, secure corporate enclaves, and disconnected air-gapped networks. Requiring a remote server to record a trace would break local development. By storing traces as files in the repository:
- Any developer or CI runner can execute tasks with zero infrastructure setup.
- Traces can be inspected, diffed, committed, or archived using standard Unix tools (`cat`, `jq`, `grep`, `git`).

### B. Tamper-Evidence via Hash Chaining
If trace files were ordinary JSON files without cryptographic linking, a rogue agent or developer could easily edit `.agent-harness/traces/records/<id>.json` after the fact, forging a passing test result or fabricating a route selection.
By calculating:
$$H_n = \text{SHA-256}(H_{n-1} + \text{payload}_n)$$
any retrospective edit to a trace file alters the hash chain. The Routing Gate (`swe-seed gate <id> --verify`) recalculates the chain from the initial `RouteSelected` genesis; any discrepancy fails closed, rejecting the merge.

### C. Human-Readable Forensics
While SQLite (`ledger.db`) enforces cryptographic chain speed and indexing, the mirrored JSON files in `.agent-harness/traces/records/` provide immediate human transparency. A reviewer can open the JSON trace in their editor, inspect the timestamps and command evidence, and verify what occurred without needing specialized database query tools.

---

## 3. Trade-offs and Consequences

| Trade-off | Description |
|---|---|
| **Local Disk Space** | Accumulating hundreds of JSON traces consumes disk space. Mitigated by `swe-seed agent-hooks compact-logs` and gitignore policies for non-essential traces. |
| **Concurrent Writes** | Concurrent tasks writing to the same SQLite ledger must manage database locks. Handled via WAL mode and short backoff retries. |

---

## 4. Invariants Protected

1. **Tamper-Evidence**: Historical events cannot be altered or removed without invalidating the cryptographic chain.
2. **Offline Verifiability**: Traces can be fully verified in air-gapped CI environments without network access.

---

## 5. Source Trail

- `crates/swe-seed-core/src/trace_ledger.rs`: SHA-256 chaining and SQLite verification.
- `crates/swe-seed-core/src/routing_gate.rs`: `route_gate()` verification.
- `crates/swe-seed-core/src/trace/record.rs`: JSON trace record serialization.
- `docs/specs/0014-trace-and-durable-decisions.md`: Specification.
