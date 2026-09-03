# Explanation: Why Host Projection Over a Custom Runtime?

This document explains why SWE_SEED projects canonical specifications into host-native tool configuration files rather than running a custom agent wrapper or building a proprietary IDE runtime.

---

## 1. The Core Question

Many agent frameworks build custom desktop applications, custom terminal front-ends, or IDE forks to govern agent behavior. Why does SWE_SEED avoid wrapping the agent runtime, instead compiling canonical specifications into host-native configuration files (`.claude/settings.json`, `.github/copilot-instructions.md`, etc.) via `swe-seed sync`?

---

## 2. Confirmed Design Rationale

### A. Meeting Engineers Where They Already Work
Developers already have established workflows with tools like Claude Code, GitHub Copilot, Codex, and Antigravity. Requiring teams to abandon their preferred tools in favor of a proprietary wrapper creates adoption friction. By projecting canonical rules directly into the host's native configuration files, developers continue using their existing tools without modification.

### B. Avoiding the "Fork and Wrapper Maintenance Trap"
AI coding assistants evolve rapidly. Building a wrapper around Claude or Copilot creates an ongoing maintenance burden: upstream CLI changes, authentication updates, or UI redesigns constantly break wrapper layers. Projection treats the host tool as a compiler target: SWE_SEED emits standard markdown, JSON, or YAML, leaving runtime execution to the host vendor.

### C. Honest Degradation via Capability Matrices
Different host tools provide different levels of capability:
- Some hosts support pre-tool execution hooks that can strictly block dangerous commands.
- Other hosts only support prompt-injection instructions (advisory guidance).

By using explicit host adapters, SWE_SEED documents the exact enforcement strength (`Strict`, `Advisory`, or `Unsupported`) in `swe-seed hosts`. This prevents the dangerous illusion that advisory prompts provide guaranteed security enforcement.

### D. Preserving Developer Edits via Managed Blocks
Using delimited markers (`<!-- BEGIN SWE_SEED MANAGED BLOCK -->` and `<!-- END SWE_SEED MANAGED BLOCK -->`), SWE_SEED updates canonical doctrine while leaving human customizations outside the markers untouched.

---

## 3. Trade-offs and Consequences

| Trade-off | Description |
|---|---|
| **Enforcement Limitations** | In hosts that lack native lifecycle hooks (e.g. Copilot prompt files), SWE_SEED cannot forcefully block commands before execution; it must rely on downstream CI proof gates. |
| **Drift Risk** | Because files reside in the workspace, users might edit projected blocks directly. Addressed by `swe-seed doctor --host <host>` which flags drift. |

---

## 4. Invariants Protected

1. **Non-Destructive Projection**: Content outside managed block delimiters is never overwritten by `swe-seed sync`.
2. **Deterministic Regeneration**: `swe-seed sync` is idempotent; running it multiple times produces identical output.

---

## 5. Source Trail

- `crates/swe-seed-core/src/adapters/host.rs`: `HostAdapter` trait.
- `crates/swe-seed-core/src/adapters/marker.rs`: Managed block delimiter engine.
- `crates/swe-seed-core/src/adapters/capabilities.rs`: `HostCapabilityMatrix`.
- `docs/specs/0004-host-adapter-contract.md`: Specification.
