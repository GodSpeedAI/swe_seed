# Agent Harness

The agent harness defines how agents in this repository route work, load context, preserve traces, use cognitive artifacts, and prove completion.

Canonical source: [HARNESS_SPEC.md](../../HARNESS_SPEC.md).

This documentation is for operating the harness. The specification remains the source of truth for requirements and conformance.

## Start Here

- New task: [Start a Task](howto/start-a-task.md)
- Before claiming work is ready: [Run Harness Checks](howto/run-harness-checks.md)
- Need durable execution state: [Capture Traces and Checkpoints](howto/capture-traces.md)
- Need tamper-evidence / routing enforcement: [Enforce Routing](howto/enforce-routing.md)
- Need signed envelopes / SEA-Forge verification: [Manage Federation Keys](howto/manage-federation-keys.md)
- Route looks wrong: [Debug Routing](howto/debug-routing.md)
- Extending the harness: [Extend the Harness](howto/extend-the-harness.md)
- Add a route: [Add a Route Card](howto/add-a-route-card.md)
- Add a harness check: [Add a Conformance Eval](howto/add-a-conformance-eval.md)
- Need to inspect harness state: [Inspect the Harness](howto/inspect-the-harness.md)
- Need generated agent surfaces: [Render Skills and Targets](howto/render-skills-and-targets.md)

## Why It Exists

- [Routing and Proof](explanations/routing-and-proof.md)
- [Context and Continuity](explanations/context-and-continuity.md)
- [Verification and Conformance](explanations/verification-and-conformance.md)
- [Hooks, Memory, and Learning](explanations/hooks-memory-and-learning.md)
- [Federation Signing and Tamper-Evident Traces](explanations/federation-signing-and-traces.md)

## References

- [CLI Recipes](references/cli-recipes.md)
- [Route Map](references/route-map.md)
- [Artifact Map](references/artifact-map.md)
- [Eval Map](references/eval-map.md)
- [Hook Events](references/hook-events.md)
- [Memory Map](references/memory-map.md)
