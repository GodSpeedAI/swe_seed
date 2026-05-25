# Capture Traces and Checkpoints

Use traces when the task needs durable state across edits, validation, restart, or handoff.

## Start the trace

```bash
python scripts/harness.py trace start "implement the requested change"
```

This creates a trace record and links it to a route decision.

## Append meaningful evidence

```bash
python scripts/harness.py trace append TRACE_ID "validation failed because the required route example is missing"
```

Append only when the note changes understanding of the task. Do not dump raw logs into the trace.

## Checkpoint at stage boundaries

```bash
python scripts/harness.py trace checkpoint TRACE_ID \
  --stage change \
  --summary "route behavior corrected; docs and evals still pending" \
  --next-action "update validation" \
  --artifact scripts/harness.py \
  --artifact .agent-harness/evals/core-conformance.md \
  --risk "full proof not rerun"
```

Use checkpoints when another agent may need to resume the task later.

## Resume from the latest checkpoint

```bash
python scripts/harness.py trace resume TRACE_ID
```

This should give the next agent enough state to continue without rereading the transcript.

## Finish only after proof

```bash
python scripts/harness.py trace finish TRACE_ID \
  --claim "contract implemented" \
  --command "just ci" \
  --result "exit 0"
```

## Keep in mind

- Route decisions show process selection, not completion.
- Traces should store commands, artifacts, and observed outcomes.
- Never store secrets or private tokens in trace files.
