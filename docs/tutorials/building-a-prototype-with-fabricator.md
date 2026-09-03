# Tutorial: Building a Prototype with Fabricator

This tutorial guides you through using the Fabricator layer to convert an informal product need into a fully validated 10-node specification chain, ready for coding agent execution.

---

## 1. Prerequisites

- Environment initialized and verified (`just doctor`).
- Clean repository working tree.

---

## 2. Step 1: Initialize a Fabricator Run

Execute the `fabricate` command with a concise description of your product need:

```bash
cargo run -q -p swe-seed -- fabricate "CLI tool that parses markdown headers and emits a table of contents"
```

### Expected Output:
```text
Initialized Fabricator run: 20260902T210000Z-md-toc
Run directory: .fabricator/runs/20260902T210000Z-md-toc/
Deriving semantic specification chain...
  [1/10] ProductSeed generated
  [2/10] JobStory generated
  [3/10] ProductHypothesis generated
  [4/10] PRD generated
  [5/10] ProductADR generated
  [6/10] EARSRequirements generated
  [7/10] SoftwareDesignSpec generated
  [8/10] GherkinScenarios generated
  [9/10] TDDPlan generated
  [10/10] AgentTask generated
Run generation completed.
```

Take note of the run identifier (referred to here as `<run_id>`).

---

## 3. Step 2: Inspect Generated Artifacts

List the files created in the run directory:

```bash
ls -la .fabricator/runs/<run_id>/
```

Key artifacts to inspect:
- `PRODUCT_SEED.md`: Bounded statement of user, problem, and constraints.
- `JOB_STORY.md`: Standardized user situation and motivation.
- `EARS_REQUIREMENTS.md`: Structured requirements using Ubiquitous and Event-Driven patterns.
- `GHERKIN_SCENARIOS.feature`: Behavioral acceptance tests (`Given/When/Then`).
- `AGENT_TASK.md`: The execution brief for the coding agent.

Notice that every file contains frontmatter linking it directly to the previous file via a `TraceabilityLink`.

---

## 4. Step 3: Validate Semantic Chain Integrity

Before handing off the task to an agent, verify that the chain has no missing links or syntax errors:

```bash
cargo run -q -p swe-seed -- fabricate validate-chain <run_id>
```

### Expected Output:
```text
Validating semantic chain for run '20260902T210000Z-md-toc'...
  Check 1: Node presence (10/10 present) ......................... PASS
  Check 2: Upstream link integrity ............................... PASS
  Check 3: Downstream link continuity ............................ PASS
  Check 4: EARS syntax compliance ................................ PASS
  Check 5: Gherkin scenario traceability ......................... PASS

Validation Result: PASSED (0 findings)
Task is approved for agent handoff.
```

If any link had been broken or corrupted, the validator would have printed a `SemanticChainValidationReport` detailing the gap and exited with status 1.

---

## 5. Step 4: Inspect the Frozen EvalSpec

The handoff package contains a frozen evaluation specification:

```bash
cat .fabricator/runs/<run_id>/EVAL_SPEC.yaml
```

Notice that the verification checks (e.g. running the generated CLI against a sample markdown file) are frozen before the agent writes a line of code.

---

## 6. Next Steps

- Learn more about the [Fabricator Semantic Chain Subsystem](../subsystems/fabricator-semantic-chain.md).
- Read the [Fabricator Specification](../../FABRICATOR_SPEC_v0.1.0.md) for deeper schema details.
