# Documentation Map

This document catalogs every technical documentation page in the SWE_SEED knowledge system, defining its purpose, target audience need, Diátaxis classification, prerequisites, and related pages.

---

## 1. Meta & Orientation Pages

| Page Path | Purpose | Primary Audience Need | Diátaxis Type | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`docs/README.md`](README.md) | Entry point, high-level summary, 1-picture architecture, and navigation hub. | Quick orientation; discovering where to go based on intent. | Orientation | None | [`getting-started.md`](getting-started.md), [`mental-model.md`](mental-model.md) |
| [`docs/getting-started.md`](getting-started.md) | Step-by-step developer environment setup and first execution. | Setting up a verified local workspace. | Tutorial | None | [`README.md`](README.md), [`tutorials/first-routed-task.md`](tutorials/first-routed-task.md) |
| [`docs/mental-model.md`](mental-model.md) | Conceptual foundations: 3-layer stack, sovereign loop, work contract. | Grasping system structure without low-level code details. | Explanation | [`README.md`](README.md) | [`architecture.md`](architecture.md), [`concepts.md`](concepts.md) |
| [`docs/architecture.md`](architecture.md) | In-depth 6-view architecture guide with Mermaid diagrams and source trails. | Understanding subsystem boundaries, execution flows, and trust limits. | Explanation | [`mental-model.md`](mental-model.md) | [`source-map.md`](source-map.md), [`subsystems/`](subsystems/) |
| [`docs/concepts.md`](concepts.md) | Formal dictionary defining 20+ domain terms, distinctions, and symbols. | Precise vocabulary clarity; eliminating terminology confusion. | Reference | [`README.md`](README.md) | [`source-map.md`](source-map.md) |
| [`docs/troubleshooting.md`](troubleshooting.md) | Diagnostic matrices, error symptoms, root causes, and recovery runbooks. | Diagnosing and recovering from failed commands or gate rejections. | How-To | [`getting-started.md`](getting-started.md) | [`reference/cli.md`](reference/cli.md) |
| [`docs/source-map.md`](source-map.md) | Comprehensive cross-reference linking architecture to Rust code and schemas. | Connecting architectural claims directly to source code evidence. | Reference | [`architecture.md`](architecture.md) | All subsystem docs |
| [`docs/documentation-map.md`](documentation-map.md) | Meta-catalog of the entire documentation system (this document). | Navigating and discovering documentation resources. | Reference | None | [`README.md`](README.md) |

---

## 2. Subsystem Guides (`docs/subsystems/`)

Each subsystem guide follows a strict 11-part template: Purpose, Responsibilities, Non-responsibilities, Position in system, Core abstractions, Internal operation, State, Lifecycle, Failure modes, Extension points, and Source trail.

| Page Path | Subsystem | Primary Audience Need | Diátaxis Type | Related Pages |
|---|---|---|---|---|
| [`subsystems/sweseed-governance.md`](subsystems/sweseed-governance.md) | SweSeed Layer | Understanding capability registration, layer boundary checks, and provenance. | Explanation / Reference | [`specs/0002-swe-seed-centralization-layer.md`](specs/0002-swe-seed-centralization-layer.md), [`specs/0018-layer-boundary-governance.md`](specs/0018-layer-boundary-governance.md) |
| [`subsystems/routing-engine.md`](subsystems/routing-engine.md) | Routing Engine | Understanding intent matching, route cards, and pre-execution contracts. | Explanation / Reference | [`reference/route-cards-reference.md`](reference/route-cards-reference.md), [`howto/add-a-route-card.md`](howto/add-a-route-card.md) |
| [`subsystems/context-budget.md`](subsystems/context-budget.md) | Context Budget Plane | Controlling context window size, token budgets, and output containment. | Explanation / Reference | [`specs/0015-context-budget-plane.md`](specs/0015-context-budget-plane.md) |
| [`subsystems/trace-ledger.md`](subsystems/trace-ledger.md) | Trace Ledger & Gate | Cryptographic hash chaining, durable task records, and merge gating. | Explanation / Reference | [`specs/0014-trace-and-durable-decisions.md`](specs/0014-trace-and-durable-decisions.md), [`workflows/task-routing-to-proof.md`](workflows/task-routing-to-proof.md) |
| [`subsystems/eval-and-proof.md`](subsystems/eval-and-proof.md) | Eval & Proof Engine | Deterministic testing, frozen eval specs, and proof dispositions. | Explanation / Reference | [`specs/0013-eval-and-proof.md`](specs/0013-eval-and-proof.md), [`explanations/why-pre-execution-proof.md`](explanations/why-pre-execution-proof.md) |
| [`subsystems/host-adapters-and-projection.md`](subsystems/host-adapters-and-projection.md) | Host Adapters | Projecting config into Claude, Copilot, Antigravity; drift detection. | Explanation / Reference | [`specs/0004-host-adapter-contract.md`](specs/0004-host-adapter-contract.md), [`reference/host-capability-matrix.md`](reference/host-capability-matrix.md) |
| [`subsystems/hook-runtime.md`](subsystems/hook-runtime.md) | Hook Runtime | Lifecycle event logging, secret redaction, and OTel telemetry export. | Explanation / Reference | [`specs/0005-normalized-hook-runtime.md`](specs/0005-normalized-hook-runtime.md) |
| [`subsystems/mcpgateway.md`](subsystems/mcpgateway.md) | MCPGate Gateway | MCP server discovery, tool namespacing, and permission proxying. | Explanation / Reference | [`specs/0020-mcpgate.md`](specs/0020-mcpgate.md), [`howto/configure-mcp-servers.md`](howto/configure-mcp-servers.md) |
| [`subsystems/learning-and-adaptation.md`](subsystems/learning-and-adaptation.md) | Learning Loop | Trace reflection, learning records, and reviewed skill promotion. | Explanation / Reference | [`specs/0016-learning-and-adaptation-loop.md`](specs/0016-learning-and-adaptation-loop.md) |
| [`subsystems/fabricator-semantic-chain.md`](subsystems/fabricator-semantic-chain.md) | Fabricator Layer | Product-to-prototype semantic chain, EARS requirements, Gherkin specs. | Explanation / Reference | [`specs/0017-fabricator-layer.md`](specs/0017-fabricator-layer.md), [`tutorials/building-a-prototype-with-fabricator.md`](tutorials/building-a-prototype-with-fabricator.md) |
| [`subsystems/federation.md`](subsystems/federation.md) | SEA Federation | Semantic event envelopes, Ed25519 signing, and SOPS encrypted keys. | Explanation / Reference | [`specs/0011-sea-loop-federation.md`](specs/0011-sea-loop-federation.md), [`howto/manage-federation-keys.md`](howto/manage-federation-keys.md) |
| [`subsystems/doctor-and-drift.md`](subsystems/doctor-and-drift.md) | Doctor & Drift | Aggregate repository health checks and host projection drift scanning. | Explanation / Reference | [`specs/0008-doctor-and-drift-detection.md`](specs/0008-doctor-and-drift-detection.md), [`howto/run-and-extend-doctor.md`](howto/run-and-extend-doctor.md) |

---

## 3. Workflows & Execution Traces (`docs/workflows/`)

Step-by-step traces through runtime execution paths with sequence diagrams and state transitions.

| Page Path | Workflow | Description | Diátaxis Type |
|---|---|---|---|
| [`workflows/startup-and-discovery.md`](workflows/startup-and-discovery.md) | Startup & Discovery | Tracing CLI argument parsing, configuration loading, and workspace resolution. | Explanation |
| [`workflows/task-routing-to-proof.md`](workflows/task-routing-to-proof.md) | Task Routing to Proof | The canonical end-to-end execution path: route -> context-plan -> trace -> proof -> gate. | Explanation |
| [`workflows/host-projection-and-sync.md`](workflows/host-projection-and-sync.md) | Host Projection & Sync | The full cycle of generating host config files, detecting drift, and rolling back snapshots. | Explanation |
| [`workflows/fabricator-product-run.md`](workflows/fabricator-product-run.md) | Fabricator Product Run | Tracing the synthesis of a ProductSeed through all 10 semantic chain nodes to a proven prototype. | Explanation |
| [`workflows/federation-signing-and-verification.md`](workflows/federation-signing-and-verification.md) | Federation Signing | Tracing Ed25519 key generation, SOPS decryption, envelope signing, and verification. | Explanation |

---

## 4. Deep Architectural Explanations (`docs/explanations/`)

In-depth essays answering **why** architectural decisions were made, distinguishing verified facts from inferences.

| Page Path | Topic | Core Question Answered | Diátaxis Type |
|---|---|---|---|
| [`explanations/why-contracts-as-data.md`](explanations/why-contracts-as-data.md) | Contracts-as-Data | Why are BAML schemas parsed purely as deterministic data without an LLM runtime? | Explanation |
| [`explanations/why-pre-execution-proof.md`](explanations/why-pre-execution-proof.md) | Pre-Execution Proof | Why must proof commands be declared before code modification rather than after? | Explanation |
| [`explanations/why-three-layers.md`](explanations/why-three-layers.md) | Three-Layer Stack | Why are SweSeed, Harness, and Fabricator strictly separated with downward governance? | Explanation |
| [`explanations/why-file-first-tamper-evident-traces.md`](explanations/why-file-first-tamper-evident-traces.md) | Tamper-Evident Traces | Why use append-only JSON files and hash-chained SQLite rather than cloud telemetry? | Explanation |
| [`explanations/why-host-projection-over-custom-runtime.md`](explanations/why-host-projection-over-custom-runtime.md) | Host Projections | Why project into native host config files rather than building a custom agent wrapper? | Explanation |

---

## 5. Guided Tutorials (`docs/tutorials/`)

Step-by-step practical learning journeys starting from zero repository knowledge.

| Page Path | Tutorial | Learning Outcome | Diátaxis Type |
|---|---|---|---|
| [`tutorials/first-routed-task.md`](tutorials/first-routed-task.md) | First Routed Task | Take a real task from route selection to verified proof and gate closure. | Tutorial |
| [`tutorials/building-a-prototype-with-fabricator.md`](tutorials/building-a-prototype-with-fabricator.md) | Fabricator Prototype | Turn a product need into a validated specification chain and working prototype. | Tutorial |
| [`tutorials/authoring-a-new-skill.md`](tutorials/authoring-a-new-skill.md) | Authoring a Skill | Ingest a skill, normalize to SkillIR, scan for security, and project to host targets. | Tutorial |

---

## 6. How-To Guides (`docs/howto/`)

Goal-oriented operational procedures for specific maintainer tasks.

| Page Path | Guide | Goal | Diátaxis Type |
|---|---|---|---|
| [`howto/add-a-route-card.md`](howto/add-a-route-card.md) | Add a Route Card | Register a new work pattern with custom context, work loop, and proof commands. | How-To |
| [`howto/add-a-host-adapter.md`](howto/add-a-host-adapter.md) | Add a Host Adapter | Implement support for a new AI coding tool by creating a projection adapter. | How-To |
| [`howto/configure-mcp-servers.md`](howto/configure-mcp-servers.md) | Configure MCP Servers | Add, namespace, and configure permission policies for MCP servers under MCPGate. | How-To |
| [`howto/debug-routing-mismatches.md`](howto/debug-routing-mismatches.md) | Debug Routing Mismatches | Diagnose why a task matched the wrong route card and refine semantic triggers. | How-To |
| [`howto/manage-federation-keys.md`](howto/manage-federation-keys.md) | Manage Federation Keys | Generate Ed25519 keypairs, encrypt with SOPS, and rotate keys safely. | How-To |
| [`howto/run-and-extend-doctor.md`](howto/run-and-extend-doctor.md) | Run & Extend Doctor | Execute doctor diagnostics and register custom health or drift checks. | How-To |

---

## 7. Technical Reference (`docs/reference/`)

Exhaustive, precise reference specifications.

| Page Path | Reference Topic | Scope | Diátaxis Type |
|---|---|---|---|
| [`reference/cli.md`](reference/cli.md) | CLI Manual | Complete documentation for all `swe-seed` commands, subcommands, arguments, flags, and exit codes. | Reference |
| [`reference/config-and-baml-schemas.md`](reference/config-and-baml-schemas.md) | Config & BAML Schemas | Detailed field-by-field reference for `.agent-harness/config.yaml`, BAML schemas, and `.swe-seed/manifest.toml`. | Reference |
| [`reference/route-cards-reference.md`](reference/route-cards-reference.md) | Route Cards Reference | Complete specifications of all 11 built-in route cards, their triggers, artifacts, and proof obligations. | Reference |
| [`reference/host-capability-matrix.md`](reference/host-capability-matrix.md) | Host Capability Matrix | Exhaustive comparison of enforcement strength (Strict, Advisory, Unsupported) across all 6 host adapters. | Reference |
