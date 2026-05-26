# SWE SEED CI/CD Development Harness Specification v0.2.0

## Purpose

This specification defines a CI/CD development harness that gives humans and agents one stable way to install tools, run checks, manage secrets, debug failures, observe harness behavior, and prove work is ready.

The desired outcome is behavioral: fewer hidden commands, fewer environment surprises, less completion theater, and faster recovery when the harness or its adapters misbehave. A contributor should know what to run, what the result means, what evidence was captured, and which document to update when the harness changes.

## Layer Position

SWE Seed is the outer layer. It packages, installs, validates, and regenerates the system without
owning every inner behavior.

```text
SWE Seed outer layer
-> Harness middle layer
-> Fabricator inner layer
```

SWE Seed MUST remain self-contained as a seed and dev-harness specification. It MAY include approved
harness and fabrication artifacts in a generated project, but it MUST NOT duplicate the inner layer
contracts.

Layer ownership:

- SWE Seed owns project scaffolding, local/CI command parity, tool installation, dev-harness
  observability, secrets workflow, root regeneration, and project packaging.
- The harness owns agent routing, Skill IR, context stewardship, traceability, proof gates,
  reflection, and supervised skill learning.
- The fabricator owns product-to-prototype artifact packets, product seeds, PRDs, SDS documents, TDD
  plans, agent tasks, prototype proof records, and fabrication learning.

SWE Seed MAY orchestrate the lower layers through files, commands, and stable identifiers. It MUST
NOT require the harness or fabricator to know about SWE Seed.

Dependency direction is one-way inward:

- SWE Seed MAY depend on the harness and fabricator.
- The harness MAY depend on the fabricator only through explicit handoff and proof artifacts.
- The harness MUST NOT depend on SWE Seed.
- The fabricator MUST NOT depend on the harness or SWE Seed.
- Inner layers MUST NOT import, call, branch on, or require outer-layer runtime behavior.

## Enterprise Completion Policy

SWE Seed MUST reject unresolved todos, placeholders, mocks, stubs, disabled checks, synthetic proof,
and unsupported completion claims. These are not accepted as implementation, validation,
regeneration, or release evidence.

If a generated project is incomplete, the system MUST record the gap as a failed gate, blocked trace,
eval issue, regression case, ADR, or explicit non-release decision. It MUST NOT label incomplete work
as done, enterprise-ready, shippable, validated, or production-ready.

Completion theater is a release-blocking defect. A release claim is valid only when local commands,
EvalResult artifacts, ProofRecords, reflection, and adaptation decisions support the claim.

## Core Principle

```text
Remote CI is not a separate truth.
The local just interface is the developer and agent API.
GitHub Actions should call the same commands wherever practical.
Observability is file-first, language-agnostic, and rebuildable.
```

## Required Command Interface

The project MUST provide a `justfile` with these recipes:

- `just bootstrap`: install or synchronize local tool dependencies.
- `just doctor`: report missing required or recommended tools.
- `just format`: run formatting checks.
- `just lint`: run static checks.
- `just test`: run harness and project tests.
- `just ci`: run the local mirror of remote CI.
- `just secrets-encrypt`: encrypt plaintext secret inputs.
- `just secrets-decrypt`: decrypt encrypted secrets into ignored local files.
- `just secrets-edit <file>`: edit a SOPS-managed secret.
- `just secrets-rotate-key`: rotate SOPS recipients or age keys.
- `just harness-eval-run <spec> [output]`: execute a local EvalSpec through the harness runner and optionally write an EvalResult file.

When the fabricator layer is packaged inside SWE Seed, the project MUST also expose these bounded
outer-layer recipes:

- `just fabricate-new <seed>`
- `just fabricate-generate <run_id>`
- `just fabricate-validate <run_id>`
- `just fabricate-handoff <run_id>`
- `just fabricate-proof <run_id>`
- `just fabricate-reflect <run_id>`
- `just fabricate-status <run_id>`

`just ci` MUST be the proof command for ordinary completion claims.

## Required Scaffold

The harness MUST include:

- GitHub Actions workflow under `.github/workflows/ci.yml`.
- Local command scripts under `scripts/`.
- `.vscode/extensions.json` and `.vscode/settings.json`.
- `.editorconfig`.
- `.gitignore`.
- `.gitattributes`.
- `.env.example`.
- `.envrc`.
- `.mise.toml`.
- `devbox.json`.
- `package.json` and lockfile when Node-based tooling is used.
- `pyproject.toml` when Python tooling is used.
- SOPS and age configuration for secret handling.
- A validation script that checks required harness files and command contracts.

When the fabricator layer is packaged inside SWE Seed, the scaffold MUST also include:

- `.fabricator/config.yaml`
- `.fabricator/runs/README.md`
- `.fabricator/schemas/` for the fabricator packet contract
- `.fabricator/templates/` for the rendered packet surfaces
- `scripts/fabricate.py` or an equivalent documented local command surface
- `docs/fabrication-layer/` operator documentation

When the fabricator layer is packaged inside SWE Seed, the validation script MUST also guard
against lower-layer contract drift that a fresh agent could otherwise miss. At minimum it SHOULD
catch:

- missing required fabricator command aliases,
- missing required fabrication docs,
- missing required schema files or template files,
- regressions where governed schema files drop required fields or array item schemas,
- regressions where packaged proof or handoff artifacts drift from the documented command contract.

The repository root MUST include these canonical root specs:

- `SWE_SEED_SPEC_v0.2.0.md`
- `HARNESS_SPEC.md`
- `FABRICATOR_SPEC_v0.1.0.md`

When BAML is enabled, the scaffold MUST include these required contracts and release-governance files:

- `.agent-harness/baml/baml_src/swe_seed.baml`
- `.agent-harness/baml/baml_src/harness.baml`
- `.agent-harness/baml/baml_src/fabricator.baml`
- `.agent-harness/baml/MIGRATION_NOTE.md`
- `.agent-harness/baml/V0_1_RELEASE_CRITERIA.md`
- `.agent-harness/evals/schemas/eval-artifacts.schema.yaml`

The scaffold SHOULD stay small. Add tools only when they remove a real ambiguity or make the proof path more reliable.

## BAML Role in SWE Seed

BAML MAY be used as an optional first-class typed artifact generation layer for SWE Seed projects.
It defines typed generation functions for seed, harness, product, context, proof, trace, reflection,
and learning artifacts.

BAML does not replace SWE Seed, the harness, Markdown, YAML, human review, or proof gates. The
relationship is:

- BAML produces structured outputs.
- Renderers convert those outputs into Markdown/YAML.
- Markdown/YAML remains the human-legible source used by agents.
- Generated artifacts MUST remain reviewable, versioned, and testable.
- BAML supports regeneration, but it MUST NOT bypass human approval or proof gates.

If BAML is enabled and not already installed, it MUST be installed through `uv`:

```bash
uv add baml-py
uv run baml-cli generate --from .agent-harness/baml/baml_src
```

Implementations SHOULD pin the BAML dependency in Python project metadata and lockfiles so regeneration is reproducible. The recommended minimum is `baml-py>=0.222.0`.

The generated BAML client is implementation machinery. It is not the operating surface for agents.

## SWE Seed Artifact Classes

SWE Seed MAY generate, validate, package, or regenerate artifacts across the system, but ownership
stays with the layer that defines the artifact contract.

| Artifact             | Owning layer          | Practical meaning                                                                                           |
| -------------------- | --------------------- | ----------------------------------------------------------------------------------------------------------- |
| `ProjectSeed`        | SWE Seed              | Bounded input describing repository purpose, toolchain, command contract, and desired scaffold outcome.     |
| `HarnessSpec`        | Harness               | Source-of-truth contract for agent routing, context, proof, traceability, and learning.                     |
| `HarnessADR`         | Harness               | Harness decision record with context, decision, consequences, validation, and rollback.                     |
| `RouteCard`          | Harness               | Executable route contract for a class of agent work.                                                        |
| `SkillIR`            | Harness               | Reusable agent behavior with triggers, procedure, evidence, outputs, guardrails, and evals.                 |
| `EvalSpec`           | Harness               | Checks that decide whether generated or agent-produced output is acceptable.                                |
| `ContextBudget`      | Harness               | Limits required, optional, excluded, stale, and forbidden context for a route.                              |
| `ContextPack`        | Harness or Fabricator | Intentionally generated context bundle for one routed task or fabrication handoff.                          |
| `HookPolicy`         | Harness               | Lifecycle event policy for routing, proof, trace capture, safety, and failure handling.                     |
| `PermissionPolicy`   | Harness or Fabricator | Allowed, approval-gated, and forbidden actions for agents or hooks.                                         |
| `TraceSchema`        | Harness               | Required fields connecting request, route, artifacts, commands, proof, risks, and claims.                   |
| `ReflectionTemplate` | Harness or Fabricator | Evidence-bearing post-run prompt set for learning capture.                                                  |
| `LearningRecord`     | Harness or Fabricator | Approved, rejected, or no-change lesson with provenance.                                                    |
| `SkillProposal`      | Harness or Fabricator | Proposed skill addition or change with evidence, evals, rollout, and rollback.                              |
| `RegressionCase`     | Harness or Fabricator | Known failure case that validation should catch in future runs.                                             |
| `ProductSeed`        | Fabricator            | Bounded input describing product idea, target user, constraints, and desired prototype outcome.             |
| `ADR`                | Fabricator            | Product or implementation decision record.                                                                  |
| `PRD`                | Fabricator            | Product requirements document with goals, non-goals, users, scope, and acceptance criteria.                 |
| `SDS`                | Fabricator            | Software design spec describing architecture, components, data flow, risks, and interfaces.                 |
| `TDDPlan`            | Fabricator            | Test-driven implementation plan with checks, implementation steps, and proof.                               |
| `AgentTask`          | Fabricator            | Bounded instruction packet used by Claude Code/OpenClaw-style agents for one prototype task.                |
| `ProofRecord`        | Fabricator or Harness | Recorded evidence that tests, checks, or manual verification passed or were explicitly skipped with reason. |

Fabricator PRDs SHOULD express requirements in EARS-structured form so they are machine-readable,
traceable, and deterministic to validate.

SWE Seed SHOULD package lower-layer artifacts and validate their presence and freshness. It SHOULD NOT
redefine the lower-layer schemas when the harness or fabricator spec already owns them.

## Semantic Specification Chain

SWE Seed packages a semantic specification chain that preserves product intent from discovery through
requirements, architecture, design, behavior, evaluation, execution, proof, and learning.

Canonical chain:

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

Purpose:

- preserve original user or system need,
- preserve situation and context,
- preserve desired outcome,
- preserve constraints,
- preserve rejected alternatives,
- preserve system structure,
- preserve behavioral examples,
- preserve eval checks,
- preserve proof evidence,
- preserve learning and adaptation outputs.

A downstream artifact is invalid if it cannot trace to an upstream artifact or an explicit approved
waiver.

### Artifact-To-Syntax Mapping

| Layer                          | Artifact                   | Required syntax          | Purpose                                                                    |
| ------------------------------ | -------------------------- | ------------------------ | -------------------------------------------------------------------------- |
| Problem anchor                 | `JTBD.md` or `ProductSeed` | Job Story                | Capture human situation, motivation, and desired outcome                   |
| Requirement                    | `PRD.md`                   | EARS                     | Convert intent into deterministic system requirements                      |
| Architecture decision          | `ADR.md`                   | Y-Statement              | Record architectural trade-off, rejected alternatives, and accepted costs  |
| Structural design              | `SDS.md`                   | C4 model + Mermaid       | Define context, containers, components, relationships, and boundaries      |
| Behavioral design or test seed | `SDS.md` and-or `TDD.md`   | Gherkin                  | Define executable examples of expected behavior                            |
| Evaluation                     | `EVAL_SPEC.yaml`           | EvalSpec                 | Define product, process, learning, and adaptation checks                   |
| Agent execution                | `AGENT_TASK.md`            | bounded task packet      | Define mission, allowed actions, forbidden actions, and definition of done |
| Proof                          | `PROOF_RECORD.md`          | observed evidence record | Record what actually passed, failed, or was manually verified              |

Clarifications:

- Gherkin MAY appear in `SDS.md` as behavioral design and in `TDD.md` as executable test seed.
- `EvalSpec` is the source of truth for what must be checked.
- `ProofRecord` is the source of truth for what actually happened.

### Stable Traceability IDs

The semantic chain SHOULD use stable IDs with these prefixes:

- `JOB-` for Job Stories
- `REQ-` for EARS requirements
- `ADR-` for architecture decisions
- `Y-` for Y-Statements
- `SDS-` for design specs
- `CMP-` for components
- `SCN-` for Gherkin scenarios
- `EVAL-` for eval specs
- `CHK-` for eval checks
- `TASK-` for agent tasks
- `PROOF-` for proof records
- `REG-` for regression cases
- `SKILL-` for skill proposals or Skill IR

Each artifact SHOULD include frontmatter or structured metadata linking upstream and downstream IDs
where practical.

## Typed Artifact Generation

SWE Seed generation SHOULD follow this path:

```text
Seed Input
-> typed BAML output
-> semantic specification chain artifacts
-> rendered Markdown/YAML artifact
-> validation
-> agent context
-> execution trace
-> proof record
-> reflection
-> learning artifact
```

Generated Markdown/YAML artifacts SHOULD include frontmatter metadata where useful:

- `artifact_type`
- `artifact_id`
- `source_spec`
- `generated_by`
- `status`
- `version`
- `requires_human_review`
- `linked_artifacts`

The metadata exists to preserve traceability, validation, and regeneration. It MUST NOT replace the
human-legible body of the artifact.

## Markdown/YAML as Operating Contracts

SWE Seed Markdown/YAML files are not informal documentation when they govern agent behavior. They
function as:

- human-legible operating contracts,
- agent context,
- validation surfaces,
- memory artifacts,
- regeneration inputs,
- traceability anchors.

Generated Markdown/YAML MUST remain readable and reviewable without requiring a BAML runtime.

## BAML Function Inventory for SWE Seed

SWE Seed owns only the outer BAML functions. Harness and fabricator BAML functions are imported,
packaged, or invoked as lower-layer contracts; SWE Seed MUST NOT duplicate their definitions.

| Function                  | Input artifacts                       | Output artifact       | Purpose                                                             | Validation requirement                                              |
| ------------------------- | ------------------------------------- | --------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `GenerateProjectSeed`     | user need, repo constraints           | `ProjectSeed`         | Define the bounded scaffold input.                                  | Seed has outcome, constraints, non-goals, and proof.                |
| `SelectLayerCapabilities` | `ProjectSeed`, available specs        | capability map        | Select which harness and fabricator capabilities the project needs. | Map names owning layer and source spec for each capability.         |
| `AssembleSeedPackage`     | `ProjectSeed`, layer capability map   | seed package manifest | Package approved lower-layer artifacts without redefining them.     | Manifest references exact artifact paths, versions, and owners.     |
| `ValidateLayerBoundaries` | specs, manifests, generated artifacts | boundary report       | Detect duplicated ownership or upward dependencies.                 | Report flags any layer that depends on a higher layer.              |
| `RegenerateSeedArtifacts` | root specs, approved artifacts        | proposed diffs        | Rebuild generated project artifacts safely.                         | Approved artifacts are preserved or changed through proposed diffs. |

The harness function inventory is defined in `HARNESS_SPEC.md`. The fabricator function inventory is
defined in `FABRICATOR_SPEC_v0.1.0.md`.

## Regeneration Contract

The whole system MUST be regenerable by another agent using only the three root specs as the
normative input:

- `SWE_SEED_SPEC_v0.2.0.md`,
- `HARNESS_SPEC.md`,
- `FABRICATOR_SPEC_v0.1.0.md`.

These filenames are canonical for bootstrap regeneration. Compatibility aliases MUST NOT be introduced as required inputs.

These three files are the bootstrap contract. A clean repository generator MUST be able to create
the scaffold, BAML source contracts, renderers, templates, route cards, skills, eval artifacts,
context policies, hook policies, proof gates, trace templates, reflection templates, learning
records, fabrication packet templates, docs, and validation commands from the root specs alone.

Derived artifacts MAY preserve explicit human decisions during incremental regeneration, but they
MUST NOT be required to generate the system from scratch. If a generated or approved artifact
conflicts with a root spec, the root spec wins unless a recorded migration note or ADR explicitly
updates the root-spec contract.

SWE Seed itself MUST be regenerable from:

- `SWE_SEED_SPEC_v0.2.0.md`,
- `HARNESS_SPEC.md`,
- `FABRICATOR_SPEC_v0.1.0.md`.

SWE Seed MAY include `HARNESS_SPEC.md`, `FABRICATOR_SPEC_v0.1.0.md`, approved route cards, approved
Skill IR files, eval specs, hook policies, learning records, regression cases, and fabrication
artifacts as incremental regeneration inputs. Those inputs remain owned by their layer specs and
MUST NOT become hidden prerequisites for bootstrap generation.

Regeneration MUST preserve:

- explicit human decisions,
- version history,
- approved constraints,
- active skills,
- deprecated skills,
- regression cases,
- existing traceability,
- proof records.

Generated output MUST NOT silently overwrite human-approved artifacts. If regeneration changes an
approved artifact, it SHOULD create a proposed diff or migration note.

## Context Control Requirements

SWE Seed MUST support context control for agents. A context contract MUST define:

- required context,
- optional context,
- excluded context,
- stale context,
- forbidden context,
- context size/budget,
- freshness rules,
- relevance rules.

Context packs SHOULD be intentionally generated from route, task, and budget inputs rather than
manually accumulated. A context pack MUST make excluded or stale context visible when it affects the
task.

## Proof and Anti-Delusion Requirements

SWE Seed MUST require proof-bearing work. The system SHOULD reject, block, or flag:

- unsupported completion claims,
- implementation without linked route or spec,
- invented requirements,
- scope expansion not approved in spec,
- unverifiable outputs,
- disabled or bypassed tests,
- stale context treated as current,
- confidence without proof,
- missing trace records,
- missing reflection after substantial work.

Completion claims MUST map to proof records or explicit skipped-check justifications.

Working proof is incomplete when it cannot trace to the semantic specification chain artifacts or an
explicit approved waiver.

## Learning and Skill Evolution

SWE Seed SHOULD support supervised learning capture. Every substantial agent run SHOULD produce one
of:

- approved lesson,
- rejected lesson with reason,
- skill proposal,
- regression case,
- `HarnessADR`,
- no-change record.

Skill promotion MUST require evidence and human approval. v0.1 MUST NOT support unsupervised skill
promotion.

## Evaluation and Adaptation Layer

SWE Seed packages the local evaluation and adaptation contract for generated projects. The harness
owns agent-process eval policy. The fabricator owns product/prototype eval details. SWE Seed owns
installation, file layout, and regeneration rules that keep those artifacts present and reviewable.

Evaluation is not only a completion gate. Evaluation is the gate that determines what the system is
allowed to learn.

No adaptation without evaluation.

Every substantial generated-project run MUST produce or explicitly waive:

- `EVAL_SPEC.yaml`
- `EVAL_RESULT.json`
- `PROOF_RECORD.md`
- `ADAPTATION_DECISION.yaml`
- `REFLECTION.md`
- `SKILL_PROPOSAL.yaml` or `NO_SKILL_PROPOSED.md`
- `REGRESSION_CASE.yaml` when a reusable failure is found

SWE Seed MUST keep `EvalSpec` and `EvalResult` file-first and tool-agnostic. External eval tools MAY
be packaged as adapters only. They MUST NOT become the source of truth or mutate active specs.

### Learning Rate and Change-Control Policy

SWE Seed uses the harness learning-rate table for generated projects:

| Band            | Artifacts                                                                |  Change speed | Approval                      |
| --------------- | ------------------------------------------------------------------------ | ------------: | ----------------------------- |
| Runtime         | generated outputs, run traces, proof records, eval results               |     immediate | none                          |
| Reflection      | reflections, lessons, no-change records                                  |          fast | author review                 |
| Candidate       | skill proposals, learning candidates, regression candidates, eval issues | fast/moderate | review required               |
| Active Behavior | active SkillIR, RouteCards, ContextBudgets                               |      moderate | approval + regression         |
| Governance      | EvalSpecs, HookPolicies, PermissionPolicies                              |          slow | approval + HarnessADR         |
| Constitution    | HARNESS_SPEC, SWE_SEED_SPEC, Fabrication spec                            |       slowest | explicit ADR + migration note |

No single run may directly modify active governance or root specs. A run MAY produce proposed diffs,
eval issues, regression cases, or ADRs for future review.

Operational rule:

```text
Let experience change fast. Let governance change slowly. Let repeated evaluated pressure move the outer layers.
```

### External Eval Adapters

Supported future adapters MAY include pytest, Playwright, DSPy, LangSmith, manual checklist, and
LLM-as-judge adapters.

Adapter rules:

- The local `EvalSpec` remains the source of truth.
- The adapter translates local eval definitions into the external tool's format.
- The external tool returns observations.
- Observations are normalized into `EvalResult`.
- No external tool may silently promote skills or modify active specs.

Do not implement DSPy or LangSmith in v0.1 unless already present.

### Semantic Chain Validation

The validator SHOULD check:

- every `REQ-*` links to `JOB-*` or an approved upstream source,
- every `ADR-*` contains at least one `Y-*`,
- every `Y-*` links to a constraint or requirement,
- every `CMP-*` links to one or more `REQ-*`,
- every `SCN-*` links to one or more `REQ-*`,
- every required `SCN-*` maps to `CHK-*` or a manual proof item,
- every `TASK-*` links to `ContextPack` and `EvalSpec`,
- every `PROOF-*` links to `EvalResult`,
- every `SkillProposal` links to proof, eval, and reflection.

If a required link is missing, the artifact is incomplete unless an approved waiver exists.

## Minimal BAML v0.1 Scope

For v0.1, SWE Seed BAML support is limited to generating:

- `ProjectSeed`
- layer capability map,
- seed package manifest,
- boundary validation report,
- proposed regeneration diffs.

The same project MAY include harness BAML generation for `RouteCard`, `SkillIR`, `EvalSpec`,
`ContextPack`, `ReflectionTemplate`, and `SkillProposal`, and fabricator BAML generation for
`ProductSeed`, `ADR`, `PRD`, `SDS`, `TDDPlan`, and `AgentTask`. Those remain owned by their layer
specs.

Do not add:

- vector memory,
- autonomous planning,
- production deployment,
- complex multi-agent orchestration,
- unsupervised skill promotion,
- hidden self-modification.

## HTML5 Game Pilot Compatibility

The first SWE Seed fabrication target MAY be a single-page HTML5 game.

Pilot constraints:

- `index.html` only,
- vanilla HTML/CSS/JS,
- no backend,
- no external assets,
- no network calls,
- playable locally,
- proof checklist required,
- reflection required,
- skill proposal optional but encouraged.

The pilot exists to validate that SWE Seed can convert a bounded product seed into generated specs,
agent context, implementation, proof, and learning.

### HTML5 Game Pilot Semantic Chain Example

```text
JOB-001: When I am playing a 60-second browser game, I want to quickly see which objects help or hurt my progress, so I can understand the core mechanic without reading instructions.

REQ-001: When the player collides with a focus token, the game shall increase the score by one and spawn a new focus token.

Y-001: In the context of the first HTML5 game pilot, facing the constraint of zero setup and local playability, we decided for a single index.html file with inline CSS/JS, and neglected bundlers and external assets, to achieve immediate browser execution, accepting that modularity and asset reuse are limited.

CMP-001 GameLoop: advances game state and renders each frame; links to REQ-001, REQ-002, REQ-003.

SCN-001: Collecting a focus token increases score.

CHK-001: Verify by manual play or automated browser check that collecting a token increments score and spawns a new token.

PROOF-001: Browser test or manual check confirms CHK-001 passed.
```

## Semantic Chain Non-Goals

- Do not create heavyweight enterprise process.
- Do not require every tiny prototype to have a full C4 diagram.
- Do not make syntax compliance more important than working proof.
- Do not let BAML or agents invent missing intent.
- Do not promote semantic-chain outputs without validation.
- Do not treat diagrams as proof.
- Do not treat Gherkin scenarios as passing tests until checked by EvalSpec, automation, or manual proof.

## Advisor Constraint

No new SWE Seed abstraction may be added unless it improves at least one of:

- prototype speed,
- proof quality,
- scope control,
- context relevance,
- reuse,
- user validation,
- safety,
- learning capture,
- regeneration reliability.

The reason MUST be recorded in the spec change, ADR, migration note, or proposal.

## Optional BAML-Aware Layout

When BAML is enabled, the preferred minimal layout is:

```text
.agent-harness/
  baml/
    MIGRATION_NOTE.md
    V0_1_RELEASE_CRITERIA.md
    baml_src/
      harness.baml
      swe_seed.baml
      fabricator.baml
  renderers/
  templates/
  generated/
  schemas/
```

Equivalent paths are allowed when they fit an existing repository convention. The layout MUST keep
typed BAML sources, renderers, generated outputs, templates, and schemas distinguishable.

## CI Requirements

GitHub Actions MUST:

- Check out the repository.
- Install `just`.
- Install the declared toolchain.
- Restore dependency caches where the tool supports safe caching.
- Install dependencies from lockfiles.
- Run `just ci`.

The workflow MAY contain CI-specific setup, but it SHOULD NOT duplicate lint, test, or format logic already expressed through `just`.

## Observability and Logging Requirements

Observability is a core requirement of the dev harness. It exists to make setup failures, CI failures, hook adapter defects, and proof mismatches inspectable without binding the project to one language runtime or one storage backend.

The observability layer MUST remain agnostic to the agent harness. The two systems SHOULD work in harmony through shared identifiers and compatible payload capture, but the dev harness MUST NOT depend on agent-harness-specific Skill IR, memory layouts, or runtime libraries.

### Architectural Rule

The observability pipeline MUST follow this shape:

```text
agent hook or command wrapper
-> router or capture shim
-> normalized event
-> append-only JSONL event log
-> optional artifact files
-> optional rusql index
-> optional vector index
-> dashboards / replay / doctor / trace
```

The append-only JSONL log is the durable source of truth. Any rusql index, FTS layer, Tantivy-style search layer, or vector store MUST be rebuildable from files and MUST NOT become the only durable record.

### Storage Levels

The observability layer SHOULD evolve only when query pressure justifies it:

- Level 0: JSONL event files on disk.
- Level 1: Optional rusql index for structured filters.
- Level 2: Optional rusql FTS or Tantivy-style search for faster textual lookup.
- Level 3: Optional vector index for semantic retrieval over prompts, errors, summaries, or tool outputs.

Implementations MUST NOT start with a database as the only source of truth.

### Event Envelope Contract

Every captured hook execution, adapter invocation, or wrapped harness command MUST emit one normalized event envelope. The normalized envelope MUST be serializable as one JSON object per line and MUST include, at minimum:

- `schema_version`
- `event_id`
- `trace_id`
- `span_id`
- `parent_span_id`
- `timestamp`
- `agent`
- `agent_version`
- `native_event`
- `event`
- `session_id`
- `turn_id`
- `cwd`
- `repo_root`
- `profile`
- `hook_id`
- `script`
- `status`
- `duration_ms`
- `exit_code`
- `stdout_ref`
- `stderr_ref`
- `native_payload_ref`
- `normalized_payload_ref`
- `result_ref`

Large payloads, stdout, stderr, and tool-specific blobs SHOULD be persisted as separate artifact files referenced by the envelope rather than inlined into the JSONL record.

### File Layout

If observability is enabled, the preferred layout is:

```text
.agent-hooks/
  config.yaml
  logs/
    events-YYYY-MM-DD.jsonl
  payloads/
    YYYY-MM-DD/
      <event_id>.native.json
      <event_id>.normalized.json
      <event_id>.result.json
  artifacts/
    YYYY-MM-DD/
      <event_id>.stdout.txt
      <event_id>.stderr.txt
  index/
    hooks.rusql
    vectors/
```

Equivalent paths are allowed, but the implementation MUST document them and MUST preserve the separation between append-only events, large artifacts, and derived indexes.

### Behavioral Requirements

The observability implementation MUST support:

- redaction before persistence,
- log rotation by date and size,
- replay from captured payloads and artifacts,
- inspection of one event without loading the whole log,
- filesystem-first backup and recovery,
- JSONL export,
- OpenTelemetry-compatible JSON export,
- JUnit-style report export,
- compatibility with scripts written in shell, Python, Rust, TypeScript, Node, Deno, Bun, or any tool that can read stdin and write stdout or stderr.

The implementation MUST NOT require project scripts or adapters to import a logging SDK. Captured programs SHOULD communicate through stdin, stdout, stderr, exit codes, and filesystem artifacts.

### Command Surface

A rebuild of the dev harness MUST expose an observability CLI surface. The preferred namespace is `agent-hooks`, though equivalent wrappers MAY be provided through `just`.

Minimum command set:

- `agent-hooks trace --last`
- `agent-hooks trace --session <id>`
- `agent-hooks replay --event <event_id>`
- `agent-hooks inspect --event <event_id>`
- `agent-hooks doctor --observability`
- `agent-hooks compact-logs`
- `agent-hooks index rebuild`
- `agent-hooks export otel`
- `agent-hooks export junit`

`replay` is the most important recovery feature. The observability layer SHOULD make adapter debugging possible from captured payloads without requiring a live rerun.

### Cross-Harness Boundary

The dev harness and the agent harness serve different layers and MUST remain separable.

The dev harness owns:

- command execution surfaces,
- local and remote CI parity,
- setup and tool diagnostics,
- secrets workflows,
- adapter and wrapper observability,
- durable event logging and derived observability indexes.

The dev harness MUST NOT depend on agent-harness route cards, Skill IR, memory files, or learning-review packet formats to function.

The dev harness SHOULD interoperate with the agent harness through shared identifiers and artifact references only. Preferred shared identifiers are `trace_id`, `session_id`, `span_id`, `event_id`, repository root, and stable artifact paths.

### Outcome-Bearing Implementation Order

An implementation that aims to maximize productive outcomes SHOULD build the dev harness in this order:

1. Command parity and proof. `just` remains the local and CI command API, and `doctor`, `bootstrap`, and `ci` provide one clear way to set up, diagnose, and prove work.

1. File-first observability. Command wrappers and hooks emit normalized JSONL events plus artifact references, and `trace`, `inspect`, and `replay` work on captured events before any index or dashboard exists.

1. Optional acceleration layers. rusql or equivalent indexes are rebuildable from JSONL, OTEL and JUnit exports remain derived outputs, and vector retrieval appears only after exact and structured lookup demonstrably fail real recovery tasks.

The implementation SHOULD NOT start with dashboards, vector retrieval, or agent-specific adapters. Those layers are only useful after the command and replay path are already reliable.

### Upgrade Triggers

The implementation SHOULD remain file-only while all of the following are true:

- fewer than 50,000 events exist for the repository,
- most queries are by date, session, or recent failure,
- `grep`, `jq`, or simple trace tooling remain sufficient,
- no always-on dashboard requires indexed lookup.

The implementation SHOULD add a rusql index when structured filters by agent, session, hook, or status become frequent, when `doctor` and `trace` need fast lookup, or when multiple agents produce concurrent events.

The implementation SHOULD add vector search only when semantic retrieval over prompts, errors, summaries, or tool outputs clearly outperforms exact and structured search for real recovery tasks such as finding similar failed runs.

## Dev Harness Documentation

The project MUST include concise operational documentation under `docs/dev-harness/`.

The documentation MUST explain how to use, maintain, and safely change the harness. It SHOULD NOT restate obvious command names unless the command contract matters. It MUST document intent, non-obvious behavior, required workflows, failure recovery, and references needed to make correct changes.

### Documentation Structure

`docs/dev-harness/README.md`
: Entry point. Explain what the harness is, what problem it solves, and the command contract: local `just` commands are the source of truth, and CI calls them where practical.

`docs/dev-harness/explanations/`
: Conceptual documentation for non-obvious design decisions.

Examples:

- `local-ci-parity.md`
- `observability-model.md`
- `toolchain-boundaries.md`
- `secrets-model.md`
- `agent-proof-commands.md`

`docs/dev-harness/howto/`
: Task-focused guides for work a developer or agent may need to perform.

Each how-to filename MUST use a verb phrase.

Examples:

- `initialize-dev-env.md`
- `run-local-ci.md`
- `add-a-ci-check.md`
- `change-node-version.md`
- `rotate-secrets-key.md`
- `debug-failing-ci.md`

`docs/dev-harness/references/`
: Stable reference material for configuration, schemas, command contracts, and external docs.

Examples:

- `just-recipes.md`
- `observability-contract.md`
- `github-actions.md`
- `mise.md`
- `devbox.md`
- `sops-age.md`
- `proof-command-map.md`

### Documentation Rules

How-to documents MUST include:

- Purpose.
- Prerequisites.
- Steps.
- Verification command.
- Common failure modes, when useful.

Explanation documents MUST include:

- Why the design exists.
- What tradeoff it makes.
- What should not be changed casually.

Reference documents MUST include:

- Relevant config files.
- Important fields or schema links.
- Ownership boundaries.
- Commands that prove the config still works.

### Required Initial Documents

The scaffold MUST generate at least:

- `docs/dev-harness/README.md`
- `docs/dev-harness/howto/initialize-dev-env.md`
- `docs/dev-harness/howto/run-local-ci.md`
- `docs/dev-harness/howto/add-a-ci-check.md`
- `docs/dev-harness/howto/debug-failing-ci.md`
- `docs/dev-harness/explanations/local-ci-parity.md`
- `docs/dev-harness/explanations/observability-model.md`
- `docs/dev-harness/explanations/secrets-model.md`
- `docs/dev-harness/references/just-recipes.md`
- `docs/dev-harness/references/observability-contract.md`
- `docs/dev-harness/references/proof-command-map.md`

`just ci` MUST verify that these required documentation files exist.

When a harness command, CI job, tool version, or secrets workflow changes, the matching documentation MUST be updated in the same change.

## Secrets Requirements

The harness MUST support SOPS with age.

Secret handling MUST follow these rules:

- Plaintext secret outputs MUST be ignored by git.
- Local age keys MUST be ignored by git.
- SOPS configuration MUST make intended encrypted paths explicit.
- Secret commands MUST fail clearly when `sops` or required key material is missing.

The spec MUST NOT require real project secrets in the scaffold.

## Proof Command Map

The documentation MUST include a proof-command map that tells humans and agents what command proves each kind of claim.

Minimum map:

| Claim                        | Proof command                    |
| ---------------------------- | -------------------------------- |
| Harness files exist          | `bash tests/validate-harness.sh` |
| Local CI passes              | `just ci`                        |
| Tooling is installed         | `just doctor`                    |
| Formatting is clean          | `just format`                    |
| Static checks pass           | `just lint`                      |
| Tests pass                   | `just test`                      |
| Secret workflow is available | `just secrets-edit <file>`       |

## Validation Requirements

The harness MUST include automated validation for:

- Required files.
- Required `just` recipes.
- CI calling `just ci`.
- Documentation files required by this spec.
- Git ignore rules for local secrets and generated cache files.
- Canonical root spec presence and canonical filenames.
- Required BAML source contracts and BAML release-governance files when BAML is enabled.
- Absence of obsolete compatibility aliases, including `FABRICATION_LAYER_SPEC_v0.1.0.md`.
- Absence of incomplete-work markers in governed artifacts and scripts, using the marker list defined by the harness validator.

When BAML is enabled, validation MUST include a generation smoke check:

```bash
uv run baml-cli generate --from .agent-harness/baml/baml_src
```

Validation MUST also include a local eval-run smoke check that executes a minimal EvalSpec through the harness and asserts a passing `EVAL_RESULT.json`.

When an observability implementation is added, validation MUST also check the event log schema, the replay surface, and the rebuildability of any derived index.

Validation SHOULD treat these observability behaviors as outcome-bearing:

- capture writes a normalized event and referenced payload or artifact files,
- trace surfaces recent and session-scoped events,
- inspect resolves one event into a readable recovery surface,
- replay rehydrates captured payloads without a live rerun,
- doctor reports layout health and index state,
- index rebuild proves derived storage is reproducible from files.

Validation SHOULD check contracts, not incidental formatting.

## Implementation Learning Loop

The harness MUST encode lessons that reduce future implementation friction. These lessons are generalizable and should be applied without adding ceremony:

- Make the command contract executable. If a required file, recipe, or doc matters, validate it.
- Separate facts by use. Put tasks in `howto/`, rationale in `explanations/`, and stable contracts in `references/`.
- Prefer one source of truth. CI should call local commands instead of reimplementing them.
- Make observability replayable. Keep JSONL as the durable event ledger and treat indexes as derived caches.
- Keep optional tools optional in local diagnostics, but make required proof commands fail clearly.
- Use lockfiles when a command depends on package-installed tools.
- Avoid placeholder success. A completion claim needs a fresh proof command and observed output.
- Fix discovered ambiguity in the spec, not only in the scaffold.

These rules are not an invitation to expand the harness. Add structure only when it improves correctness, repeatability, or recovery from failure.

## Implementation Checklist

- Create the required scaffold files.
- Create the required documentation files.
- Add validation for the scaffold and documentation.
- Define the observability contract and required docs before implementing adapters or indexes.
- Implement command parity before adding convenience wrappers, dashboards, or remote-only logic.
- Implement capture, inspect, and replay before adding derived indexes or semantic retrieval.
- Keep the dev harness agent-agnostic by sharing identifiers and artifacts rather than importing the agent harness runtime.
- Run the validation script and observe failure before creating missing required outputs.
- Run `just ci` after implementation.
- Update this spec when implementation reveals a reusable requirement or ambiguity.
