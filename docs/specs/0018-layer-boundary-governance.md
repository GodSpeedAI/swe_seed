# 0018 — Layer Boundary Governance

## Purpose

Define the outer **SweSeed-layer** governance: the 3-layer model
(`SweSeed → Harness → Fabricator`), capability assembly into a `SeedPackageManifest`, layer-
boundary validation (`BoundaryReport`), and idempotent regeneration. This is the centralization
spine specs 0002/0003 were reaching for, now expressed in the proven vocabulary.

## Non-goals

- Not a host adapter (0004) — boundary governance is host-neutral.
- Not runtime enforcement of every call — it validates *artifacts and layer ownership*.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `swe_seed.baml` | `LayerName{SweSeed,Harness,Fabricator}`, `ProjectSeed`, `LayerCapability{id,owner,source_spec,artifact_paths,purpose,required}`, `SeedPackageManifest{project_seed_path,capabilities,generated_paths}`, `BoundaryFinding{layer,artifact_path,issue,severity}`, `BoundaryReport{findings,passed}`, `SeedRegenerationInput`, `SeedRegenerationPlan`, `ArtifactMetadata`, `SeedArtifactStatus`, `ReviewRequirement`; functions `SelectLayerCapabilities`, `AssembleSeedPackage`, `ValidateLayerBoundaries`, `RegenerateSeedArtifacts` |

## SWE_Seed requirements

1. **3-layer ownership**: every capability/artifact has an `owner` `LayerName`. A lower layer
   may not own an upper layer's concern; cross-layer references go downward only
   (`SweSeed → Harness → Fabricator`).
2. **Capability assembly**: `SelectLayerCapabilities(project_seed, specs)` chooses required
   `LayerCapability`s; `AssembleSeedPackage` produces a `SeedPackageManifest`, the assembled
   registry artifact that replaces the earlier spec 0003 `registry.toml` placeholder.
3. **Boundary validation** (`ValidateLayerBoundaries` → `BoundaryReport`): findings for
   misplaced artifacts, upward references, missing required capabilities, or
   ownership/`source_spec` mismatches. `passed=false` blocks release. Run by `doctor` (0008).
4. **ProjectSeed** declares repository purpose, toolchain, command contract, scaffold
   outcome, constraints, non-goals, proof requirements — the seed of a centralized repo.
5. **Idempotent regeneration**: `RegenerateSeedArtifacts(input) → SeedRegenerationPlan` with
   `proposed_diffs`, `migration_notes`, `preserved_decisions`. Regeneration must preserve
   approved decisions and be a no-op when inputs are unchanged (deterministic).
6. **ArtifactMetadata** carries provenance + `SeedArtifactStatus` (Draft/Candidate/Active/
   Deprecated/Approved/Rejected) + `ReviewRequirement` for every governed artifact (0009).

## Data model

All `swe_seed.baml` types canonical (0019). `SeedPackageManifest` is the top-level registry
artifact; there is no separate authoritative `registry.toml` file in the reconciled model.

```toml
# LayerCapability
id = "trace-subsystem"
owner = "Harness"
source_spec = "docs/specs/0014-trace-and-durable-decisions.md"
artifact_paths = ["swe_seed::trace", ".agent-harness/traces/"]
purpose = "durable decisions + session continuity"
required = true
```

- **Rust**: `swe_seed::seed` (`project_seed.rs`, `capability.rs`, `manifest.rs`,
  `boundary.rs`, `regenerate.rs`).
- **Validation**: ownership/source_spec consistent; no upward references; required caps present.

## CLI behavior

```
swe-seed seed assemble [--project-seed <path>]     # → SeedPackageManifest
swe-seed seed validate-boundaries                  # → BoundaryReport (also run by doctor)
swe-seed seed regenerate [--input <path>]          # → SeedRegenerationPlan (idempotent)
```

## Generated files

`SeedPackageManifest` (the registry artifact), `BoundaryReport`, `SeedRegenerationPlan` under
`.swe-seed/` (or `.agent-harness/`), each with `ArtifactMetadata` + content hash.

## Rust module boundaries

`swe_seed::seed` is the outer layer; consumes registry concepts (0003) and is validated by
`doctor` (0008). Host adapters (0004) project from the assembled manifest.

## Security and provenance considerations

Boundary governance prevents layer leakage (e.g. Fabricator owning Harness policy).
Regeneration preserves approved decisions — no silent loss. Every artifact has provenance +
review status.

## Tests

- An artifact owned by the wrong layer → `BoundaryReport.passed = false`.
- An upward cross-layer reference → finding.
- `regenerate` with unchanged inputs → empty diff (idempotent); preserves approved decisions.
- Missing a `required` capability → boundary fail.

## Resolved registry shape

`SeedPackageManifest` is the registry artifact. Any legacy `registry.toml` wording in older
specs maps to this manifest.

## Acceptance criteria

- [ ] 3-layer ownership enforced; downward-only references.
- [ ] `assemble`/`validate-boundaries`/`regenerate` implemented; doctor runs boundary checks.
- [ ] Regeneration idempotent and decision-preserving; every artifact carries provenance.
