# Semantic Chain v0.1 Backlog

## Purpose

Capture the remaining implementable work needed to move the semantic specification chain from typed contract coverage into deeper deterministic generation and validation.

## Validator Backlog

- Parse `JOB-*`, `REQ-*`, `ADR-*`, `Y-*`, `CMP-*`, `SCN-*`, `CHK-*`, `TASK-*`, and `PROOF-*` IDs from rendered artifacts.
- Verify every `REQ-*` links to a `JOB-*` or approved upstream source.
- Verify every `ADR-*` contains at least one `Y-*`.
- Verify every `Y-*` links to one or more constraints or requirements.
- Verify every `CMP-*` links to one or more `REQ-*`.
- Verify every `SCN-*` links to one or more `REQ-*`.
- Verify every required `SCN-*` maps to `CHK-*`, an automated test, or manual proof item.
- Verify every `TASK-*` links to `ContextPack` and `EvalSpec`.
- Verify every `PROOF-*` links to `EvalResult`.
- Verify every SkillProposal links to proof, eval, and reflection.
- Verify waivers are explicit, scoped, and approved.

## Generator Backlog

- Render `JOB_HYPOTHESIS.md` from the business-canvas fields in `ProductHypothesis`.
- Render EARS requirements as both structured fields and reviewable sentences in `PRD.md`.
- Render Y-Statements into `ADR.md` with linked rejected alternatives and accepted costs.
- Render SDS component contracts and Mermaid snippets from typed `SDSComponent` data.
- Render Gherkin scenarios into `SDS.md` or `TDD.md` depending on artifact intent.
- Generate `EvalSpec` checks directly from linked `REQ-*`, `Y-*`, `CMP-*`, `SCN-*`, and permission rules.
- Generate `AgentTask` packets that preserve upstream IDs and allowed-forbidden action bounds.
- Generate `ProofRecord` templates that cite `CHK-*`, `EVAL-*`, and observed evidence references.

## v0.1 Scope Guard

- Keep the validator file-first and repository-local.
- Prefer explicit waivers over silent gap filling.
- Do not invent missing intent during generation.
- Do not require a full diagram when a component table is enough.
- Do not treat syntax artifacts as proof until EvalSpec and ProofRecord confirm them.
