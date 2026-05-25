# Context and Continuity

The harness treats context as a limited resource.

Agents fail when they burn the context window on raw logs, broad searches, and transcript replay. The harness counters that with a small set of rules: read route-required context first, keep high-volume tool output out of the conversation, use code for bulk analysis, and preserve state in durable artifacts instead of relying on the chat transcript.

The main entry point is:

```bash
python scripts/harness.py context-plan "task"
```

That command combines the semantic route with local context policy. It tells the agent what to read first, how to contain output, and where continuity belongs.

Continuity comes from artifacts, not memory of the conversation. The important ones are:

- route decision records for why a route was chosen,
- trace records for what changed and what proof ran,
- checkpoints for stage, next action, and unresolved risks,
- memory artifacts for durable lessons,
- spec changes when the contract itself moved.

This matters whenever a session is compacted, handed off, or resumed later. A later agent should be able to recover from the trace and the named artifacts without rereading the whole session.

The practical rule is also simple:

- Prefer summaries, counts, and focused excerpts to raw dumps.
- Start a trace before material work.
- Checkpoint when the task crosses a meaningful boundary.
- Resume from trace artifacts, not transcript archaeology.
