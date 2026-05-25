# Failure Patterns

## Use this when

Read this before debugging, implementation, harness improvement, or review. These are known ways the harness can look complete while failing to move work toward the outcome.

## Keep in mind

The recurring failure is symbolic completion: files exist, labels exist, or commands exist, but they do not force a better next action.

## Patterns To Avoid

- Job type lists without route cards. Labels do not route work.
- Route output that names a category but omits context, skills, work loop, proof, or next action.
- Validation that checks file existence but not behavior, freshness, or contract content.
- Documentation that restates commands without explaining when to use them, why they exist, or how to recover from failure.
- CI YAML that duplicates local check logic and drifts from `just ci`.
- Completion claims without fresh proof-command output.
- Memory files that become long transcripts. They hide useful lessons instead of sharpening attention.
- Package-installed tools without lockfiles. Local CI then fails before it can test behavior.
- Harness changes that skip the spec. The next agent cannot tell whether the behavior is intended.
- Work that feels productive but does not move outcome: large summaries, broad rewrites, clever nudges, or new files that do not change routing, proof, memory, or execution.

## Outcome Anti-Patterns

- "Knowledge only" additions. If an instruction does not select a next action, block a failure mode, or improve proof quality, it is not harness progress.
- "Completion theater." More files, longer docs, or prettier checklists can hide the absence of executable behavior.
- "Semantic decoration." Route labels, skill names, or memory categories have no value unless they change what the agent reads, does, verifies, or records.
- "Unpaid abstraction." A new layer must remove a real decision burden or make a useful failure observable.
- "Proof outsourcing." Do not let CI, hooks, or route cards become words agents trust without reading actual command output.

## Recovery Move

When one of these appears, convert it into an executable contract: route card field, validation check, proof command, or concise memory rule.
