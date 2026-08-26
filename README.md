# SWE_SEED

Your coding agent wrote a convincing summary of the change. Did the change actually happen — and does it pass the checks that matter?

SWE_SEED is a CLI harness that turns a coding-agent request into a verifiable work contract: a selected route, the context that route requires, the artifact the task must produce, the proof commands chosen before work starts, and a trace that records what happened. The agent's final message ends the conversation. It does not close the work.

It works with the agents you already use — Claude Code, Codex, OpenCode, GitHub Copilot, Antigravity — and it runs locally against the repository, with no server and no model provider of its own.

```bash
just harness-route "fix the failing checkout test"
```

## The failure pattern

AI coding agents made code cheap. They did not make proof cheap. The verification burden did not disappear; it moved onto whoever reviews the agent's work.

The familiar loop:

```text
prompt → agent activity → completion summary → human forensic investigation
```

The summary says "implemented, tested, complete." The repository may show missing files, omitted tests, unrelated edits, or a check that never ran. The reviewer becomes the forensic layer: open the diff, rediscover the original request, figure out which context the agent missed, run the checks it skipped, and decide whether the narration matches reality.

The failure has structure, not just symptoms:

- **Narration closes claims.** The agent writes the code, chooses which tests to run, and grades its own homework. Asking one system to do all three is an efficient way to automate optimism.
- **Context selection is a lottery.** The agent can read the whole repository without knowing which files carry the decisive constraints. Repository access is not the same as relevant context.
- **Proof is chosen after the work.** Tests selected after the diff exists are vulnerable to convenience — easy checks get run, expensive ones get skipped, and "tested" loses its meaning.
- **Every host gets a different operating system.** The team's rules live in one file for Claude, another for Copilot, a third in somebody's head. They drift, and nobody notices until behavior diverges.
- **Failures evaporate.** A session that failed for an instructive reason leaves nothing structured behind. The next session starts from the same ignorance.

The problem is not that agents are careless. The problem is that nothing in the default workflow requires the work to prove itself.

## What SWE_SEED changes

The correction is to move proof from after the summary to before it, and to make the terms of the work explicit before execution:

```text
work request
→ route            which work pattern applies
→ bounded context  which files the agent must read
→ required artifact what must exist when the work is done
→ proof commands   which checks must pass, declared up front
→ trace            a structured record of the whole chain
```

The vocabulary, defined once:

- A **route** is the work pattern selected for a request. This repository ships route cards for bugfix, implementation, refactor, review, documentation, test, release, and more — each declaring its required context, work loop, artifacts, proof commands, and done conditions. A bugfix route demands a root cause before a fix; a review route demands findings before a summary.
- **Bounded context** is the explicit list of files the route requires the agent to read, instead of the whole repository treated as equally relevant.
- The **artifact** is what the task must produce — a diff, a test, a root-cause note, a reproduction. A convincing explanation without the artifact is not success.
- **Proof** is the set of commands bound to the route before work begins. If a required check never ran, "tested" has no standing.
- The **trace** is a JSON record linking request, route, context, events, and completion claim — written to `.agent-harness/traces/records/` so a reviewer inspects evidence instead of reconstructing a transcript.

Together these form the **work contract**: route, context, artifact, proof, trace, agreed before execution. That is the whole mechanism. It does not make the agent smarter; it makes the agent's claims falsifiable.

One boundary stays explicit: SWE_SEED makes the claim falsifiable. It does not grant the claim final standing. Review, security judgment, and settlement of the outcome still belong to humans — or, in the wider GodSpeed stack, to a separate settlement system.

## See it work

Prerequisites: Rust 1.75+ (`cargo`), and [`just`](https://github.com/casey/just). Clone, then:

```bash
just bootstrap   # install or sync local tools
just doctor      # report anything missing
```

Route a real request:

```bash
just harness-route "fix the failing checkout test"
```

What it prints is the work contract for that task, as JSON:

```json
{
  "job_type": "bugfix",
  "route_card": ".agent-harness/routes/bugfix.json",
  "required_context": ["AGENTS.md", ".agent-harness/playbooks/30-debug-from-symptom.md", "..."],
  "required_skills": ["debug-discipline"],
  "work_loop": ["establish reliable reproduction ...", "trace the fail path end-to-end ...", "..."],
  "required_artifacts": ["reliable reproduction", "observed failure", "root cause note", "..."],
  "proof": ["just ci"],
  "done_when": ["original failure no longer reproduces", "root cause is connected to the fix", "..."],
  "next_action": "Read required context, then execute work_loop[0]: establish reliable reproduction ..."
}
```

That output is the point. Before anyone — human or agent — touches a file, the task already has a defined shape: what to read, what to produce, what to run, and what "done" means. Add `--record` (or use `just harness-route-record`) to write the route decision into the trace ledger.

Record the work as it happens:

```bash
just harness-trace-start "fix the failing checkout test"
just harness-trace-finish <trace-id> "fixed checkout regression" "just ci" "pass"
```

This writes a trace record under `.agent-harness/traces/records/` containing the route, timestamped events, the completion claim, and the proof command with its result. Inspect it directly — it is plain JSON.

What failure looks like: `swe-seed gate <trace-id> --verify` exits non-zero if the trace was never routed or its chain fails verification. The `gate-merge` just recipe uses this as a merge gate — a change that bypassed routing cannot produce a passing gate. Missing proof is a failed obligation, not an empty field.

Two more commands worth running in the first five minutes:

```bash
just harness-context-plan "write onboarding docs"   # the bounded context set for a task
just harness-validate                                # structural integrity of the harness itself
```

## What the mechanism buys you

| Without SWE_SEED | What changes | What you can verify |
| --- | --- | --- |
| The agent improvises a procedure from one prompt | The route card fixes the work pattern for the task type | `swe-seed route "<task>"` shows the selected route and its obligations |
| The agent reads whatever it finds | The route names the required context up front | `swe-seed context-plan "<task>"` shows the bounded context set |
| "Done" means the agent stopped talking | The route declares required artifacts and done conditions | The route output lists `required_artifacts` and `done_when` |
| Tests are chosen after the diff exists | Proof commands are bound to the route before work starts | `proof: [...]` in the route; `swe-seed gate` fails on an unrouted trace |
| The summary is the only record | A JSON trace links request, route, events, claim, and proof | `.agent-harness/traces/records/*.json` |
| Every host gets hand-maintained instructions | Host files are regenerated projections of one canonical harness | `swe-seed sync --host <host> --dry-run`; `swe-seed doctor --host <host>` detects drift |
| Failed sessions teach nothing | Traces can be reflected into reviewed learning candidates | `swe-seed reflect <trace-id>` proposes; nothing promotes without review |

## How it works

SWE_SEED is a Rust CLI (`swe-seed`) plus a directory of canonical harness state (`.agent-harness/`) that lives in the repository. There is no daemon and no network surface of its own.

**Routing.** `swe-seed route` matches a request against the route cards in `.agent-harness/routes/` and returns the card's obligations as JSON. Route cards are data, not prompts: they declare required context, skills, an ordered work loop, artifacts, proof commands, done conditions, and failure modes.

**Traces.** The trace lifecycle (`trace start`, `append`, `checkpoint`, `resume`, `finish`) records what happened while work ran. Records are JSON files under `.agent-harness/traces/records/`; route decisions land in `traces/route-decisions/`. The `gate` command verifies a trace chain and exits non-zero on a trace that was never routed — the merge gate in `justfile` builds on this.

**Host projection.** Canonical policy, hooks, and skills live once in `.agent-harness/`; `swe-seed sync --host <host>` renders them into the host's own format (for Claude, that is `.claude/settings.json`). `swe-seed hosts` lists the supported adapters and their capability matrices — what each host can actually enforce, not what the docs promise. `doctor --host <host>` detects drift when a generated file was hand-edited; `rollback --host <host>` restores the last snapshot. Edit the harness source; regenerate the projections.

**Hooks.** Where a host supports lifecycle hooks, `.agent-hooks/` responds to agent events — routing on prompt submission, safety checks before tool calls, evidence capture after commands, completion-language review at turn end. Enforcement strength depends on the host's capability matrix; advisory instructions are not marketed as enforcement.

**Eval and proof.** `swe-seed eval run --spec <spec>` executes eval specs against the harness; `just ci` is the default local proof command — the same contract CI calls, so local and remote results stay aligned.

**Learning.** `swe-seed reflect <trace-id>` turns a finished trace into a proposed learning record; `learn` promotes reviewed records into skill proposals or regression cases. The harness does not silently rewrite its own rules because one session felt awkward — a proposal names evidence, risk, and rollback, and a human approves material changes.

**Contracts.** The data model for all layers is defined as `.baml` contracts under `.agent-harness/baml/baml_src/` and consumed as data. No language model is called at run time.

Three layers, each governed by the one above it:

```text
SweSeed      (SWE_SEED_SPEC_v0.2.0.md)    capability assembly, layer boundaries
  Harness    (HARNESS_SPEC.md)            routing, proof, context, hooks, traces, learning
    Fabricator (FABRICATOR_SPEC_v0.1.0.md)  bounded product-to-prototype runs
```

## What it is not

- **Not a coding agent.** SWE_SEED does not reason, edit, or run tools. The host agent does the work; SWE_SEED defines the terms and keeps the evidence.
- **Not an agent platform or orchestrator.** No dashboard, no server, no hosted control plane. A CLI and files in the repository.
- **Not a CI replacement.** Keep CI. SWE_SEED binds proof to the task before work starts and uses CI output as proof; CI remains the independent downstream check.
- **Not a settlement authority.** SWE_SEED produces the route, artifact, proof, and trace needed to judge whether work is complete. It does not decide that the outcome counts.
- **Not a runtime authority.** Host hooks can check risky tool use where the host supports it. Governing consequential side effects — commits, deploys, secrets — is a different problem, owned by a different layer.

## Where it fits in the GodSpeed stack

SWE_SEED is useful standalone: one repository, one supported agent, one routed task with proof is a complete deployment.

In the full stack, the boundaries are deliberate:

> DomainForge defines the domain. SEA Forge governs the work. SWE_SEED proves the change. GodSpeed-Agent compounds the capability.

- **Context Kernel** owns context authority — bounded, cited context packets. SWE_SEED requests and consumes context; it does not become a competing context source. (Federation support for this exists behind `swe-seed federation`.)
- **SEA Forge** owns pre-action authority and governed side effects. SWE_SEED emits work and proof events; it does not enforce runtime authority.
- **[godspeed_agent](https://github.com/GodSpeedAI/GodSpeed-Agent)** owns settlement classification and the capability lifecycle. SWE_SEED produces evidence; it consumes evidence and decides what the outcome counts for.

Integration is opt-in. Standalone SWE_SEED requires none of it.

## Current status

This is a draft reference implementation. The core harness contracts are expressed in specs, route cards, validation, and the CLI; expect the shape to evolve as real usage shows which constraints help and which add noise.

| Status | Capability |
| --- | --- |
| **Implemented** | Routing and route cards; context planning; trace lifecycle and JSON trace records; harness structure validation (`swe-seed harness`); host projection (`sync`), drift detection (`doctor --host`), and rollback for Claude, Codex, OpenCode, GitHub Copilot, Antigravity, and CI; hook runtime and exports (OTel, JUnit); eval specs (`eval run`); learning reflection and reviewed promotion; seed assembly, boundary validation, and provenance verification; trace-chain merge gate (`gate --verify`) |
| **Implemented, narrower than the vision** | Host enforcement strength varies by host capability matrix — check `swe-seed hosts` before assuming pre-action blocking; the learning store (rusql/vector retrieval) is opt-in tooling, not the default file-based path |
| **Experimental** | MCP gateway (`swe-seed gateway`, spec 0020); federation envelope exchange and SEA Forge verification (`swe-seed federation`, spec 0011) |
| **Architectural target** | Full Context Kernel integration; GodSpeed-Agent settlement handoff; cross-host behavioral parity beyond projection determinism |
| **Roadmap** | Vector retrieval for trace search (proposed only when files stop answering real recovery questions); broader host coverage |

The honest summary: the routing → trace → proof → gate loop is real and tested (337 tests in the workspace at last count). Host projection is real for the six hosts listed. Deeper stack integration is where the proof thins out — treat it as design direction, not shipped capability.

## Adopt it in your own project

Two paths:

1. **Start from this repository.** Clone it, keep the harness structure, and adapt the specs, route cards, skills, and checks to your project. Best when you want the working reference implementation and its validation.
2. **Rebuild from the specs.** Copy `HARNESS_SPEC.md`, `SWE_SEED_SPEC_v0.2.0.md`, `AGENTS.md`, and `.github/copilot-instructions.md` into a target project, then have your agent follow the specs to implement the harness there. Best when your repository already has its own structure and you want to port the operating model.

Either way, start small: one repository, one agent, one task type, one proof set. Prove failure behavior early — a missing route, a missing artifact, a failing proof — because the harness is credible when failure is explicit, not when success is easy.

## Technical reference

### Commands

Everyday commands (via `just`; the `swe-seed` CLI runs through `cargo run -q -p swe-seed --`):

```bash
just bootstrap             # install or synchronize local tools
just doctor                # report missing required or recommended tools
just ci                    # the local CI contract — the default proof command
just format / lint / test  # the individual CI stages

just harness-route "task"              # select a route and print the work contract
just harness-route-record "task"       # route and write the decision to the ledger
just harness-context-plan "task"       # plan bounded context for a task
just harness-validate                  # validate harness structure
just harness-doctor                    # doctor checks, including drift
just harness-render-skills             # render skills to host formats

just harness-trace-start "task"                              # open a trace
just harness-trace-append <id> "note"                        # append an event
just harness-trace-checkpoint <id> <stage> "summary" ["next"] # checkpoint for recovery
just harness-trace-resume <id>                               # resume from a trace
just harness-trace-finish <id> "claim" ["command" "result"]  # close with claim + proof
```

Direct CLI commands with no `just` wrapper (see `swe-seed --help` for the full surface):

```bash
swe-seed hosts                          # supported hosts and capability matrices
swe-seed sync --host <host> [--dry-run] # project harness state into a host
swe-seed rollback --host <host>         # restore the last projection snapshot
swe-seed gate <trace-id> --verify       # routing/chain verification; exits non-zero on failure
swe-seed eval run --spec <spec>         # run an eval spec
swe-seed reflect <trace-id>             # propose a learning record from a trace
swe-seed seed assemble                  # build the seed package manifest
swe-seed provenance verify              # fail closed on missing hash or license
```

### Repository layout

- `AGENTS.md` — the operating contract agents are expected to follow.
- `.agent-harness/routes/` — route cards mapping work types to context, work loops, artifacts, and proof.
- `.agent-harness/skills/`, `playbooks/`, `memory/` — reusable procedures, short workflows, and durable project context.
- `.agent-harness/traces/` — route decisions and trace records (JSON); the sqlite learning store is opt-in.
- `.agent-harness/baml/baml_src/` — the contract schemas, consumed as data.
- `.agent-harness/reflections/` — learning review packets, kept separate from active instructions.
- `.agent-hooks/` — the hook surface for agent lifecycle events.
- `crates/swe-seed`, `crates/swe-seed-core` — the CLI and the core library.
- `docs/specs/0001`–`0020` — the numbered design specifications; `0012` is authoritative on vocabulary conflicts.
- `SWE_SEED_SPEC_v0.2.0.md`, `HARNESS_SPEC.md`, `FABRICATOR_SPEC_v0.1.0.md` — the root layer contracts.

### Documentation map

- [Agent operating contract](AGENTS.md)
- [Harness specification](HARNESS_SPEC.md)
- [SWE_SEED layer specification](SWE_SEED_SPEC_v0.2.0.md)
- [Fabricator specification](FABRICATOR_SPEC_v0.1.0.md)
- [Dev harness guide](docs/dev-harness/README.md)
- [Agent harness spec index](docs/specs/agentic-swe-harness.md)

### Security and privacy

SWE_SEED stores its state in the repository and local files; it has no network surface of its own. It does not store secrets in traces, memory, examples, or rendered instructions — treat that as a convention the specs and validation enforce, not a guarantee against a determined agent writing to a trace. Federation keys, when used, are SOPS-encrypted at rest (see the `secrets-*` and `federation-encrypt-key` recipes). Host hooks can check tool calls before they run where the host supports it, but that is a host capability, not an authorization boundary — consequential actions need a real authority layer.

### License

No license file is included yet. Add one before publishing this project for reuse outside your organization.
