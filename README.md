# SWE_SEED

### Decide what will prove the work before the agent starts doing it.

Your coding agent finishes a task and gives you a good summary:

```text
Implemented the checkout fix.
Added tests.
All checks pass.
```

Now you have another task.

Verify the summary.

Open the diff. Recover the original request. Figure out which files mattered. Check whether the agent found them. Work out which tests would actually prove the change. Run the tests. Notice one never ran. Find an unrelated edit. Decide whether the result is complete.

AI made implementation faster.

It did not remove the work required to establish that implementation actually happened.

**SWE_SEED is a lightweight verification harness for AI-assisted software work. It gives each task a route, required context, expected artifact, proof contract, and trace so completion claims are tied to executable evidence rather than an agent's summary.**

Plainly:

> SWE_SEED decides what kind of work this is, what the agent needs to read, what it must produce, and what must pass before anybody gets to call the work done.

The core contract is:

```text
work request
    ↓
route
    ↓
required context
    ↓
required artifact
    ↓
proof declared up front
    ↓
agent does the work
    ↓
trace
    ↓
evidence-backed completion claim
```

SWE_SEED does not write the code.

It does not provide a model.

It does not replace CI.

It makes the software-work contract explicit before execution begins.

---

## The reviewer became the missing harness

A normal coding-agent loop looks roughly like this:

```text
request
   ↓
agent searches repository
   ↓
agent edits
   ↓
agent chooses tests
   ↓
agent runs some checks
   ↓
agent summarizes
   ↓
human verifies everything important
```

That last step is doing more work than it appears to.

The reviewer has to reconstruct several things the agent was allowed to improvise:

```text
What kind of work was this?

Which repository context actually mattered?

What artifact should exist?

Which checks establish the requested behavior?

Did those checks run?

Did the work follow the repository's operating rules?

What remains unsupported by evidence?
```

As agents get faster, that burden grows rather than disappears.

One developer can now produce more changes than the same developer can deeply inspect.

Parallel agents widen the gap further.

SWE_SEED moves some of that structure in front of execution.

---

## Start with one task

Route a real request:

```bash
just harness-route "fix the failing checkout test"
```

SWE_SEED returns a work contract.

For a bug fix, that might include:

```json
{
  "job_type": "bugfix",
  "route_card": ".agent-harness/routes/bugfix.json",
  "required_context": [
    "AGENTS.md",
    ".agent-harness/playbooks/30-debug-from-symptom.md"
  ],
  "required_skills": ["debug-discipline"],
  "work_loop": [
    "establish a reliable reproduction",
    "trace the failure path",
    "identify the root cause",
    "implement the smallest correction"
  ],
  "required_artifacts": [
    "reliable reproduction",
    "observed failure",
    "root cause note",
    "implementation change"
  ],
  "proof": ["just ci"],
  "done_when": [
    "original failure no longer reproduces",
    "root cause is connected to the fix",
    "required proof passes"
  ]
}
```

Before the agent touches a file, several questions have stopped being improvisational:

```text
What work pattern applies?

What must be read?

What must be produced?

Which checks will count?

What does completion require?
```

That is the useful part.

---

## Five things make up the work contract

### Route

A **route** is the work pattern selected for the request.

Different work should not inherit the same generic instruction:

```text
"Inspect the repo, make the change, test it."
```

A bug fix requires different evidence from a documentation task.

A refactor has different obligations from an implementation.

A code review should produce findings, not an implementation.

SWE_SEED ships route cards for work types such as:

```text
bugfix
implementation
refactor
review
documentation
test
release
```

Each route can declare:

```text
required context
required skills
work loop
required artifacts
proof commands
done conditions
failure modes
```

The route gives the task a shape before the executor starts filling it in.

---

### Required context

Repository access is not the same as useful context.

An agent with permission to read 20,000 files still has to decide which twenty carry the constraints that matter.

SWE_SEED makes that decision explicit:

```bash
just harness-context-plan "write onboarding docs"
```

The resulting context plan identifies the material the route expects the agent to read.

For example:

```text
AGENTS.md
active specification
relevant implementation
nearby tests
applicable playbook
repository conventions
```

That does not mean the agent may never inspect another file.

It means there is a minimum body of context the work should not accidentally skip.

---

### Artifact

Every route declares what the task must actually produce.

For a bug fix:

```text
reproduction
root-cause evidence
code change
regression test
```

For documentation:

```text
documentation artifact
validated commands or examples
```

For review:

```text
findings
evidence
severity or disposition
```

This closes a surprisingly common gap:

```text
good explanation
≠
required artifact
```

The task is not complete because the agent can describe what it meant to produce.

---

### Proof

Proof commands are selected before the work begins.

That timing matters.

Suppose the agent writes a change and then decides which test would demonstrate success.

The implementation now influences the test selection.

Easy checks are attractive.

Expensive checks are inconvenient.

A narrow unit test may happen to agree perfectly with the implementation that just created it.

SWE_SEED instead binds proof to the route:

```text
task classified
      ↓
route selected
      ↓
proof declared
      ↓
implementation begins
```

If the route requires:

```text
just ci
```

then "I ran a smaller unit test instead" does not satisfy the same obligation.

That does not make `just ci` universally correct.

It makes the proof contract explicit and reviewable before the result exists.

---

### Trace

The agent conversation is not the durable work record.

SWE_SEED records a structured trace linking:

```text
request
route
context
events
completion claim
proof command
proof result
```

Trace records live under:

```text
.agent-harness/traces/records/
```

They are plain JSON.

The point is not to preserve every token the model generated.

The point is to preserve enough structure that another person or system can inspect what the completion claim rests on.

---

## Record a run

Start a trace:

```bash
just harness-trace-start "fix the failing checkout test"
```

As work progresses:

```bash
just harness-trace-append <trace-id> "root cause isolated to stale checkout state"
```

Checkpoint longer work:

```bash
just harness-trace-checkpoint \
  <trace-id> \
  implementation \
  "fix implemented; regression test remains" \
  "add regression test"
```

Finish the trace:

```bash
just harness-trace-finish \
  <trace-id> \
  "fixed checkout regression" \
  "just ci" \
  "pass"
```

Now the completion claim and the claimed proof live in the same structured record.

A reviewer can inspect that record instead of reconstructing the work entirely from a chat transcript.

---

## Missing proof is not an empty field

Run:

```bash
swe-seed gate <trace-id> --verify
```

The gate exits non-zero when the trace chain does not satisfy its required structure.

An unrouted task cannot quietly appear equivalent to a routed task.

A missing proof obligation does not become:

```text
proof: null
```

and then disappear into a summary.

For merge gating, the repository can use:

```text
gate-merge
```

to make route and trace verification consequential.

This does not replace CI.

It proves a different thing.

CI answers:

> Did these configured checks pass against this repository state?

SWE_SEED also asks:

> Were these the checks this kind of work was required to run, and is the completion claim connected to them?

---

## A test result and a completion claim are different objects

Suppose:

```text
just ci → exit 0
```

That is useful evidence.

It does not automatically mean:

```text
the requested artifact exists

the task was routed correctly

the decisive context was read

the bug's root cause was identified

the route's done conditions were satisfied

the agent changed only what the task required
```

SWE_SEED does not try to make one test command answer all those questions.

It preserves the pieces separately.

That makes disagreement inspectable.

---

## Bug fixes should not become implementations with a different label

Route cards let different work types demand different behavior.

A bugfix route can require:

```text
reproduce
   ↓
observe failure
   ↓
trace the failure path
   ↓
identify root cause
   ↓
make the smallest correction
   ↓
prove original failure is gone
```

That is materially different from:

```text
read ticket
   ↓
change code until tests pass
```

Why require the root cause?

Because otherwise the agent can remove the symptom while leaving the mechanism intact.

A future change reactivates it.

Now the same problem gets bought twice.

The route gives the work enough structure to distinguish repair from coincidence.

---

## Reviews should not quietly become rewrites

The same principle applies to review work.

A review route can require:

```text
inspect
   ↓
identify findings
   ↓
attach evidence
   ↓
classify severity
   ↓
report
```

The expected artifact is the finding set.

Not a rewritten subsystem.

This seems obvious until a capable coding agent sees a problem during review and helpfully starts fixing it.

SWE_SEED keeps the requested work type explicit.

---

## Context can be bounded without pretending the rest of the repository does not exist

A context plan is not a security boundary.

It is a representational aid.

The task may require:

```text
AGENTS.md
specific implementation
nearby tests
one specification
one playbook
```

Starting there is cheaper than treating every repository file as equally relevant.

The agent can expand outward when evidence says it needs more.

This creates a useful progression:

```text
minimum required context
        ↓
work begins
        ↓
new uncertainty appears
        ↓
inspect additional evidence
        ↓
continue
```

instead of:

```text
load everything possible
        ↓
hope attention lands on the right thing
```

In the broader GodSpeed architecture, Context Kernel can eventually supply stronger bounded, cited context packets.

SWE_SEED consumes that context.

It does not need to become the authoritative context service itself.

---

## One harness, several coding agents

Teams rarely use only one coding agent forever.

Today it may be Claude Code.

Tomorrow someone uses Codex.

Another developer prefers OpenCode.

A separate environment runs GitHub Copilot or Antigravity.

Without a shared harness, operating rules start getting copied into:

```text
CLAUDE.md
AGENTS.md
Copilot instructions
host settings
hooks
local developer conventions
```

Then they drift.

SWE_SEED keeps canonical harness state under:

```text
.agent-harness/
```

and projects host-specific forms from it.

List supported hosts:

```bash
swe-seed hosts
```

Project the harness:

```bash
swe-seed sync --host <host>
```

Inspect the change first:

```bash
swe-seed sync --host <host> --dry-run
```

Detect drift:

```bash
swe-seed doctor --host <host>
```

Roll back the last projection:

```bash
swe-seed rollback --host <host>
```

The rule is:

> Edit the harness source. Regenerate the host projections.

Do not maintain six independent interpretations of the same operating contract if they can be derived from one source.

---

## Host capability still matters

Projection does not make every agent host equally enforceable.

Some hosts support pre-tool hooks.

Some expose only advisory instructions.

Some expose lifecycle events that others do not.

SWE_SEED records those differences in host capability matrices:

```bash
swe-seed hosts
```

That prevents a generated configuration file from being marketed as stronger control than the underlying host actually provides.

A rule appearing in a prompt is not equivalent to a rule enforced before a tool call.

SWE_SEED keeps that distinction visible.

---

## Hooks can capture useful events

Where the host supports them, `.agent-hooks/` can respond to agent lifecycle events.

Examples include:

```text
prompt submitted
      ↓
route request

tool about to execute
      ↓
safety / policy-oriented check

command completed
      ↓
evidence capture

turn ending
      ↓
completion-language review
```

These hooks strengthen the harness where the host exposes the necessary surface.

They do not turn SWE_SEED into a universal runtime authority layer.

If a consequential action needs enforceable authorization across tools and systems, that is a different control problem.

---

## Trace state survives the conversation

Longer coding sessions fail in mundane ways.

The terminal closes.

The model context truncates.

A machine restarts.

The developer changes agent hosts.

The conversation gets abandoned overnight.

A trace can preserve structured checkpoints:

```bash
just harness-trace-checkpoint \
  <id> \
  debugging \
  "reproduction stable; root cause unresolved" \
  "inspect transaction boundary"
```

Resume later:

```bash
just harness-trace-resume <id>
```

Recovery becomes:

```text
read durable state
```

rather than:

```text
ask a new agent to infer what the previous agent was doing
```

That is a small difference until the work is expensive.

Then it becomes a large one.

---

## Failed work can become useful evidence

Most coding-agent failures disappear.

A bad session is closed.

Another prompt is tried.

The next agent starts with little more than the repository state the failed attempt left behind.

SWE_SEED can reflect on a finished trace:

```bash
swe-seed reflect <trace-id>
```

Reflection produces a **proposed** learning record.

It does not immediately rewrite the active harness.

That distinction matters.

One awkward session can suggest:

```text
this route may be missing context

this proof command may be too weak

this playbook may need another step

this failure should become a regression case
```

Those are candidates.

Promotion into active rules or skills requires review.

The harness should learn from evidence without letting every local annoyance permanently mutate the operating system.

---

## Faster is not automatically better

Suppose a route originally requires:

```text
reproduce
root-cause note
implementation
regression test
just ci
```

Later someone proposes:

```text
implementation
unit test
```

The second route is faster.

It may even have a higher task-completion rate.

But if the removed steps were carrying useful information, the route did not become better.

It became cheaper by lowering the standard.

SWE_SEED preserves route, proof, and trace history so changes to the harness can be evaluated against what they remove as well as what they accelerate.

---

## The harness can be evaluated too

Run an evaluation spec:

```bash
swe-seed eval run --spec <spec>
```

This lets harness behavior itself become testable.

Useful evaluation questions include:

```text
Does this task route correctly?

Does the route require the expected context?

Does a missing route fail the gate?

Does missing proof remain failure?

Does host projection remain deterministic?

Does drift detection catch a hand-edited projection?

Can the trace resume from its recorded checkpoint?
```

The harness should not be exempt from the proof discipline it asks software work to follow.

---

## The implementation is deliberately small

SWE_SEED consists primarily of:

```text
Rust CLI
+
repository-local harness data
+
JSON traces
+
host projections
+
specifications
```

There is no daemon required for ordinary standalone use.

There is no hosted control plane.

There is no bundled model.

There is no mandatory network service.

The canonical repository state lives under:

```text
.agent-harness/
```

That makes a simple deployment possible:

```text
one repository
one coding agent
one task type
one proof contract
```

You do not need to adopt the larger architecture to test whether the mechanism helps.

---

## Three layers

The repository separates three concerns:

```text
SWE_SEED
capability assembly and layer boundaries
        ↓
Harness
routing, context, proof, traces, hooks, learning
        ↓
Fabricator
bounded product-to-prototype runs
```

Their root specifications are:

```text
SWE_SEED_SPEC_v0.2.0.md
HARNESS_SPEC.md
FABRICATOR_SPEC_v0.1.0.md
```

The numbered design specifications under:

```text
docs/specs/
```

define narrower pieces of behavior.

When vocabulary conflicts, use the repository's authoritative specification rather than inferring meaning from old examples.

---

## Contracts are data

Core harness contracts are defined under:

```text
.agent-harness/baml/baml_src/
```

They describe the data structures used by the harness.

SWE_SEED does not require a language-model call at runtime to interpret those contracts.

The model participates as the coding executor through the host.

The harness itself remains deterministic where its job is deterministic.

---

## Install and bootstrap

Prerequisites:

- Rust 1.75+
- `cargo`
- [`just`](https://github.com/casey/just)

Clone the repository, then:

```bash
just bootstrap
just doctor
```

Run the normal repository checks:

```bash
just ci
```

Individual stages are available as:

```bash
just format
just lint
just test
```

Validate the harness itself:

```bash
just harness-validate
```

---

## First five minutes

Route a task:

```bash
just harness-route "fix the failing checkout test"
```

Inspect the required context:

```bash
just harness-context-plan "fix the failing checkout test"
```

Record the route:

```bash
just harness-route-record "fix the failing checkout test"
```

Start a trace:

```bash
just harness-trace-start "fix the failing checkout test"
```

Finish it with proof:

```bash
just harness-trace-finish \
  <trace-id> \
  "fixed checkout regression" \
  "just ci" \
  "pass"
```

Verify the trace:

```bash
swe-seed gate <trace-id> --verify
```

That is enough to experience the core mechanism.

---

## Everyday commands

### Repository setup

```bash
just bootstrap
just doctor
just ci
just format
just lint
just test
```

### Routing and context

```bash
just harness-route "task"
just harness-route-record "task"
just harness-context-plan "task"
just harness-validate
just harness-doctor
```

### Traces

```bash
just harness-trace-start "task"
just harness-trace-append <id> "note"
just harness-trace-checkpoint <id> <stage> "summary" ["next"]
just harness-trace-resume <id>
just harness-trace-finish <id> "claim" ["command" "result"]
```

### Host projection

```bash
swe-seed hosts
swe-seed sync --host <host>
swe-seed sync --host <host> --dry-run
swe-seed rollback --host <host>
swe-seed doctor --host <host>
```

### Verification and improvement

```bash
swe-seed gate <trace-id> --verify
swe-seed eval run --spec <spec>
swe-seed reflect <trace-id>
swe-seed seed assemble
swe-seed provenance verify
```

See:

```bash
swe-seed --help
```

for the complete command surface.

---

## Repository layout

```text
AGENTS.md
```

The repository operating contract for coding agents.

```text
.agent-harness/routes/
```

Route cards describing work types.

```text
.agent-harness/skills/
.agent-harness/playbooks/
.agent-harness/memory/
```

Reusable procedures and durable project context.

```text
.agent-harness/traces/
```

Route decisions and trace records.

```text
.agent-harness/baml/baml_src/
```

Harness contract schemas.

```text
.agent-harness/reflections/
```

Learning proposals kept separate from active instructions.

```text
.agent-hooks/
```

Host lifecycle hooks.

```text
crates/swe-seed/
crates/swe-seed-core/
```

CLI and core Rust implementation.

```text
docs/specs/
```

Numbered design specifications.

---

## What SWE_SEED proves — and what it does not

SWE_SEED is strongest when its claims remain narrow.

A valid routed trace with recorded proof can support a statement such as:

```text
This work was routed as a bugfix,
required these artifacts,
declared this proof,
recorded this execution chain,
and produced this proof result.
```

It does not automatically prove:

```text
the product requirement was correct

the code is secure

the business outcome succeeded

the agent now possesses durable capability

the runtime action was authorized by organizational policy
```

Those are different judgments.

Keeping them separate prevents the harness from turning evidence into stronger claims than the evidence supports.

---

## What SWE_SEED is not

### Not a coding agent

SWE_SEED does not reason through the implementation, edit files, or decide what code to write.

The host agent does the work.

SWE_SEED defines the work contract and preserves evidence about how that contract was exercised.

### Not an orchestrator

It does not provide a long-running multi-agent control runtime.

For work that branches, retries, delegates, recovers, and runs over long horizons, a dedicated execution harness such as Gauntlet solves a different problem.

### Not CI

Keep CI.

SWE_SEED decides which proof belongs to the work before execution and records whether that proof was exercised.

CI performs the downstream checks.

### Not runtime authority

Hooks may inspect tool use where a host exposes that capability.

They are not a substitute for an independent authority layer governing consequential side effects such as deployments, secrets, merges, or external systems.

### Not settlement authority

SWE_SEED can establish that a work contract produced its required artifact and proof evidence.

That does not automatically settle the larger outcome.

### Not capability formation

One successful routed task is one successful routed task.

Reliable capability requires repetition under variation, transfer, reduced orchestration burden, and recovery.

SWE_SEED preserves evidence that can support that later judgment.

It does not manufacture the judgment itself.

---

## Where it fits

SWE_SEED works standalone.

That should remain the default way to understand it:

```text
coding request
      ↓
SWE_SEED
route + context + artifact + proof contract
      ↓
coding agent
      ↓
trace + evidence
```

Within the broader GodSpeed architecture:

```text
.sea
represents consequential domain meaning
        ↓
DomainForge
compiles and validates that meaning

Context Kernel
supplies bounded evidence-bearing context
        ↓
SWE_SEED
defines the software-work contract
        ↓
SEA-Forge
authorizes consequential action
        ↓
Gauntlet
executes long-horizon work when needed
        ↓
RealityTrace
binds expected and observed results
        ↓
GodSpeed-Agent
updates developmental memory and capability
```

Not every task needs every layer.

A small coding change may need only:

```text
SWE_SEED
+
existing coding agent
+
CI
```

Use additional structure when the work earns it.

---

## Current status

The core routing → trace → proof → gate loop is implemented and exercised by the repository's test suite.

### Implemented

- routing and route cards
- context planning
- trace lifecycle
- JSON trace records
- harness structure validation
- host projection
- drift detection
- host rollback
- hook runtime
- OpenTelemetry and JUnit exports
- eval specs
- learning reflection
- reviewed learning promotion
- seed assembly
- boundary validation
- provenance verification
- trace-chain merge gating

Host projection currently covers:

```text
Claude
Codex
OpenCode
GitHub Copilot
Antigravity
CI
```

### Implemented with narrower guarantees

Host enforcement strength varies.

Check:

```bash
swe-seed hosts
```

before assuming a rule can be enforced before action on a particular host.

The optional learning store and vector retrieval path are not required for normal file-based use.

### Experimental

- MCP gateway
- federation envelope exchange
- SEA-Forge verification integration

### Architectural target

- deeper Context Kernel integration
- GodSpeed-Agent settlement handoff
- stronger cross-host behavioral parity

Treat those targets as direction until the corresponding proof exists.

---

## Adopt it without modeling your whole software organization

The most expensive way to adopt SWE_SEED would be to design every possible route before running one task.

Do the opposite.

Pick:

```text
one repository
one coding agent
one work type
one proof set
```

Bugfix is a good first route because failure behavior is easy to understand.

Make sure these cases work:

```text
correctly routed task
missing route
missing artifact
failing proof
missing proof
successful proof
trace resume
```

A harness becomes credible when its failure behavior is clear.

Not when its happy-path demo is impressive.

---

## Two adoption paths

### Use this repository as the starting point

Clone the reference implementation and adapt:

```text
route cards
skills
playbooks
proof commands
host projections
```

This is the easier route when you want the existing runtime and validation.

### Rebuild from the specifications

Bring the relevant specifications into an existing repository and implement the harness around that repository's structure.

Start with:

```text
HARNESS_SPEC.md
SWE_SEED_SPEC_v0.2.0.md
AGENTS.md
.github/copilot-instructions.md
```

This path makes more sense when the repository already has mature conventions you do not want to reorganize around the reference project.

---

## Security and privacy

SWE_SEED stores its ordinary state in repository-local files.

It exposes no mandatory network service of its own.

Secrets should not be written into:

```text
traces
memory
examples
rendered instructions
```

The specifications and validation treat that as an operating rule.

They cannot prevent a sufficiently privileged or misbehaving external agent from deliberately writing secret material to an ordinary file.

Federation keys, when federation is used, can be stored encrypted with SOPS.

Host hooks can inspect tool calls only where the host exposes the required hook surface.

Do not confuse that with an independent authorization boundary.

---

## Documentation

- [Agent operating contract](AGENTS.md)
- [Harness specification](HARNESS_SPEC.md)
- [SWE_SEED specification](SWE_SEED_SPEC_v0.2.0.md)
- [Fabricator specification](FABRICATOR_SPEC_v0.1.0.md)
- [Development harness guide](docs/dev-harness/README.md)
- [Agent harness specification index](docs/specs/agentic-swe-harness.md)

---

## The short version

Before the coding agent starts, decide:

```text
what kind of work this is

what it must read

what it must produce

what will prove it
```

Then let the agent work.

Record what happened.

Keep missing proof missing.

Let CI remain CI.

Do not turn one completion summary into stronger evidence than it deserves.

If the work is important enough to verify after the agent finishes, it is often important enough to decide **how it will be verified before the agent begins**.

That is SWE_SEED.

---

## License

No license file is currently included.

Add one before publishing the project for reuse outside your organization.
