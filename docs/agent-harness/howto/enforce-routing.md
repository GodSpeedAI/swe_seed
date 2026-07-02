# Enforce Routing

> Design: [Federation Signing and Tamper-Evident Traces](../explanations/federation-signing-and-traces.md).
> Why routing is mandatory: [`AGENTS.md`](../../../AGENTS.md) → "Routing Is Mandatory".

`AGENTS.md` tells a cooperative agent to route; the routing gate makes
bypassing **detectable and blockable**. The chain records that routing happened
(evidence); the gate refuses to proceed when it didn't (teeth).

## The lifecycle, end to end

```bash
# 1. Route the task — this appends the RouteSelected genesis to the chain.
swe-seed trace start "fix the login bug"
# → { "trace_id": "20260630T...-fix-the-login-bug", ... }

# 2. Make the trace active for the session (the gate + host hook read this).
export SWE_SEED_TRACE=20260630T...-fix-the-login-bug
```

From this point, every recorded event (`trace append`, `checkpoint`, `finish`)
chains off the genesis, and the gate treats the trace as routed.

## The gate

```bash
swe-seed gate "$SWE_SEED_TRACE"             # 0 = routed, 1 = never routed (fail-closed)
swe-seed gate "$SWE_SEED_TRACE" --verify    # also recomputes the full chain (CI merge gate)
```

`--verify` detects a tampered payload or a severed link in the ledger, not just
a missing genesis. Use it for the merge boundary; use the plain gate for the
fast in-flight check.

## Host hook (PreToolUse enforcement)

`swe-seed agent-hooks route-gate` is the host hook command:

```bash
swe-seed agent-hooks route-gate                         # reads SWE_SEED_TRACE
swe-seed agent-hooks route-gate --trace-id <id>         # explicit trace
```

It evaluates the gate, logs a `PreToolUse` event (allowed/blocked) to the hook
log, and exits 0 (allow) / 1 (block). **When `SWE_SEED_TRACE` is unset it
allows** — so a host running this on every tool call is not blocked outside a
traced session.

The Phase-9 host adapters **project this as the PreToolUse hook** in generated
host config (claude, codex, antigravity, opencode):

```bash
swe-seed sync --host claude            # writes the projected config incl. the route-gate PreToolUse hook
```

Once a host loads that config, every tool call runs the gate: routed trace →
allow; unrouted active trace → block.

## CI merge gate

```bash
export SWE_SEED_TRACE=<the merge's trace id>
just gate-merge     # exits non-zero if unrouted or the chain fails verification
```

Wire `gate-merge` into the merge/PR pipeline to reject work whose trace was
never routed or whose chain has been altered.

## What "blocked" means

- Outside a traced session (`SWE_SEED_TRACE` unset): **allow** — normal tool use.
- Active trace that was routed: **allow**.
- Active trace that was **never routed**: **block** (exit 1) — the agent skipped
  `trace start`/routing, and the gate refuses to proceed.

A bypassing agent cannot fabricate a passing gate without appending a
`RouteSelected` genesis to the chain — and that genesis is the very thing the
gate requires. Signing the chain root (see
[Manage Federation Keys](manage-federation-keys.md)) lets a third party (SEA-Forge
or CI) independently confirm the chain was produced by SWE_Seed and not altered.
