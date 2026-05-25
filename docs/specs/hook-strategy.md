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

Another implementation must also define the binding contract, not only the event names. At minimum, document:

- how the native agent invokes each hook,
- the payload shape passed in,
- what identifiers are preserved across hooks and traces,
- what the hook may return,
- which failures are advisory versus blocking.

The preferred portable contract is stdin JSON plus stdout JSON or structured text, with large payloads written to filesystem artifacts. When a dev-harness observability layer exists, hooks should preserve shared identifiers such as `trace_id`, `session_id`, and `span_id` so routing, replay, and proof artifacts remain correlated without coupling the two harnesses at runtime.
