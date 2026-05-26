# BAML Artifact Compiler Migration Note

## What Changed

`HARNESS_SPEC.md` now defines BAML as an optional typed artifact compiler for harness artifacts.
BAML produces structured outputs, and renderers convert those outputs into the Markdown/YAML files
agents read and operators review.

`SWE_SEED_SPEC_v0.2.0.md` now defines BAML as an optional first-class typed artifact generation
layer for SWE Seed projects. It defines SWE Seed as the outer layer and delegates harness and
fabrication generation details to their owning specs.

`FABRICATOR_SPEC_v0.1.0.md` now defines the inner product fabrication layer. It owns product seeds,
PRDs, SDS documents, TDD plans, agent tasks, prototype proof, reflection, and fabrication learning.

The specs now treat evaluation and adaptation as first-class. Local `EvalSpec`, `EvalResult`,
`AdaptationDecision`, `ProofRecord`, `RegressionCase`, and `LearningCandidate` artifacts control what
the system may learn. External eval tools remain adapters only.

Fabricator hypothesis modeling is now represented as a job-hypothesis business canvas, preserving
the valuable measurable-success and riskiest-assumption fields while adding explicit value,
adoption, and alternative-solution context.

Fabricator PRD requirements are now represented in machine-readable EARS structure with stable
requirement IDs so traceability and deterministic validation can operate on typed fields instead of
free-form prose.

The root specs now define a semantic specification chain:

```text
JTBD Job Story
-> EARS Requirement
-> Y-Statement ADR
-> C4/Mermaid Structural SDS
-> Gherkin Behavioral Scenario
-> EvalSpec
-> AgentTask
-> ProofRecord
```

This change makes the product path reconstructible from the three root specs alone and turns each
layer into a constrained syntax rather than a free-form document shape.

Markdown/YAML remains the human-legible operating contract, agent context, validation surface,
memory artifact, and regeneration input. BAML does not replace route cards, Skill IR, evals, traces,
product specs, proof records, human approval, or proof gates.

## Why

The harness already uses Markdown/YAML artifacts as operational inputs. Typed BAML schemas make those
artifacts easier to generate, validate, compare, and regenerate without turning them into informal
documentation or hidden prompt text.

SWE Seed also needs the same typed generation path so future regenerated projects can package lower
layers without duplicating their contracts. The three specs work together by stable artifacts and
identifiers with one-way inward dependencies: SWE Seed may depend on lower layers, the harness may
depend on fabricator handoff/proof artifacts, and the fabricator depends on neither outer layer.

## Installation Constraint

When implemented, BAML must be installed through `uv`:

```bash
uv add baml-py
uv run baml-cli generate --from .agent-harness/baml/baml_src
```

The generated client code is an implementation detail. Reviewed harness artifacts remain versioned
in git and must pass the existing proof gates.

## Migration Boundary

The harness v0.1 scope is limited to generation for `RouteCard`, `SkillIR`, `EvalSpec`,
`ContextPack`, `ReflectionTemplate`, and `SkillProposal`.

The SWE Seed v0.1 scope adds `ProjectSeed`, layer capability selection, seed package manifests,
boundary validation, and regeneration planning. Product artifacts remain fabricator-owned.

This migration does not add vector memory, autonomous planning, production deployment, complex
multi-agent orchestration, unsupervised skill promotion, or hidden self-modification.

Fabricator typed source contracts are present under `.agent-harness/baml/baml_src/` so another agent
can regenerate all three layers from the three root specs without a hidden fourth source.

## Evaluation Boundary

The active `EvalSpec` freezes after implementation handoff. If a run finds a weak or wrong eval, it
creates an eval issue, regression case, ADR, or future EvalSpec revision. The current run does not
weaken its own eval unless a human explicitly waives or restarts it.

## Root-Spec Regeneration

Another agent should be able to regenerate the whole system from only:

- `SWE_SEED_SPEC_v0.2.0.md`
- `HARNESS_SPEC.md`
- `FABRICATOR_SPEC_v0.1.0.md`

Everything else is either generated from those specs or an approved human decision that must be
preserved as an explicit artifact.

## Remaining v0.1 Work

The typed contract and root specs now define the semantic chain, but deeper deterministic validators
and renderers remain implementation work. The current backlog is tracked in:

- `.agent-harness/baml/SEMANTIC_CHAIN_V0_1_BACKLOG.md`
