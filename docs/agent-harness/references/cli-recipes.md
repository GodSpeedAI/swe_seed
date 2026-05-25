# CLI Recipes

These are the main harness entry points.

## Python CLI

```bash
python scripts/harness.py validate
python scripts/harness.py doctor
python scripts/harness.py render-skills
python scripts/harness.py route "task"
python scripts/harness.py route --record "task"
python scripts/harness.py route --capture-hook --agent copilot --agent-version local --session-id SESSION_ID --trace-id TRACE_ID --span-id SPAN_ID "task"
python scripts/harness.py inspect debug-discipline
python scripts/harness.py context-plan "task"
python scripts/harness.py trace start "task"
python scripts/harness.py trace start --capture-hook --agent copilot --agent-version local --session-id SESSION_ID --span-id TRACE_START_SPAN "task"
python scripts/harness.py trace append TRACE_ID "note"
python scripts/harness.py trace checkpoint TRACE_ID --stage change --summary "summary"
python scripts/harness.py trace checkpoint --capture-hook --agent copilot --agent-version local --session-id SESSION_ID --span-id TRACE_CHECKPOINT_SPAN TRACE_ID --stage change --summary "summary"
python scripts/harness.py trace resume TRACE_ID
python scripts/harness.py trace distill TRACE_ID
python scripts/harness.py trace finish TRACE_ID --claim "claim" --command "just ci" --result "exit 0"
.agent-harness/hooks/hook-router.sh prompt.submit --task "task" --agent copilot --agent-version local --session-id SESSION_ID --trace-id TRACE_ID --span-id SPAN_ID --capture
```

## Just recipes

```bash
just harness-validate
just harness-doctor
just harness-render-skills
just harness-route "task"
just harness-route-record "task"
just harness-inspect debug-discipline
just harness-context-plan "task"
just harness-trace-start "task"
just harness-trace-append TRACE_ID "note"
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

Use the Python CLI when you need the full argument surface. Use `just` when you want stable local recipes.
