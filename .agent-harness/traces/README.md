# Trace Records

## Use this when

Use this when an implementation, bugfix, refactor, release, or harness improvement needs a durable record of what route was selected, what changed, what proof ran, and what risk remains.

## Route decision

Every trace begins with a Route decision. The route decision captures the task text, selected route card, confidence, required context, work loop, proof commands, and next action. Create it with:

```bash
python scripts/harness.py route --record "implement the requested change"
```

Route decisions live under `.agent-harness/traces/route-decisions/`. They are evidence of process selection, not proof that the work is complete.

## Trace record

A Trace record is the working ledger for the task. Start one before material changes:

```bash
python scripts/harness.py trace start "implement the requested change"
```

Append notes when the task crosses a meaningful boundary:

```bash
python scripts/harness.py trace append TRACE_ID "validation failed because the required doc is missing"
```

Checkpoint when the task reaches a stage boundary or needs a clean handoff:

```bash
python scripts/harness.py trace checkpoint TRACE_ID \
  --stage change \
  --summary "spec updated; validation delta still pending" \
  --next-action "update conformance checks" \
  --artifact HARNESS_SPEC.md \
  --artifact .agent-harness/evals/core-conformance.md \
  --risk "proof command not rerun yet"
```

Resume from the latest handoff packet:

```bash
python scripts/harness.py trace resume TRACE_ID
```

The resume output should be enough for a later agent to continue without rereading the transcript.

Finish it only after proof has been read:

```bash
python scripts/harness.py trace finish TRACE_ID --claim "contract implemented" --command "just ci" --result "exit 0"
```

Do not store secrets, credentials, customer data, or private tokens in trace files. Store references to commands, files, and observed outcomes.

## Done when

A useful trace lets a later agent answer four questions without rereading the whole session: what route governed the work, what evidence changed the plan, what proof was run, and what risk remains. A useful checkpoint adds the current stage and next action in one compact packet.
