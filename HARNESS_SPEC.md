# HARNESS_SPEC

Root contract for the **Harness layer**: the middle layer that turns a request into routed,
proven work. This file is a stable root spec. Detailed design lives in `docs/specs/`; the
reconciliation in `docs/specs/0012-existing-harness-reconciliation.md` is authoritative on
any vocabulary conflict.

## Position in the stack

The Harness layer sits between the SweSeed layer (`SWE_SEED_SPEC_v0.2.0.md`) and the
Fabricator layer (`FABRICATOR_SPEC_v0.1.0.md`). It owns routing, proof, context, hooks,
traces, and learning. It is governed by SweSeed and governs Fabricator.

## Operating contract

Routing is mandatory: every request resolves to a `RouteCard` for one of the required job
types before work begins. Completion claims depend on proof. The full operating contract is
`AGENTS.md`.

## Owned concepts

Canonical schema: `.agent-harness/baml/baml_src/harness.baml` (contracts as data, no LLM
runtime; see `docs/specs/0019-baml-contracts-as-data.md`).

- `RouteCard` — job type, required context, work loop, required artifacts, proof, done-when.
- `SkillIR` — normalized, portable skill procedure with evidence and render targets.
- `EvalSpec` / `EvalCheck` / `EvalResult` — the deterministic evaluation gate.
- `ProofRecord` / `ProofDisposition` — claims with evidence, skipped checks, unresolved risks.
- `ContextBudget` / `ContextPack` — what context may enter, and the assembled pack.
- `HookPolicy` / `PermissionPolicy` — lifecycle gating and allowed or forbidden actions.
- `TraceSchema` — durable decision and trace record contract.
- `ReflectionTemplate`, `LearningRecord`, `LearningCandidate`, `SkillProposal`,
  `RegressionCase`, `AdaptationDecision` — the learning and adaptation loop.

## Layer rules

1. Routing first: read required context before editing (context discipline,
   `.agent-harness/context/budget-policy.yaml`).
2. Proof gate: an `EvalSpec` is frozen after handoff and cannot be edited to make work pass.
   A capability or lesson is promoted only on a live `Pass`. A waived check needs a reason.
3. Durable decisions: route decisions and traces survive compaction; on restart, read the
   latest route decision, trace record, changed specs, and unresolved risks first.
4. Hook runtime: events are logged with redaction; forbidden actions are blocked; risky
   actions are approval gated.
5. Learning is enforceable: a lesson becomes a `RegressionCase` linked to an `EvalCheck`, and
   a `SkillProposal` always includes a rollback plan.

## Evaluation and Adaptation Layer

Evaluation and adaptation are specified in this root `HARNESS_SPEC.md` and in
`docs/specs/0013-eval-and-proof.md`. This root contract preserves the stable section marker
used by harness smoke evals.

## Required job types

Each job type has a route card in `.agent-harness/routes/`:
`research`, `spec`, `implementation`, `bugfix`, `refactor`, `test`, `review`, `release`,
`documentation`, `harness_improvement`, `skill_authoring`.

## Commands (target Rust surface)

```
swe-seed route <task> [--record]                 # select a RouteCard, record the decision
swe-seed trace start|append|checkpoint|resume|distill|finish
swe-seed eval run --spec <path>                  # produce an EvalResult
swe-seed validate                                # static root and artifact checks
swe-seed doctor                                  # validate + eval + boundary checks
swe-seed context-plan <task>                     # produce a ContextPack
swe-seed render-skills                           # render SkillIR to host targets
swe-seed reflect <trace> ; swe-seed adapt <run>  # learning and adaptation loop
```

## Detailed specs

- `docs/specs/0004-host-adapter-contract.md`, `0005-normalized-hook-runtime.md`,
  `0006-mcp-gateway-integration.md`, `0007-skill-ingestion-and-scan-gate.md`,
  `0008-doctor-and-drift-detection.md`.
- `docs/specs/0013-eval-and-proof.md`, `0014-trace-and-durable-decisions.md`,
  `0015-context-budget-plane.md`, `0016-learning-and-adaptation-loop.md`.
- `.agents/plans/0001-swe-seed-v0-1-implementation.md` — Rust rewrite plan of record.

## Status

Stable root contract. Implementation is migrating from the Python reference harness
(`scripts/harness.py`, `scripts/agent_hooks.py`) to Rust per the plan of record.
