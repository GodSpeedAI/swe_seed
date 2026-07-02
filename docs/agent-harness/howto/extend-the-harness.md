# Extend the Harness

Use this when the harness itself needs a new behavior, route, eval, doc, or operator surface.

This is not the right path for product code. Use it only when routing, proof, context handling, traces, memory, hooks, Skill IR, or harness validation must change.

## 1. Identify the governing artifact

Before editing, decide which artifact actually controls the behavior:

- route behavior: `.agent-harness/routes/` or `crates/swe-seed/src/cli.rs`
- harness contract: `HARNESS_SPEC.md`
- operator guidance: `docs/agent-harness/`
- agent discovery docs: `docs/specs/`
- deterministic checks: `.agent-harness/evals/` and `just harness-validate`
- durable lessons: `.agent-harness/memory/`
- lifecycle guidance: `.agent-harness/hooks/`

Do not change several layers at once unless the contract truly moved.

## 2. Route the work and read the harness-improvement contract

Run:

```bash
just harness-route "describe the harness change"
```

For material harness work, the expected route is usually `harness_improvement`. Read that route card before changing files.

## 3. Update the spec first when the contract changes

If the change alters what the harness promises, start in `HARNESS_SPEC.md`.

Examples:

- a new route fallback rule,
- a new trace checkpoint requirement,
- a new hook responsibility,
- a new required doc or required file,
- a new conformance rule.

If the behavior is already part of the spec and only the implementation or docs are missing, you can skip the spec edit.

## 4. Add deterministic validation

The harness should fail usefully when the behavior is missing.

Typical validation surfaces are:

- `.agent-harness/evals/core-conformance.md` for named conformance cases,
- `.agent-harness/evals/route-conflicts.md` for route ambiguity,
- `.agent-harness/evals/negative-conformance.md` for known failure modes,
- `just harness-validate` for deterministic scaffold checks,
- `just harness-validate` for structural or phrase-level validation.

Prefer the smallest check that can prove the behavior.

## 5. Make the smallest implementation change

Keep the change narrow.

Good harness changes:

- tighten route scoring for a known prompt class,
- add one canonical route example,
- add one missing required file check,
- add one operator doc for an already-supported behavior,
- add one trace field that improves restart.

Weak harness changes:

- broad rewrites of several route cards with no new proof,
- adding new concepts without changing agent movement,
- duplicating the spec in several docs,
- adding files that validation never checks.

## 6. Update operator docs when humans need the behavior

If a human maintainer must know how to run, inspect, or extend the harness, add or update `docs/agent-harness/`.

If an agent only needs the contract, prefer `HARNESS_SPEC.md`, `docs/specs/`, route cards, and validation.

## 7. Run proof in order

For harness changes, use:

```bash
just harness-validate
just harness-validate
just ci
```

Read the output. Do not stop at “command ran.”

## 8. Capture durable state when needed

For longer harness work, start a trace and checkpoint stage boundaries:

```bash
just harness-trace-start "extend harness"
just harness-trace-checkpoint TRACE_ID change "spec updated" "run validation"
```

## Done when

The governing contract is updated where needed, validation enforces the new behavior, operator docs reflect the current state when relevant, and the full proof chain passes.
