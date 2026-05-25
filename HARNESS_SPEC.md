# Agentic SWE Harness and Skill IR Specification

Status: Draft v1 (language-agnostic)

Purpose: Define a self-contained development harness for AI coding agents that routes work by desired outcome, invokes portable skills through a canonical Skill Intermediate Representation (Skill IR), verifies software-engineering outcomes, records cognitive artifacts, and supports evidence-based harness improvement.

## Normative Language

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, and
`OPTIONAL` in this document are to be interpreted as described in RFC 2119.

`Implementation-defined` means the behavior is part of the implementation contract, but this
specification does not prescribe one universal policy. Implementations MUST document the selected
behavior.

## 1. Problem Statement

AI coding agents can generate code, tests, documentation, and implementation plans, but successful
software-engineering outcomes require more than model capability. They require a repeatable operating
system for:

- understanding the desired outcome,
- selecting the correct development procedure,
- loading the right project context,
- invoking the right skill at the right time,
- enforcing proof obligations,
- preserving useful cognitive artifacts,
- preventing process drift,
- and improving the harness based on evidence.

Existing instruction systems are fragmented across tools. A reusable procedure may exist as a Claude
Code `SKILL.md`, a Copilot instruction file, a Codex `AGENTS.md` section, a hook-injected prompt, a
checklist, or a project playbook. These formats differ, but the operative object is often the same:
a language-defined behavioral contract that causes an agent to perform a job in a particular way.

This specification defines a harness that treats skills as portable behavioral contracts rather than
agent-specific prompt files. It introduces a Skill Intermediate Representation (Skill IR) that can be
rendered into multiple agent surfaces while preserving the same job-to-be-done, procedure, evidence
requirements, forbidden behaviors, and output contract.

The harness is not merely an orchestration layer. It SHOULD use strategic behavior shaping: exploit known agent tendencies, heuristics, failure modes, and low-cost defaults only when doing so improves outcome production. Examples include routing labels into executable next actions, forcing proof before completion language, constraining context to reduce drift, validating generated artifacts, and using formatting or configuration defaults that remove avoidable choices.

Behavior shaping MUST remain outcome-bound. A nudge, configuration, or instruction has no value if it does not improve the agent's ability to produce the requested result, improve proof quality, reduce recovery time, or prevent a known failure mode.

Implementations SHOULD validate behavior-shaping rules where practical. Required operating instructions and successful-pattern memory SHOULD mention behavior shaping, agent tendency, and outcome production so these ideas remain tied to the harness contract rather than becoming informal taste.

Agent-facing prose is a behavior-shaping surface. Implementations SHOULD constrain completion-adjacent
phrasing that causes premature stopping. This repository validates three low-cost constraints:
avoid em dashes, no praise before verification, and avoid should work. These constraints are not style
preferences; they exist to keep claims tied to observed proof.

The harness solves five operational problems:

- It routes work by outcome instead of relying on ad hoc agent behavior.
- It converts reusable procedures into portable Skill IR instead of copying tool-specific prompts.
- It ties implementation work to verification evidence and traceability.
- It uses hooks and cognitive artifacts to create a closed-loop development process.
- It improves itself through proposal-based learning rather than uncontrolled instruction mutation.
- It exploits agent behavior patterns strategically where the leverage is cheap and the benefit is tied to verified outcomes.

Important boundary:

- The harness is not a coding agent, model provider, or IDE.
- The harness is an agent-facing development operating system.
- Native agents still perform reasoning, editing, command execution, and tool use.
- The harness governs process, context, skill invocation, verification, traceability, and learning.

## 2. Goals and Non-Goals

### 2.1 Goals

- Provide a lightweight entry instruction that routes agents into a project-owned harness.
- Define `AGENTS.md` as the primary task router and operating contract.
- Define a repository directory contract for specs, skills, hooks, memory, traces, evals, and reflections.
- Define a canonical Skill IR for portable behavioral skills.
- Allow external skill systems to be normalized into Skill IR without becoming runtime dependencies.
- Render Skill IR into multiple agent surfaces such as Claude skills, Codex instructions, Copilot instructions, hook prompts, checklists, and eval criteria.
- Route work by job-to-be-done (JTBD) and desired outcome.
- Define verification contracts for each job type.
- Define behavior-shaping rules that exploit agent tendencies only when they support the desired outcome.
- Preserve traceability from requirement to implementation to proof.
- Maintain cognitive artifacts that improve future agent performance.
- Support hook-driven lifecycle integration.
- Support evidence-based harness improvement proposals.
- Provide tests and validation profiles sufficient for autonomous implementation by coding agents.

### 2.2 Non-Goals

- Replacing native coding agents or their tool protocols.
- Mandating one model provider or one IDE.
- Mandating automatic self-modification of instructions.
- Creating a general-purpose workflow engine.
- Creating a full project-management system.
- Requiring vector databases or embeddings for conformance.
- Requiring persistent databases for conformance.
- Requiring a web UI for conformance.
- Treating copied prompt text as equivalent across all agents without validation.

## 3. System Overview

### 3.1 Main Components

1. `Entry Instruction`
   - Minimal agent-specific entry point.
   - Directs the agent to read `AGENTS.md` before acting.
   - SHOULD NOT contain complex project logic.

2. `AGENTS.md Router`
   - Primary project operating manual for agents.
   - Classifies requested work into job types.
   - Selects governing specs, skills, hooks, memory artifacts, and verification requirements.

3. `Specification Layer`
   - Stores project and harness specifications.
   - Defines source-of-truth requirements and acceptance criteria.

4. `Skill IR Registry`
   - Stores normalized, portable behavioral skills.
   - Defines triggers, JTBD, procedures, evidence requirements, forbidden behaviors, and output contracts.

5. `Skill Renderer`
   - Renders Skill IR into target surfaces such as `SKILL.md`, `.instructions.md`, `AGENTS.md` sections, hook prompt fragments, checklists, and eval definitions.

6. `Hook Strategy Layer`
   - Connects agent lifecycle events to routing, context loading, skill selection, safety checks, trace capture, verification, and reflection.

7. `Memory and Cognitive Artifact Layer`
   - Stores durable project cognition such as repo maps, decisions, constraints, failure patterns, glossary, open questions, and successful patterns.

8. `Context Stewardship Layer`
   - Controls context budget, tool-output containment, bulk-analysis strategy, and restart state.
   - SHOULD absorb context-mode-style mechanisms without requiring `mksglu/context-mode` as a dependency.
   - Prefers route-required context first, focused excerpts over raw dumps, code-generated summaries over manual token processing, and durable traces over transcript dependence.

9. `Verification Layer`
   - Defines required evidence for job completion.
   - Runs tests, linting, traceability checks, and acceptance-criteria mapping.

10. `Traceability Layer`

- Maps requirements to code changes, tests, commands, and verification results.

11. `Reflection and Learning Layer`
    - Records session reflections and recurring failure patterns.
    - Proposes harness improvements.
    - Does not auto-apply material harness changes without review unless explicitly configured.

12. `Skill Normalizer`
    - Converts external skills, playbooks, postmortems, and successful traces into Skill IR candidates.

13. `Harness CLI` (RECOMMENDED)
    - Provides commands for validation, rendering, testing, tracing, and improvement proposal review.

### 3.2 Abstraction Levels

The harness is easiest to port when kept in these layers:

1. `Outcome Layer`
   - Job type, JTBD, desired result, acceptance criteria, and definition of done.

2. `Routing Layer`
   - `AGENTS.md` rules that classify tasks and select process.

3. `Skill Layer`
   - Skill IR behavior contracts and rendered target-specific instructions.

4. `Context Layer`
   - Specs, cognitive artifacts, memory files, relevant prior decisions, context budget, and restart state.

5. `Execution Layer`
   - Native agents, tools, hooks, shell commands, and scripts.

6. `Verification Layer`
   - Tests, checks, proof obligations, traceability, and completion gates.

7. `Learning Layer`
   - Reflections, postmortems, skill updates, and harness improvement proposals.

### 3.3 External Dependencies

A conforming implementation MAY depend only on the filesystem and native agent instruction surfaces.

Optional dependencies include:

- native agent hooks,
- a universal hook router,
- SQLite, rusql,  or another local trace store,
- vector search or embeddings,
- test runners and linters,
- CI systems,
- project-management APIs,
- local or remote dashboards.

## 4. Directory Contract

### 4.1 Recommended Repository Layout

A conforming repository SHOULD use this layout:

```text
copilot-instructions.md
AGENTS.md
.github/
  instructions/
    *.instructions.md
docs/
  specs/
    *.md
.agent-harness/
  config.yaml
  skills/
  routes/
  render-targets/
  hooks/
  context/
  memory/
  traces/
  evals/
  reflections/
  playbooks/
  imports/
```

### 4.2 Entry Instruction Files

The harness MAY support multiple agent-specific entry files.

Examples:

- `copilot-instructions.md`
- `.github/copilot-instructions.md`
- `.github/instructions/*.instructions.md`
- `CLAUDE.md`
- `AGENTS.md`
- `.codex/instructions.md`

Entry files SHOULD be lightweight.

Required behavior:

- Entry instructions MUST direct the agent to read `AGENTS.md` before making material changes.
- Entry instructions SHOULD direct the agent to follow the job-type router.
- Entry instructions SHOULD NOT duplicate full skill procedures unless the target agent requires it.

### 4.3 `AGENTS.md` Router

`AGENTS.md` is the authoritative routing document.

It MUST define or reference:

- job types,
- semantic routing rules,
- executable route cards,
- required specs,
- required skills,
- required cognitive artifacts,
- verification contracts,
- completion rules,
- harness improvement rules.

`AGENTS.md` MUST move the agent from task text to action. It MUST NOT merely list job types. It SHOULD remain human-readable and compact by delegating detailed procedure to route cards, Skill IR, or rendered skill files.

### 4.4 Specs Directory

`docs/specs/` stores source-of-truth specifications.

Recommended files:

```text
docs/specs/
  development-harness.md
  skill-ir.md
  verification-system.md
  memory-system.md
  hook-strategy.md
  product-or-feature-specific-spec.md
```

Specs SHOULD define:

- problem statement,
- goals and non-goals,
- domain model,
- contracts,
- algorithms,
- validation matrix,
- implementation checklist.

### 4.5 Harness Directory

`.agent-harness/` stores harness implementation artifacts.

Subdirectories:

- `skills/`
  - Skill IR definitions and rendered skills.

- `routes/`
  - executable route cards selected by the semantic router.

- `render-targets/`
  - generated target-specific instruction artifacts.
  - current core render targets are `claude/`, `copilot/`, `hooks/`, and `checklists/`.

- `hooks/`
  - hook scripts and lifecycle bindings.

- `context/`
  - context budget policy, context-mode normalization notes, and operating rules for tool-output containment, think-in-code analysis, and session continuity.

- `memory/`
  - durable cognitive artifacts.

- `traces/`
  - execution traces and proof records.

- `evals/`
  - harness and skill evaluation cases.
  - core evals SHOULD be deterministic command-based cases with expected output and rationale.

- `reflections/`
  - session reflections and improvement proposals.

- `playbooks/`
  - human-readable procedures not yet normalized into Skill IR.
  - the core playbook set SHOULD be MECE and composable, with no more than six core playbooks unless one is removed or merged.

Current core playbooks:

- `00-orient-and-route.md`
- `10-frame-outcome.md`
- `20-change-with-proof.md`
- `30-debug-from-symptom.md`
- `40-review-for-risk.md`
- `50-capture-learning.md`

- `imports/`
  - imported external skill sources and normalization notes.

## 5. Core Domain Model

### 5.1 Entities

#### 5.1.1 Job Type

A category of work defined by desired outcome. A job type is not operational by itself. It MUST resolve to an executable route card before the agent begins material work.

Fields:

- `id` (string)
- `name` (string)
- `jtbd` (string)
- `triggers` (list of strings)
- `required_inputs` (list)
- `required_skills` (list of skill IDs)
- `optional_skills` (list of skill IDs)
- `required_artifacts` (list)
- `verification` (list of checks)
- `done_when` (list of conditions)
- `failure_modes` (list)

#### 5.1.2 Route Card

Executable contract selected by the semantic router for a job type.

Fields:

- `id` (string)
- `job_type` (string)
- `purpose` (string)
- `semantic_triggers` (list of strings)
- `positive_examples` (list of strings)
- `negative_examples` (list of strings)
- `required_context` (list of paths)
- `required_skills` (list of skill IDs)
- `work_loop` (ordered list of actions)
- `required_artifacts` (list)
- `proof` (list of commands or checks)
- `done_when` (list of conditions)
- `failure_modes` (list)
- `fallback_policy` (string)

#### 5.1.3 Skill IR

Canonical representation of a portable behavioral skill.

Fields:

- `id` (string)
- `version` (integer or semantic version string)
- `category` (string)
- `jtbd` (string)
- `description` (string)
- `triggers` (list)
- `inputs` (object)
- `procedure` (ordered list)
- `evidence_required` (list)
- `forbidden_behaviors` (list)
- `outputs` (list)
- `success_criteria` (list)
- `failure_modes` (list)
- `render_targets` (list)
- `source` (object, OPTIONAL)
- `status` (enum)

#### 5.1.4 Render Target

A concrete format emitted from Skill IR.

Fields:

- `target` (string)
  - Example: `claude_skill`, `copilot_instruction`, `codex_agents_section`, `hook_prompt`, `checklist`, `eval`.

- `path` (path string)
- `format` (string)
- `agent` (string or null)
- `generated_from` (skill ID)
- `generated_at` (timestamp)

#### 5.1.5 Cognitive Artifact

Durable memory file intended to improve future agent behavior.

Fields:

- `id`
- `path`
- `purpose`
- `owner`
- `update_policy`
- `read_policy`
- `max_size_policy`

#### 5.1.6 Verification Contract

Proof obligations for job completion.

Fields:

- `job_type`
- `required_checks`
- `required_evidence`
- `acceptable_exceptions`
- `failure_response`

#### 5.1.7 Traceability Record

Mapping from intended outcome to implementation proof.

Fields:

- `task_id` (string or null)
- `job_type`
- `requirements`
- `files_changed`
- `tests_or_checks_run`
- `results`
- `unresolved_risks`
- `completion_claim`

#### 5.1.8 Reflection Record

Post-run learning artifact.

Fields:

- `session_id`
- `job_type`
- `what_worked`
- `what_failed`
- `friction_points`
- `missed_context`
- `candidate_improvements`
- `evidence_links`

#### 5.1.9 Harness Improvement Proposal

A proposed change to improve harness behavior.

Fields:

- `id`
- `observed_problem`
- `evidence`
- `proposed_change`
- `affected_files`
- `expected_benefit`
- `risk`
- `rollback_plan`
- `validation_plan`
- `status`

### 5.2 Stable Identifiers and Naming Rules

- Skill IDs MUST be lowercase kebab-case.
- Job type IDs MUST be lowercase snake_case or kebab-case, consistently selected by implementation.
- Artifact paths SHOULD be stable across sessions.
- Rendered files MUST record their source Skill IR ID.
- Improvement proposal IDs SHOULD be stable and unique.

## 6. Job-Type Router Specification

### 6.1 Semantic Router Purpose

The router MUST be a semantic router: it maps the user's intended outcome to an executable route plan. It MUST use job type labels only as an intermediate classification step.

The router MUST consider outcome intent, JTBD, semantic triggers, positive examples, negative examples, explicit file/spec references, and safety implications. Implementations MAY use embeddings or model-based classification, but they MUST provide a deterministic fallback suitable for CI validation.

The router output MUST tell the agent what to do next. A conforming route result includes:

- selected job type,
- selected route card path,
- confidence or documented assumption,
- required context to read,
- required skills to load,
- ordered work loop,
- required artifacts,
- proof commands,
- done conditions,
- first next action.

### 6.2 Required Job Types

A conforming harness SHOULD define at least these job types:

- `research`
- `spec`
- `implementation`
- `bugfix`
- `refactor`
- `test`
- `review`
- `release`
- `documentation`
- `harness_improvement`
- `skill_authoring`

### 6.3 Route Cards

Each required job type MUST have a route card under `.agent-harness/routes/`.

Route cards are executable contracts. They MUST translate a classified job type into movement toward the outcome: context loading, skill selection, work loop, artifacts, proof, and completion gates.

Route cards MUST include:

- `id`
- `job_type`
- `purpose`
- `semantic_triggers`
- `positive_examples`
- `negative_examples`
- `required_context`
- `required_skills`
- `work_loop`
- `required_artifacts`
- `proof`
- `done_when`
- `failure_modes`
- `fallback_policy`

If a route card is missing or incomplete, the harness MUST treat that as a harness defect.

Route cards SHOULD be action-dense enough to move work without extra interpretation. At minimum, a practical route card SHOULD include:

- at least four semantic triggers;
- at least two positive examples;
- at least two negative examples;
- at least three required context entries, including one relevant core playbook;
- at least five work-loop steps;
- at least three required artifacts;
- at least three done conditions;
- at least two failure modes;
- explicit proof execution and proof-output reading;
- failure modes that tell the agent when to stop, switch routes, or ask.

### 6.4 Routing Behavior

The router MUST determine:

1. What kind of job this is.
2. What outcome is sought.
3. Which route card applies.
4. What source-of-truth spec applies.
5. What skills are required.
6. What cognitive artifacts should be read.
7. What verification evidence is required.
8. What output artifacts are expected.
9. What first action should happen next.
10. Whether missing context blocks execution.

For fresh-session build prompts that express broad creation intent but do not yet name concrete
acceptance criteria, implementation target, or changed artifact, the router SHOULD bootstrap into
the `spec` route rather than defaulting to an unrelated maintenance route. The first step should
clarify the contract before code.

### 6.5 Routing Failure Behavior

If the router cannot classify a task:

- It SHOULD ask for clarification only if classification materially changes the work.
- If clarification is unavailable, it SHOULD choose the safest likely job type and record the assumption.
- It MUST NOT bypass verification obligations merely because the task was ambiguous.
- It MUST still produce a route plan or an explicit blocked result.

### 6.6 Example Route Card

```yaml
id: bugfix
job_type: bugfix
purpose: Diagnose and fix a defect without guessing.
semantic_triggers:
  - failing test
  - regression
  - unexpected behavior
positive_examples:
  - "fix the failing login test"
  - "debug why CI errors on checkout"
negative_examples:
  - "implement a new feature"
required_context:
  - AGENTS.md
  - .agent-harness/memory/repo-map.md
  - .agent-harness/memory/failure-patterns.md
required_skills:
  - debug-discipline
work_loop:
  - reproduce the original failure
  - identify the failing path
  - implement the smallest fix
  - add regression protection when appropriate
  - run proof commands
required_artifacts:
  - observed failure
  - root cause note
  - changed files summary
  - validation evidence
proof:
  - just ci
done_when:
  - original failure no longer reproduces
  - root cause is connected to the fix
  - proof commands pass or skipped checks are justified
failure_modes:
  - failure cannot be reproduced
  - route lacks required context
fallback_policy: Ask for clarification if the missing symptom changes the diagnosis path.
```

## 7. Skill IR Specification

### 7.1 Purpose

Skill IR is the canonical representation of reusable agent behavior.

A skill is not primarily a file. A skill is a behavioral contract that can be rendered into multiple agent-specific surfaces.

Skill IR is the canonical source for skill behavior. Render targets are generated projections. If two paths need identical bytes, one path MUST be a symlink to the canonical file. If two files differ because they target different consumers, they MUST carry canonical-source metadata and validation MUST prove they are fresh.

### 7.2 Required Fields

Every Skill IR file MUST include:

- `id`
- `version`
- `category`
- `jtbd`
- `triggers`
- `procedure`
- `evidence_required`
- `forbidden_behaviors`
- `outputs`
- `success_criteria`

### 7.3 Recommended Fields

Skill IR SHOULD include:

- `failure_modes`
- `examples`
- `non_examples`
- `rendering_notes`
- `source`
- `status`

### 7.4 Skill Status Values

Allowed status values:

- `draft`
- `active`
- `deprecated`
- `experimental`
- `imported`
- `candidate`

### 7.5 Behavioral Invariants

Rendered skills MUST preserve behavioral invariants.

Behavioral invariants include:

- the JTBD,
- trigger conditions,
- required evidence,
- forbidden behaviors,
- completion criteria,
- output expectations.

Rendered text MAY differ per target agent if the invariant behavior is preserved.

### 7.6 Example Skill IR

```yaml
id: debug-discipline
version: 1
category: debug
jtbd: diagnose and fix a defect without guessing
triggers:
  - failing test
  - regression
  - unexpected behavior
inputs:
  required:
    - symptom_or_failure
procedure:
  - reproduce the failure or document why reproduction is unavailable
  - identify the actual failing path
  - form falsifiable hypotheses
  - test one hypothesis at a time
  - implement the smallest fix that explains the failure
  - validate against the original repro
  - add regression protection when appropriate
evidence_required:
  - repro command or scenario
  - observed failure
  - root cause explanation
  - validation result
forbidden_behaviors:
  - speculative fix without repro or explicit uncertainty
  - broad rewrite before root cause
  - declaring done without rerunning proof
outputs:
  - root cause note
  - changed files summary
  - validation evidence
success_criteria:
  - original failure no longer reproduces
  - root cause is connected to fix
  - relevant tests pass
status: active
render_targets:
  - claude_skill
  - copilot_instruction
  - codex_agents_section
  - hook_prompt
  - checklist
  - eval
```

## 8. Skill Normalization and Import Specification

### 8.1 External Skill Sources

The harness MAY import or reference external skill sources, including:

- Claude-style skills,
- Copilot instruction files,
- Codex instructions,
- team playbooks,
- postmortems,
- successful traces,
- external public skill repositories.

### 8.2 Import Boundary

External skills MUST NOT become authoritative merely by being imported.

Import flow:

1. Store raw source under `.agent-harness/imports/` or reference it in metadata.
2. Extract behavioral invariants.
3. Convert to Skill IR candidate.
4. Review for overlap, safety, and fit.
5. Promote to active skill if accepted.

### 8.3 Skill Normalization Requirements

A normalizer MUST extract or require human/agent completion of:

- JTBD,
- triggers,
- ordered procedure,
- evidence requirements,
- forbidden behaviors,
- output contract,
- success criteria,
- known failure modes.

If these cannot be extracted, the imported skill MUST remain `candidate` or `experimental`.

The current implementation has normalized selected process invariants from `thananon/9arm-skills` without making that repository a runtime dependency:

- debugging requires reliable reproduction, fail-path tracing, hypothesis disproof, and a breadcrumb ledger;
- engineering post-mortems require reliable reproduction, known root cause, identified fix, and honest validation coverage;
- review requires questioning intent, checking for a simpler alternative, tracing the actual path, and separating claim from verification.

These imported invariants MUST live in local Skill IR, route cards, playbooks, or validation before they influence agent behavior.

### 8.4 Isomorphic Skill Principle

The harness treats skills as behaviorally isomorphic when two renderings preserve the same operative behavior despite different file formats or wording.

Conformance rule:

- Behavioral equivalence is determined by invariant behavior and outcome verification, not byte-identical prompt text.

## 9. Skill Rendering Specification

### 9.1 Render Targets

Supported render targets MAY include:

- `claude_skill`
- `copilot_instruction`
- `codex_agents_section`
- `github_instruction`
- `hook_prompt`
- `checklist`
- `eval`
- `human_playbook`

### 9.2 Rendering Requirements

Renderers MUST:

- preserve behavioral invariants,
- include required evidence expectations,
- include forbidden behaviors where relevant,
- include the source Skill IR ID and version,
- avoid duplicating conflicting instructions.
- generate target-specific surfaces from Skill IR rather than requiring manual edits to generated files.
- fail validation when a generated artifact is stale.

Renderers SHOULD:

- adapt tone and structure to the target agent,
- keep entry-point instructions short,
- avoid overloading target files with all skills at once.

Current core render targets:

- `claude_skill`: 9arm/Claude-style `SKILL.md` with YAML frontmatter containing `name` and `description`, stored under a bucket directory and skill directory.
- `copilot_instruction`: `.instructions.md`-style instruction surface.
- `hook_prompt`: hook-consumable prompt fragment.
- `checklist`: human/agent checklist surface.

Generated render targets SHOULD include these sections when applicable:

- `Use this when`
- `What to do`
- `Evidence required`
- `Forbidden behavior`
- `Done when`

### 9.3 Rendered Artifact Header

Generated artifacts SHOULD include a header:

```text
Generated from Skill IR: <skill-id>@<version>
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.
```

### 9.4 Rendering Failure Behavior

If a skill cannot be rendered without losing required behavior:

- the renderer MUST fail with a typed error,
- the skill MUST NOT be silently omitted,
- the failure MUST be operator-visible.

## 10. Hook Strategy Specification

### 10.1 Hook Purpose

Hooks connect lifecycle events to harness behavior.

Hooks SHOULD enforce process, gather context, record evidence, and prevent unsafe actions. Hooks SHOULD NOT replace agent reasoning.

### 10.2 Canonical Hook Events

Recommended events:

- `session.start`
- `prompt.submit`
- `tool.pre`
- `tool.post`
- `turn.stop`
- `session.end`
- `compact.pre`
- `subagent.start`
- `subagent.stop`

### 10.3 Hook Responsibilities

Recommended behavior:

- `session.start`
  - Load `AGENTS.md`.
  - Load repo map and active specs.
  - Load relevant memory artifacts.

- `prompt.submit`
  - Classify job type.
  - Select required skills.
  - Detect missing acceptance criteria.

- `tool.pre`
  - Enforce workspace boundary.
  - Block dangerous commands.
  - Require justification for destructive actions.

- `tool.post`
  - Capture changed files.
  - Update traceability draft.
  - Record command/check result.

- `turn.stop`
  - Compare claimed completion against proof obligations.
  - Run lightweight verification checks where configured.
  - Prompt for unresolved evidence gaps.

- `session.end`
  - Write session summary.
  - Update reflection record.
  - Propose harness improvements when evidence warrants.

### 10.4 Hook Safety

Hooks MUST:

- enforce timeouts,
- avoid infinite loops,
- avoid writing secrets to logs,
- preserve traceability,
- fail safely when unavailable.

## 11. Memory and Cognitive Artifact Specification

### 11.1 Purpose

Cognitive artifacts store durable project knowledge that improves agent performance without relying solely on hidden context or chat history.

### 11.2 Required or Recommended Artifacts

Recommended files:

```text
.agent-harness/memory/repo-map.md
.agent-harness/memory/decisions.md
.agent-harness/memory/open-questions.md
.agent-harness/memory/failure-patterns.md
.agent-harness/memory/successful-patterns.md
.agent-harness/memory/glossary.md
.agent-harness/memory/constraints.md
```

### 11.3 Artifact Update Rules

- Memory updates SHOULD be concise.
- Memory updates SHOULD cite evidence such as trace IDs, test results, specs, or changed files.
- Memory artifacts MUST NOT become unbounded dumps.
- Durable memory updates SHOULD be reviewed or gated for material changes.
- Required memory files SHOULD include `Use this when` and `Keep in mind` sections.
- Memory artifacts SHOULD change future behavior. They SHOULD NOT store transcripts, one-off preferences, or generic commentary.

### 11.4 Artifact Read Rules

The router SHOULD specify which artifacts are required for each job type.

Example:

```yaml
job_types:
  implementation:
    read_artifacts:
      - repo-map.md
      - decisions.md
      - constraints.md
  bugfix:
    read_artifacts:
      - repo-map.md
      - failure-patterns.md
      - constraints.md
```

## 12. Verification and Traceability Specification

### 12.1 Verification Contract

Each job type MUST define done conditions and required evidence.

The harness MUST distinguish:

- work performed,
- proof gathered,
- completion claimed.

### 12.2 Required Verification Types

Implementations SHOULD support:

- test commands,
- lint/typecheck commands,
- static checks,
- acceptance-criteria mapping,
- changed-file review,
- spec traceability,
- manual evidence notes.

### 12.3 Traceability Map

For implementation-like tasks, the agent SHOULD produce or update a traceability record:

```yaml
traceability:
  requirement: "..."
  files_changed:
    - path: "..."
      reason: "..."
  verification:
    - command: "..."
      result: "passed|failed|skipped"
      evidence: "..."
  unresolved:
    - "..."
```

### 12.4 Completion Claim Rules

An agent MUST NOT claim completion unless:

- required checks have passed,
- skipped checks are explicitly justified,
- acceptance criteria are mapped,
- unresolved risks are disclosed.

## 13. Reflection and Learning Specification

### 13.1 Learning Boundary

The harness MAY learn from outcomes, but MUST distinguish proposals from applied changes.

Default rule:

- The harness SHOULD propose improvements.
- The harness SHOULD NOT auto-apply material changes to routing, skills, specs, or hooks without explicit approval or documented policy.

### 13.2 Reflection Record

After meaningful sessions, the harness SHOULD create a reflection record.

Minimum fields:

- session summary,
- job type,
- what worked,
- what failed,
- missing context,
- verification gaps,
- suggested harness improvements.

Reflection records SHOULD be used only when the session produced reusable learning, exposed process friction, changed a harness contract, or revealed a repeated failure pattern. They SHOULD NOT be used for routine successful edits or command transcripts.

Reflection templates SHOULD require evidence such as command output, route result, changed file, validation failure, user correction, or repeated friction.

### 13.3 Improvement Proposal Flow

Improvement flow:

1. Observe friction or failure.
2. Record evidence.
3. Propose a harness change.
4. Review proposal.
5. Apply accepted change.
6. Validate improvement.
7. Roll back if harmful.

### 13.4 Improvement Proposal Format

```yaml
id: hip-0001
observed_problem: "Agent repeatedly declared completion without mapping acceptance criteria."
evidence:
  - trace_id: "trace-123"
  - failed_check: "missing_acceptance_mapping"
proposed_change:
  target: "skill"
  file: ".agent-harness/skills/core/traceability-map.yaml"
  summary: "Add explicit completion gate requiring requirement-to-test mapping."
expected_benefit: "Fewer false completion claims."
risk: "May increase completion latency."
rollback_plan: "Revert skill version from 2 to 1."
validation_plan: "Run evals/completion-claim cases."
status: "proposed"
```

Improvement proposals SHOULD distinguish weak evidence from actionable evidence. If evidence is weak, implementations SHOULD record an open question, keep a local reflection, wait for another occurrence, or ask the user for policy instead of mutating routing, skills, hooks, specs, or validation.

Proposal states SHOULD include `proposed`, `accepted`, `implemented`, `rejected`, and `superseded`.

### 13.5 Anti-Drift Rule

Harness self-improvement MUST be evidence-based.

The system MUST NOT treat a single subjective preference as sufficient reason to rewrite stable operating instructions unless the operator explicitly requests it.

## 14. Harness Configuration Specification

### 14.1 Config File

Primary config file:

```text
.agent-harness/config.yaml
```

### 14.2 Config Schema

Top-level keys:

- `version`
- `entrypoints`
- `job_types`
- `routes`
- `skills`
- `render_targets`
- `hooks`
- `memory`
- `verification`
- `learning`
- `observability`

Unknown keys SHOULD be ignored for forward compatibility unless strict mode is enabled.

### 14.3 Example Config

```yaml
version: 1

entrypoints:
  primary_router: AGENTS.md

routes:
  registry: .agent-harness/routes
  semantic_fallback: deterministic_token_overlap

skills:
  registry: .agent-harness/skills
  render_targets:
    - copilot_instruction
    - codex_agents_section
    - hook_prompt

learning:
  auto_apply: false
  proposal_path: .agent-harness/reflections/harness-improvement-proposals.md

verification:
  require_traceability_for:
    - implementation
    - bugfix
    - refactor
```

### 14.4 Dynamic Reload Semantics

Implementations MAY support dynamic reload.

If implemented:

- invalid reloads MUST preserve last known good config,
- active executions SHOULD NOT be interrupted unless explicitly configured,
- reload errors MUST be visible.

## 15. CLI and Tooling Specification

### 15.1 Recommended CLI

A harness implementation SHOULD provide a CLI.

Suggested commands:

```text
harness init
harness validate
harness route <task>
harness route --record <task>
harness route --json <task>
harness inspect <item>
harness context-plan <task>
harness render-skills
harness import-skill <path-or-url>
harness normalize-skill <source>
harness test-skill <skill-id>
harness verify
harness trace start <task>
harness trace append <trace> <note>
harness trace checkpoint <trace> --stage <stage> --summary <summary>
harness trace resume <trace>
harness trace finish <trace> --claim <claim> [--command <command> --result <result>]
harness reflect
harness propose-improvement
harness doctor
```

This repository implements `validate`, `doctor`, `render-skills`, `route`, `route --record`,
`inspect`, `context-plan`, and `trace start|append|checkpoint|resume|finish` in `scripts/harness.py`.
`just` recipes expose the same operations for local use.

### 15.2 `validate`

Validation MUST check:

- config schema,
- `AGENTS.md` existence,
- job type definitions,
- route card existence and schema,
- route cards for every required job type,
- semantic router examples,
- skill IR schema,
- render target validity,
- duplicate IDs,
- missing required skills,
- verification command availability where applicable.

### 15.3 `doctor`

Doctor SHOULD check:

- entry instruction files,
- agent-specific instruction surfaces,
- hook availability,
- required commands,
- shell compatibility,
- generated artifacts freshness,
- trace directory writability.

### 15.4 `render-skills`

Rendering MUST:

- read Skill IR,
- generate configured render targets,
- preserve source metadata,
- fail on behaviorally lossy renderings.

### 15.5 `route --record`

Route recording MUST write a route decision ledger entry before material work when the selected
route needs auditability. A route decision record MUST include task text, selected job type, route
card path, confidence, required context, required skills, work loop, proof commands, done conditions,
and next action.

Route decision records MUST NOT be treated as completion evidence. They prove process selection only.

### 15.6 `inspect`

Inspection SHOULD answer "what governs this harness item?" for a route, skill, eval, memory topic, or
rendered target. A useful inspect result SHOULD include canonical source paths, generated projections,
related routes, proof commands, and relevant eval or memory files.

### 15.7 `trace`

Trace commands SHOULD create and update filesystem trace records:

- `trace start <task>` creates a trace record and linked route decision.
- `trace append <trace> <note>` records meaningful evidence or decision boundaries.
- `trace checkpoint <trace> --stage <stage> --summary <summary>` records a resumable handoff packet.
- `trace resume <trace>` returns the latest checkpoint together with the governing route and open risks.
- `trace finish <trace> --claim <claim>` records the completion claim and optional proof command.

Trace commands MUST NOT store secrets. Generated trace records SHOULD be ignored by version control
unless the repository intentionally stores evidence fixtures.

### 15.8 `context-plan`

`context-plan <task>` SHOULD combine the semantic route result with the local context budget policy.
The output SHOULD tell the agent what to read first, how to contain high-volume tool output, when to
use code for bulk analysis, and where durable session continuity belongs.

The command MUST NOT require `mksglu/context-mode` or any MCP server. It absorbs the useful mechanism:
route-required context first, tool-output containment, think in code, and trace-backed session
continuity.

## 15A. Context Stewardship Specification

### 15A.1 Purpose

Context stewardship prevents the agent from spending the context window on raw data that does not
move the task. It is a behavior-shaping layer tied to outcome production.

### 15A.2 Required Mechanisms

Implementations SHOULD provide:

- a context budget policy,
- tool-output containment guidance,
- a think-in-code rule for bulk analysis,
- session continuity through durable records,
- resumable checkpoints that condense the current stage, next action, and unresolved risks,
- hook or route pressure that reminds the agent before high-volume tools run.

### 15A.3 Imported Context-Mode Invariants

The harness MAY learn from `mksglu/context-mode` without adding it as a dependency. The portable
invariants are:

- keep raw tool output out of the working context where possible,
- retain task continuity in durable indexed or searchable artifacts,
- emit compact restart packets instead of relying on the full transcript,
- generate code or shell pipelines to compute over large inputs,
- route or hook the agent toward context-saving behavior,
- keep context rules outcome-bound rather than stylistic.

### 15A.4 Local Contract

This repository stores the contract in `.agent-harness/context/`, validates it through
`scripts/harness.py validate`, and exposes it through `scripts/harness.py context-plan <task>`,
`scripts/harness.py trace checkpoint ...`, and `scripts/harness.py trace resume <trace>`.

### 15A.5 Checkpoint and Resume

Implementations SHOULD support a lightweight checkpoint operation on active traces.

A checkpoint SHOULD record:

- current stage,
- compact summary of the current state,
- next action,
- relevant artifacts to reopen,
- unresolved risks that could block the next agent.

A resume operation SHOULD return the latest checkpoint together with the governing route and open
risks. The goal is not archival completeness. The goal is a restart packet that lets the next
agent continue surgically.

## 16. Observability Specification

### 16.1 Required Logs

The harness SHOULD log:

- job classification,
- selected skills,
- loaded artifacts,
- verification checks,
- completion claims,
- failed proof obligations,
- improvement proposals.

### 16.2 Trace Records

Trace records SHOULD include:

- session ID,
- job type,
- selected route,
- route decision record,
- zero or more checkpoints with stage, summary, next action, relevant artifacts, and unresolved risks,
- selected skills,
- files changed,
- commands run,
- verification results,
- reflections created,
- improvement proposals created.

The trace store SHOULD separate source templates and generated records. This repository uses
`.agent-harness/traces/traceability-template.yaml` for the source template,
`.agent-harness/traces/route-decisions/` for route decision records, and
`.agent-harness/traces/records/` for task traces.

Checkpoint events SHOULD be small enough that a later agent can resume the task by reading the
trace and the named artifacts instead of reconstructing state from the conversation transcript.

### 16.3 Observability Failure Behavior

Observability failures MUST NOT corrupt source files or prevent safe agent execution.

If trace writing fails, the harness SHOULD warn the operator and continue where safe.

## 17. Security and Operational Safety

### 17.1 Trust Boundary

The harness assumes that instructions, skills, and hooks can materially affect agent behavior.

Implementations MUST document:

- whether skills are trusted,
- whether imported skills require review,
- whether hooks can run arbitrary commands,
- whether memory files can be edited automatically,
- whether generated instructions are reviewed.

### 17.2 Secret Handling

- Secrets MUST NOT be written to memory artifacts or traces.
- Environment variables containing secrets MUST be redacted in logs.
- Imported skills MUST be reviewed for prompt-injection risks before promotion.

### 17.3 Hook Command Safety

Hook scripts SHOULD:

- run with timeouts,
- run in a validated working directory,
- avoid destructive operations unless explicitly allowed,
- emit structured results when possible.

### 17.4 Skill Import Safety

Imported skill text MUST be treated as untrusted until normalized and reviewed.

The normalizer SHOULD detect:

- instructions to exfiltrate data,
- attempts to override system/project rules,
- broad destructive commands,
- hidden self-modification instructions,
- missing evidence requirements.

## 18. Reference Algorithms

### 18.1 Task Routing

```text
function route_task(task):
  read AGENTS.md
  classify job_type
  if classification uncertain and materially important:
    ask or record assumption
  load job_type contract
  load required specs
  load required skills
  load required memory artifacts
  produce route_plan
  return route_plan
```

### 18.2 Skill Rendering

```text
function render_skill(skill_ir, target):
  validate skill_ir
  renderer = select_renderer(target)
  artifact = renderer.render(skill_ir)
  verify behavioral_invariants_preserved(skill_ir, artifact)
  write artifact with source metadata
  return artifact
```

### 18.3 Completion Verification

```text
function verify_completion(job_type, traceability_record):
  contract = load_verification_contract(job_type)
  for check in contract.required_checks:
    ensure check has result or justified skip
  ensure acceptance criteria are mapped
  ensure unresolved risks are disclosed
  if requirements unmet:
    return incomplete_with_reasons
  return complete
```

### 18.4 Improvement Proposal Generation

```text
function propose_harness_improvement(reflection, traces):
  if evidence is insufficient:
    do not propose material change
  identify repeated failure or high-impact friction
  create proposal with evidence, risk, rollback, and validation plan
  store proposal as proposed
  do not apply unless policy allows
```

### 18.5 Skill Normalization

```text
function normalize_external_skill(source_text):
  extract jtbd
  extract triggers
  extract procedure
  extract evidence requirements
  extract forbidden behaviors
  extract outputs
  extract success criteria
  if required fields missing:
    mark candidate and request completion
  else:
    create Skill IR candidate
  return candidate
```

## 19. Test and Validation Matrix

A conforming implementation SHOULD include tests for the behaviors defined in this specification.

Validation profiles:

- `Core Conformance`: deterministic tests REQUIRED for conforming implementations.
- `Extension Conformance`: REQUIRED only for OPTIONAL features shipped by the implementation.
- `Real Agent Profile`: environment-dependent smoke tests RECOMMENDED before production use.

Core eval files SHOULD define named cases. Each case SHOULD include:

- command,
- expected output or observable condition,
- why the behavior matters,
- the harness contract it protects.

Core evals SHOULD avoid duplicating every implementation detail. They should protect movement toward outcomes: routing produces action, render targets stay fresh, hooks guide lifecycle behavior, memory/playbooks/reflections remain operational, and proof commands support completion claims.

### 19.1 Directory and Config Conformance

- Required directories can be initialized.
- `AGENTS.md` exists and is referenced by entry instructions.
- Config validates against schema.
- Unknown keys are ignored or rejected according to strict-mode policy.
- Missing required skills are detected.
- Duplicate skill IDs are rejected.
- Core eval file contains deterministic cases with command, expected result, and rationale.

### 19.2 Router Conformance

- Known task examples route to expected job types.
- Ambiguous tasks produce either clarification or documented assumption.
- Route results include route card path, required context, skills, work loop, artifacts, proof, done conditions, and next action.
- Route cards load required skills and artifacts.
- Router refuses to bypass verification requirements.
- Every required job type has a route card.
- Route cards are validated as executable contracts, not descriptive labels.
- Route cards include relevant core playbooks in required context.
- Route work loops require reading proof output before completion claims.
- Canonical fresh-session build prompts bootstrap into a spec-first route instead of a maintenance route.

### 19.3 Skill IR Conformance

- Valid Skill IR passes schema validation.
- Missing required fields fail validation.
- Invalid status values fail validation.
- Behavioral invariants are available to renderers.
- Imported skills remain candidate until normalized.
- Active debug discipline includes reliable reproduction, fail-path tracing, hypothesis disproof, and breadcrumb ledger requirements.

### 19.4 Rendering Conformance

- Skill IR renders to configured targets.
- Rendered artifacts include source metadata.
- Lossy rendering fails with typed error.
- Generated artifacts are deterministic for the same input.
- Generated artifacts are fresh relative to Skill IR.
- Render targets that need identical bytes use symlinks instead of duplicate regular files.
- Generated Claude skill targets use 9arm-style `SKILL.md` frontmatter with `name` and `description`.
- Render targets include use, action, evidence, forbidden behavior, and done-condition sections where applicable.

### 19.5 Hook Conformance

- Session start loads expected artifacts.
- Prompt submit classifies job type.
- Tool pre-check can block unsafe operations when configured.
- Tool post-check captures changed files or command results.
- Turn stop detects missing proof obligations.
- Session end creates reflection when configured.
- Hook guidance includes purpose, action, and boundary for each canonical event.

### 19.6 Verification Conformance

- Completion without required checks is rejected.
- Skipped checks require justification.
- Acceptance criteria mapping is required for implementation-like jobs.
- Traceability records include files, checks, and results.
- Checkpoint or resume surfaces, when shipped, must expose latest stage, next action, and unresolved risks without replaying raw transcript state.
- Bugfix verification requires reliable reproduction or an explicit blocker, fail-path evidence, hypothesis disproof, and validation against the original symptom.

### 19.7 Learning Conformance

- Reflection records are created after meaningful sessions.
- Improvement proposals include evidence, risk, rollback, and validation plan.
- Material harness changes are not auto-applied when `learning.auto_apply=false`.
- Rejected proposals do not modify active skills or router rules.
- Memory artifacts include operational use guidance and remain concise.
- Core playbooks are exactly the MECE set selected by the implementation unless the spec or validation changes.
- Review playbooks preserve simpler-alternative, actual-path tracing, and claim-vs-verification checks.
- Learning playbooks preserve post-mortem gates and honest validation coverage.

### 19.8 Security Conformance

- Secrets are redacted from traces.
- Imported skills are not promoted without normalization.
- Hook timeouts are enforced.
- Dangerous hook commands are blocked or require explicit policy allowance.

### 19.9 Real Agent Profile

Recommended checks:

- Copilot instruction rendering works in the target repository.
- Codex or AGENTS.md routing works for a real task.
- Claude-style skill rendering works if Claude Code is used.
- Hooks fire in the target agent runtime if hook integration is enabled.
- A real implementation task produces traceability evidence.

## 20. Implementation Checklist

### 20.1 Required for Core Conformance

- Repository directory initializer.
- Lightweight entry instruction generator.
- `AGENTS.md` router template.
- Semantic router with deterministic fallback.
- Route card schema.
- Route card for every required job type.
- Job type schema.
- Skill IR schema.
- Skill registry loader.
- Skill renderer abstraction.
- Render targets for Claude/9arm-style skill, Copilot instruction, hook prompt, and checklist.
- Generated-render freshness validation.
- Duplicate render-target content detection with symlink policy.
- Operational memory artifacts with quality gates.
- Core playbook set with quality gates.
- Hook router guidance with purpose, action, and boundary.
- Verification contract model.
- Traceability record model.
- Reflection record model.
- Improvement proposal model.
- Reflection and proposal quality gates.
- Validation command.
- Security redaction for secrets.
- 9arm-derived debug, review, and post-mortem process invariants normalized into local artifacts.

### 20.2 Recommended for Practical Use

- Render targets for Codex/AGENTS.md and eval criteria.
- Hook integration with a universal hook router.
- Trace store using files or SQLite.
- Skill import/normalization command.
- Doctor command.
- Real-agent smoke tests.
- Built-in MECE core skill set.

### 20.3 Recommended Extensions

- Vector retrieval for memory artifacts.
- SQLite-backed trace and skill performance store.
- Dashboard or local status server.
- CI integration.
- Signed skill packages.
- WASM skill execution.
- Policy-engine integration.

## Appendix A. Recommended Core Skill Taxonomy

```text
.agent-harness/skills/

00-routing/
  classify-job
  select-skills
  define-done

10-understand/
  read-spec
  repo-map
  assumption-register

20-design/
  design-options
  risk-register
  scrutinize

30-implement/
  implement-from-spec
  minimal-change
  refactor-safely

40-debug/
  debug-discipline
  repro-builder
  fail-path-tracer

50-verify/
  verification-planner
  test-runner
  traceability-map

60-document/
  post-mortem
  decision-record
  management-talk

70-review/
  code-review
  spec-review
  skill-review

80-learn/
  reflection-ledger
  harness-improvement-proposal
  skill-author
```

## Appendix B. Example `AGENTS.md` Skeleton

```markdown
# Agent Operating Contract

Read this file before making material changes.

## Routing Rule

Run or mentally apply the semantic router before acting. The router maps the user's requested outcome to a route card.

The route card is the procedure contract. Follow its required context, skills, work loop, artifacts, proof commands, and done conditions.

## Source of Truth

Use `docs/specs/` for governing requirements.

## Completion Rule

Do not claim completion until the job-type verification contract is satisfied or skipped checks are justified.

## Job Types

- research
- spec
- implementation
- bugfix
- refactor
- test
- review
- release
- documentation
- harness_improvement
- skill_authoring

## Required Behavior

- Select one route card before material work.
- Load required context from that route card.
- Load required skills from that route card.
- Execute the route card's work loop in order.
- Produce required artifacts from that route card.
- Preserve traceability from requirement to change to verification.
- Record assumptions when context is missing.
- Propose harness improvements through the improvement proposal flow, not by directly mutating core instructions.
```

## Appendix C. Example Copilot Entry Instruction

```markdown
# Copilot Entry Instructions

Before making material changes, read `AGENTS.md` and follow the project router.

Use `docs/specs/` as source of truth.

Use the selected job type's required skills and verification contract.

Do not claim completion without verification evidence or documented skipped checks.
```

## Appendix D. Example Skill Import Metadata

```yaml
source:
  kind: external_skill_repository
  name: example-skills
  url: https://example.invalid/repo
  imported_at: 2026-05-24T00:00:00Z
  license: unknown
normalization:
  status: candidate
  notes:
    - extracted behavioral invariants
    - evidence requirements added during normalization
review:
  required: true
  reviewed_by: null
```

## Appendix E. Example Traceability Record

```yaml
task:
  job_type: bugfix
  summary: Fix failure when session hook payload lacks session_id.
requirements:
  - Adapter must tolerate missing session_id.
files_changed:
  - path: src/adapters/claude.rs
    reason: Add fallback session identifier handling.
verification:
  - command: cargo test missing_session_id
    result: passed
  - command: cargo test
    result: passed
unresolved_risks: []
completion_claim: All required checks passed and fallback behavior is covered by regression test.
```

## Appendix F. Example Reflection Record

```yaml
session_id: session-123
job_type: bugfix
what_worked:
  - Repro-first debugging prevented speculative changes.
what_failed:
  - Existing test fixtures did not include missing session_id payloads.
friction_points:
  - Adapter payload docs were hard to locate.
missed_context:
  - Compatibility matrix lacked nullability notes.
candidate_improvements:
  - Add adapter fixture coverage requirement for missing optional fields.
evidence_links:
  - trace-456
```
