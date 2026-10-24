# Route Cards Reference

This document provides complete reference specifications for all 11 canonical `RouteCard` files located in `.agent-harness/routes/`.

---

## 1. `bugfix`
- **Purpose**: Identify root cause, establish reliable reproduction, and apply targeted bug fixes.
- **Card Path**: `.agent-harness/routes/bugfix.json`
- **Semantic Triggers**: `fix`, `bug`, `issue`, `regression`, `broken`, `fails`, `error`, `crash`.
- **Required Context**:
  - `AGENTS.md`
  - `.agent-harness/playbooks/30-debug-from-symptom.md`
  - `docs/dev-harness/README.md`
  - `.agent-harness/memory/constraints.md`
- **Required Skills**: `debug-discipline`
- **Work Loop**:
  1. Establish reliable reproduction before editing code.
  2. Trace fail path end-to-end to identify root cause.
  3. Apply minimal fix addressing root cause.
  4. Run proof commands and observe passing output.
- **Required Artifacts**: Reliable reproduction, observed failure, root cause note, targeted fix, passing proof output.
- **Proof**: `just ci`
- **Done When**: Original failure no longer reproduces; root cause is connected to fix; proof commands pass.

---

## 2. `implementation`
- **Purpose**: Implement planned features according to established specifications.
- **Card Path**: `.agent-harness/routes/implementation.json`
- **Semantic Triggers**: `implement`, `feature`, `build`, `add`, `create`, `support`.
- **Required Context**:
  - `AGENTS.md`
  - `.agent-harness/playbooks/20-change-with-proof.md`
  - `docs/dev-harness/README.md`
  - Relevant subsystem specification under `docs/specs/`
- **Required Skills**: None
- **Work Loop**:
  1. Review governing specifications and tests.
  2. Implement behavior in dependency order.
  3. Add or update automated tests verifying new behavior.
  4. Run proof commands.
- **Required Artifacts**: Implementation source, automated tests, proof evidence.
- **Proof**: `just ci`
- **Done When**: Specified capabilities are implemented and passing verification.

---

## 3. `documentation`
- **Purpose**: Create or update documentation that helps a reader act correctly.
- **Card Path**: `.agent-harness/routes/documentation.json`
- **Semantic Triggers**: `document`, `docs`, `readme`, `guide`, `how to`, `explain`.
- **Required Context**:
  - `AGENTS.md`
  - `.agent-harness/playbooks/10-frame-outcome.md`
  - `.agent-harness/playbooks/20-change-with-proof.md`
  - `docs/dev-harness/README.md`
  - `.agent-harness/memory/constraints.md`
- **Required Skills**: None
- **Work Loop**:
  1. Identify the reader and action the doc must support.
  2. Choose how-to, explanation, tutorial, or reference format.
  3. Write concise task-oriented content.
  4. Run proof commands and inspect output.
- **Required Artifacts**: Documentation change, reader/action note, verification evidence.
- **Proof**: `just ci`
- **Done When**: The doc supports a concrete reader action in the right information shape; proof passes.

---

## 4. `refactor`
- **Purpose**: Restructure code to improve maintainability or clarity without altering external behavior.
- **Card Path**: `.agent-harness/routes/refactor.json`
- **Semantic Triggers**: `refactor`, `clean up`, `restructure`, `simplify`, `deduplicate`.
- **Required Context**:
  - `AGENTS.md`
  - `.agent-harness/playbooks/20-change-with-proof.md`
  - `docs/dev-harness/README.md`
- **Required Skills**: None
- **Work Loop**:
  1. Verify existing test suite passes cleanly before modifying code.
  2. Apply structural changes incrementally.
  3. Run test suite between increments to confirm zero behavioral regressions.
- **Required Artifacts**: Refactored code, regression-free test output.
- **Proof**: `just ci`
- **Done When**: Code is restructured; zero behavioral changes; full CI passes.

---

## 5. `test`
- **Purpose**: Author or improve automated test suites and evaluation checks.
- **Card Path**: `.agent-harness/routes/test.json`
- **Semantic Triggers**: `test`, `coverage`, `assert`, `unit test`, `integration test`, `eval`.
- **Required Context**:
  - `AGENTS.md`
  - `docs/specs/0013-eval-and-proof.md`
  - `docs/dev-harness/README.md`
- **Required Skills**: None
- **Work Loop**:
  1. Identify untested edge cases or regression scenarios.
  2. Author targeted tests asserting required invariants.
  3. Confirm new tests fail against broken behavior and pass against valid behavior.
- **Required Artifacts**: Automated test files, observed passing execution.
- **Proof**: `just ci`
- **Done When**: New tests execute reliably and demonstrate deterministic results.

---

## 6. `review`
- **Purpose**: Perform structured risk analysis, technical debt audits, or pull request reviews.
- **Card Path**: `.agent-harness/routes/review.json`
- **Semantic Triggers**: `review`, `audit`, `inspect`, `check diff`, `risk analysis`.
- **Required Context**:
  - `AGENTS.md`
  - `.agent-harness/playbooks/40-review-for-risk.md`
  - Target diff or commit range
- **Required Skills**: `neatcode`
- **Work Loop**:
  1. Inspect changed artifacts and diff scope.
  2. Check compliance against layer boundary rules and system invariants.
  3. Compile concrete findings with file paths and line numbers.
- **Required Artifacts**: Structured review findings report.
- **Proof**: Explicit reviewed findings documented.
- **Done When**: Findings are source-backed and categorized by severity.

---

## 7. `research`
- **Purpose**: Investigate questions, explore codebase patterns, or analyze third-party protocols.
- **Card Path**: `.agent-harness/routes/research.json`
- **Semantic Triggers**: `research`, `investigate`, `explore`, `why does`, `how does`.
- **Required Context**:
  - `AGENTS.md`
  - Relevant subsystem specifications under `docs/specs/`
- **Required Skills**: None
- **Work Loop**:
  1. Formulate specific technical questions.
  2. Collect concrete evidence from code, specs, and execution output.
  3. Synthesize findings, distinguishing verified facts from inferences.
- **Required Artifacts**: Research note or report.
- **Proof**: Source citations supporting all conclusions.
- **Done When**: Questions are answered with evidence-backed citations.

---

## 8. `spec`
- **Purpose**: Author or amend numbered subsystem specifications.
- **Card Path**: `.agent-harness/routes/spec.json`
- **Semantic Triggers**: `spec`, `specification`, `RFC`, `design doc`, `contract`.
- **Required Context**:
  - `SWE_SEED_SPEC_v0.2.0.md`
  - `HARNESS_SPEC.md`
  - `docs/specs/0012-existing-harness-reconciliation.md`
- **Required Skills**: None
- **Work Loop**:
  1. Define purpose, non-goals, data models, and CLI behaviors.
  2. Check vocabulary alignment against spec 0012.
  3. Update BAML contracts in `.agent-harness/baml/baml_src/`.
- **Required Artifacts**: Numbered spec under `docs/specs/`, updated BAML schemas.
- **Proof**: `just harness-validate`
- **Done When**: Spec is structurally complete and validated.

---

## 9. `release`
- **Purpose**: Package, tag, and verify release artifacts and changelogs.
- **Card Path**: `.agent-harness/routes/release.json`
- **Semantic Triggers**: `release`, `tag`, `publish`, `version bump`, `changelog`.
- **Required Context**:
  - `Cargo.toml`
  - `docs/dev-harness/README.md`
- **Required Skills**: None
- **Work Loop**:
  1. Confirm working directory is clean and all tests pass.
  2. Bump version numbers across workspace manifests.
  3. Generate release notes and verify provenance signatures.
- **Required Artifacts**: Version bump diff, release notes, signed trace.
- **Proof**: `just parity`
- **Done When**: Release artifacts are built and verified.

---

## 10. `harness_improvement`
- **Purpose**: Repair, extend, or optimize the SWE_SEED harness and its route cards.
- **Card Path**: `.agent-harness/routes/harness_improvement.json`
- **Semantic Triggers**: `harness`, `route card`, `behavior shaping`, `nudge`, `governance`.
- **Required Context**:
  - `AGENTS.md`
  - `HARNESS_SPEC.md`
  - `.agent-harness/memory/constraints.md`
- **Required Skills**: None
- **Work Loop**:
  1. Identify harness defect, missing route card, or drifting rule.
  2. Implement minimal structural adjustment.
  3. Verify against `harness_validate` and golden test suites.
- **Required Artifacts**: Harness change, proof evidence.
- **Proof**: `just harness-validate && just ci`
- **Done When**: Harness validation and CI pass with zero regressions.

---

## 11. `skill_authoring`
- **Purpose**: Ingest, normalize, scan, and render reusable skills.
- **Card Path**: `.agent-harness/routes/skill_authoring.json`
- **Semantic Triggers**: `skill`, `author skill`, `SkillIR`, `render skills`.
- **Required Context**:
  - `docs/specs/0007-skill-ingestion-and-scan-gate.md`
  - `.agent-harness/skills/`
- **Required Skills**: None
- **Work Loop**:
  1. Author or import skill in `.agent-harness/skills/<name>/SKILL.md`.
  2. Run security scan gate against skill scripts and instructions.
  3. Render skill to host targets via `swe-seed render-skills`.
- **Required Artifacts**: Skill definition, rendered target projection.
- **Proof**: `just harness-render-skills && just harness-validate`
- **Done When**: Skill validates and renders cleanly to targets.
