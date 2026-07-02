# Add a Conformance Eval

Use this when the harness needs a deterministic check for behavior that must not regress.

Conformance evals protect the harness contract. They are not a place to restate implementation details or add decorative examples.

## 1. Choose the right eval surface

Use the narrowest file that matches the behavior:

- `.agent-harness/evals/core-conformance.md` for positive core behavior,
- `.agent-harness/evals/negative-conformance.md` for breakages the harness must catch,
- `.agent-harness/evals/route-conflicts.md` for ambiguous prompts where precedence matters.

If the check also belongs in a deterministic shell assertion, add a matching line to `just harness-validate`.

## 2. Write the smallest useful case

Each case should prove one thing.

Good cases:

- one canonical route prompt,
- one trace capability,
- one hook contract,
- one validation failure mode.

Weak cases:

- a large scenario that mixes several behaviors,
- implementation trivia that would fail on harmless refactors,
- a restatement of the spec with no executable command.

## 3. Use the standard structure

For core evals, include:

- case title,
- command,
- expected output or condition,
- why it matters.

For negative evals, include:

- failure title,
- breakage,
- expected failure,
- why it matters.

For route conflicts, include:

- conflict title,
- prompt,
- expected route,
- precedence rule.

## 4. Prefer observable behavior

Commands should check output, files, or processable conditions.

Prefer:

- `just harness-route "prompt"`
- `just harness-validate`
- `just harness-validate`

Avoid cases that require subjective interpretation when a small CLI assertion could prove the same thing.

## 5. Add the matching deterministic check when useful

If the behavior is cheap to assert in the validation script, add a line to `just harness-validate`.

That script is the scaffold guardrail. The eval markdown explains the contract. The shell check proves it in CI.

## 6. Rerun proof

```bash
just harness-validate
just harness-validate
just ci
```

## Done when

The new case protects one real harness behavior, it is executable or directly tied to an executable check, and the proof chain passes.
