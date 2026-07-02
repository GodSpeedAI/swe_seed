# Capture Traces and Checkpoints

Use traces when the task needs durable state across edits, validation, restart, or handoff.

## Start the trace

```bash
just harness-trace-start "implement the requested change"
```

This creates a trace record and links it to a route decision.

## Append meaningful evidence

```bash
just harness-trace-append TRACE_ID "validation failed because the required route example is missing"
```

Append only when the note changes understanding of the task. Do not dump raw logs into the trace.

## Checkpoint at stage boundaries

```bash
just harness-trace-checkpoint TRACE_ID \
  change \
  "route behavior corrected; docs and evals still pending" \
  "update validation"
```

Use checkpoints when another agent may need to resume the task later.

If you need artifact and risk fields, use the full Rust CLI surface:

```bash
cargo run -q -p swe-seed -- trace checkpoint TRACE_ID \
  --stage change \
  --summary "route behavior corrected; docs and evals still pending" \
  --next-action "update validation" \
  --artifact crates/swe-seed/src/cli.rs \
  --risk "full proof not rerun"
```

## Resume from the latest checkpoint

```bash
just harness-trace-resume TRACE_ID
```

This should give the next agent enough state to continue without rereading the transcript.

## Finish only after proof

```bash
just harness-trace-finish TRACE_ID \
  "contract implemented" \
  "just ci" \
  "exit 0"
```

## Keep in mind

- Route decisions show process selection, not completion.
- Traces should store commands, artifacts, and observed outcomes.
- Shared IDs such as `trace_id`, `session_id`, and `span_id` let traces and replay line up without coupling runtimes.
- Never store secrets or private tokens in trace files.
