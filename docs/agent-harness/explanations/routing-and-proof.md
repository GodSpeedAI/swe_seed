# Routing and Proof

The agent harness exists to make agent behavior operational instead of aspirational.

Without a router, a task starts as free-form interpretation. Different agents can read the same request and choose different work. Some will implement too early. Some will review when they should debug. Some will stop after editing files and never run proof. The harness removes that ambiguity by mapping a task to a route card with required context, a work loop, expected artifacts, proof commands, and done conditions.

This is why the route result includes more than a job label. A useful route tells the agent what to do next. In this repository that means a route card under `.agent-harness/routes/`, relevant playbooks, and explicit proof. If the route does not change the next action, it is decorative.

Proof is the second half of the contract. The harness separates work performed, proof gathered, and completion claimed. Editing files is not proof. A route is not proof. A trace record is not proof. Proof comes from the route's required commands and any justified skips. The default project proof command is `just ci`, but some routes also require focused harness checks such as `just harness-validate`.

The practical rule is simple:

- Route first.
- Read the route's required context.
- Follow the work loop.
- Run the proof commands.
- Read the output before claiming completion.

That sequence is the harness.
