# BAML v0.1 Release Criteria

## Required Capabilities

The v0.1 BAML layer is releasable only when these capabilities are present and validated:

- `baml-py` is installed through `uv` and pinned in project dependency files.
- `.agent-harness/baml/baml_src/` contains typed source contracts for SWE Seed, Harness, and
  Fabricator artifacts.
- Fabricator typed contracts represent hypothesis as a job-hypothesis business canvas.
- Fabricator PRD contracts represent requirements in machine-readable EARS structure with stable IDs.
- Fabricator typed contracts represent Job Stories, Y-Statements, SDS components, Gherkin scenarios, and semantic-chain trace links.
- `uv run baml-cli generate --from .agent-harness/baml/baml_src` exits with code 0.
- Generated BAML client code is ignored by git and cannot become the operating source of truth.
- Markdown/YAML artifacts remain reviewable, versioned, and testable.
- Local eval artifacts are file-first and do not depend on hosted services.
- Regeneration commands produce reviewed files or proposed diffs, never silent overwrites of approved
  artifacts.

## Release Gates

- `pnpm exec prettier --check .`
- `uv run ruff check .`
- `uv run baml-cli generate --from .agent-harness/baml/baml_src`
- `grep -q 'enum EARSPattern' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'class EARSRequirement' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'target_job string' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'class JobStory' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'class YStatement' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'class SDSComponent' .agent-harness/baml/baml_src/fabricator.baml`
- `grep -q 'class GherkinScenario' .agent-harness/baml/baml_src/fabricator.baml`
- `python scripts/harness.py validate`
- `bash tests/validate-harness.sh`
- `just ci`

## Prohibited Release Evidence

The release cannot rely on unresolved task notes, placeholders, mocks, stubs, synthetic proof,
disabled checks, stale context, or unsupported completion claims. A missing capability is recorded as
a failed release gate or an explicit non-release decision, not as completed work.
