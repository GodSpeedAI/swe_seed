// BAML source contract for the SWE Seed outer layer.
// Harness generation lives in harness.baml.
// Fabricator generation lives in fabricator.baml.

enum SeedArtifactStatus {
  Draft
  Candidate
  Active
  Deprecated
  Approved
  Rejected
}

enum ReviewRequirement {
  Required
  Optional
  NotRequired
}

enum LayerName {
  SweSeed
  Harness
  Fabricator
}

class SeedSourceRef {
  path string
  summary string
}

class SeedValidationRequirement {
  check string
  blocking bool
  evidence string
}

class SeedNeed {
  request string
  desired_outcome string
  constraints string[]
  non_goals string[]
  proof_requirements string[]
}

class ArtifactMetadata {
  artifact_type string
  artifact_id string
  source_spec string
  generated_by string
  status SeedArtifactStatus
  version string
  requires_human_review ReviewRequirement
  linked_artifacts string[]
}

class ProjectSeed {
  metadata ArtifactMetadata
  repository_purpose string
  toolchain string[]
  command_contract string[]
  scaffold_outcome string
  constraints string[]
  non_goals string[]
  proof_requirements string[]
  validation SeedValidationRequirement[]
}

class LayerCapability {
  id string
  owner LayerName
  source_spec string
  artifact_paths string[]
  purpose string
  required bool
}

class SeedPackageManifest {
  metadata ArtifactMetadata
  project_seed_path string
  capabilities LayerCapability[]
  generated_paths string[]
  validation SeedValidationRequirement[]
}

class BoundaryFinding {
  layer LayerName
  artifact_path string
  issue string
  severity string
}

class BoundaryReport {
  metadata ArtifactMetadata
  findings BoundaryFinding[]
  passed bool
  validation SeedValidationRequirement[]
}

class SeedRegenerationInput {
  swe_seed_spec_path string
  approved_project_seeds string[]
  approved_seed_package_manifests string[]
  approved_layer_capability_maps string[]
  approved_lower_layer_artifact_refs string[]
}

class SeedRegenerationPlan {
  source_artifacts SeedSourceRef[]
  proposed_diffs string[]
  migration_notes string[]
  preserved_decisions string[]
  validation SeedValidationRequirement[]
}

function GenerateProjectSeed(need: SeedNeed, repo_constraints: string[]) -> ProjectSeed {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a bounded SWE Seed ProjectSeed.
    Preserve outcome, constraints, non-goals, command contract, and proof requirements.

    Need:
    {{ need }}

    Repository constraints:
    {{ repo_constraints }}

    {{ ctx.output_format }}
  "#
}

function SelectLayerCapabilities(project_seed: ProjectSeed, available_specs: SeedSourceRef[]) -> LayerCapability[] {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Select the required layer capabilities for this project seed.
    Do not redefine harness or fabricator contracts.
    Reference the owning layer and source spec for each capability.

    Project seed:
    {{ project_seed }}

    Available specs:
    {{ available_specs }}

    {{ ctx.output_format }}
  "#
}

function AssembleSeedPackage(project_seed: ProjectSeed, capabilities: LayerCapability[]) -> SeedPackageManifest {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Assemble a seed package manifest from the project seed and layer capabilities.
    Package approved lower-layer artifacts by reference instead of copying their schemas.

    Project seed:
    {{ project_seed }}

    Capabilities:
    {{ capabilities }}

    {{ ctx.output_format }}
  "#
}

function ValidateLayerBoundaries(specs: SeedSourceRef[], manifests: SeedPackageManifest[]) -> BoundaryReport {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Validate layer boundaries across SWE Seed, Harness, and Fabricator artifacts.
    Dependencies may point inward only: SWE Seed may depend on Harness or Fabricator,
    Harness may depend on Fabricator handoff/proof artifacts, and Fabricator must not
    depend on either outer layer. Flag duplicated ownership and any inward-layer
    dependency on an outer layer.

    Specs:
    {{ specs }}

    Manifests:
    {{ manifests }}

    {{ ctx.output_format }}
  "#
}

function RegenerateSeedArtifacts(inputs: SeedRegenerationInput) -> SeedRegenerationPlan {
  client "openai-responses/gpt-5-mini"
  prompt #"
    Generate a regeneration plan for SWE Seed outer-layer artifacts.
    Preserve human decisions, version history, constraints, and lower-layer artifact references.
    Produce proposed diffs or migration notes instead of overwriting approved artifacts.

    Inputs:
    {{ inputs }}

    {{ ctx.output_format }}
  "#
}
