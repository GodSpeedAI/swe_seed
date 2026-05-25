# Capture Traces and Checkpoints

Use traces when the task needs durable state across edits, validation, restart, or handoff.

## Start the trace

```bash
python scripts/harness.py trace start "implement the requested change"
```

This creates a trace record and links it to a route decision.

If you want the trace itself to enter the shared observability path, run:

```bash
python scripts/harness.py trace start --capture-hook --agent copilot --agent-version local --session-id SESSION_ID --span-id TRACE_START_SPAN "implement the requested change"
```

That records a `trace.start` event through `scripts/agent-hooks` while preserving the generated `trace_id`.

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

If you want the checkpoint boundary in the same replay surface, run:

```bash
python scripts/harness.py trace checkpoint --capture-hook --agent copilot --agent-version local --session-id SESSION_ID --span-id TRACE_CHECKPOINT_SPAN TRACE_ID \
  --stage change \
  --summary "route behavior corrected; docs and evals still pending" \
  --next-action "update validation" \
  --artifact scripts/harness.py \
  --risk "full proof not rerun"
```

That records a `trace.checkpoint` event with the same shared-ID observability path used by route capture.

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
- Shared IDs such as `trace_id`, `session_id`, and `span_id` let traces and replay line up without coupling runtimes.
- Never store secrets or private tokens in trace files.
