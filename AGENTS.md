# Agent Operating Contract

Read this file before making material changes.

## Normative Language

The key words MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in RFC 2119.

Implementation-defined means the behavior is part of the implementation contract, but this specification does not prescribe one universal policy. Implementations MUST document the selected behavior.

## Routing Is Mandatory

Do not treat job types as labels to remember. A job type only matters because it selects a route card that tells you what to do next.

For every task, run or mentally apply the semantic router:

```bash
swe-seed route "<user task>"
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
swe-seed context-plan "<user task>"
```

Use tool-output containment for high-volume reads and commands. Do not pour raw logs, whole directories, or broad search output into the conversation when a count, path list, focused excerpt, or trace note will move the task.

Use think in code for bulk analysis: write or run a small script, shell pipeline, parser, or query that computes the answer, then bring back the result. The agent should generate the analysis mechanism rather than manually processing large data in the context window.

Preserve session continuity through trace records, route decisions, memory updates, and compact restart notes. The next agent should recover the task state from durable artifacts instead of relying on the conversation transcript.

## Local Agent Memory

Use `.agents/` as local, working memory for active agent handoffs. These files are **ignored by git and are non-authoritative scratch** — never a source of truth. Treat `.agents/current_status.yml`, `.agents/DEBT.md`, `.agents/OPEN_QUESTIONS.md`, `.agents/reports/`, `.agents/plans/`, and `.agents/lessons/` as per-agent scratch that may lag or diverge from committed reality; verify against the committed sources of truth (specs in `docs/specs/`, harness memory in `.agent-harness/memory/`, and the code itself) before relying on them. Agents SHOULD keep them current enough that the next agent can recover the latest outcome, blocker, and next action without reading the full transcript, but they are not load-bearing.

Before material work, read any existing local memory that can change the next action:

- `.agents/current_status.yml` for the current outcome, route, working state, proof status, blockers, and next action.
- `.agents/OPEN_QUESTIONS.md` for load-bearing questions that cannot be answered from the codebase alone.
- `.agents/DEBT.md` for out-of-scope issues already noticed.
- `.agents/lessons/` for durable lessons from prior coding challenges or repeated mistakes.

After material work, update local memory as follows:

- Update `.agents/current_status.yml` when the pursued outcome, route, changed artifacts, proof status, blocker, or next action changes. Keep it short enough to serve as a handoff, not a transcript.
- Update `.agents/OPEN_QUESTIONS.md` only for questions that block or materially change the outcome and cannot be resolved from code, docs, tests, or command output. Include a recommendation, known options or tradeoffs, and the decision needed from a human.
- Update `.agents/DEBT.md` for problems, risks, or cleanup noticed while working that are real but out of scope for the current task. Include evidence, impact, and a suggested follow-up.
- Add a note under `.agents/lessons/` only when the lesson is generalizable across future work in this codebase. Do not record one-off surprises, personal preferences, raw logs, or chat transcripts. Lessons that recur MAY later be promoted into `AGENTS.md` or `.agent-harness/memory/`.

All local memory entries MUST be concrete, concise, source-backed when practical, and free of secrets. If a required `.agents/` file or directory is missing, create the smallest useful file or directory before relying on it.

## Source of Truth

Normative sources are committed; local memory is not.

- **Specs:** the numbered subsystem specs (`0001`–`0020`) live under `docs/specs/` and are the authoritative design source. The root layer contracts `SWE_SEED_SPEC_v0.2.0.md`, `HARNESS_SPEC.md`, and `FABRICATOR_SPEC_v0.1.0.md` sit at the repo root and defer detail to `docs/specs/`. `.agents/specs/` MUST NOT exist — it is migration-only; if a spec is normative it belongs in `docs/specs/`. `.baml` contracts under `.agent-harness/baml/baml_src/` are generated/reviewed from these specs.
- **Harness memory:** `.agent-harness/memory/` (committed) is canonical for durable harness memory — `constraints.md`, `decisions.md`, `failure-patterns.md`, `glossary.md`, `open-questions.md`, `repo-map.md`, `successful-patterns.md`. Route cards and validators read from here.
- **Plans:** implementation plans are execution artifacts, not normative. They live as scratch (`.agents/plans/`); if a plan's requirement becomes normative, promote it back into a numbered spec under `docs/specs/`. Do not treat plans as a source of truth.
- **Federation signing keys (spec 0011, Phase B):** SWE_Seed signs envelopes with an Ed25519 private key in gitignored `.swe-seed/federation/keys/` (SOPS-encrypt at rest via `just federation-encrypt-key <id>`). Public keys are committed under `.agent-harness/federation/keys/<id>.pub` and shared with SEA-Forge. SEA's own key lives in SEA. Workflow: `swe-seed federation keygen <id>` → `just federation-encrypt-key <id>` → sign with `swe-seed federation sign <trace> --key <id>`; verify with `swe-seed federation verify <file> --key <id>`.

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
