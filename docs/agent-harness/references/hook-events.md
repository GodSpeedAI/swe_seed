# Hook Events

These are the canonical hook events surfaced by `.agent-harness/hooks/hook-router.sh`.

| Event           | Purpose                                           | Typical action                                                |
| --------------- | ------------------------------------------------- | ------------------------------------------------------------- |
| `session.start` | Orient the agent before work begins               | read `AGENTS.md`, context guidance, repo map, and constraints |
| `prompt.submit` | Convert user intent into an executable route plan | run or mirror the router and require a next action            |
| `tool.pre`      | Prevent expensive or unsafe tool misuse           | check workspace, intent, context budget, and destructive risk |
| `tool.post`     | Preserve evidence while it is fresh               | capture changed files, results, and proof relevance           |
| `turn.stop`     | Block false completion claims                     | compare proof obligations against observed evidence           |
| `session.end`   | Capture reusable learning without drift           | write reflection or proposal only when evidence warrants      |

The hook router supports two modes:

- guidance mode, which emits `purpose`, `action`, and `boundary` text for the canonical event,
- capture mode, which emits the same guidance through `scripts/agent-hooks capture` with shared identifiers such as `trace_id`, `session_id`, and `span_id`.

For `prompt.submit`, capture mode also records a route preview when a `--task` value is provided. This keeps hook observability aligned with routing without making the dev harness depend on the agent harness runtime.
