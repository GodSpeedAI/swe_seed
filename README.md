# SWE_SEED

SWE_SEED is a low-touch development harness for teams that use AI coding agents and still need dependable software outcomes.

It gives humans and agents one shared way to decide what kind of work is being requested, load only the context that matters, make the change, and prove the result before anyone calls it done. When hooks are enabled, the harness can guide that loop as the agent works instead of waiting for a human to remember every step.

## Boundary

SWE_SEED is the **dev-work harness**: it owns **routing** (prompt → route card),
**hooks**, **traces**, and **proof gating** (completion claims depend on proof
commands). It is the sole repo that should be called "harness" in this stack.

It does **not** own settlement classification or the capability lifecycle — that
is [godspeed_agent](../godspeed_agent)'s job (the **settlement / navigation
runtime**). SWE_SEED emits `WorkRequested`, `ContextRequired`, `RouteSelected`,
`ProofStarted`/`ProofCompleted`; godspeed_agent consumes evidence and emits
`SettlementRecorded` / `CapabilityUpdated`. Keeping these boundaries prevents
the duplicated hook/trace/learning machinery both repos drifted toward (F-08).


## Why It Exists

AI agents can move fast, but speed does not help when the work stops at a confident summary instead of a verified change. SWE_SEED changes the default state from "trust the agent" to "follow the route, produce the artifact, show the proof."

Use it when you want a project where:

- a vague request turns into a concrete next action,
- the agent reads the right project context before editing,
- every job type has a clear route,
- completion claims depend on proof commands,
- failures leave enough evidence for the next person or agent to recover,
- useful lessons improve the harness without turning it into a pile of prompts,
- the system can grow from files to indexes only when the project actually needs that power.

The result is a calmer development loop. Contributors know where to start, agents know what procedure to follow, and reviewers can ask for evidence instead of reconstructing intent from a transcript.

SWE_SEED is designed to feel powerful without feeling heavy. Hooks can respond to the agent lifecycle, route new prompts, check risky tool use before it happens, capture proof after commands run, and block premature completion language at turn end. The harness does the routine steering, so humans can focus on intent, judgment, and review.

It also solves a concrete team problem: process gets better only when someone notices friction, writes it down, and turns it into a safer default. SWE_SEED makes that part of the loop. Sessions can produce reflections and improvement proposals with evidence, risk, rollback, and validation plans. The harness can recommend better routes, skills, checks, memory, or hook behavior as the project teaches it what actually fails.

## What This Repository Provides

SWE_SEED is both a reference implementation and a portable specification.

- `AGENTS.md` gives agents the operating contract.
- `.agent-harness/routes/` maps work types to required context, work loops, artifacts, and proof.
- `.agent-harness/skills/` stores reusable work procedures in a portable format.
- `.agent-harness/playbooks/` keeps common workflows short and inspectable.
- `.agent-harness/memory/` captures durable project context, decisions, constraints, and patterns.
- `.agent-harness/traces/` records route decisions and recoverable work history.
- `.agent-harness/hooks/` connects agent lifecycle events to routing, safety checks, trace capture, verification, and reflection.
- `.agent-harness/reflections/` keeps learning review packets and improvement proposals separate from active instructions.
- `scripts/harness.py` exposes routing, validation, context planning, skill rendering, and trace commands.
- `justfile` gives humans, agents, and CI one command surface.
- `SWE_SEED_SPEC_v0.2.0.md`, `HARNESS_SPEC.md`, and `FABRICATOR_SPEC_v0.1.0.md` are the root layer contracts; `docs/specs/` holds the detailed design.

## Layered Architecture

SWE_SEED is a three-layer stack. Each layer is governed by the one above it, and ownership flows downward only.

```
SweSeed     (SWE_SEED_SPEC_v0.2.0.md)   governance, capability assembly, layer boundaries
  Harness   (HARNESS_SPEC.md)           routing, proof, context, hooks, traces, learning
    Fabricator (FABRICATOR_SPEC_v0.1.0.md)  product to prototype semantic chain
```

- The **SweSeed** layer centralizes and governs capabilities, assembles a capability package, and validates layer boundaries.
- The **Harness** layer routes each request to a route card, gates completion on proof, manages context budgets, hooks, traces, and the learning loop.
- The **Fabricator** layer runs a bounded product-to-prototype pass over a traceable semantic chain.

`LayerName` is `{ SweSeed, Harness, Fabricator }`. The data model for all three layers is defined as `.baml` contracts under `.agent-harness/baml/baml_src/` and consumed as data, with no language model called at run time.

## Design Specs and the Rust Rewrite

The design lives in committed specs:

- `docs/specs/0001`–`0020` define each subsystem. `0012` reconciles these specs with the existing harness and is authoritative on any vocabulary conflict.
- `.agents/plans/0001-swe-seed-v0-1-implementation.md` is the plan of record: a full rewrite of all three layers in Rust, building to the specs, with golden-file parity against the current Python harness, after which the Python implementation is removed.

The current `scripts/harness.py` (plus `agent_hooks.py`, `fabricate.py`) is the Python reference implementation that the Rust rewrite reproduces and supersedes.

## The Approach

SWE_SEED treats process as part of the product. The harness does not try to make the agent smarter by adding more instructions everywhere. It narrows the moment:

1. Route the task by the outcome the user wants.
2. Read the context that route requires.
3. Follow the route's work loop.
4. Produce the required artifact.
5. Run the proof command.
6. Leave a trace when recovery or learning matters.

This keeps attention on the work that changes the outcome. The harness uses small, visible constraints because small constraints are easier to trust, debug, and improve.

The hook layer is what makes the harness low touch. A prompt can trigger routing. A tool call can trigger a safety check. A command result can trigger evidence capture. A final response can trigger proof review. Instead of asking every contributor to memorize the operating model, SWE_SEED puts the right reminder at the moment it matters.

The learning layer is how it improves without becoming reckless. SWE_SEED does not silently rewrite its own rules because one session felt awkward. It captures evidence, proposes the change, names the risk, defines rollback, and asks for validation. That gives teams the benefit of a self-improving harness while keeping humans in control of material changes.

The storage model follows the same discipline. The baseline is files because files are easy to inspect, review, back up, and rebuild. As the project grows, the harness can propose a rusql index when recent failures, sessions, hooks, or trace lookups become too slow for simple tools. It can propose vector search later, only when exact and structured search stop answering real recovery questions. The project gets stronger memory when it earns the weight.

## When It Helps

Use SWE_SEED when your team asks an agent to:

- implement a feature and prove it with tests,
- fix a bug without guessing at the cause,
- review code with findings before summaries,
- write documentation that helps a reader act,
- prepare a release with explicit checks,
- keep agent sessions on track through lifecycle hooks,
- turn repeated friction into reviewed improvement proposals,
- decide when trace search has outgrown files and needs rusql or vector retrieval,
- improve the harness only when evidence shows the change is useful.

The reader should not need to translate "process" into value. The value is fewer hidden assumptions, fewer missed checks, and a shorter path from request to verified result.

## Quick Start

Prerequisites:

- Python 3.12 or newer
- `just`
- `pnpm` for formatting checks
- `uv` for Python linting
- `mise`, recommended for tool installation

Set up the repository:

```bash
git clone https://github.com/GodSpeedAI/swe_seed.git
cd swe_seed
just bootstrap
just doctor
just ci
```

Use the harness directly:

```bash
python scripts/harness.py route "fix the failing checkout test"
python scripts/harness.py context-plan "write onboarding docs"
python scripts/harness.py validate
```

The most important command is:

```bash
just ci
```

That is the default local proof command. CI should call the same contract, so the local result and remote result stay aligned.

## Adopt It In Another Project

You can use SWE_SEED in two ways.

### 1. Start From This Repository

Clone it, keep the harness structure, and adapt the specs, routes, skills, and checks to your project.

This is the best path when you want the working reference implementation and validation scripts.

### 2. Rebuild From The Specs

Copy these files into a target project:

- `HARNESS_SPEC.md`
- `SWE_SEED_SPEC_v0.2.0.md`
- `AGENTS.md`
- `.github/copilot-instructions.md`

Then ask your agent to follow the specs and implement the harness in that environment. This is the best path when your repository already has its own structure and you want to port the operating model.

## How To Work With An Agent

Start with the outcome, not the implementation guess.

Good requests look like:

```text
Fix the failing CI check and show the proof.
```

```text
Add documentation that helps a new contributor run local CI.
```

```text
Review this PR for regressions and missing tests.
```

The harness routes those requests into different procedures. A bugfix needs cause before fix. Documentation needs a reader and action. A review needs findings before summary. The route keeps those differences visible.

## Safety Boundaries

SWE_SEED is intentionally small.

- It is not a coding agent.
- It is not an IDE.
- It is not a project management system.
- It does not require a database, dashboard, or model provider.
- It does not store secrets in traces, memory, examples, or rendered instructions.
- It does not auto-apply material harness changes without an explicit policy.

The harness governs process, context, evidence, and recovery. Native agents still do the reasoning, editing, command execution, and tool use.

That is the core tradeoff: SWE_SEED stays ridiculously lightweight at the start, then proposes heavier capabilities only when they solve an observed job.

## Project Status

This project is a draft reference implementation. The core harness contracts are expressed in specs, route cards, validation scripts, and local commands. Expect the shape to evolve as proof from real usage shows which constraints help and which ones add noise.

If you change the harness, change it with evidence. The standard is not "more process." The standard is whether the change helps a future contributor produce a better verified outcome with less recovery cost.

## Documentation Map

- [Agent operating contract](AGENTS.md)
- [Harness specification](HARNESS_SPEC.md)
- [SWE_SEED dev harness specification](SWE_SEED_SPEC_v0.2.0.md)
- [Dev harness guide](docs/dev-harness/README.md)
- [Agent harness spec index](docs/specs/agentic-swe-harness.md)

## Development Commands

```bash
just bootstrap          # install or synchronize local tools
just doctor             # report missing required or recommended tools
just format             # check formatting
just lint               # run static checks
just test               # run harness validation tests
just ci                 # run the local CI contract
```

Harness commands:

```bash
python scripts/harness.py validate
python scripts/harness.py doctor
python scripts/harness.py render-skills
python scripts/harness.py route "task"
python scripts/harness.py context-plan "task"
python scripts/harness.py trace start "task"
```

## License

No license file is included yet. Add one before publishing this project for reuse outside your organization.
