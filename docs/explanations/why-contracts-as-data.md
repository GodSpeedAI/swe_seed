# Explanation: Why Contracts as Data?

This document explores the design rationale behind SWE_SEED's **Contracts-as-Data** architecture and the intentional exclusion of an LLM runtime from the core harness engine.

---

## 1. The Core Question

BAML (Boundary Anonymized Markdown Language) is commonly associated with prompt templating and LLM function calling. Why does SWE_SEED use `.baml` schema files (`swe_seed.baml`, `harness.baml`, `fabricator.baml`) purely as static data definitions, completely omitting an LLM client or runtime inside the Rust engine?

---

## 2. Confirmed Design Rationale

Per specification [0019](specs/0019-baml-contracts-as-data.md) and the authoritative reconciliation in [0012](specs/0012-existing-harness-reconciliation.md), the decision to treat BAML contracts as data rests on three confirmed architectural requirements:

### A. Zero Hallucination in Governance
The harness is responsible for enforcing contracts, verifying proofs, and validating boundaries. If the governance harness itself relied on LLM calls to parse schemas or determine route selection:
- Non-deterministic responses could cause identical tasks to route differently across runs.
- Hallucinations could corrupt data models or miss critical boundary findings.
- Governance would be vulnerable to upstream model outages, API rate limits, and network latency.

Treating BAML schemas as static type contracts ensures that schema parsing and validation are 100% deterministic, reproducible, and instantaneous.

### B. Single-Binary, Offline Operation
SWE_SEED is engineered as a standalone CLI tool. Requiring an LLM runtime would introduce massive dependency overhead (HTTP clients, TLS certificates, prompt templates, API key management). By consuming schemas as pure data (`serde`), the binary compiles into a lean native executable that operates completely offline with zero API credentials.

### C. Clear Separation of Concerns
The host coding agent (Claude Code, GitHub Copilot, Codex, Antigravity) is the entity that reasons, plans, and writes code using language models. SWE_SEED does not attempt to be an agent; it provides the **operating terms, constraints, and audit ledger** for the agent. Mixing LLM reasoning into the harness would blur the boundary between the worker (the agent) and the contract governor (the harness).

---

## 3. Trade-offs and Rejected Alternatives

| Alternative Considered | Why Rejected | Consequence of Chosen Design |
|---|---|---|
| **Porting BAML Python / Rust LLM Runtime** | Rejected in spec 0012. Adding LLM dependencies introduces API keys, cost, latency, and non-determinism into static commands like `swe-seed route`. | The CLI runs sub-millisecond local commands with zero API credentials. |
| **Using Pure JSON-Schema or Protobuf** | Considered, but BAML was already adopted across the wider GodSpeed ecosystem for declaring entity models and cross-layer schemas. | Using BAML as data maintains ecosystem schema compatibility while preserving local runtime simplicity. |

---

## 4. Invariants Protected

1. **Deterministic Execution**: Given identical inputs and route cards, `swe-seed route` and `swe-seed validate` will produce bit-for-bit identical outputs every time.
2. **Zero Network Requirement**: Core harness execution (`route`, `trace`, `seed`, `doctor`) executes with network disconnected.

---

## 5. Source Trail

- `.agent-harness/baml/baml_src/swe_seed.baml`: Canonical outer layer schema.
- `.agent-harness/baml/baml_src/harness.baml`: Canonical harness layer schema.
- `.agent-harness/baml/baml_src/fabricator.baml`: Canonical inner layer schema.
- `crates/swe-seed-core/src/contracts/`: Data structures generated/aligned with BAML.
- `docs/specs/0019-baml-contracts-as-data.md`: Normative specification.
- `docs/specs/0012-existing-harness-reconciliation.md`: Reconciliation decision.
