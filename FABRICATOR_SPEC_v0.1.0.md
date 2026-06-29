# FABRICATOR_SPEC v0.1.0

Root contract for the **Fabricator layer**: the inner layer that turns a product need into a
proven prototype through a traceable semantic chain. This file is a stable root spec.
Detailed design lives in `.agents/specs/0017-fabricator-layer.md`; the reconciliation in
`.agents/specs/0012-existing-harness-reconciliation.md` is authoritative on any vocabulary
conflict.

## Position in the stack

The Fabricator layer is the innermost layer, governed by the Harness layer
(`HARNESS_SPEC.md`) and the SweSeed layer (`SWE_SEED_SPEC_v0.2.0.md`). Its eval, proof, and
learning types mirror the Harness ones at product scope.

## Purpose

Run a single bounded pass from a product need to a proven prototype. The Fabricator does not
loop autonomously; it produces a traceable chain plus a frozen eval spec and a proof record,
then hands off.

## Semantic Specification Chain

Each artifact derives from and traces to its predecessor:

```
ProductSeed -> JobStory -> ProductHypothesis -> PRD -> ProductADR (Y-statement)
  -> EARSRequirements -> SDS / SDSComponents -> GherkinScenarios -> TDDPlan
  -> AgentTask -> FabricatorEvalSpec -> FabricatorProofRecord
```

Canonical schema: `.agent-harness/baml/baml_src/fabricator.baml` (contracts as data, no LLM
runtime; see `.agents/specs/0019-baml-contracts-as-data.md`). The chain edge is
`TraceabilityLink`.

## Layer rules

1. Chain integrity: `ValidateSemanticChain` produces a `SemanticChainValidationReport`. A
   broken upstream link is a finding; when `passed = false`, handoff is blocked.
2. Requirements use the EARS patterns; scenarios are Gherkin. Both are validated for shape.
3. Proof-gated handoff: the `AgentTask` hands off only with a frozen `FabricatorEvalSpec` and
   the required regression cases.
4. Every artifact carries `ArtifactMetadata` provenance.

## Commands (target Rust surface)

```
swe-seed fabricate <product-need>            # bounded run: chain + eval spec + proof
swe-seed fabricate validate-chain <run>      # produce a SemanticChainValidationReport
```

Configuration and templates: `.fabricator/config.yaml`, `.fabricator/templates/`.

## Detailed specs

- `.agents/specs/0017-fabricator-layer.md` — full semantic chain and validation.
- `.agents/specs/0013-eval-and-proof.md` — eval and proof model reused at product scope.
- `.agents/plans/0001-swe-seed-v0-1-implementation.md` — Rust rewrite plan of record.

## Root Compatibility Markers

Detailed Fabricator contracts remain in `.agents/specs/`, but the root spec preserves the
validation markers used by the harness smoke suite:

- `eval-spec.schema.yaml`
- `NO_SKILL_PROPOSED.md.j2`
- `EVAL_RESULT.json`
- Deterministic Serialization

## Job Story Syntax

See `.agents/specs/FABRICATOR_SPEC_v0.1.0.md` for the full syntax.

## Y-Statement ADR Syntax

See `.agents/specs/FABRICATOR_SPEC_v0.1.0.md` for the full syntax.

## SDS Structural Requirements

See `.agents/specs/FABRICATOR_SPEC_v0.1.0.md` for the full structural requirements.

## Gherkin Behavioral Syntax

No Gherkin scenario may be considered satisfied unless its Given, When, and Then clauses trace
to the semantic chain.

## Status

Version 0.1.0. Implementation is migrating from the Python reference (`scripts/fabricate.py`)
to Rust per the plan of record.
