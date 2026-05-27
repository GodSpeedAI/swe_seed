# Fabricator Specification v0.1.0

## Purpose

This specification defines a self-contained product fabrication layer that can be packaged by SWE
Seed or used with an equivalent outer project scaffold.

The fabrication layer converts a bounded product seed into a complete, traceable, agent-executable prototype packet. It exists to help humans and agents move from a job-hypothesis canvas to a working prototype without relying on ad hoc prompting, unbounded agent behavior, or unverifiable completion claims.

The desired outcome is behavioral: fewer vague builds, fewer unsupported prototypes, less scope drift, faster prototype cycles, clearer proof, and more reusable learning after each run.

## Layer Position

The fabricator is the inner layer in the SWE Seed system.

```text
SWE Seed outer layer
-> Harness middle layer
-> Fabricator inner layer
```

The fabricator MUST be self-contained. It MAY be packaged by SWE Seed and executed through harness
routes or dev-harness commands, but it MUST NOT require SWE Seed or the harness to generate a
fabrication packet, validate artifact completeness, create a handoff, record prototype proof, or
capture fabrication learning.

References to SWE Seed and the harness in this document define packaging and handoff boundaries only.
Fabricator behavior MUST NOT branch on the presence of either outer layer.

Dependency direction is one-way inward. Because the fabricator is the inner layer, it has no inward
layer to depend on in this stack. It MUST NOT import, call, require, or branch on harness or SWE Seed
runtime behavior.

The fabricator MUST remain focused on product-to-prototype work. It MUST NOT define agent routing,
general Skill IR registries, dev-harness CI parity, secrets workflows, or repo-wide observability.
Those are outer-layer responsibilities.

Layer integration MUST use explicit artifacts and identifiers: product seed path, run ID, artifact
IDs, context pack path, agent task path, prototype path, proof record path, reflection path, and
optional trace ID.

## Enterprise Completion Policy

The fabricator MUST reject unresolved todos, placeholders, mocks, stubs, disabled checks, synthetic
proof, and unsupported prototype completion claims. These are not accepted as product behavior,
artifact packets, eval results, proof records, or release evidence.

If a prototype or artifact packet is incomplete, the run MUST record a failed gate, blocked handoff,
eval issue, regression case, ADR, or explicit non-release decision. It MUST NOT label incomplete work
as done, enterprise-ready, shippable, validated, or production-ready.

Completion theater is a blocking defect. A prototype is complete only when the generated artifacts,
local execution evidence, EvalResult, ProofRecord, reflection, and adaptation decision support the
claim.

A contributor or agent should know:

- what product job is being tested,
- what artifact must be generated next,
- what constraints govern the prototype,
- what context the implementation agent may use,
- what proof is required,
- what counts as done,
- what lesson or reusable skill should be captured afterward.

## Core Principle

```text
A prototype is not complete because an agent generated code.
A prototype is complete when a bounded job-hypothesis canvas has been converted into a runnable artifact with linked specs, explicit constraints, observed proof, and captured learning.
```

Markdown/YAML artifacts are not informal notes. They are human-legible operating contracts, agent context, validation surfaces, memory artifacts, and regeneration inputs.

BAML MAY be used as a typed generation layer. BAML produces structured outputs; renderers convert those outputs into Markdown/YAML artifacts. The rendered artifacts remain the operating surface used by humans, agents, validators, and future regeneration.

## Required Command Interface

A conforming implementation SHOULD expose fabrication commands through a small project CLI or scripts.
Outer layers MAY provide `just` aliases, but the fabricator does not require `just` for conformance.

Preferred command surface:

- `fabricate new <seed>`: create a new fabrication run from a bounded product seed.
- `fabricate generate <run_id>`: generate required fabrication artifacts for the run.
- `fabricate validate <run_id>`: validate artifact completeness, traceability, constraints, and readiness.
- `fabricate handoff <run_id>`: prepare the agent task and context pack for Claude Code, OpenClaw, or similar agent execution.
- `fabricate proof <run_id>`: run proof checks for the generated prototype.
- `fabricate reflect <run_id>`: create or validate reflection and learning artifacts.
- `fabricate status <run_id>`: report run status, missing artifacts, proof state, and next action.

Recommended outer-layer aliases:

- `just fabricate-new <seed>`
- `just fabricate-generate <run_id>`
- `just fabricate-validate <run_id>`
- `just fabricate-handoff <run_id>`
- `just fabricate-proof <run_id>`
- `just fabricate-reflect <run_id>`
- `just fabricate-status <run_id>`

Equivalent commands are allowed, but the implementation MUST document the selected command names and proof obligations.

The SWE Seed reference packaging MAY implement this command surface as `python scripts/fabricate.py`
plus matching `just fabricate-*` aliases.

At minimum, v0.1 MUST provide one command or script that can:

```text
seed input -> generated artifact packet -> validation result
```

The command MUST fail clearly when required artifacts are missing or invalid.

## Required Scaffold

A conforming fabrication layer SHOULD include this layout:

```text
.fabricator/
  config.yaml
  baml/
    fabricator.baml
    clients.baml
    tests.baml
  schemas/
    product-seed.schema.yaml
    jtbd.schema.yaml
    job-hypothesis.schema.yaml
    hypothesis.schema.yaml
    adr.schema.yaml
    prd.schema.yaml
    sds.schema.yaml
    tdd-plan.schema.yaml
    context-pack.schema.yaml
    agent-task.schema.yaml
    eval-checklist.schema.yaml
    eval-spec.schema.yaml
    eval-result.schema.yaml
    proof-record.schema.yaml
    reflection.schema.yaml
    adaptation-decision.schema.yaml
    skill-proposal.schema.yaml
  templates/
    PRODUCT_SEED.md.j2
    JTBD.md.j2
    JOB_HYPOTHESIS.md.j2
    HYPOTHESIS.md.j2
    ADR.md.j2
    PRD.md.j2
    SDS.md.j2
    TDD.md.j2
    CONTEXT_PACK.md.j2
    AGENT_TASK.md.j2
    EVAL_CHECKLIST.md.j2
    EVAL_SPEC.yaml.j2
    PROOF_RECORD.md.j2
    REFLECTION.md.j2
    ADAPTATION_DECISION.yaml.j2
    NO_SKILL_PROPOSED.md.j2
    SKILL_PROPOSAL.yaml.j2
  runs/
    README.md
    <run_id>/
      input.yaml
      generated/
      prototype/
      proof/
      trace.md
```

Equivalent paths are allowed, but the implementation MUST document them.

Reference implementations MAY render templates directly from a single script instead of a dedicated
`renderers/` directory if the command surface remains documented and regenerable.

Structured schema files under `.fabricator/schemas/` are normative contract artifacts, not
placeholder examples. They MUST encode every field this specification marks as required for the
governed artifact shapes they cover.

Schema lessons learned for v0.1:

- array-valued properties in governed schemas MUST declare item schemas,
- required fields named in this specification MUST appear in the schema or be covered by a
  documented alias,
- aliases such as `current_artifact_chain` and `artifact_chain` or `allowed_files` and
  `files_allowed` MUST be explicit rather than implied by prose,
- a schema is incomplete if it names a collection but leaves element structure unconstrained.

The scaffold MUST stay small. Add generators, schemas, renderers, or runtime integrations only when they improve artifact quality, traceability, proof, or learning capture.

When packaged inside SWE Seed, the fabricator BAML contract MUST also be exposed at
`.agent-harness/baml/baml_src/fabricator.baml` so bootstrap regeneration can resolve all three
root-layer BAML source contracts from one canonical package layout.

The fabricator MUST NOT create `.agent-harness/`, route-card registries, general Skill IR
registries, dev-harness docs, CI workflows, secrets configuration, or observability indexes. It MAY
reference those artifacts when an outer layer provides them.

## Required Artifact Packet

Every fabrication run MUST produce or explicitly waive these artifacts:

- `PRODUCT_SEED.md`
- `JTBD.md`
- `JOB_HYPOTHESIS.md`
- `HYPOTHESIS.md`
- `ADR.md`
- `PRD.md`
- `SDS.md`
- `TDD.md`
- `CONTEXT_PACK.md`
- `AGENT_TASK.md`
- `EVAL_CHECKLIST.md`
- `EVAL_SPEC.yaml`
- `PROOF_RECORD.md`
- `ADAPTATION_DECISION.yaml`
- `REFLECTION.md`
- `SKILL_PROPOSAL.yaml` or `NO_SKILL_PROPOSED.md`

`EVAL_RESULT.json` MUST be produced by the proof command before a prototype may be treated as
complete.

A waiver MUST include:

- artifact waived,
- reason,
- approver or author,
- risk introduced,
- replacement proof, if any.

A generated prototype MUST NOT be treated as complete if proof and reflection are missing.

`JOB_HYPOTHESIS.md` is the canonical hypothesis artifact. `HYPOTHESIS.md` MAY be emitted as a
compatibility mirror during migration, but it MUST preserve the same IDs and claims as
`JOB_HYPOTHESIS.md`.

## Optional Strategy Context

The fabrication layer MAY accept upstream provenance from a standalone Strategy Layer when that
improves traceability.

Optional fields MAY be carried in `ProductSeed`, `ContextPack`, or `AgentTask`, including:

- `strategy_option_id`
- `strategy_decision_record_id`
- `strategy_evidence_ids`
- `strategy_evidence_gap_ids`
- `strategy_question`

These fields are optional provenance only. They MUST NOT make fabrication depend on the existence
of a strategy layer.

## Semantic Specification Chain

The fabrication layer uses a semantic specification chain to preserve intent from product discovery
through implementation proof.

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

The chain exists so each layer uses a constrained syntax that humans can review, tools can parse,
and agents can follow.

The chain MUST preserve:

- original user or system need,
- situation and context,
- desired outcome,
- constraints,
- rejected alternatives,
- system structure,
- behavioral examples,
- eval checks,
- proof evidence,
- learning and adaptation outputs.

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

## Product Seed Requirements

A `ProductSeed` is the minimal bounded input used to start a fabrication run.

It MUST include:

- `run_id`
- `product_type`
- `user`
- `situation`
- `desired_outcome`
- `prototype_goal`
- `constraints`
- `non_goals`
- `success_conditions`
- `proof_expectations`

It SHOULD include:

- `current_workaround`
- `failure_cost`
- `riskiest_assumption`
- `target_runtime`
- `allowed_files`
- `forbidden_files`
- `allowed_tools`
- `forbidden_tools`

Example:

```yaml
run_id: "0001-focus-runner"
product_type: "single_page_html5_game"
user: "casual player with 60 seconds"
situation: "player opens a local browser game with no setup"
desired_outcome: "player experiences how distractions compete with focus"
prototype_goal: "create a playable one-file HTML5 canvas game"
constraints:
  - "index.html only"
  - "vanilla HTML/CSS/JS"
  - "no backend"
  - "no external assets"
  - "no network calls"
non_goals:
  - "no multiplayer"
  - "no account system"
  - "no build system"
success_conditions:
  - "game opens locally"
  - "player can move"
  - "score changes"
  - "win/loss condition works"
  - "restart works"
proof_expectations:
  - "manual browser checklist passes"
  - "no console errors on load"
```

## Job Hypothesis Canvas Requirements

The fabrication layer MUST represent the hypothesis as a job-hypothesis business canvas.

The canonical canvas fields are:

- target job,
- target user segment,
- situation context,
- pains,
- desired gains,
- value proposition,
- alternatives today,
- channels,
- adoption triggers,
- adoption barriers,
- measurable success,
- riskiest assumptions.

This keeps the valuable structure from the current hypothesis model while making assumptions,
adoption risks, and value tradeoffs explicit and machine-readable.

## Job Story Syntax

Canonical Job Story syntax:

```text
When [situation], I want to [motivation or action], so I can [desired outcome].
```

Rules:

- the `When` clause MUST describe a concrete situation, not a persona label,
- the `I want to` clause MUST describe the user's motivation or needed capability,
- the `so I can` clause MUST describe the observable desired outcome,
- vague outcomes are not allowed unless operationalized.

Example:

```text
When I am playing a 60-second browser game, I want to understand which objects help or hurt my progress, so I can make fast decisions without reading instructions.
```

## BAML Role in the Fabrication Layer

BAML MAY be used to define typed generation functions for fabrication artifacts.

BAML exists to make the artifact packet:

- structured,
- typed,
- regenerable,
- testable,
- easier to validate,
- easier to render consistently.

BAML MUST NOT replace human review, proof gates, or artifact validation.

The required relationship is:

```text
ProductSeed
-> BAML typed output
-> Markdown/YAML rendered artifact
-> validation
-> agent context
-> execution
-> proof
-> reflection
-> learning artifact
```

BAML outputs SHOULD be treated as drafts until validation passes and, where required, human approval occurs.

Implementations SHOULD pin the BAML dependency in Python project metadata and lockfiles so typed
artifact generation is reproducible. The recommended minimum is `baml-py>=0.222.0`.

## BAML Function Inventory

A v0.1 implementation SHOULD define these BAML functions or equivalent typed generation functions:

| Function                     | Input                              | Output               | Purpose                                                     | Validation requirement                                         |
| ---------------------------- | ---------------------------------- | -------------------- | ----------------------------------------------------------- | -------------------------------------------------------------- |
| `GenerateJTBD`               | `ProductSeed`                      | `JTBD`               | Clarify the user, situation, struggle, and desired outcome. | Must preserve seed constraints and user.                       |
| `GenerateProductHypothesis`  | `ProductSeed`, `JTBD`              | `ProductHypothesis`  | Convert the seed into a job-hypothesis business canvas.     | Must include measurable success and riskiest assumption.       |
| `GenerateADR`                | `ProductSeed`, `ProductHypothesis` | `ADR`                | Select the prototype architecture and constraints.          | Must state decision, consequences, and non-goals.              |
| `GeneratePRD`                | `JTBD`, `ProductHypothesis`, `ADR` | `PRD`                | Define product requirements and acceptance criteria.        | Must include testable requirements in EARS format with IDs.    |
| `GenerateSDS`                | `PRD`, `ADR`                       | `SDS`                | Define implementation design, components, files, and risks. | Every component must trace to a requirement.                   |
| `GenerateTDDPlan`            | `PRD`, `SDS`                       | `TDDPlan`            | Define tests and manual checks before implementation.       | Every must-have requirement must have a check.                 |
| `GenerateContextPack`        | Artifact packet                    | `ContextPack`        | Produce compact implementation context.                     | Must include constraints, non-goals, success conditions.       |
| `GenerateAgentTask`          | `ContextPack`, `TDDPlan`           | `AgentTask`          | Produce bounded implementation instructions.                | Must include allowed/forbidden actions and definition of done. |
| `GenerateEvalSpec`           | `PRD`, `SDS`, `TDDPlan`            | `EvalSpec`           | Define local checks for the prototype.                      | Must cover product, process, learning, and adaptation classes. |
| `GenerateEvalChecklist`      | `EvalSpec`                         | `EvalChecklist`      | Render manual proof checks for the prototype.               | Must map each required check to evidence.                      |
| `GenerateProofRecord`        | `EvalResult`, observed results     | `ProofRecord`        | Record observed evidence after implementation.              | Must cite checks or manual observations.                       |
| `GenerateReflectionTemplate` | `EvalResult`, `ProofRecord`        | `ReflectionTemplate` | Create a post-run learning structure.                       | Must require eval and proof citations.                         |
| `GenerateAdaptationDecision` | `EvalResult`, `Reflection`         | `AdaptationDecision` | Decide allowed or blocked learning.                         | Must apply the eval-to-adaptation map.                         |
| `ProposeRegressionCase`      | failed eval, proof                 | `RegressionCase`     | Capture reusable fabrication failure.                       | Must link to an eval check and future rule.                    |
| `ProposeLearningCandidate`   | `Reflection`, `EvalResult`         | `LearningCandidate`  | Capture potential reusable lesson.                          | Must include claim, evidence, scope, and confidence.           |
| `ProposeSkillFromEvaluation` | `LearningCandidate`, evals         | `SkillProposal`      | Propose reusable behavior from evaluated runs.              | Must include trigger, guardrails, and proposed eval.           |

The implementation MAY add more functions after v0.1 only when a real fabrication run shows repeated friction.

Semantic-chain support SHOULD additionally provide these typed functions or equivalent generation and
validation steps:

- `GenerateJobStory`
- `GenerateEARSRequirements`
- `GenerateYStatementADR`
- `GenerateSDSComponents`
- `GenerateGherkinScenarios`
- `GenerateEvalSpecFromSemanticChain`
- `GenerateAgentTaskFromSemanticChain`
- `ValidateSemanticChain`
- `ProposeTraceabilityRepair`

Rules:

- BAML may generate drafts.
- Generated artifacts require validation.
- BAML must preserve upstream IDs.
- BAML must not invent unlinked requirements or design choices.
- If source intent is ambiguous, generated output must mark ambiguity instead of filling gaps silently.

## BAML Non-Goals

BAML MUST NOT be used to:

- autonomously approve artifacts,
- autonomously promote skills,
- bypass proof gates,
- deploy production changes,
- make unverified outputs true,
- silently overwrite human-approved artifacts,
- generate broad abstractions that are not used by the fabrication layer,
- replace `just ci`, tests, manual checks, or observed proof.

## Evaluation and Adaptation Layer

Evaluation is not only a completion gate. Evaluation is the gate that determines what the fabricator
is allowed to learn.

No adaptation without evaluation.

Every substantial fabrication run MUST produce or explicitly waive:

- `EVAL_SPEC.yaml`
- `EVAL_RESULT.json`
- `PROOF_RECORD.md`
- `ADAPTATION_DECISION.yaml`
- `REFLECTION.md`
- `SKILL_PROPOSAL.yaml` or `NO_SKILL_PROPOSED.md`
- `REGRESSION_CASE.yaml` when a reusable failure is found

The active `EvalSpec` is frozen after implementation handoff. The implementation agent MUST NOT
modify the active eval, pass conditions, proof gates, or constraints during the same run. If the eval
is wrong or too weak, create an eval issue, regression case, fabrication ADR, or future EvalSpec
revision.

Fabrication evals SHOULD cover:

- product outcome: prototype works,
- process compliance: context, permissions, non-goals, and proof were respected,
- learning quality: reflection and skill proposals cite eval/proof evidence,
- adaptation eligibility: allowed changes are blocked or permitted by the eval-to-adaptation map.

`EvalSpec` checks SHOULD be generated from:

- EARS requirements,
- Y-Statement constraints,
- SDS component contracts,
- Gherkin scenarios,
- AgentTask permissions,
- prior RegressionCases.

Rules:

- product evals should check behavior and runtime outcome,
- process evals should check whether the agent followed route, context, constraints, and non-goals,
- learning evals should check whether reflections and skill proposals are grounded in proof,
- adaptation evals should check what changes are allowed, blocked, or deferred.

No Gherkin scenario may be considered satisfied until represented in an EvalSpec check, automated
test, or explicit manual proof item.

External eval tools MAY be adapters only. Local `EvalSpec` and `EvalResult` artifacts remain the
source of truth.

## Artifact Metadata Requirements

Generated Markdown/YAML artifacts SHOULD include frontmatter where practical.

Minimum recommended frontmatter:

```yaml
---
artifact_type: "PRD"
artifact_id: "prd-0001-focus-runner"
run_id: "0001-focus-runner"
source_spec:
  - "FABRICATOR_SPEC_v0.1.0.md"
generated_by: "baml"
status: "draft"
version: "0.1.0"
requires_human_review: true
linked_artifacts:
  - "JTBD.md"
  - "JOB_HYPOTHESIS.md"
  - "ADR.md"
---
```

The implementation SHOULD preserve these fields through regeneration.

## Artifact Chain

A conforming fabrication run MUST preserve this chain:

```text
ProductSeed
-> JTBD
-> ProductHypothesis
-> ADR
-> PRD
-> SDS
-> TDDPlan
-> ContextPack
-> AgentTask
-> Prototype
-> EvalChecklist
-> ProofRecord
-> Reflection
-> SkillProposal or NoSkillRecord
```

The chain exists to preserve traceability from observed product intent to implementation proof.

If any link is missing, the run status MUST become `blocked`, `waived`, or `incomplete`.

## PRD EARS Requirements

PRD requirements MUST be represented in EARS form so they are machine-readable and deterministic to
validate.

Every PRD requirement MUST include:

- requirement ID,
- EARS pattern type,
- system subject,
- condition or trigger fields required by that EARS pattern,
- system response,
- source artifact link.

Allowed EARS patterns:

- ubiquitous,
- event-driven,
- state-driven,
- optional feature,
- unwanted behavior.

A textual EARS sentence SHOULD be stored alongside structured fields to simplify review and diffing.

Supported EARS patterns:

1. Ubiquitous:

```text
The [system] shall [response].
```

1. Event-driven:

```text
When [trigger], the [system] shall [response].
```

1. State-driven:

```text
While [state], the [system] shall [response].
```

1. Unwanted behavior:

```text
If [unwanted condition], then the [system] shall [mitigation response].
```

1. Optional feature:

```text
Where [feature is included], the [system] shall [response].
```

Additional rules:

- each requirement MUST use one supported EARS pattern or carry an explicit waiver,
- each requirement MUST have a stable ID,
- each requirement MUST link to a Job Story, `ProductSeed`, or approved upstream source,
- requirements MUST avoid multiple unrelated system responses in one statement,
- requirements MUST be testable or explicitly marked non-testable with reason.

Example:

```text
REQ-001: When the player collides with a focus token, the game shall increase the score by one and spawn a new focus token.
```

## Y-Statement ADR Syntax

Canonical Y-Statement syntax:

```text
In the context of [use case or component], facing [constraint or quality attribute], we decided for [chosen option], and neglected [alternatives], to achieve [benefit], accepting that [cost or drawback].
```

Rules:

- each ADR MUST include at least one Y-Statement,
- each Y-Statement MUST name the chosen option and at least one rejected alternative,
- each Y-Statement MUST state the accepted cost or drawback,
- each ADR MUST link to affected `ProductSeed`, Job Story, or PRD requirements,
- architecture choices without a Y-Statement are incomplete or speculative.

Example:

```text
In the context of the first HTML5 game pilot, facing the constraint of zero setup and local playability, we decided for a single index.html file with inline CSS/JS, and neglected bundlers and external assets, to achieve immediate browser execution, accepting that modularity and asset reuse are limited.
```

## SDS Structural Requirements

Each `SDS` SHOULD include:

1. C4 level selection:

- Context: system and external actors.
- Container: major runtime or deployment units.
- Component: internal responsibilities.
- Code: optional and only when useful.

1. Mermaid diagram where useful:

- `flowchart`
- `sequenceDiagram`
- `classDiagram`
- `stateDiagram`
- or C4-compatible Mermaid syntax when supported by repository tooling.

1. Component contracts.

Each component contract MUST define:

- responsibility,
- inputs,
- outputs,
- state owned,
- errors or failure modes,
- linked PRD requirements,
- linked Gherkin scenarios or tests.

Rules:

- trivial prototypes do not require diagrams when a component table is clearer,
- for the v0.1 HTML5 game pilot, a simple component table MAY satisfy structural requirements,
- diagrams must clarify architecture rather than decorate the document.

Example component contract:

```yaml
component: GameLoop
responsibility: "Advance game state and render each frame."
inputs:
  - "keyboard state"
  - "current game state"
outputs:
  - "updated entity positions"
  - "rendered canvas frame"
state_owned:
  - "lastFrameTimestamp"
failure_modes:
  - "animation loop stops"
linked_requirements:
  - "REQ-004"
linked_scenarios:
  - "SCN-002"
```

## Gherkin Behavioral Syntax

Canonical Gherkin syntax:

```gherkin
Scenario: [behavior name]
  Given [initial context or state]
  When [action or event]
  Then [observable result]
```

Rules:

- each must-have PRD requirement MUST have at least one Gherkin scenario or explicit waiver,
- each Gherkin scenario MUST link to one or more PRD requirements,
- each Gherkin scenario MUST be convertible into a manual check, automated test, or EvalSpec check,
- scenarios should avoid implementation details unless those details are required by SDS or ADR.

Example:

```gherkin
Scenario: Collecting a focus token increases score
  Given the game is running and a focus token is visible
  When the player collides with the focus token
  Then the score increases by one
  And a new focus token appears
```

## Traceability Requirements

Traceability MUST be checked before implementation handoff.

Minimum requirements:

- Every PRD requirement MUST link to the ProductSeed, JTBD, or ProductHypothesis.
- Every PRD requirement MUST be EARS-conformant and have a stable ID.
- Every SDS component MUST link to one or more PRD requirements.
- Every TDD case or manual check MUST link to one or more PRD requirements.
- Every AgentTask MUST link to a ContextPack and TDDPlan.
- Every ProofRecord MUST link to an EvalChecklist.
- Every SkillProposal MUST link to a Reflection and ProofRecord.

Traceability SHOULD be represented through stable IDs rather than prose-only references.

Recommended prefixes:

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

Example requirement ID:

```text
REQ-0001-R1
```

Example test ID:

```text
TDD-0001-T1
```

The validator SHOULD check:

- every `REQ-*` links to `JOB-*` or approved source,
- every `ADR-*` contains at least one `Y-*`,
- every `Y-*` links to a constraint or requirement,
- every `CMP-*` links to one or more `REQ-*`,
- every `SCN-*` links to one or more `REQ-*`,
- every required `SCN-*` maps to `CHK-*` or manual proof item,
- every `TASK-*` links to `ContextPack` and `EvalSpec`,
- every `PROOF-*` links to `EvalResult`,
- every SkillProposal links to proof, eval, and reflection.

If a required link is missing, the artifact is incomplete unless an approved waiver exists.

## Context Pack Requirements

The context pack is the main implementation context for the coding agent.

It MUST be intentionally generated. It MUST NOT be an accumulated dump of every related file.

A `ContextPack` MUST include:

- objective,
- current artifact chain,
- required context,
- constraints,
- non-goals,
- allowed files,
- forbidden files,
- allowed actions,
- forbidden actions,
- success conditions,
- proof expectations,
- escalation triggers.

It SHOULD include:

- context size or token budget,
- stale context warnings,
- freshness rules,
- relevant prior lessons,
- excluded context,
- summary of open ambiguities.

If context becomes ambiguous, stale, or contradictory, the implementation agent MUST escalate instead of inventing requirements.

## Agent Task Requirements

An `AgentTask` MUST be bounded enough for Claude Code, OpenClaw, or another coding agent to execute without interpreting broad strategy.

It MUST include:

- role,
- mission,
- input artifacts,
- files allowed to create or edit,
- files forbidden to touch,
- implementation steps,
- proof steps,
- definition of done,
- forbidden behaviors,
- final response requirements.

It MUST NOT instruct the agent to:

- expand scope,
- add unrequested infrastructure,
- install dependencies without approval,
- access secrets,
- call external services without approval,
- disable or bypass tests,
- claim completion without proof.

## Permission and Safety Requirements

A fabrication handoff MUST define action boundaries.

Minimum policy:

```yaml
allowed:
  - read_generated_artifacts
  - create_or_edit_declared_prototype_files
  - run_declared_local_checks
  - update_proof_record
approval_required:
  - install_dependency
  - modify_global_config
  - add_network_call
  - change_security_boundary
  - add_backend_service
forbidden:
  - read_env_files
  - access_secrets
  - delete_unrelated_files
  - push_to_main
  - disable_tests
  - bypass_eval_gate
  - exfiltrate_project_data
```

The implementation SHOULD use existing harness hooks or command wrappers to enforce these policies where practical.

When no harness is present, the fabricator MUST still render the permission policy into the
`AgentTask` and validate it before handoff. Hook enforcement is optional integration, not a
fabricator dependency.

## Proof Requirements

A prototype MUST have an observed proof record.

A `ProofRecord` MUST include:

- run ID,
- prototype artifact path,
- proof command or manual check list,
- observed result,
- pass/fail state,
- failures,
- limitations,
- author or agent,
- timestamp.

For the HTML5 game pilot, proof MAY be manual plus static checks.

Minimum proof checklist:

- prototype file exists,
- prototype opens locally,
- no network calls are required,
- no external assets are required,
- required user interaction works,
- win/loss or completion condition works where applicable,
- restart or reset works where applicable,
- no console errors on initial load,
- non-goals were not implemented.

The proof record MUST NOT be inferred only from agent claims.

## HTML5 Game Pilot Requirements

The first fabrication target SHOULD be a single-page HTML5 game.

Purpose:

```text
Validate that the fabrication layer can convert one bounded product seed into generated specs, bounded agent context, a working prototype, proof, reflection, and reusable learning.
```

Pilot constraints:

- `index.html` only.
- Vanilla HTML/CSS/JS.
- No backend.
- No package manager.
- No external assets.
- No network calls.
- Local browser runtime.
- Manual proof checklist required.
- Reflection required.
- Skill proposal optional but encouraged.

The first game SHOULD be small enough to implement in one agent session.

The pilot MUST NOT optimize for game quality over fabrication proof. The primary outcome is validating the product-fabrication path.

### HTML5 Game Pilot Semantic Chain Example

Job Story:

```text
JOB-001: When I am playing a 60-second browser game, I want to quickly see which objects help or hurt my progress, so I can understand the core mechanic without reading instructions.
```

EARS:

```text
REQ-001: When the player collides with a focus token, the game shall increase the score by one and spawn a new focus token.
```

Y-Statement:

```text
Y-001: In the context of the first HTML5 game pilot, facing the constraint of zero setup and local playability, we decided for a single index.html file with inline CSS and JS, and neglected bundlers and external assets, to achieve immediate browser execution, accepting that modularity and asset reuse are limited.
```

SDS Component:

```text
CMP-001 GameLoop: advances game state and renders each frame; links to REQ-001, REQ-002, REQ-003.
```

Gherkin:

```gherkin
SCN-001: Collecting a focus token increases score
  Given the game is running and a focus token is visible
  When the player collides with the focus token
  Then the score increases by one
  And a new focus token appears
```

Eval check:

```text
CHK-001: Verify by manual play or automated browser check that collecting a token increments score and spawns a new token.
```

Proof:

```text
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

Minimum HTML5 game `EvalSpec` example:

```yaml
id: eval-html5-game-v0-1
version: "0.1.0"
run_id: "<run_id>"
target_type: "single_page_html5_game"
target_path: "index.html"
purpose: "Verify that the game pilot is local, playable, scoped, and proof-backed."
eval_classes:
  - product_outcome
  - process_compliance
  - learning_quality
  - adaptation_eligibility
checks:
  - id: html-file-exists
    class: product_outcome
    type: file_exists
    target: index.html
    required: true
    rule: "index.html exists"
    evidence_required: "file path"
  - id: no-network-patterns
    class: process_compliance
    type: static_forbidden_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "http://"
        - "https://"
        - "fetch("
        - "XMLHttpRequest"
        - "WebSocket"
    evidence_required: "static scan result"
  - id: no-external-script-or-link
    class: process_compliance
    type: static_forbidden_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "<script[^>]+src="
        - "<link[^>]+href="
    evidence_required: "static scan result"
  - id: canvas-present
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "<canvas"
    evidence_required: "static scan result"
  - id: animation-loop
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "requestAnimationFrame"
    evidence_required: "static scan result"
  - id: keyboard-input
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "keydown"
        - "keyup"
    evidence_required: "static scan result"
  - id: score-or-state-change
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "score"
        - "state"
    evidence_required: "static scan result"
  - id: win-loss-or-completion
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "win"
        - "lose"
        - "gameOver"
        - "complete"
    evidence_required: "static scan result"
  - id: restart-reset
    class: product_outcome
    type: static_required_patterns
    target: index.html
    required: true
    rule:
      patterns:
        - "restart"
        - "reset"
    evidence_required: "static scan result"
  - id: manual-play-check
    class: product_outcome
    type: manual_check
    target: local_browser
    required: true
    rule: "manual browser play check completed"
    evidence_required: "checked by author or agent"
  - id: non-goals-preserved
    class: process_compliance
    type: artifact_consistency
    target: artifact_packet
    required: true
    rule: "non-goals were not implemented"
    evidence_required: "diff and artifact review"
pass_condition: "all required checks pass or have explicit human waiver"
outputs:
  - EVAL_RESULT.json
  - PROOF_RECORD.md
  - ADAPTATION_DECISION.yaml
```

## Observability and Run Logging Requirements

The fabrication layer MUST produce a run trace.

The trace SHOULD be file-first and MAY interoperate with the dev harness observability layer through shared identifiers.

Minimum run trace:

```text
.fabricator/runs/<run_id>/trace.md
```

The trace MUST include:

- run ID,
- seed path,
- generated artifacts,
- validation result,
- handoff status,
- prototype artifact path,
- proof record path,
- reflection path,
- skill proposal path or no-skill record,
- unresolved risks,
- next action.

The run trace MUST be append-safe. Single-entry updates MUST append without read-modify-write file
replacement so concurrent or repeated writes do not silently lose prior trace entries.

If JSONL observability exists, fabrication events SHOULD emit normalized events with shared identifiers:

- `trace_id`
- `run_id`
- `event_id`
- `artifact_id`
- `prototype_path`
- `proof_record_path`

The fabrication layer MUST NOT require a database for v0.1 conformance.

## Storage Levels

The fabrication layer SHOULD evolve storage only when real query pressure justifies it:

- Level 0: files only under `.fabricator/runs/`.
- Level 1: optional JSONL run events.
- Level 2: optional SQLite/rusql index for run status, artifact lookup, and proof search.
- Level 3: optional full-text search.
- Level 4: optional vector retrieval over prior artifact packets, reflections, and skill proposals.

The append-only run files and rendered Markdown/YAML artifacts are the durable source of truth.

Indexes MUST be rebuildable from files.

## Regeneration Contract

The fabricator spec is one of the three root specs used to regenerate the whole SWE Seed system:

- `SWE_SEED_SPEC_v0.2.0.md`
- `HARNESS_SPEC.md`
- `FABRICATOR_SPEC_v0.1.0.md`

These root-spec filenames are canonical for bootstrap regeneration and MUST be validated as exact
paths. Compatibility aliases MUST NOT be required, and known obsolete aliases (including
`FABRICATION_LAYER_SPEC_v0.1.0.md`) SHOULD be rejected.

An agent regenerating the system from the root specs MUST be able to derive the fabrication scaffold,
artifact packet, eval/proof requirements, HTML5 game pilot constraints, BAML source contracts,
validation expectations, and learning policy from this document without reading a hidden
implementation source.

The fabrication layer MUST be regenerable from:

- `FABRICATOR_SPEC_v0.1.0.md`.

Approved BAML schemas/functions, templates, prior-run artifacts, learning records, and skill
proposals MAY constrain incremental regeneration, but they MUST NOT be required to bootstrap the
fabrication layer from the root spec. They are derived or approved operating artifacts, not hidden
source material.

Regeneration MUST preserve:

- explicit human decisions,
- version history,
- approved constraints,
- active skills,
- deprecated skills,
- proof records,
- existing traceability,
- manual edits marked as approved.

Regeneration MUST NOT silently overwrite approved artifacts.

If regeneration changes an approved artifact, it MUST create a proposed diff or migration note.

## Fabrication Documentation

The project SHOULD include concise operational documentation under:

```text
docs/fabrication-layer/
```

Recommended structure:

```text
docs/fabrication-layer/
  README.md
  howto/
    create-a-product-seed.md
    generate-a-fabrication-packet.md
    hand-off-to-claude-code.md
    verify-a-prototype.md
    capture-run-learning.md
  explanations/
    artifact-chain.md
    baml-generation-model.md
    proof-before-completion.md
  references/
    artifact-schemas.md
    command-contract.md
    html5-game-pilot.md
```

Documentation MUST explain:

- what the fabrication layer is,
- when to use it,
- how to create a seed,
- how to generate artifacts,
- how to validate a run,
- how to hand off to an agent,
- how to record proof,
- how to capture learning.

Docs SHOULD NOT restate obvious command names unless the command contract matters.

When the fabrication layer is packaged inside SWE Seed, the documentation under
`docs/fabrication-layer/` becomes required operator surface rather than optional explanatory
material.

## Proof Command Map

The documentation MUST include a proof-command map.

Minimum map:

| Claim                       | Proof command or check                                 |
| --------------------------- | ------------------------------------------------------ |
| Fabrication scaffold exists | `fabricate validate <run_id>` or equivalent            |
| Artifact packet complete    | artifact completeness validation                       |
| Traceability passes         | traceability validation                                |
| Context pack is ready       | context-pack validation                                |
| Agent task is ready         | agent-task validation                                  |
| Prototype exists            | file existence check                                   |
| Prototype runs              | declared runtime check or manual proof                 |
| HTML5 game opens locally    | manual browser check                                   |
| Proof recorded              | `PROOF_RECORD.md` exists and is complete               |
| Reflection captured         | `REFLECTION.md` exists and is complete                 |
| Skill decision captured     | `SKILL_PROPOSAL.yaml` or `NO_SKILL_PROPOSED.md` exists |

## Validation Requirements

The fabrication layer MUST include automated validation for:

- required artifact directories,
- required generated artifact files,
- required frontmatter fields where used,
- artifact chain completeness,
- traceability links,
- PRD EARS conformance and requirement ID stability,
- context pack constraints and non-goals,
- agent task allowed and forbidden actions,
- proof record presence,
- reflection presence,
- skill proposal or no-skill record presence.

Validation lessons learned for v0.1:

- malformed structured artifacts such as `EVAL_SPEC.yaml` MUST be reported as validation failures,
  not uncaught runtime exceptions,
- validation of required artifacts MUST support run-subdirectory routing such as generated versus
  proof artifacts instead of assuming one directory for all outputs,
- one-of artifact requirements such as `SKILL_PROPOSAL.yaml` or `NO_SKILL_PROPOSED.md` MUST be
  represented explicitly rather than by accidental omission,
- handoff preparation MUST fail clearly when prerequisite generated artifacts are missing and MUST
  identify which files are absent.

Validation SHOULD treat these as outcome-bearing:

- PRD requirements are testable,
- SDS components map to requirements,
- TDD checks cover must-have requirements,
- context pack excludes forbidden scope,
- agent task has a definition of done,
- proof record includes observed evidence,
- reflection captures at least one lesson, no-change reason, or unresolved risk.

Validation SHOULD check contracts, not incidental formatting.

## Deterministic Serialization

The reference loader MAY accept JSON or a constrained YAML subset to avoid mandatory parser
dependencies.

When an artifact path ends with `.yaml`, the file MAY contain JSON object text if that is the most
deterministic way to represent nested arrays of objects for a dependency-free loader.

Implementations MUST document this choice in the command contract or artifact reference docs and
MUST keep the artifact object-shaped and reviewable by humans.

Emitter lessons learned for v0.1:

- YAML emitters MUST serialize nested sequences and mappings as valid YAML rather than relying on
  indentation that only looks plausible,
- YAML emitters MUST quote ambiguous plain-scalar keywords when string semantics are intended,
- dependency-free loaders and emitters MUST be tested against the shapes they are expected to read
  and write.

## Scope Drift Requirements

The fabrication layer MUST detect or flag scope drift.

Scope drift includes:

- implementing non-goals,
- adding infrastructure not required by ADR/PRD/SDS,
- adding backend when forbidden,
- adding dependencies when forbidden,
- adding network calls when forbidden,
- changing runtime target,
- modifying files outside the allowed set,
- omitting required proof,
- replacing a bounded prototype with a broader platform.

A drift finding MUST produce one of:

- correction request,
- waiver with reason,
- HarnessADR or FabricationADR,
- regression case,
- rejected run.

## Learning Capture Requirements

Every substantial fabrication run MUST produce one of:

- approved lesson,
- rejected lesson with reason,
- skill proposal,
- regression case,
- FabricationADR,
- no-change record.

A skill proposal MUST include:

- trigger,
- inputs,
- output,
- procedure,
- guardrails,
- evals,
- example run ID,
- proof link,
- promotion status.

Skill promotion MUST require human approval in v0.1.

The implementation MUST NOT mutate active skills automatically based only on one run.

## Outcome Metrics

The fabrication layer SHOULD track these metrics:

| Metric                  | Meaning                                                          |
| ----------------------- | ---------------------------------------------------------------- |
| Time to Seed            | Time from idea capture to valid ProductSeed.                     |
| Time to Artifact Packet | Time from ProductSeed to complete generated artifacts.           |
| Time to Handoff         | Time from ProductSeed to valid AgentTask and ContextPack.        |
| Time to Prototype       | Time from ProductSeed to runnable prototype.                     |
| Proof Pass Rate         | Percent of runs with passing proof.                              |
| Scope Drift Count       | Number of detected scope deviations per run.                     |
| Context Relevance Score | Operator or validator rating of context usefulness.              |
| Human Correction Count  | Number of human corrections required before proof.               |
| Skill Proposal Rate     | Percent of runs producing a skill proposal.                      |
| Skill Reuse Rate        | Percent of runs using an existing skill.                         |
| Demo Success Rate       | Percent of prototypes that can be demonstrated.                  |
| User Validation Rate    | Percent of prototypes tested with a real or representative user. |

Metrics SHOULD be stored in the run trace or proof record before any dashboard is built.

## Minimal v0.1 Implementation

v0.1 SHOULD implement only:

- ProductSeed input.
- BAML or equivalent typed generation for core artifacts.
- Markdown/YAML rendering.
- Artifact packet validation.
- Traceability validation.
- EVAL_SPEC.yaml generation.
- EVAL_RESULT.json capture.
- ADAPTATION_DECISION.yaml generation.
- REGRESSION_CASE.yaml when applicable.
- ContextPack generation.
- AgentTask generation.
- HTML5 game pilot.
- Manual proof checklist.
- Reflection capture.
- SkillProposal or NoSkillRecord.

v0.1 SHOULD NOT implement:

- production deployment,
- backend services,
- multi-agent orchestration,
- vector memory,
- autonomous skill promotion,
- autonomous product strategy,
- dashboards,
- complex database-backed memory,
- general-purpose workflow engine,
- hosted eval dependency,
- eval mutation during active runs.

## Advisor Constraint

No new fabrication abstraction may be added unless it improves at least one of:

- prototype speed,
- artifact quality,
- proof quality,
- scope control,
- context relevance,
- reuse,
- user validation,
- safety,
- learning capture,
- regeneration reliability.

If a proposed abstraction does not improve one of those outcomes, it belongs in a parking-lot note, not the active spec.

## Implementation Learning Loop

The fabrication layer MUST encode lessons that reduce future fabrication friction.

General rules:

- Start with one seed and one prototype target.
- Keep prototype constraints explicit.
- Generate context intentionally.
- Write proof expectations before implementation.
- Prefer one runnable artifact over broad architecture.
- Reject completion without proof.
- Capture drift as a learning signal.
- Convert repeated successful behavior into skills.
- Convert repeated failure into regression cases.
- Improve the spec when a real run reveals reusable ambiguity.

These rules are not an invitation to expand the layer. Add structure only when it improves correctness, repeatability, proof, or learning.

## Implementation Checklist

- Create `FABRICATOR_SPEC_v0.1.0.md`.
- Create `.fabricator/config.yaml`.
- Create `.fabricator/runs/`.
- Create initial schema files for ProductSeed, PRD, SDS, TDDPlan, ContextPack, AgentTask, ProofRecord, Reflection, and SkillProposal.
- Create BAML function contract or equivalent typed generator.
- Create Markdown/YAML templates.
- Create a renderer.
- Create a validation command.
- Create the HTML5 game pilot seed.
- Generate the first artifact packet.
- Validate traceability before implementation handoff.
- Hand off to Claude Code/OpenClaw-style agent with only `CONTEXT_PACK.md` and `AGENT_TASK.md` plus linked specs.
- Build `index.html`.
- Record proof.
- Record reflection.
- Propose or reject a reusable skill.
- Run the fabricator validation command and any outer-layer proof command configured by the project.
- Update this spec only when implementation reveals a reusable requirement or ambiguity.
