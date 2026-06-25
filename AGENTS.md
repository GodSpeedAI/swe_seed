# Agent Operating Contract

Read this file before making material changes.

## Normative Language

The key words MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in RFC 2119.

Implementation-defined means the behavior is part of the implementation contract, but this specification does not prescribe one universal policy. Implementations MUST document the selected behavior.

## Routing Is Mandatory

Do not treat job types as labels to remember. A job type only matters because it selects a route card that tells you what to do next.

For every task, run or mentally apply the semantic router:

```bash
python scripts/harness.py route "<user task>"
```

The route result is the operating plan. It provides:

- selected job type
- route card path under `.agent-harness/routes/`
- required context to read
- required skills to load
- ordered work loop
- required artifacts
- proof commands
- done conditions
- next action

## Execution Rule

After routing:

1. Read the required context from the route result.
2. Load or apply the required skills.
3. Execute the work loop in order.
4. Produce the required artifacts.
5. Run the proof commands.
6. Do not claim completion until the done conditions are satisfied or skipped checks are justified.

If the route card is missing, incomplete, or does not move the task toward the requested outcome, treat that as a harness defect and use the `harness_improvement` route.

## Strategic Behavior Shaping

The harness is not only an orchestrator. It should use behavior shaping to exploit known agent tendency patterns when that improves outcome production.

Use cheap levers first: route cards, proof gates, narrow context, generated files, validation, checklists, formatting defaults, and explicit “done when” conditions. These work because agents tend to follow local structure, over-trust labels, drift into summaries, and claim completion too early.

Do not add a nudge because it is clever. A behavior-shaping rule has no value unless it helps produce the requested outcome, improves proof quality, reduces recovery time, or prevents a known failure mode.

## Context Discipline

Before broad exploration, apply the context budget:

```bash
python scripts/harness.py context-plan "<user task>"
```

Use tool-output containment for high-volume reads and commands. Do not pour raw logs, whole directories, or broad search output into the conversation when a count, path list, focused excerpt, or trace note will move the task.

Use think in code for bulk analysis: write or run a small script, shell pipeline, parser, or query that computes the answer, then bring back the result. The agent should generate the analysis mechanism rather than manually processing large data in the context window.

Preserve session continuity through trace records, route decisions, memory updates, and compact restart notes. The next agent should recover the task state from durable artifacts instead of relying on the conversation transcript.

## Source of Truth

Use `HARNESS_SPEC.md` and `docs/specs/` for harness requirements. Use project specs when a task names a feature or product behavior.

## Required Job Types

Each job type MUST have a route card in `.agent-harness/routes/`.

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

## Completion Rule

Completion is not a statement of effort. Completion requires proof.

For implementation-like work, the default proof command is:

```bash
just ci
```

Use narrower proof commands only when the selected route card says they are enough for the claim.

Other forms of proofs are references to files, created or modified, test results (or the commands to generate them), or other side-effect/change/result/output artifacts. The proof commands are the minimum required to claim completion, but they may not be sufficient to guarantee it. Use judgment to determine if additional proof is needed before claiming completion.

## Prose And Claim Discipline

Agent prose is part of the harness because it shapes whether future work moves or stalls.

- avoid em dashes unless quoted source text or a project style rule requires them.
- no praise before verification. Do not celebrate, congratulate, or declare quality before reading proof output.
- avoid should work, probably works, looks good, and similar completion-adjacent language. State the observed evidence or state the remaining gap.
- Prefer concrete next actions over descriptive summaries when the task is not done.
- Keep final answers tied to changed artifacts, proof commands, and unresolved risks.
