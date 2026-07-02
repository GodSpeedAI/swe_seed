# Context Mode Normalization

## Source mechanism

The source mechanism is `mksglu/context-mode` at `https://github.com/mksglu/context-mode`. Its README describes four ideas this harness can absorb without adding the package as a dependency: keep raw tool data out of the context window, track session continuity in durable records, use code to compute bulk analysis, and use hooks or routing pressure to make the model prefer context-saving paths.

## Imported invariant

- Contain raw tool output before it consumes the working context.
- Keep durable session continuity outside the transcript.
- Prefer think in code for bulk analysis.
- Add hook-time or route-time nudges only when they reduce context waste or improve recovery.
- Preserve model freedom for final prose except where local harness rules prevent premature completion claims.

## Local artifact

This repository implements the mechanism through local harness artifacts:

- `.agent-harness/context/README.md` defines the operating discipline.
- `.agent-harness/context/budget-policy.yaml` defines the context budget.
- `just harness-context-plan "task"` creates an executable context plan from the semantic route.
- `.agent-harness/traces/` stores route decisions, trace records, and restart evidence.
- `AGENTS.md` requires context budget, tool-output containment, think in code, and session continuity.

## Validation

`just harness-validate` checks that the context discipline exists and names the imported invariants. `just harness-validate` checks that the `context-plan` command is exposed and returns a context budget.
