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

The hook router is guidance, not an engine. It should tell the agent what to do and what boundary not to cross.
