# How-To: Add a Route Card

This guide explains how to add a new `RouteCard` to SWE_SEED, register its semantic triggers, and validate it against harness integrity rules.

---

## 1. Goal

Introduce a new standardized work pattern (for example, a custom migration or benchmark task) that binds specific context, an ordered work loop, expected artifacts, and verification commands.

---

## 2. Prerequisites

- Environment initialized (`just doctor`).
- Clean repository working tree.

---

## 3. Procedure

### Step 1: Create the Route Card JSON File
Create a new file in `.agent-harness/routes/<card-id>.json`. For example, `.agent-harness/routes/benchmark.json`:

```json
{
  "id": "benchmark",
  "job_type": "benchmark",
  "purpose": "Execute performance profiling and regression benchmarks against system targets.",
  "semantic_triggers": [
    "benchmark",
    "perf",
    "profile",
    "latency",
    "throughput"
  ],
  "positive_examples": [
    "run latency benchmarks for the router",
    "profile memory allocation in gateway proxy"
  ],
  "negative_examples": [
    "fix the failing unit test",
    "document the CLI commands"
  ],
  "required_context": [
    "AGENTS.md",
    "docs/dev-harness/README.md",
    ".agent-harness/memory/constraints.md"
  ],
  "required_skills": [],
  "work_loop": [
    "verify baseline benchmark environment",
    "execute benchmark suite under quiet conditions",
    "record throughput and latency metrics",
    "compile comparative analysis note"
  ],
  "required_artifacts": [
    "benchmark output",
    "comparative metrics note"
  ],
  "proof": [
    "cargo bench --no-run"
  ],
  "done_when": [
    "benchmarks complete without errors",
    "metrics are recorded in trace ledger"
  ],
  "failure_modes": [
    "noisy neighbor interference during benchmark execution"
  ],
  "fallback_policy": "If performance metrics fluctuate wildly, repeat run after reboot."
}
```

### Step 2: Validate the Route Card
Execute the static harness validator:

```bash
just harness-validate
```

If the card JSON is malformed or missing required schema fields, `harness-validate` reports the exact line and error.

### Step 3: Verify Route Selection
Test that the router matches your new card:

```bash
cargo run -q -p swe-seed -- route "run latency benchmarks for the router"
```

Verify that the output selects `"job_type": "benchmark"`.

---

## 4. Common Failure Symptoms

- **Schema Validation Error**: `missing field 'proof'`: Ensure all mandatory fields from `harness.baml` are present.
- **Router Ignored Card**: If a competing card (e.g. `test.json`) matches instead, add more specific `positive_examples` to your card and add negative examples to the competing card.
