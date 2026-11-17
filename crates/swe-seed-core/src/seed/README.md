# seed

## Overview

SweSeed outer layer (specs 0003/0009/0018): 3-layer ownership governance
(`SweSeed -> Harness -> Fabricator`), capability assembly into
`SeedPackageManifest`, downward-only boundary validation, the self-describing
`ProjectSeed`, and idempotent regeneration planning. Every governed artifact
carries `ArtifactMetadata` (provenance + status). Types are baml-faithful and
assert `BamlParity`.

## Interface

- `pub enum LayerName { SweSeed, Harness, Fabricator }` with `rank()`
  (SweSeed=1, Harness=2, Fabricator=3; outer-to-inner) and `BamlParity`
  [mod.rs:LayerName], [mod.rs:rank]
- `pub enum SeedArtifactStatus { Draft, Candidate, Active, Deprecated,
Approved, Rejected }` and `pub enum ReviewRequirement { Required, Optional,
NotRequired }` (both `BamlParity`, PascalCase serde) [mod.rs:SeedArtifactStatus],
  [mod.rs:ReviewRequirement]
- `SeedSourceRef { path, summary }`, `SeedValidationRequirement { check,
blocking, evidence }`, `SeedNeed { request, desired_outcome, constraints,
non_goals, proof_requirements }`, `ArtifactMetadata { artifact_type,
artifact_id, source_spec, generated_by, status, version,
requires_human_review, linked_artifacts }` (all `BamlParity`)
  [mod.rs:SeedSourceRef], [mod.rs:SeedValidationRequirement], [mod.rs:SeedNeed],
  [mod.rs:ArtifactMetadata]
- `pub struct LayerCapability { id, owner, source_spec, artifact_paths,
purpose, required }` (`BamlParity`) and `builtin_capabilities()`: the built-in
  registry of 13 capabilities across the three layers (e.g. `layer-governance`,
  `provenance`, `routing`, `traces`, `eval-proof`, `doctor`, `fabricator`)
  [capability.rs:LayerCapability], [capability.rs:builtin_capabilities]
- `pub struct ProjectSeed { metadata, repository_purpose, toolchain,
command_contract, scaffold_outcome, constraints, non_goals,
proof_requirements, validation }` (`BamlParity`) and
  `self_project_seed()`: the self-describing seed for this repository
  [project_seed.rs:ProjectSeed], [project_seed.rs:self_project_seed]
- `pub struct SeedPackageManifest { metadata, project_seed_path, capabilities,
generated_paths, validation }` (`BamlParity`);
  `pub const DEFAULT_MANIFEST_PATH: &str =
".swe-seed/seed-package-manifest.json"`; `assemble(seed_path, seed,
capabilities)`, `assemble_default()`, `write_manifest(manifest, path)`,
  `read_manifest(path)` [manifest.rs:SeedPackageManifest],
  [manifest.rs:DEFAULT_MANIFEST_PATH], [manifest.rs:assemble],
  [manifest.rs:assemble_default], [manifest.rs:write_manifest],
  [manifest.rs:read_manifest]
- `infer_layer(path) -> Option<LayerName>`: infers a path's layer from its
  prefix (`.fabricator` / `src/fabricator` -> Fabricator; `.agent-harness`,
  `src/route`, `src/trace`, ... -> Harness; `.swe-seed`, `crates/swe-seed`,
  `src/seed`, ... -> SweSeed) [boundary.rs:infer_layer]
- `validate_boundaries(manifest) -> BoundaryReport` with `BoundaryFinding
{ layer, artifact_path, issue, severity }` and `BoundaryReport { metadata,
findings, passed, validation }` (both `BamlParity`)
  [boundary.rs:validate_boundaries], [boundary.rs:BoundaryFinding],
  [boundary.rs:BoundaryReport]
- `SeedRegenerationInput` and `SeedRegenerationPlan { source_artifacts,
proposed_diffs, migration_notes, preserved_decisions, validation }` (both
  `BamlParity`) with `regenerate(root, input) -> SeedRegenerationPlan`
  [regenerate.rs:SeedRegenerationInput], [regenerate.rs:SeedRegenerationPlan],
  [regenerate.rs:regenerate]

## Invariants

- Layer order is outer-to-inner: a layer may reference only itself or a more
  inner layer; `infer_layer` returns `None` for unrecognized paths so they
  surface as findings instead of silently counting as valid outer-layer
  artifacts [mod.rs:LayerName], [boundary.rs:infer_layer]
- `passed=false` blocks release: any duplicate capability id, unclassified
  artifact path (medium), upward reference (high), or missing required
  registry capability (high) is a finding and fails the report
  [boundary.rs:validate_boundaries]
- Dependencies must point inward only: a capability owned by layer rank `r`
  referencing an artifact inferred at an outer rank is an upward-reference
  finding [boundary.rs:validate_boundaries]
- `assemble` records the real `project_seed_path` (the resolvable source of the
  seed) rather than a fixed placeholder, so manifests preserve provenance for
  later reads and regeneration [manifest.rs:assemble]
- Regeneration is idempotent: a present artifact is preserved, a missing one
  becomes a `recreate <kind>: <path>` proposed diff, and output vectors are
  sorted, so identical inputs yield byte-identical plans
  [regenerate.rs:regenerate]
- The self seed's constraints pin the governance contract: 3-layer
  downward-only references, no LLM runtime (`.baml` is contracts-as-data), and
  the inner stack passing with federation disabled (standalone invariant)
  [project_seed.rs:self_project_seed]

## Side Effects

- `write_manifest` creates parent directories and writes pretty JSON
  [manifest.rs:write_manifest]
- `read_manifest` reads the manifest JSON file [manifest.rs:read_manifest]
- `regenerate` stats each approved artifact path under `root`
  (`root.join(path).exists()`) [regenerate.rs:resolve_one]

## Verification

- Run: `cargo test -p swe-seed-core`
