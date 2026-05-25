# Glossary

## Use this when

Read this when terms in the harness could be confused or used as decoration. If a term does not change behavior, do not add it.

## Keep in mind

Definitions should support action. Prefer terms that help route work, choose proof, or preserve context.

`Semantic router`
: Mechanism that maps user intent to an executable route plan. It should consider outcome, verbs, referenced artifacts, triggers, examples, and risk. It must return next action, not only a label.

`Route card`
: Executable contract for a job type. It lists required context, skills, work loop, artifacts, proof commands, done conditions, and failure policy.

`Job type`
: Intermediate classification such as `implementation`, `bugfix`, or `harness_improvement`. It has value only when it selects a route card.

`Skill IR`
: Portable behavioral skill contract that can render into agent-specific surfaces while preserving triggers, evidence, forbidden behavior, and success criteria.

`Proof command`
: Command whose observed output supports a completion claim. The default proof command is `just ci`.

`Traceability record`
: Mapping from intended outcome to changed files, checks run, results, unresolved risks, and completion claim.

`Memory artifact`
: Small durable note that improves future agent attention. It is not a transcript or a dumping ground.
