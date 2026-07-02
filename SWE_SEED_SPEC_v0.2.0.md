# SWE_SEED_SPEC v0.2.0

Root contract for the **SweSeed layer**: the outer governance layer of the three-layer
stack. This file is a stable root spec. Detailed design lives in `docs/specs/`; the
reconciliation in `docs/specs/0012-existing-harness-reconciliation.md` is authoritative on
any vocabulary conflict.

## Layer stack

```
SweSeed     (this spec)  governance, capability assembly, layer boundaries
  Harness   (HARNESS_SPEC.md)  routing, proof, context, hooks, traces, learning
    Fabricator (FABRICATOR_SPEC_v0.1.0.md)  product to prototype semantic chain
```

Ownership flows downward only: an inner layer never owns an outer layer's concern, and
cross-layer references point from outer to inner. `LayerName` is `{ SweSeed, Harness,
Fabricator }`.

## Purpose

SWE_Seed centralizes and governs capabilities (skills, MCP servers, hooks, agents, commands,
rules, doctrine) across coding-agent tools. It is the outer layer that assembles capabilities
into a package, validates layer boundaries, and projects into host tools. It is not a skill
pack and not an LLM gateway.

## Owned concepts

Canonical schema: `.agent-harness/baml/baml_src/swe_seed.baml` (contracts as data, no LLM
runtime; see `docs/specs/0019-baml-contracts-as-data.md`).

- `ProjectSeed` — repository purpose, toolchain, command contract, proof requirements.
- `LayerCapability` — an owned capability with `owner` layer, `source_spec`, artifact paths.
- `SeedPackageManifest` — the assembled capability registry.
- `BoundaryReport` / `BoundaryFinding` — layer-boundary validation result.
- `ArtifactMetadata`, `SeedArtifactStatus`, `ReviewRequirement` — provenance and review state.
- `SeedRegenerationInput` / `SeedRegenerationPlan` — idempotent artifact regeneration.

## Governance rules

1. Every governed artifact carries `ArtifactMetadata` provenance and a review status.
2. `ValidateLayerBoundaries` blocks release when ownership is violated or a required
   capability is absent (`BoundaryReport.passed = false`).
3. Regeneration preserves approved decisions and is a no-op when inputs are unchanged.
4. Promotion of any capability requires a live proof pass (see HARNESS_SPEC.md and
   `docs/specs/0013-eval-and-proof.md`). A simulated or waived result never promotes.
5. Provenance completeness is a release gate (`docs/specs/0009-license-and-provenance-boundaries.md`).

## Commands (target Rust surface)

```
swe-seed seed assemble                 # build the SeedPackageManifest
swe-seed seed validate-boundaries      # produce a BoundaryReport (also run by doctor)
swe-seed seed regenerate               # produce an idempotent SeedRegenerationPlan
swe-seed provenance verify             # fail closed on missing hash or license
just fabricate-new <seed>              # run scripts/fabricate.py new
```

## Detailed specs

- `docs/specs/0002-swe-seed-centralization-layer.md` — overview.
- `docs/specs/0003-capability-registry.md` — registry and metadata.
- `docs/specs/0018-layer-boundary-governance.md` — boundary governance.
- `docs/specs/0009-license-and-provenance-boundaries.md` — provenance and clean room.
- `.agents/plans/0001-swe-seed-v0-1-implementation.md` — Rust rewrite plan of record.

## Semantic Specification Chain

The detailed semantic chain contract lives in the numbered specs under `docs/specs/`,
especially `docs/specs/0018-layer-boundary-governance.md` for SweSeed layer governance.
This root spec keeps the stable entry point required by harness validation and points to
the expanded source-of-truth sections.

### Semantic Chain Validation

Semantic Chain Validation is specified in the detailed SweSeed and Fabricator specs. The root
contract keeps the validation marker so harness smoke checks can confirm that chain validation
remains part of the release gate.

## Status

Version 0.2.0. Implementation language is migrating from the Python reference harness to
Rust per the plan of record. `.baml` contracts are the canonical data model.
