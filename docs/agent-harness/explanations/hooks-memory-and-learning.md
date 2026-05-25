# Hooks, Memory, and Learning

The harness does more than route tasks. It also shapes lifecycle behavior around the task.

Hooks provide the timing. They connect lifecycle events such as session start, prompt submit, tool execution, turn stop, and session end to routing, context discipline, safety, trace capture, and learning. In this repository the hook surface is intentionally lightweight: `.agent-harness/hooks/hook-router.sh` emits purpose, action, and boundary guidance for the canonical events. The hooks guide behavior. They do not replace the agent's reasoning.

Memory provides durable cognition. The harness keeps a repo map, decisions, constraints, failure patterns, successful patterns, a glossary, and open questions under `.agent-harness/memory/`. These files are not archives. They are small operating artifacts meant to sharpen future behavior. When they become transcripts or generic notes, they stop helping.

Learning provides controlled change. The harness can record reflections and improvement proposals, but it does not automatically rewrite its own core behavior just because one session had an idea. Material changes should move through spec, validation, implementation, and proof. That keeps learning tied to evidence instead of taste.

These three surfaces work together:

- hooks prompt the right action at the right time,
- memory keeps reusable context small and durable,
- learning captures repeated friction without destabilizing the system.

That is how the harness improves while staying governable.
