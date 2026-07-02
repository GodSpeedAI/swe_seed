# Federation Signing and Tamper-Evident Traces

> Governing spec: [`docs/specs/0011-sea-loop-federation.md`](../../specs/0011-sea-loop-federation.md).
> SEA-Forge consumer: `docs/specs/domains/agentic_capability_loop/signing.md` (in the SEA repo).

Routing is recorded in a tamper-evident chain, the chain root is signed with
Ed25519, and SEA-Forge verifies both. This is how SWE_Seed proves that a task
was routed, that its trace was not altered, and that the envelope genuinely
came from SWE_Seed. It is also how routing becomes *enforced*, not merely
advised: the chain is the evidence, and the gate is the teeth.

## Why this exists

`AGENTS.md` makes a cooperative agent route voluntarily, but an agent can ignore
it — prose is not compulsion. A bypassing agent cannot, however, fabricate a
valid signed chain without SWE_Seed's private key. So the harness:

1. **Records** every route decision and trace event in an append-only hash chain
   (tamper-evident evidence).
2. **Signs** the chain root so SEA-Forge (or CI) can verify provenance.
3. **Enforces** at the host: the projected PreToolUse hook blocks tool use on an
   active trace that was never routed.

Shape at the front, gate at the back, audit in the middle.

## The tamper-evident ledger

Every trace's events live in a hash-chained append-only log in SQLite
(`.agent-harness/traces/ledger.sqlite3`, gitignored):

```
entry_n.hash = SHA256(entry_n.payload_json ‖ entry_{n-1}.hash)
```

- The **genesis entry (seq 0) is the `RouteSelected` decision** — so the fact
  that a task was routed is the first thing in the chain, by construction.
- `trace start` appends the genesis; `trace append` / `checkpoint` / `finish`
  append subsequent events.
- `chain_root(trace_id)` is the head hash — a single value that covers the whole
  chain.
- `verify_chain(trace_id)` recomputes every hash from stored payloads and
  detects a tampered payload or a severed link.

**Honest limit:** a hash chain in a database the writer controls is
tamper-*evident* against partial edits, not tamper-*resistant* against a full
rewrite by the writer. Full resistance arrives only when the chain root is
**anchored outside the writer's control** — which is exactly what the signature
+ SEA-Forge verification provide.

## Ed25519 signing (mutual)

Each party has its own Ed25519 keypair:

| Party | Private key | Public key |
|---|---|---|
| SWE_Seed | signs route/trace chain roots; lives in gitignored `.swe-seed/` (SOPS/age at rest) | committed in `.agent-harness/federation/keys/`; shared with SEA |
| SEA-Forge | signs `AuthorityChecked`; lives in SEA (SOPS/age at rest) | shared with SWE_Seed |

SWE_Seed **signs the chain root** (one signature covers the whole trace via the
hash chain). SEA-Forge verifies the signature (provenance: SWE_Seed produced
this root) **and** recomputes the chain to the root (integrity: no entry was
altered). The signature is attached to the envelope payload as
`signature: {algorithm: "ed25519", key_id, value}` alongside `trace_chain_root`.

**The key never crosses the trust boundary in the wrong direction.** SWE_Seed's
private key stays in SWE_Seed; SEA holds only SWE_Seed's public key — otherwise
SEA could forge the signatures it is verifying and the check would be vacuous.

## The canonical signing string (the cross-repo contract)

Ed25519 signs bytes; both the Rust signer (SWE_Seed) and the Python verifier
(SEA-Forge) must produce **byte-identical preimages**. The canonical signing
string is:

```
{namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}
```

- `compact_sorted_payload_json` is the payload with **sorted keys** and **no
  whitespace**, with the `signature` field removed.
- `event_id` is **excluded** — it is an envelope identifier, not content.
- Ed25519 signs these UTF-8 bytes directly (no pre-hash).

This is pinned by a **committed test vector** that both repos must reproduce:
`tests/fixtures/federation-sign-vector.json` (SWE_Seed direction) and
`tests/fixtures/sea-authority-vector.json` (SEA direction). These vectors are
the executable parity contract — drift fails CI on whichever side breaks it.

## Routing enforcement (the gate)

- `swe-seed gate <trace>` exits 0 if the trace has a `RouteSelected` genesis, 1
  otherwise (fail-closed: no ledger or no genesis → block).
- `swe-seed gate <trace> --verify` also recomputes the full chain (CI merge gate).
- `swe-seed agent-hooks route-gate` is the host hook: it reads the active trace
  from `SWE_SEED_TRACE`, evaluates the gate, logs a `PreToolUse` event, and
  exits 0 (allow) / 1 (block). **When no trace is active it allows** — so a host
  running it on every tool call is not blocked outside a traced session.

## Host projection

The Phase-9 host adapters project the **enforcing** `route-gate` as the
PreToolUse hook (claude, codex, antigravity, opencode). Every other lifecycle
event projects the logging `capture` command. So once a host loads the
projected config and `SWE_SEED_TRACE` is set (after `trace start`), every tool
call runs through the gate; an agent that bypassed routing is blocked at the
host.

## What is proven vs operational

- **Proven:** the ledger is tamper-evident; signatures verify cross-repo (Rust ↔
  Python) in both directions; the gate blocks unrouted traces; host adapters
  project the gate.
- **Operational (yours to set):** replace the placeholder age recipient in
  `.sops.yaml` with your real key; exchange public keys between SWE_Seed and SEA.

## See also

- [Manage federation keys](../howto/manage-federation-keys.md) — keygen, SOPS/age, sign, verify.
- [Enforce routing](../howto/enforce-routing.md) — traces, the gate, host hooks, CI.
- [Hooks, Memory, and Learning](hooks-memory-and-learning.md) — the lifecycle the gate plugs into.
