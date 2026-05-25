# Hook Strategy

Hooks connect agent lifecycle events to routing, safety, trace capture, verification, and reflection.

This scaffold provides `.agent-harness/hooks/hook-router.sh` as a portable placeholder for these events:

- `session.start`
- `prompt.submit`
- `tool.pre`
- `tool.post`
- `turn.stop`
- `session.end`

Hooks should fail safely, avoid secrets in logs, and avoid destructive operations unless policy explicitly allows them.
