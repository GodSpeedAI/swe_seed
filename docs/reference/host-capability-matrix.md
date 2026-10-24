# Host Capability Matrix

This document provides the canonical comparison matrix of host adapters supported by SWE_SEED, detailing their actual enforcement capabilities across different AI coding tools.

---

## 1. Enforcement Levels Defined

- **`Strict`**: The host tool provides a native execution hook that intercepts actions and can programmatically block execution before tool calls run.
- **`Advisory`**: The host tool injects instructions into the model's prompt context. The model is advised to follow the rule, but the harness cannot forcefully intercept or block tool calls in flight.
- **`Unsupported`**: The host tool does not support this integration point.

---

## 2. Comparison Matrix

| Host Adapter | Pre-Prompt Routing | Pre-Tool Safety Blocking | Post-Tool Logging | Token Budget Containment | Managed Markers | Drift Detection | Snapshot Rollback |
|---|---|---|---|---|---|---|---|
| **Claude Code** (`claude`) | `Advisory` | `Strict` (via hooks) | `Strict` | `Advisory` | `Yes` | `Yes` | `Yes` |
| **GitHub Copilot** (`copilot`) | `Advisory` | `Unsupported` | `Unsupported` | `Advisory` | `Yes` | `Yes` | `Yes` |
| **Antigravity** (`antigravity`) | `Advisory` | `Strict` (via hooks) | `Strict` | `Strict` | `Yes` | `Yes` | `Yes` |
| **Codex** (`codex`) | `Advisory` | `Unsupported` | `Unsupported` | `Advisory` | `Yes` | `Yes` | `Yes` |
| **OpenCode** (`opencode`) | `Advisory` | `Advisory` | `Advisory` | `Advisory` | `Yes` | `Yes` | `Yes` |
| **CI / Automation** (`ci`) | `Strict` (gate) | `Strict` (gate) | `Strict` | `Strict` | `Yes` | `Yes` | `N/A` |

---

## 3. Host Adapter Profiles

### Claude Code (`claude`)
- **Target Files**: `.claude/settings.json`, `CLAUDE.md`.
- **Strengths**: Supports rich hook configurations in `settings.json`, enabling pre-tool execution checks and post-tool event emission.
- **Limitations**: Top-level prompt routing remains advisory; agents must be instructed to run `swe-seed route`.

### GitHub Copilot (`copilot`)
- **Target Files**: `.github/copilot-instructions.md`.
- **Strengths**: Broad native editor integration in VS Code and JetBrains.
- **Limitations**: Copilot instructions are prompt-injected only; no pre-tool execution blocking exists. Downstream verification must rely on CI gates (`just ci`).

### Antigravity (`antigravity`)
- **Target Files**: `.agent-rules/`, Antigravity guidelines.
- **Strengths**: Deep integration with agentic coding hooks, supporting strict pre-tool approval gates and automated context budgeting.
- **Limitations**: Requires Antigravity environment.

### CI Engine (`ci`)
- **Target Files**: `.github/workflows/ci.yml`, `scripts/ci.sh`.
- **Strengths**: Absolute enforcement authority. Pull requests cannot merge without passing `swe-seed gate <trace> --verify` and `just ci`.
- **Limitations**: Evaluates after changes are committed rather than interactively during generation.

---

## 4. Querying the Matrix from the CLI

To view the live capability matrix directly in your terminal:

```bash
cargo run -q -p swe-seed -- hosts
```
