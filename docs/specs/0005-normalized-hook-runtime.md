# 0005 — Normalized Hook Runtime

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** The
> normalized event vocabulary stays, but gating is expressed via proven **HookPolicy** +
> **PermissionPolicy** (`CapabilityProfile` → PermissionPolicy), and the runtime ports the
> proven `agent_hooks` engine (JSONL log + SQLite index + redaction + OTEL/JUnit export +
> compaction). Context sourcing is **ContextBudget/ContextPack** (0015), not an ad-hoc
> ContextBuild. Types canonical in `.baml` ([0019](0019-baml-contracts-as-data.md)).

## Purpose

Define a host-neutral lifecycle/event model so a single `Hook` definition (0003) can be
projected to whichever hosts support the corresponding event, and degrade where they do
not. SWE_Seed normalizes divergent host hook systems into one vocabulary.

## Non-goals

- Not a re-implementation of any host's hook engine.
- Not the 50+ default hook set of multi-host runtime prior-art — SWE_Seed ships **zero** default hooks.
- v0.1 supports only five events; the rest are reserved names.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| multi-host runtime prior-art | `(prior-art path redacted)` | dir | Hook bundled with a loader, event-driven | Hooks are first-class capabilities tied to lifecycle events |
| multi-host runtime prior-art | `(prior-art path redacted)` | file | Host runtime constructs its hook set | Per-host hook projection; runtime owns wiring |
| multi-host runtime prior-art | `(prior-art path redacted)`, `injection-cache.ts` | files | Inject doctrine into context, cached | `ContextBuild` event semantics |
| gateway prior-art | `src/gateway/meta_mcp/invoke.rs` | file | Pre/post invocation handling around tool calls | `PreToolUse`/`PostToolUse` semantics (re-derived) |

> Note: multi-host runtime prior-art is noncommercial-licensed; only the *existence and shape* of its
> hook wiring was observed, no code read for reuse.

## SWE_Seed requirements

### Full normalized lifecycle (reserved vocabulary)

```
SessionStart ContextBuild MessageIn MessageOut
PreToolUse PostToolUse PreEdit PostEdit
PreShell PostShell TaskStart TaskEnd Error Recovery
```

### v0.1 required events (must be supported end-to-end)

```
SessionStart  ContextBuild  PreToolUse  PostToolUse  TaskEnd
```

All other events are accepted in the registry but marked `reserved` and not projected in
v0.1 (doctor warns if used).

### Per-event spec

Each event defines: **trigger / input payload / output payload / allowed side effects /
blocking / failure behavior / host support / logging / tests.**

#### SessionStart
- **Trigger**: a host session begins.
- **Input**: `{session_id, host_id, cwd, profile}`.
- **Output**: optional `{context_additions[]}`.
- **Side effects**: read-only env probe; may emit context.
- **Blocking**: non-blocking (best-effort); host may ignore.
- **Failure**: log + continue (never abort a session).
- **Host support**: Claude yes; OpenCode yes; Codex no (→ project as a startup note in
  AGENTS.md instead); CI maps to job start.
- **Logging**: one structured line per fire (`event, hook_id, status, ms`).
- **Tests**: fires once per session; failure does not abort.

#### ContextBuild
- **Trigger**: host assembles context (doctrine injection point).
- **Input**: `{session_id, existing_context_refs}`.
- **Output**: `{doctrine_fragments[], capability_summaries[]}`.
- **Side effects**: read-only; produces injected text only.
- **Federation (optional, 0011)**: with `federation.context = external|hybrid`, this event
  may emit `ContextRequired` and merge a consumed `ContextPacketCreated` from Context
  Kernel. With the flag off (default), context is built **fully locally** — fail-soft to
  local on any external timeout.
- **Blocking**: non-blocking; cached (injection-cache concept).
- **Failure**: log + skip injection (degraded, not fatal).
- **Host support**: Claude/OpenCode via AGENTS.md injection; Codex via static AGENTS.md;
  Copilot via instructions file.
- **Logging**: fragment count + cache hit/miss.
- **Tests**: deterministic fragment ordering; cache invalidation on registry change.

#### PreToolUse
- **Trigger**: before a tool/MCP invocation.
- **Input**: `{tool_name, args_digest, profile, capability_id?}`.
- **Output**: `{decision: allow|deny|warn, message?}`.
- **Side effects**: policy evaluation; may consult scan status / approval flags.
- **Federation (optional, 0011)**: with `federation.authority = delegate|hybrid`, the
  decision additionally consults an external `AuthorityChecked` (allow/deny/escalate) from
  SEA-Forge. With the flag off (default), the decision is **fully local** (profiles + scan
  gate). External `allow` never overrides a local `deny` unless explicitly opted in.
- **Blocking**: **blocking** when `decision=deny` (the gate use-case).
- **Failure**: fail-closed for `dangerous` profile, fail-open (warn) otherwise — explicit
  per `on_failure`.
- **Host support**: Claude yes; OpenCode subset; Codex no (gate enforced at gateway 0006
  instead); CI yes.
- **Logging**: decision + reason for every fire (audit).
- **Tests**: deny actually blocks; fail-closed honored for dangerous profile.

#### PostToolUse
- **Trigger**: after a tool/MCP invocation returns.
- **Input**: `{tool_name, status, duration_ms, result_digest}`.
- **Output**: optional `{annotations[]}`.
- **Side effects**: metrics, provenance updates; no mutation of result.
- **Blocking**: non-blocking.
- **Failure**: log + continue.
- **Host support**: Claude/OpenCode yes; Codex no; CI yes.
- **Logging**: status + duration.
- **Tests**: fires once per invocation incl. error path.

#### TaskEnd
- **Trigger**: a task/subagent run completes.
- **Input**: `{task_id, status, summary_digest}`.
- **Output**: optional `{report_ref}`.
- **Side effects**: write run summary to log; update doctor state.
- **Blocking**: non-blocking.
- **Failure**: log + continue.
- **Host support**: Claude/OpenCode yes; Codex/CI map to job end.
- **Logging**: terminal status.
- **Tests**: fires on success and failure paths.

### Host support mapping (summary)

| Event | Claude | Codex | OpenCode | Copilot/VSCode | CI |
|---|---|---|---|---|---|
| SessionStart | yes | static | yes | no | job-start |
| ContextBuild | yes | static | yes | instructions | job-start |
| PreToolUse | yes | gateway | subset | no | yes |
| PostToolUse | yes | no | yes | no | yes |
| TaskEnd | yes | job-end | yes | no | job-end |

"static" = realized as AGENTS.md content rather than a live hook. "gateway" = enforced in
the MCP gateway plane (0006), not as a host hook.

## CLI behavior, if applicable

`swe-seed hooks list`, `swe-seed hooks test <id> --host H` (dry-fire with synthetic
payload).

## Generated files, if applicable

Host-specific hook config via adapters (0004) — e.g. Claude `.claude/settings.json` hooks
region. Hooks never written to hosts that lack the event (degraded to static/gateway).

## Rust module boundaries

`swe_seed::hooks` with `mod event` (enum + reserved vs required), `mod runtime`
(dispatch), `mod payload`. Projection handled by `swe_seed::adapters`.

## Security and provenance considerations

`PreToolUse` is the in-host enforcement point for scan/approval gates; for hosts without
it, enforcement falls to the gateway (0006). Every blocking decision is logged for audit.

## Tests

- Required-event matrix covered; reserved events rejected from projection.
- Blocking `deny` verified to block; fail-closed verified for dangerous profile.
- Deterministic logging format (one line/event).

## Decisions

- **OpenCode hook subset**: implement the 5 required events; mark any event OpenCode cannot
  honor as `unsupported` (honest degradation). Precise mapping finalized during M4 — no
  guessing in the spec.
- **`Error`/`Recovery`**: **not** in v0.1. Keep the 5 required events; the full vocabulary
  stays reserved (reserved names cost nothing).

## Acceptance criteria

- [ ] Five required events fully specified with host mapping that degrades honestly.
- [ ] SWE_Seed ships zero default hooks.
- [ ] No host is assumed to support all events.
