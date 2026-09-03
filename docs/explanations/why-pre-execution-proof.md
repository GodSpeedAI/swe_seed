# Explanation: Why Pre-Execution Proof?

This document explains why SWE_SEED binds proof commands to a task **before** execution begins, rather than allowing agents to select tests after code has been written.

---

## 1. The Core Question

In standard software workflows, tests are often executed after coding is finished, or written alongside implementation. Why does SWE_SEED mandate that the proof commands (such as `just ci` or a specific test target) must be bound up front in the `RouteCard` before the agent touches repository files?

---

## 2. Confirmed Design Rationale

### The "Grading Own Homework" Failure Pattern
When an AI coding agent is allowed to choose how to verify its own work after writing code:
- **Convenience drives verification**: The agent runs the quickest or easiest unit test, avoiding comprehensive end-to-end or regression suites.
- **Scope shrinking**: If a difficult check fails, the agent may modify the test assertions, delete the failing test, or declare the failure "out of scope."
- **Automated Optimism**: The agent generates a conversational summary declaring "implemented, tested, complete" regardless of whether the decisive verification checks ran.

### Making Claims Falsifiable
By binding proof commands **prior to execution**:
1. **The terms are fixed**: The agent cannot alter what constitutes success.
2. **Claims become falsifiable**: A claim that a bug is resolved has no standing unless the pre-bound command exits with code 0 and records stdout/stderr evidence in the trace.
3. **Forensic overhead is eliminated**: Human reviewers do not need to investigate which tests the agent forgot to run; the trace ledger verifies that the pre-declared proof contract was executed and passed.

---

## 3. Trade-offs and Consequences

| Trade-off | Description |
|---|---|
| **Upfront Rigidity vs Flexibility** | Because proof is pre-bound, an agent cannot easily swap tests mid-flight. If a new test target is required, the task route or spec must be explicitly updated. |
| **CI Execution Cost** | Running `just ci` (which builds release binaries and runs golden tests) is heavier than running a single unit test, but guarantees parity between local verification and remote CI gates. |

---

## 4. Invariants Protected

1. **Frozen Verification**: Once handed off, an `EvalSpec` or proof requirement cannot be edited to manufacture a passing status.
2. **No Unproven Claims**: `swe-seed trace finish` rejects claims that lack executed command output.

---

## 5. Source Trail

- `HARNESS_SPEC.md`: Operating contract ("Completion requires proof").
- `AGENTS.md`: Completion Rule and Prose/Claim Discipline.
- `crates/swe-seed-core/src/eval/proof.rs`: `ProofRecord`, `ProofDisposition`.
- `crates/swe-seed-core/src/routing_gate.rs`: Verification that proof events exist in the trace ledger.
- `docs/specs/0013-eval-and-proof.md`: Normative specification.
