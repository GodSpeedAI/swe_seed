# 0017 — Fabricator Layer (Product → Prototype)

## Purpose

Define the innermost layer: the bounded, spec-driven pipeline that turns a product need into
a proven prototype via a **semantic chain** of artifacts
(ProductSeed → JobStory → PRD → SDS → TDDPlan → AgentTask → EvalSpec → ProofRecord), with
chain-integrity validation. Reproduces `scripts/fabricate.py` + `fabricator.baml`.

## Non-goals

- Not invoking LLMs at runtime — the `.baml` `Generate*` functions are offline-generation
  contracts; the Rust binary validates, links, and renders the resulting artifacts
  (contracts-as-data, 0019).
- Not a general project scaffolder beyond the bounded fabricate run.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `fabricator.baml` | `ProductSeed`, `JobStory`, `ProductHypothesis`, `EARSRequirement`+`EARSPattern`, `ProductADR`+`YStatement`, `PRD`, `SDS`+`SDSComponent`, `GherkinScenario`, `TDDPlan`, `AgentTask`, `FabricatorEvalSpec`+`FabricatorEvalCheck`, `FabricatorProofRecord`, `FabricatorReflectionTemplate`, `FabricatorAdaptationDecision`, `FabricatorRegressionCase`, `FabricatorLearningCandidate`, `SemanticChainValidationReport`, `TraceabilityLink` |
| `scripts/fabricate.py` (1169 LOC) | Bounded product→prototype CLI; `.fabricator/{config.yaml,templates}` |

## SWE_Seed requirements

1. **Semantic chain** (the spine): each artifact is derived from and traceable to its
   predecessor: `ProductSeed → JobStory → ProductHypothesis → PRD → ProductADR (Y-statement)
   → EARSRequirements → SDS/SDSComponents → GherkinScenarios → TDDPlan → AgentTask →
   FabricatorEvalSpec → FabricatorProofRecord`.
2. **Chain integrity** (`ValidateSemanticChain` → `SemanticChainValidationReport`): every
   link must trace upstream; broken links are findings; `ProposeTraceabilityRepair` suggests
   fixes. A fabricate run with broken chain integrity cannot hand off.
3. **EARS + Gherkin discipline**: requirements use `EARSPattern`; scenarios are Gherkin —
   both are validated for shape, not prose quality.
4. **Bounded run**: fabricate is a single bounded pass producing the chain + an eval spec +
   a proof record; it does not loop autonomously.
5. **Proof-gated handoff**: the `AgentTask` is handed off only with a frozen
   `FabricatorEvalSpec` (0013 frozen-after-handoff) and required regression cases.
6. The Fabricator layer is **governed by the Harness layer**: its eval/proof/learning types
   mirror the Harness ones (0013/0016) at product scope.

## Data model

All `Fabricator*` types canonical from `fabricator.baml` (0019). `TraceabilityLink{from,to,
relation}` is the chain edge.

```toml
# SemanticChainValidationReport (rendered)
passed = false
[[findings]]
link = "PRD -> SDS"
issue = "SDS component lacks PRD requirement reference"
severity = "high"
```

- **Rust**: `swe_seed::fabricator` (`chain.rs`, `artifacts.rs`, `validate.rs`, `render.rs`).
- **Validation**: each artifact references its parent; chain report `passed` gates handoff.

## CLI behavior

```
swe-seed fabricate <product-need> [--config .fabricator/config.yaml]   # bounded run → chain + eval spec + proof
swe-seed fabricate validate-chain <run>                                 # → SemanticChainValidationReport
```

## Generated files

Chain artifacts under `.fabricator/<run>/` from `.fabricator/templates`; eval spec + proof
records as in 0013. Formats frozen as golden-file parity targets vs current Python.

## Rust module boundaries

`swe_seed::fabricator`; reuses `eval` (0013), `learning` (0016); governed by boundary
validation (0018).

## Security and provenance considerations

Every artifact carries `ArtifactMetadata` provenance (0009/0018). Chain integrity prevents
"prototype without traceable requirement."

## Tests

- A broken traceability link → `SemanticChainValidationReport.passed = false` → handoff blocked.
- EARS/Gherkin shape validation rejects malformed requirements/scenarios.
- A fabricate run produces a complete, traceable chain + frozen eval spec.
- Golden-file parity for a sample fabricate run vs current Python output.

## Open questions

- v0.1 includes the full chain (maintainer chose all three layers); confirm the
  `.fabricator/templates` set is the rendering source of truth.

## Acceptance criteria

- [ ] Full semantic chain implemented with traceability validation.
- [ ] Chain-integrity gate blocks handoff on broken links.
- [ ] Proof-gated, frozen eval spec at handoff; provenance on every artifact.
