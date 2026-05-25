# SWE SEED CI/CD Development Harness Specification v0.2.0

## Purpose

This specification defines a CI/CD development harness that gives humans and agents one stable way to install tools, run checks, manage secrets, debug failures, observe harness behavior, and prove work is ready.

The desired outcome is behavioral: fewer hidden commands, fewer environment surprises, less completion theater, and faster recovery when the harness or its adapters misbehave. A contributor should know what to run, what the result means, what evidence was captured, and which document to update when the harness changes.

## Core Principle

```text
Remote CI is not a separate truth.
The local just interface is the developer and agent API.
GitHub Actions should call the same commands wherever practical.
Observability is file-first, language-agnostic, and rebuildable.
```

## Required Command Interface

The project MUST provide a `justfile` with these recipes:

- `just bootstrap`: install or synchronize local tool dependencies.
- `just doctor`: report missing required or recommended tools.
- `just format`: run formatting checks.
- `just lint`: run static checks.
- `just test`: run harness and project tests.
- `just ci`: run the local mirror of remote CI.
- `just secrets-encrypt`: encrypt plaintext secret inputs.
- `just secrets-decrypt`: decrypt encrypted secrets into ignored local files.
- `just secrets-edit <file>`: edit a SOPS-managed secret.
- `just secrets-rotate-key`: rotate SOPS recipients or age keys.

`just ci` MUST be the proof command for ordinary completion claims.

## Required Scaffold

The harness MUST include:

- GitHub Actions workflow under `.github/workflows/ci.yml`.
- Local command scripts under `scripts/`.
- `.vscode/extensions.json` and `.vscode/settings.json`.
- `.editorconfig`.
- `.gitignore`.
- `.gitattributes`.
- `.env.example`.
- `.envrc`.
- `.mise.toml`.
- `devbox.json`.
- `package.json` and lockfile when Node-based tooling is used.
- `pyproject.toml` when Python tooling is used.
- SOPS and age configuration for secret handling.
- A validation script that checks required harness files and command contracts.

The scaffold SHOULD stay small. Add tools only when they remove a real ambiguity or make the proof path more reliable.

## CI Requirements

GitHub Actions MUST:

- Check out the repository.
- Install `just`.
- Install the declared toolchain.
- Restore dependency caches where the tool supports safe caching.
- Install dependencies from lockfiles.
- Run `just ci`.

The workflow MAY contain CI-specific setup, but it SHOULD NOT duplicate lint, test, or format logic already expressed through `just`.

## Observability and Logging Requirements

Observability is a core requirement of the dev harness. It exists to make setup failures, CI failures, hook adapter defects, and proof mismatches inspectable without binding the project to one language runtime or one storage backend.

The observability layer MUST remain agnostic to the agent harness. The two systems SHOULD work in harmony through shared identifiers and compatible payload capture, but the dev harness MUST NOT depend on agent-harness-specific Skill IR, memory layouts, or runtime libraries.

### Architectural Rule

The observability pipeline MUST follow this shape:

```text
agent hook or command wrapper
-> router or capture shim
-> normalized event
-> append-only JSONL event log
-> optional artifact files
-> optional rusql index
-> optional vector index
-> dashboards / replay / doctor / trace
```

The append-only JSONL log is the durable source of truth. Any rusql index, FTS layer, Tantivy-style search layer, or vector store MUST be rebuildable from files and MUST NOT become the only durable record.

### Storage Levels

The observability layer SHOULD evolve only when query pressure justifies it:

- Level 0: JSONL event files on disk.
- Level 1: Optional rusql index for structured filters.
- Level 2: Optional rusql FTS or Tantivy-style search for faster textual lookup.
- Level 3: Optional vector index for semantic retrieval over prompts, errors, summaries, or tool outputs.

Implementations MUST NOT start with a database as the only source of truth.

### Event Envelope Contract

Every captured hook execution, adapter invocation, or wrapped harness command MUST emit one normalized event envelope. The normalized envelope MUST be serializable as one JSON object per line and MUST include, at minimum:

- `schema_version`
- `event_id`
- `trace_id`
- `span_id`
- `parent_span_id`
- `timestamp`
- `agent`
- `agent_version`
- `native_event`
- `event`
- `session_id`
- `turn_id`
- `cwd`
- `repo_root`
- `profile`
- `hook_id`
- `script`
- `status`
- `duration_ms`
- `exit_code`
- `stdout_ref`
- `stderr_ref`
- `native_payload_ref`
- `normalized_payload_ref`
- `result_ref`

Large payloads, stdout, stderr, and tool-specific blobs SHOULD be persisted as separate artifact files referenced by the envelope rather than inlined into the JSONL record.

### File Layout

If observability is enabled, the preferred layout is:

```text
.agent-hooks/
  config.yaml
  logs/
    events-YYYY-MM-DD.jsonl
  payloads/
    YYYY-MM-DD/
      <event_id>.native.json
      <event_id>.normalized.json
      <event_id>.result.json
  artifacts/
    YYYY-MM-DD/
      <event_id>.stdout.txt
      <event_id>.stderr.txt
  index/
    hooks.rusql
    vectors/
```

Equivalent paths are allowed, but the implementation MUST document them and MUST preserve the separation between append-only events, large artifacts, and derived indexes.

### Behavioral Requirements

The observability implementation MUST support:

- redaction before persistence,
- log rotation by date and size,
- replay from captured payloads and artifacts,
- inspection of one event without loading the whole log,
- filesystem-first backup and recovery,
- JSONL export,
- OpenTelemetry-compatible JSON export,
- JUnit-style report export,
- compatibility with scripts written in shell, Python, Rust, TypeScript, Node, Deno, Bun, or any tool that can read stdin and write stdout or stderr.

The implementation MUST NOT require project scripts or adapters to import a logging SDK. Captured programs SHOULD communicate through stdin, stdout, stderr, exit codes, and filesystem artifacts.

### Command Surface

A rebuild of the dev harness MUST expose an observability CLI surface. The preferred namespace is `agent-hooks`, though equivalent wrappers MAY be provided through `just`.

Minimum command set:

- `agent-hooks trace --last`
- `agent-hooks trace --session <id>`
- `agent-hooks replay --event <event_id>`
- `agent-hooks inspect --event <event_id>`
- `agent-hooks doctor --observability`
- `agent-hooks compact-logs`
- `agent-hooks index rebuild`
- `agent-hooks export otel`
- `agent-hooks export junit`

`replay` is the most important recovery feature. The observability layer SHOULD make adapter debugging possible from captured payloads without requiring a live rerun.

### Cross-Harness Boundary

The dev harness and the agent harness serve different layers and MUST remain separable.

The dev harness owns:

- command execution surfaces,
- local and remote CI parity,
- setup and tool diagnostics,
- secrets workflows,
- adapter and wrapper observability,
- durable event logging and derived observability indexes.

The dev harness MUST NOT depend on agent-harness route cards, Skill IR, memory files, or learning-review packet formats to function.

The dev harness SHOULD interoperate with the agent harness through shared identifiers and artifact references only. Preferred shared identifiers are `trace_id`, `session_id`, `span_id`, `event_id`, repository root, and stable artifact paths.

### Outcome-Bearing Implementation Order

An implementation that aims to maximize productive outcomes SHOULD build the dev harness in this order:

1. Command parity and proof. `just` remains the local and CI command API, and `doctor`, `bootstrap`, and `ci` provide one clear way to set up, diagnose, and prove work.

1. File-first observability. Command wrappers and hooks emit normalized JSONL events plus artifact references, and `trace`, `inspect`, and `replay` work on captured events before any index or dashboard exists.

1. Optional acceleration layers. rusql or equivalent indexes are rebuildable from JSONL, OTEL and JUnit exports remain derived outputs, and vector retrieval appears only after exact and structured lookup demonstrably fail real recovery tasks.

The implementation SHOULD NOT start with dashboards, vector retrieval, or agent-specific adapters. Those layers are only useful after the command and replay path are already reliable.

### Upgrade Triggers

The implementation SHOULD remain file-only while all of the following are true:

- fewer than 50,000 events exist for the repository,
- most queries are by date, session, or recent failure,
- `grep`, `jq`, or simple trace tooling remain sufficient,
- no always-on dashboard requires indexed lookup.

The implementation SHOULD add a rusql index when structured filters by agent, session, hook, or status become frequent, when `doctor` and `trace` need fast lookup, or when multiple agents produce concurrent events.

The implementation SHOULD add vector search only when semantic retrieval over prompts, errors, summaries, or tool outputs clearly outperforms exact and structured search for real recovery tasks such as finding similar failed runs.

## Dev Harness Documentation

The project MUST include concise operational documentation under `docs/dev-harness/`.

The documentation MUST explain how to use, maintain, and safely change the harness. It SHOULD NOT restate obvious command names unless the command contract matters. It MUST document intent, non-obvious behavior, required workflows, failure recovery, and references needed to make correct changes.

### Documentation Structure

`docs/dev-harness/README.md`
: Entry point. Explain what the harness is, what problem it solves, and the command contract: local `just` commands are the source of truth, and CI calls them where practical.

`docs/dev-harness/explanations/`
: Conceptual documentation for non-obvious design decisions.

Examples:

- `local-ci-parity.md`
- `observability-model.md`
- `toolchain-boundaries.md`
- `secrets-model.md`
- `agent-proof-commands.md`

`docs/dev-harness/howto/`
: Task-focused guides for work a developer or agent may need to perform.

Each how-to filename MUST use a verb phrase.

Examples:

- `initialize-dev-env.md`
- `run-local-ci.md`
- `add-a-ci-check.md`
- `change-node-version.md`
- `rotate-secrets-key.md`
- `debug-failing-ci.md`

`docs/dev-harness/references/`
: Stable reference material for configuration, schemas, command contracts, and external docs.

Examples:

- `just-recipes.md`
- `observability-contract.md`
- `github-actions.md`
- `mise.md`
- `devbox.md`
- `sops-age.md`
- `proof-command-map.md`

### Documentation Rules

How-to documents MUST include:

- Purpose.
- Prerequisites.
- Steps.
- Verification command.
- Common failure modes, when useful.

Explanation documents MUST include:

- Why the design exists.
- What tradeoff it makes.
- What should not be changed casually.

Reference documents MUST include:

- Relevant config files.
- Important fields or schema links.
- Ownership boundaries.
- Commands that prove the config still works.

### Required Initial Documents

The scaffold MUST generate at least:

- `docs/dev-harness/README.md`
- `docs/dev-harness/howto/initialize-dev-env.md`
- `docs/dev-harness/howto/run-local-ci.md`
- `docs/dev-harness/howto/add-a-ci-check.md`
- `docs/dev-harness/howto/debug-failing-ci.md`
- `docs/dev-harness/explanations/local-ci-parity.md`
- `docs/dev-harness/explanations/observability-model.md`
- `docs/dev-harness/explanations/secrets-model.md`
- `docs/dev-harness/references/just-recipes.md`
- `docs/dev-harness/references/observability-contract.md`
- `docs/dev-harness/references/proof-command-map.md`

`just ci` MUST verify that these required documentation files exist.

When a harness command, CI job, tool version, or secrets workflow changes, the matching documentation MUST be updated in the same change.

## Secrets Requirements

The harness MUST support SOPS with age.

Secret handling MUST follow these rules:

- Plaintext secret outputs MUST be ignored by git.
- Local age keys MUST be ignored by git.
- SOPS configuration MUST make intended encrypted paths explicit.
- Secret commands MUST fail clearly when `sops` or required key material is missing.

The spec MUST NOT require real project secrets in the scaffold.

## Proof Command Map

The documentation MUST include a proof-command map that tells humans and agents what command proves each kind of claim.

Minimum map:

| Claim                        | Proof command                    |
| ---------------------------- | -------------------------------- |
| Harness files exist          | `bash tests/validate-harness.sh` |
| Local CI passes              | `just ci`                        |
| Tooling is installed         | `just doctor`                    |
| Formatting is clean          | `just format`                    |
| Static checks pass           | `just lint`                      |
| Tests pass                   | `just test`                      |
| Secret workflow is available | `just secrets-edit <file>`       |

## Validation Requirements

The harness MUST include automated validation for:

- Required files.
- Required `just` recipes.
- CI calling `just ci`.
- Documentation files required by this spec.
- Git ignore rules for local secrets and generated cache files.

When an observability implementation is added, validation MUST also check the event log schema, the replay surface, and the rebuildability of any derived index.

Validation SHOULD treat these observability behaviors as outcome-bearing:

- capture writes a normalized event and referenced payload or artifact files,
- trace surfaces recent and session-scoped events,
- inspect resolves one event into a readable recovery surface,
- replay rehydrates captured payloads without a live rerun,
- doctor reports layout health and index state,
- index rebuild proves derived storage is reproducible from files.

Validation SHOULD check contracts, not incidental formatting.

## Implementation Learning Loop

The harness MUST encode lessons that reduce future implementation friction. These lessons are generalizable and should be applied without adding ceremony:

- Make the command contract executable. If a required file, recipe, or doc matters, validate it.
- Separate facts by use. Put tasks in `howto/`, rationale in `explanations/`, and stable contracts in `references/`.
- Prefer one source of truth. CI should call local commands instead of reimplementing them.
- Make observability replayable. Keep JSONL as the durable event ledger and treat indexes as derived caches.
- Keep optional tools optional in local diagnostics, but make required proof commands fail clearly.
- Use lockfiles when a command depends on package-installed tools.
- Avoid placeholder success. A completion claim needs a fresh proof command and observed output.
- Fix discovered ambiguity in the spec, not only in the scaffold.

These rules are not an invitation to expand the harness. Add structure only when it improves correctness, repeatability, or recovery from failure.

## Implementation Checklist

- Create the required scaffold files.
- Create the required documentation files.
- Add validation for the scaffold and documentation.
- Define the observability contract and required docs before implementing adapters or indexes.
- Implement command parity before adding convenience wrappers, dashboards, or remote-only logic.
- Implement capture, inspect, and replay before adding derived indexes or semantic retrieval.
- Keep the dev harness agent-agnostic by sharing identifiers and artifacts rather than importing the agent harness runtime.
- Run the validation script and observe failure before creating missing required outputs.
- Run `just ci` after implementation.
- Update this spec when implementation reveals a reusable requirement or ambiguity.
