# Tutorial: Your First Routed Task

This tutorial guides you through executing a real task under SWE_SEED from beginning to end: selecting a route, planning bounded context, tracking execution with a trace, running proof, and verifying the routing gate.

---

## 1. Prerequisites

Before starting, ensure your local environment is initialized:
- Rust 1.75+ and `just` installed.
- Repository bootstrapped:
  ```bash
  just bootstrap
  just doctor
  ```
- Harness integrity verified:
  ```bash
  just harness-validate
  ```

---

## 2. Step 1: Select the Route and Record Genesis

Imagine you are asked to resolve a bug: *"fix the checkout race condition"*.

Execute the semantic router with `--record` to bind the work contract and establish trace genesis:

```bash
just harness-route-record "fix the checkout race condition"
```

### Expected Output:
```json
{
  "job_type": "bugfix",
  "route_card": ".agent-harness/routes/bugfix.json",
  "required_context": [
    "AGENTS.md",
    ".agent-harness/playbooks/30-debug-from-symptom.md",
    "docs/dev-harness/README.md",
    ".agent-harness/memory/constraints.md"
  ],
  "required_skills": [
    "debug-discipline"
  ],
  "work_loop": [
    "establish reliable reproduction before editing code",
    "trace the fail path end-to-end to identify root cause",
    "apply minimal fix addressing the root cause",
    "run proof commands and observe passing output"
  ],
  "required_artifacts": [
    "reliable reproduction",
    "observed failure",
    "root cause note",
    "targeted fix",
    "passing proof output"
  ],
  "proof": [
    "just ci"
  ],
  "done_when": [
    "original failure no longer reproduces",
    "root cause is connected to the fix",
    "proof commands pass with zero exit code"
  ],
  "next_action": "Read required context, then execute work_loop[0]: establish reliable reproduction before editing code"
}
```

Notice that the task is classified as `bugfix`, the required proof command is pre-declared as `just ci`, and the genesis event is committed to `.agent-harness/traces/ledger.db`.

---

## 3. Step 2: Inspect Bounded Context

Before reading random repository files, check the bounded context pack:

```bash
just harness-context-plan "fix the checkout race condition"
```

### Expected Output:
```text
Context pack for 'fix the checkout race condition':
  1. AGENTS.md
  2. .agent-harness/playbooks/30-debug-from-symptom.md
  3. docs/dev-harness/README.md
  4. .agent-harness/memory/constraints.md
```

Read these files first. They contain the debugging discipline and operational constraints for this repository.

---

## 4. Step 3: Start the Trace

Open a new execution trace:

```bash
just harness-trace-start "fix the checkout race condition"
```

### Expected Output:
```text
Started trace: 20260902T204500Z-fix-checkout-race
Record: .agent-harness/traces/records/20260902T204500Z-fix-checkout-race.json
```

Save the trace ID printed (referred to here as `<trace_id>`).

---

## 5. Step 4: Record Progress Checkpoints

As you investigate and write code, record durable checkpoints so the task state is preserved:

```bash
just harness-trace-checkpoint <trace_id> "reproduction" "Identified race condition in lock contention test"
```

Inspect the JSON file at `.agent-harness/traces/records/<trace_id>.json` to observe the new milestone.

---

## 6. Step 5: Execute Proof Commands

Run the proof command declared in the work contract:

```bash
just ci
```

Observe the output. All formatting, linting, harness validation, and cargo tests will execute. Confirm that the final line states:
```text
All checks passed!
```

---

## 7. Step 6: Finish the Trace with Evidence

Seal the trace by submitting your completion claim alongside the proof evidence:

```bash
just harness-trace-finish <trace_id> "Resolved lock contention race condition" "just ci" "pass"
```

---

## 8. Step 7: Verify the Routing Gate

Verify that the trace satisfies the tamper-evident routing gate:

```bash
swe-seed gate <trace_id> --verify
```

### Expected Output:
```text
Gate check PASSED for trace '20260902T204500Z-fix-checkout-race'
Genesis: RouteSelected (.agent-harness/routes/bugfix.json)
Chain integrity: 4/4 events cryptographically verified (SHA-256)
```

The process exits with code 0. Your task is verified and eligible for merge.

---

## 9. Next Steps

- Explore the [Troubleshooting Guide](../troubleshooting.md) to understand what happens if a gate check fails.
- Learn about the [Trace Ledger & Gate Subsystem](../subsystems/trace-ledger.md) to understand the underlying SHA-256 hash chaining.
