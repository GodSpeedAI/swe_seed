# Configuration & BAML Schemas Reference

This document provides a field-by-field reference for SWE_SEED configuration files and canonical BAML contract schemas.

---

## 1. `.agent-harness/config.yaml`

The top-level configuration file governing harness behavior:

```yaml
version: "0.2.0"

# Routing engine settings
routing:
  default_job_type: "implementation"
  cards_dir: ".agent-harness/routes"
  enable_recording: true

# Context budget settings
context:
  policy_file: ".agent-harness/context/budget-policy.yaml"
  max_pack_bytes: 524288 # 512 KB
  max_pack_tokens: 32000

# Hook runtime settings: hooks load these from .agent-hooks/config.yaml, not this file
hooks:
  # Log directory; one file per UTC date: logs/events-<YYYY-MM-DD>.jsonl
  logs: ".agent-hooks/logs"
  payloads: ".agent-hooks/payloads"
  artifacts: ".agent-hooks/artifacts"
  # SQLite index rebuilt from the JSONL logs by `swe-seed agent-hooks index`
  index_db: ".agent-hooks/index/hooks.rusql"
  vector_index_root: ".agent-hooks/index/vectors"

# Host projection settings
projection:
  snapshots_dir: ".agent-harness/snapshots"
  managed_marker_begin: "<!-- BEGIN SWE_SEED MANAGED BLOCK -->"
  managed_marker_end: "<!-- END SWE_SEED MANAGED BLOCK -->"

# Federation settings
federation:
  enabled: false
  active_key_id: "default"
  keys_dir: ".agent-harness/federation/keys"
  private_keys_dir: ".swe-seed/federation/keys"

# Continuous learning settings
learning:
  auto_apply: false
  reflections_dir: ".agent-harness/reflections"
```

---

## 2. BAML Schemas (`.agent-harness/baml/baml_src/`)

### `swe_seed.baml` (Outer Layer)

#### `enum LayerName`
- `SweSeed`: Outer governance layer.
- `Harness`: Middle routing and execution layer.
- `Fabricator`: Inner prototyping layer.

#### `class ProjectSeed`
- `name: string`: Project identifier.
- `purpose: string`: Core mission statement.
- `toolchain: string[]`: Required tool binaries (e.g. `["cargo", "just", "git"]`).
- `command_contract: string`: Default command runner (e.g. `"just"`).
- `proof_requirements: string[]`: Minimum proof commands (e.g. `["just ci"]`).

#### `class LayerCapability`
- `id: string`: Unique capability identifier.
- `owner: LayerName`: Assigned owning layer.
- `kind: string`: Variant (`skill`, `mcp`, `rule`, `hook`, `command`, `doctrine`).
- `source_spec: string`: Path to defining specification.
- `artifacts: string[]`: Concrete files associated with this capability.
- `metadata: ArtifactMetadata`: Provenance tracking.

#### `class BoundaryReport`
- `passed: bool`: True if zero layer violations exist.
- `findings: BoundaryFinding[]`: List of detected violations.
- `timestamp: string`: ISO 8601 evaluation timestamp.

---

### `harness.baml` (Middle Layer)

#### `class RouteCard`
- `id: string`: Card identifier.
- `job_type: string`: Corresponding job type from the 11 canonical types.
- `purpose: string`: Explanation of when this work pattern applies.
- `semantic_triggers: string[]`: Keywords triggering this route.
- `positive_examples: string[]`: Representative user prompts.
- `negative_examples: string[]`: Prompts that must NOT match this card.
- `required_context: string[]`: File paths the agent is required to read.
- `required_skills: string[]`: Active skill identifiers required.
- `work_loop: string[]`: Ordered execution stages.
- `required_artifacts: string[]`: Tangible deliverables demanded.
- `proof: string[]`: Commands that must pass before claiming completion.
- `done_when: string[]`: Explicit completion conditions.
- `failure_modes: string[]`: Common failure patterns to avoid.

#### `class TraceRecord`
- `trace_id: string`: Unique timestamped identifier.
- `task: string`: Original prompt.
- `job_type: string`: Selected route.
- `route_card: string`: Path to selected route card.
- `events: TraceEvent[]`: Ordered list of lifecycle transitions.
- `checkpoints: TraceCheckpoint[]`: Milestone summaries.
- `claim: string?`: Agent completion assertion.
- `proof_record: ProofRecord?`: Executed command evidence.

---

### `fabricator.baml` (Inner Layer)

#### `class TraceabilityLink`
- `parent_artifact: string`: Path or ID of upstream parent file.
- `parent_sha256: string`: Cryptographic hash of parent content.
- `relationship: string`: Nature of link (e.g. `derives_from`, `satisfies`, `tests`).

#### `class SemanticChainValidationReport`
- `run_id: string`: Identifier of target fabricator run.
- `passed: bool`: True if all 10 links are unbroken and valid.
- `missing_nodes: string[]`: Any absent chain artifacts.
- `broken_links: string[]`: Links pointing to invalid parents.
- `syntax_errors: string[]`: EARS or Gherkin parsing errors.
