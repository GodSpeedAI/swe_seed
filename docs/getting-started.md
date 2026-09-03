# Getting Started with SWE_SEED

This guide walks through establishing a verified local development environment and running your first routed command with SWE_SEED.

---

## 1. Prerequisites

### Required Tools
- **Rust Toolchain**: Rust 1.75+ (including `cargo`, `rustc`). Verify with `cargo --version`.
- **Just**: Command runner used for developer workflows. Verify with `just --version`.
- **Git**: Modern version control system.

### Recommended (Optional) Tools
The repository operates standalone without these, but they are recommended for extended workflows:
- **devbox** or **mise**: Environment and package management.
- **sops** and **age**: For encrypting and decrypting federation private keys at rest.
- **pnpm**: For Prettier formatting checks (`package.json`).
- **uv**: For Python-based linting checks (`ruff`).

---

## 2. Environment Setup

Clone the repository and enter the workspace:

```bash
git clone <repository-url> swe-seed
cd swe-seed
```

Initialize the environment and verify toolchain availability:

```bash
# Bootstrap local tools (mise, pnpm, uv if installed)
just bootstrap

# Run doctor checks to verify required commands
just doctor
```

Expected output from `just doctor`:
```text
Required development commands are available
```
If any recommended tools are missing, `just doctor` reports them as notices without blocking execution.

---

## 3. Verify Harness Integrity

Before performing work, verify that the internal specifications, route cards, and contract schemas are structurally sound:

```bash
just harness-validate
```

Expected output:
```text
Harness validation passed
```

This command executes `swe-seed harness`, running static validation over:
- Root specification presence (`SWE_SEED_SPEC_v0.2.0.md`, `HARNESS_SPEC.md`, `FABRICATOR_SPEC_v0.1.0.md`).
- Canonical BAML contract definitions in `.agent-harness/baml/baml_src/`.
- The 11 mandatory route cards in `.agent-harness/routes/`.
- Absence of incomplete work markers in specs and source files.

---

## 4. Run Your First Routed Command

SWE_SEED mandates that all tasks begin with routing. To route a bugfix request:

```bash
just harness-route "fix the failing checkout test"
```

Behind the scenes, this runs:
```bash
cargo run -q -p swe-seed -- route "fix the failing checkout test"
```

### Understanding the Output

The router prints a structured JSON work contract:

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

This output forms the pre-execution contract:
- **job_type**: Specifies which operational pattern applies (`bugfix`).
- **required_context**: Exactly what files the agent must load into memory before editing code.
- **work_loop**: Ordered stages required to complete the task.
- **proof**: The exact verification command (`just ci`) that must pass before completion can be claimed.

---

## 5. Planning Bounded Context

To inspect the bounded context pack computed for this task:

```bash
just harness-context-plan "fix the failing checkout test"
```

Expected output:
```text
Context pack for 'fix the failing checkout test':
  1. AGENTS.md
  2. .agent-harness/playbooks/30-debug-from-symptom.md
  3. docs/dev-harness/README.md
  4. .agent-harness/memory/constraints.md
```

This ensures the agent reads only the files relevant to the bugfix route rather than flooding its context window with the entire repository.

---

## 6. Running Local Verification (Proof)

The default proof command declared across implementation routes is `just ci`. Execute the local verification suite:

```bash
just ci
```

`just ci` executes in sequence:
1. `doctor`: Verifies toolchain presence.
2. `format`: Checks formatting with Prettier (if `pnpm` is installed).
3. `lint`: Executes Python linter `ruff` (if `uv` is installed).
4. `test`:
   - Runs `swe-seed harness` validation.
   - Executes the full Rust test suite (`cargo test`).
   - Builds the release binary (`cargo build --release`).
   - Runs CLI golden tests (`tests/cli_golden.rs`).

---

## 7. Next Steps

Now that your local environment is verified:
- Review the [System Mental Model](mental-model.md) to understand how the three layers interact.
- Read the [Architecture Overview](architecture.md) for deep logical and runtime models.
- Step through the [First Routed Task Tutorial](tutorials/first-routed-task.md) to record and gate a real task end-to-end.
- Consult the [CLI Reference](reference/cli.md) for direct `swe-seed` commands.
