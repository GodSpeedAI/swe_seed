# CLI Recipes

These are the main harness entry points.

## Rust CLI Recipes

```bash
just harness-validate
just harness-doctor
just harness-render-skills
just harness-route "task"
just harness-route-record "task"
just harness-context-plan "task"
just harness-trace-start "task"
just harness-trace-append TRACE_ID "note"
just harness-trace-checkpoint TRACE_ID change "summary"
just harness-trace-resume TRACE_ID
just harness-trace-distill TRACE_ID
just harness-trace-finish TRACE_ID "claim" "just ci" "exit 0"
.agent-harness/hooks/hook-router.sh prompt.submit --task "task" --agent copilot --agent-version local --session-id SESSION_ID --trace-id TRACE_ID --span-id SPAN_ID --capture
```

## Just recipes

```bash
just harness-validate
just harness-doctor
just harness-render-skills
just harness-route "task"
just harness-route-record "task"
just harness-context-plan "task"
just harness-trace-start "task"
just harness-trace-append TRACE_ID "note"
just harness-trace-checkpoint TRACE_ID change "summary"
just harness-trace-resume TRACE_ID
just harness-trace-distill TRACE_ID
just harness-plan-learning-store both
just harness-sync-learning-store
just harness-query-learning-store summaries 10 any any
just harness-eval-learning-retrieval
just harness-trace-finish TRACE_ID "claim"
```

## Project proof

```bash
just ci
```

Use `just` for stable local recipes. Use `cargo run -q -p swe-seed -- <command>` or a release `swe-seed` binary when you need the full CLI argument surface.
