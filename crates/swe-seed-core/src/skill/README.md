# skill

## Overview

Skill ingestion and render (spec 0007). `SkillIR` mirrors `harness.baml` and
loads tolerantly from the richer runtime JSON (`skills/*.json`). The ingest
pipeline is `discover -> fetch -> scan -> normalize -> project -> verify`: a
blocking SkillSpector scan prevents projection, and activation requires
`source_hash` plus a terminal, non-blocking scan. Render writes deterministic
host templates per `render_target` kind.

## Interface

- `pub struct SkillIR` (harness.baml `SkillIR`: `id`, `version`, `category`,
  `jtbd`, `description`, `triggers`, `inputs`, `procedure`,
  `evidence_required`, `forbidden_behaviors`, `outputs`, `success_criteria`,
  `failure_modes`, `render_targets`, `status`, `validation`, `sources`) with
  `BamlParity`; `SkillSourceRef` [ir.rs:SkillIR], [ir.rs:BamlParity],
  [ir.rs:SkillSourceRef]
- `pub enum ArtifactStatus { Draft, Candidate, Active, Deprecated, Rejected }`
  (lowercase serde variant naming matches the runtime JSON, e.g. `"active"`)
  [ir.rs:ArtifactStatus]
- `pub struct SkillRecord { ir, source_hash, scan, path }`: an ingested skill
  with its provenance hash and scan result [ingest.rs:SkillRecord]
- `skills_dir(root)` = `.agent-harness/skills`; `discover(root)` (all
  `skills/**/*.json`, sorted), `fetch(path)`, `normalize(raw)`
  [ingest.rs:skills_dir], [ingest.rs:discover], [ingest.rs:fetch],
  [ingest.rs:normalize]
- `ingest_one(path)`: full ingest; the SkillSpector scan runs BEFORE
  normalization [ingest.rs:ingest_one]
- `ingest_with_scan(path, scan)`: ingest with an injected scan result (tests)
  [ingest.rs:ingest_with_scan]
- `pipeline(root, exceptions)` and `pipeline_scanning(root, exceptions,
scanner)`: ordered pipeline over all discovered skills, returning
  `(records, blocked_ids)` [ingest.rs:pipeline], [ingest.rs:pipeline_scanning]
- `RenderTarget { kind, rel_path, content }`; `render_skill(skill) ->
Vec<RenderTarget>` (pure, deterministic); `render_all(root, out_root,
exceptions) -> (rendered_count, blocked_ids)`
  [render.rs:RenderTarget], [render.rs:render_skill], [render.rs:render_all]

## Invariants

- Scan before normalize: `ingest_one` runs the scan immediately after the file
  is confirmed readable and before any normalization/projection
  [ingest.rs:ingest_one]
- A blocking scan prevents projection/render: blocked skills are reported by id
  and never rendered [ingest.rs:pipeline_scanning], [render.rs:render_all],
  [gate.rs via security::scan_blocks_projection]
- Tolerant loading has fixed boundaries: `version` accepts a JSON string or
  number only; `inputs` accepts a string array or `{"required": [...]}` only;
  non-string entries and other shapes are deserialization errors; extra JSON
  fields are ignored [ir.rs:flex_string], [ir.rs:flex_string_list]
- `render_skill` is a pure function, deterministic for the same input (the
  basis of the golden test) [render.rs:render_skill]
- Render paths are traversal-safe: `safe_component` drops path separators,
  control characters, and dot components from `skill.id`/`category`, falling
  back to `"unsafe-skill"`, so a malicious identifier cannot escape the
  render-targets hierarchy [render.rs:safe_component]
- Front matter is YAML-scalar-safe: `yaml_scalar` collapses newlines/control
  characters and quotes strings containing `:`, `#`, `---`, or YAML indicator
  prefixes [render.rs:yaml_scalar]
- Generated files are marked as generated: every template carries the header
  "Generated from Skill IR: <id>@<version> ... Do not edit this generated file
  directly." [render.rs:render_skill]
- An unsupported `render_target` kind yields an explicit placeholder file under
  `render-targets/other/` rather than being silently skipped
  [render.rs:render_skill]

## Side Effects

- `discover` recursively walks `.agent-harness/skills/` reading directory
  entries [ingest.rs:discover]
- `fetch` / `ingest_one` / `ingest_with_scan` read skill JSON files from disk
  [ingest.rs:fetch], [ingest.rs:ingest_one]
- `pipeline` and `render_all` invoke the external SkillSpector binary per skill
  (via `run_skillspector` or the injected scanner)
  [ingest.rs:pipeline], [render.rs:render_all]
- `render_all` creates target directories and writes rendered files under
  `out_root/.agent-harness/render-targets/...` [render.rs:render_all]

## Verification

- Run: `cargo test -p swe-seed-core`
