# Source Map: Architecture to Implementation

This document provides a comprehensive mapping connecting SWE_SEED architectural concepts and capabilities to their concrete implementation locations across the Rust workspace, BAML contracts, specifications, and test suites.

---

## 1. SweSeed Layer: Governance & Capability Assembly

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **Capability Registry & Manifest** | `crates/swe-seed-core/src/seed/manifest.rs` | `SeedPackageManifest`, `assemble_seed_package()` | `.agent-harness/baml/baml_src/swe_seed.baml` | `crates/swe-seed/tests/cli_golden.rs` | [0002](specs/0002-swe-seed-centralization-layer.md), [0003](specs/0003-capability-registry.md) |
| **Layer Capabilities** | `crates/swe-seed-core/src/seed/capability.rs` | `LayerCapability`, `select_layer_capabilities()` | `swe_seed.baml` (`LayerCapability`, `LayerName`) | `crates/swe-seed-core/src/seed/capability.rs` (unit tests) | [0003](specs/0003-capability-registry.md) |
| **Layer Boundary Governance** | `crates/swe-seed-core/src/seed/boundary.rs` | `validate_layer_boundaries()`, `BoundaryReport`, `BoundaryFinding` | `swe_seed.baml` (`BoundaryReport`) | `crates/swe-seed-core/src/seed/boundary.rs` (unit tests) | [0018](specs/0018-layer-boundary-governance.md) |
| **Idempotent Regeneration** | `crates/swe-seed-core/src/seed/regenerate.rs` | `regenerate_seed_package()`, `SeedRegenerationPlan` | `swe_seed.baml` (`SeedRegenerationPlan`) | `crates/swe-seed-core/src/seed/regenerate.rs` (unit tests) | [0002](specs/0002-swe-seed-centralization-layer.md), [0018](specs/0018-layer-boundary-governance.md) |
| **Provenance Verification** | `crates/swe-seed-core/src/provenance/verify.rs` | `verify_provenance()`, `compute_sha256()` | `swe_seed.baml` (`ArtifactMetadata`, `SourceRef`) | `crates/swe-seed-core/src/provenance/verify.rs` (unit tests) | [0009](specs/0009-license-and-provenance-boundaries.md) |

---

## 2. Harness Layer: Routing, Context, Proof & Traces

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **Semantic Router** | `crates/swe-seed-core/src/route/mod.rs` | `RouteCard`, `RouteDecision`, `match_route()`, `load_route_cards()` | `.agent-harness/baml/baml_src/harness.baml` (`RouteCard`) | `crates/swe-seed/tests/route_golden.rs` | [0004](specs/0004-host-adapter-contract.md), [0012](specs/0012-existing-harness-reconciliation.md) |
| **Route Cards (11 Job Types)** | `.agent-harness/routes/*.json` | JSON Route Card definitions | `harness.baml` (`JobType`, `RouteCard`) | `crates/swe-seed-core/src/harness_validate.rs` | `AGENTS.md`, [0012](specs/0012-existing-harness-reconciliation.md) |
| **Context Budget & Packing** | `crates/swe-seed-core/src/context/pack.rs`, `budget.rs` | `ContextPack`, `ContextBudget`, `plan_context()` | `.agent-harness/context/budget-policy.yaml`, `harness.baml` | `crates/swe-seed/tests/cli_golden.rs` | [0015](specs/0015-context-budget-plane.md) |
| **Trace Lifecycle** | `crates/swe-seed-core/src/trace/lifecycle.rs` | `trace_start()`, `trace_append()`, `trace_checkpoint()`, `trace_resume()`, `trace_distill()`, `trace_finish()` | `harness.baml` (`TraceRecord`, `TraceStage`) | `crates/swe-seed-core/src/trace/lifecycle.rs` (unit tests) | [0014](specs/0014-trace-and-durable-decisions.md) |
| **Trace Ledger & Cryptographic Chain** | `crates/swe-seed-core/src/trace_ledger.rs` | `TraceLedger`, `record_event()`, `verify_chain()`, `has_genesis()` | SQLite schema (`events` table) | `crates/swe-seed-core/src/trace_ledger.rs` (unit tests) | [0011](specs/0011-sea-loop-federation.md), [0014](specs/0014-trace-and-durable-decisions.md) |
| **Routing Gate** | `crates/swe-seed-core/src/routing_gate.rs` | `route_gate()`, `RouteGate` (`Allow`, `Block`) | Trace Ledger genesis requirement | `crates/swe-seed/src/gate_cli.rs` | `just gate-merge`, `AGENTS.md` |
| **Deterministic Eval Runner** | `crates/swe-seed-core/src/eval/` | `EvalSpec`, `EvalCheck`, `EvalResult`, `run_eval()` | `harness.baml` (`EvalSpec`, `EvalResult`) | `crates/swe-seed-core/src/eval/check.rs` (unit tests) | [0013](specs/0013-eval-and-proof.md) |
| **Proof Records & Disposition** | `crates/swe-seed-core/src/eval/proof.rs` | `ProofRecord`, `ProofDisposition` | `harness.baml` (`ProofRecord`) | `crates/swe-seed-core/src/eval/proof.rs` (unit tests) | [0013](specs/0013-eval-and-proof.md) |
| **Harness Structure Validation** | `crates/swe-seed-core/src/harness_validate.rs` | `validate_harness()`, `HarnessValidationReport` | Root specs, BAML contracts, route cards | `tests/validate-harness.sh`, `just harness-validate` | `HARNESS_SPEC.md` |

---

## 3. Host Adapters & Hook Runtime

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **Host Adapters & Projection** | `crates/swe-seed-core/src/adapters/mod.rs` | `HostAdapter`, `Projection`, `sync()`, `rollback()` | `.agent-harness/baml/baml_src/swe_seed.baml` (`HostAdapter`) | `crates/swe-seed-core/src/adapters/mod.rs` (unit tests) | [0004](specs/0004-host-adapter-contract.md) |
| **Claude Adapter** | `crates/swe-seed-core/src/adapters/claude.rs` | `ClaudeAdapter` (targets `.claude/settings.json`, `CLAUDE.md`) | `HostCapabilityMatrix` | Adapter unit tests | [0004](specs/0004-host-adapter-contract.md) |
| **Copilot Adapter** | `crates/swe-seed-core/src/adapters/github_copilot.rs` | `GithubCopilotAdapter` (`.github/copilot-instructions.md`) | `HostCapabilityMatrix` | Adapter unit tests | [0004](specs/0004-host-adapter-contract.md) |
| **Antigravity Adapter** | `crates/swe-seed-core/src/adapters/antigravity.rs` | `AntigravityAdapter` (`.agent-rules/`, Antigravity guidelines) | `HostCapabilityMatrix` | Adapter unit tests | [0004](specs/0004-host-adapter-contract.md) |
| **Managed Comment Blocks** | `crates/swe-seed-core/src/adapters/marker.rs` | `replace_managed_block()`, marker constants | Comment markers | `crates/swe-seed-core/src/adapters/marker.rs` (unit tests) | [0004](specs/0004-host-adapter-contract.md) |
| **Hook Runtime** | `crates/swe-seed-core/src/hooks/runtime.rs` | `HookRuntime`, `record_event()`, `compact_logs()` | `.agent-hooks/events.jsonl` | `crates/swe-seed-core/src/hooks/runtime.rs` (unit tests) | [0005](specs/0005-normalized-hook-runtime.md) |
| **Hook Redaction Engine** | `crates/swe-seed-core/src/hooks/redact.rs` | `redact_secrets()`, secret regex patterns | Redaction rules | `crates/swe-seed-core/src/hooks/redact.rs` (unit tests) | [0005](specs/0005-normalized-hook-runtime.md) |
| **Telemetry Export (OTel/JUnit)** | `crates/swe-seed-core/src/hooks/export.rs` | `export_otel()`, `export_junit()` | OpenTelemetry & JUnit XML | `crates/swe-seed-core/src/hooks/export.rs` (unit tests) | [0005](specs/0005-normalized-hook-runtime.md) |
| **Doctor & Drift Detection** | `crates/swe-seed-core/src/doctor/` | `run_doctor()`, `detect_host_drift()`, `DoctorReport` | JSON report format | `crates/swe-seed/tests/cli_golden.rs` | [0008](specs/0008-doctor-and-drift-detection.md) |

---

## 4. Fabricator Layer: Product to Prototype Pipeline

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **Semantic Chain Artifacts** | `crates/swe-seed-core/src/fabricator/artifacts.rs` | `ProductSeed`, `JobStory`, `PRD`, `ProductADR`, `EARSRequirements`, `SDS`, `GherkinScenarios`, `TDDPlan`, `AgentTask` | `.agent-harness/baml/baml_src/fabricator.baml` | `crates/swe-seed-core/src/fabricator/artifacts.rs` (unit tests) | [0017](specs/0017-fabricator-layer.md), `FABRICATOR_SPEC_v0.1.0.md` |
| **Chain Integrity Validation** | `crates/swe-seed-core/src/fabricator/chain.rs` | `validate_semantic_chain()`, `SemanticChainValidationReport` | `fabricator.baml` (`TraceabilityLink`) | `crates/swe-seed-core/src/fabricator/chain.rs` (unit tests) | [0017](specs/0017-fabricator-layer.md) |
| **Fabricator CLI Execution** | `crates/swe-seed/src/fabricate_cli.rs` | `run_fabricate()`, `FabricateAction` | `.fabricator/config.yaml` | `crates/swe-seed/src/fabricate_cli.rs` | `FABRICATOR_SPEC_v0.1.0.md` |

---

## 5. Gateway & Federation Subsystems

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **MCPGate Core Proxy** | `crates/swe-seed-core/src/gateway/serve.rs` | `GatewayServer`, `serve_stdio()`, `serve_sse()` | JSON-RPC 2.0 protocol | `crates/swe-seed-core/src/gateway/serve.rs` (unit tests) | [0020](specs/0020-mcpgate.md), [0006](specs/0006-mcp-gateway-integration.md) |
| **Tool Catalog & Routing** | `crates/swe-seed-core/src/gateway/catalog.rs`, `routing.rs` | `ToolCatalog`, `namespaced_name()`, `route_call()` | MCP Tool definitions | Catalog unit tests | [0020](specs/0020-mcpgate.md) |
| **Gateway Policy Governance** | `crates/swe-seed-core/src/gateway/governance.rs` | `GovernanceEngine`, `evaluate_tool_call()` | `PermissionPolicy` | Governance unit tests | [0020](specs/0020-mcpgate.md) |
| **SEA-Loop Federation Envelopes** | `crates/swe-seed-core/src/federation/envelope.rs` | `SemanticEventEnvelope`, `emit()`, `consume()` | Spec 0011 schema | `crates/swe-seed-core/src/federation/envelope.rs` (unit tests) | [0011](specs/0011-sea-loop-federation.md) |
| **Cryptographic Signing (Ed25519)** | `crates/swe-seed-core/src/federation/signing.rs` | `Keypair`, `sign_envelope()`, `verify_signature()` | Ed25519 byte format | `crates/swe-seed-core/src/federation/signing.rs` (unit tests) | [0011](specs/0011-sea-loop-federation.md) |

---

## 6. Learning & Adaptation Subsystem

| Capability / Concept | Rust Module | Primary Types & Symbols | Schemas & Contracts | Tests & Verification | Specifications |
|---|---|---|---|---|---|
| **Trace Reflection** | `crates/swe-seed-core/src/learning/reflection.rs` | `reflect_trace()`, `ReflectionTemplate` | `.agent-harness/baml/baml_src/harness.baml` | Reflection unit tests | [0016](specs/0016-learning-and-adaptation-loop.md) |
| **Learning Records & Candidates** | `crates/swe-seed-core/src/learning/record.rs`, `candidate.rs` | `LearningRecord`, `LearningCandidate` | `harness.baml` | Learning unit tests | [0016](specs/0016-learning-and-adaptation-loop.md) |
| **Skill Proposals & Regressions** | `crates/swe-seed-core/src/learning/proposal.rs`, `regression.rs` | `SkillProposal`, `RegressionCase` | `harness.baml` | Promotion unit tests | [0016](specs/0016-learning-and-adaptation-loop.md) |
| **Adaptation Decisions** | `crates/swe-seed-core/src/learning/adaptation.rs` | `build_adaptation_decision()`, `AdaptationDecision` | `harness.baml` | Adaptation unit tests | [0016](specs/0016-learning-and-adaptation-loop.md) |
