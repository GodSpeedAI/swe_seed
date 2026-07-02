# 0006 — MCP Gateway Integration

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** Profile
> gating is **PermissionPolicy** (`CapabilityProfile` → PermissionPolicy); the context plane is
> **ContextBudget/ContextPack** + `budget-policy.yaml` (0015). This MCP/tool plane sits in the
> Harness layer of the `SweSeed → Harness → Fabricator` stack.

## Purpose

Define SWE_Seed's capability/tool plane: a single gateway endpoint that fronts many backend
MCP servers, exposes a compact discovery surface instead of dumping all tool schemas into
context, and routes invocations under capability profiles and approval gates.

## Non-goals

- SWE_Seed is **not** an LLM/chat-completions gateway (mirrors gateway prior-art's own
  `ARCHITECTURE.md` Routing Boundary disclaimer).
- v0.1 does **not** reimplement gateway prior-art. It may either (a) embed a minimal SWE_Seed
  gateway, or (b) generate config that points hosts at an external MCP gateway. Proposed
  v0.1: **generate host MCP config + a minimal local router**; full embedded gateway is a
  later milestone.
- No EE security modules (firewall, tool_integrity, message_signing, etc.) are reproduced.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `ARCHITECTURE.md` | System Overview + Meta-Tools | `/mcp` endpoint, `BackendRegistry` (stdio/http/capability), discover-then-invoke meta-tools, ~95% token savings | One SWE_Seed endpoint → backend MCPs; compact discovery surface |
| gateway prior-art | `ARCHITECTURE.md` | Meta-Tools table | compact discovery meta-tools (list-servers / list-tools / search / invoke / stats) | SWE_Seed base meta-tools (renamed) |
| gateway prior-art | `src/discovery/config_scanner.rs`, `process_scanner.rs` | files | Discover backends from existing client configs + running processes | `swe-seed gateway import` |
| gateway prior-art | `src/projection/{mode.rs,role.rs}` | files | Modes/roles shape exposed surface | Capability profiles `readonly/coding/dangerous` |
| gateway prior-art | `src/commands/doctor.rs:182-253` | lines | Backend env + HTTP/stdio reachability checks | Gateway doctor checks (0008) |
| gateway prior-art | `src/commands/config_export/` | dir | Export config to client formats | Host MCP config generation (0004) |

## SWE_Seed requirements

Topology:

```
AI client → one SWE_Seed gateway endpoint → backend MCP servers → tools/resources/prompts
```

v0.1 gateway must support:

1. **Register backend MCP servers** from the registry (`MCPServer`, 0003): stdio + http.
2. **Compact discovery surface** — base meta-tools (SWE_Seed-named):
   - `swe_seed_list_servers` — backends + status/health.
   - `swe_seed_list_tools` — tools for one/all backends (cached).
   - `swe_seed_search_tools` — keyword/ranked search across backends.
   - `swe_seed_invoke` — invoke a tool on a backend; applies profile + approval gate.
3. **Route invocation** to the correct backend; preserve MCP semantics; relay
   server→client requests (sampling/elicitation/roots) transparently.
4. **Host-specific MCP config generation** via adapters (0004): `.vscode/mcp.json`,
   `~/.codex/config.toml`, Claude settings, etc.
5. **Capability profiles**: `readonly` (no mutating tools), `coding` (default working
   set), `dangerous` (destructive/network tools, requires explicit approval).
6. **Approval flags** for risky tool groups: a tool tagged `dangerous` requires
   `approval=explicit` before `swe_seed_invoke` will route it.
7. **Doctor checks** for backend availability (reachable, env set) — see 0008.
8. **Provenance records** for every registered MCP (source + hash + license tag, 0009).

### Profile model

```toml
[profile.readonly]
allow_tool_tags = ["read", "search"]
deny_tool_tags  = ["write", "exec", "network"]

[profile.coding]
allow_tool_tags = ["read", "search", "write"]
require_approval_tags = ["exec"]

[profile.dangerous]
allow_tool_tags = ["*"]
require_approval_tags = ["exec", "network", "delete"]
```

### Invocation gate (pseudocode)

```
on swe_seed_invoke(tool, args):
    p = active_profile
    if tool.tags ∩ p.deny_tool_tags: return Denied
    if tool.tags ∩ p.require_approval_tags and not approved(tool): return NeedsApproval
    record_provenance_use(tool)
    route_to_backend(tool, args)
```

## CLI behavior, if applicable

`swe-seed gateway register <mcp-id>`, `swe-seed gateway import` (scan existing configs),
`swe-seed gateway list`, `swe-seed gateway export --host H`, `swe-seed gateway doctor`.

## Generated files, if applicable

Host MCP config files (per 0004). If embedding a local router: `.swe-seed/gateway.toml`
listing backends + profile bindings.

## Rust module boundaries

`swe_seed::gateway` with `mod registry` (backends), `mod meta` (discovery surface),
`mod route` (invoke + relay), `mod profile`, `mod import`. **No `security/firewall`,
`tool_integrity`, `message_signing`, `cost_accounting`, `key_server`, `transparency_log`
modules** — those mirror EE files and are out-of-bounds (0009).

## Security and provenance considerations

- **Authority is local by default.** The profile/approval gate here is the sole authority
  when `federation.authority = local` (default). With `delegate|hybrid` (0011), invocations
  of risk-tagged tools additionally consult an external `AuthorityChecked` from SEA-Forge —
  but the gateway still functions fully standalone with federation off.
- Approval gate is mandatory for risky tool groups; dangerous profile fails closed.
- Each backend has a `ProvenanceRecord`; unverified backends cannot be registered under a
  `require_scan` profile.
- Tool-poisoning/integrity is a **goal** addressed via source hashing + scan gate (0007),
  derived from public MCP/OWASP material, not EE internals.

## Tests

- Discovery surface lists tools without loading full schemas into the client.
- Profile gate: readonly denies write tools; dangerous requires approval for exec.
- Routing: stdio + http backend invocation round-trips.
- Import: scanning an existing client config discovers its backends.

## Decisions

- **Config-generation first**, minimal router second. v0.1 generates host MCP config
  pointing at backends; an embedded routing engine is deferred until that proves
  insufficient. Delivers value without a long-lived router to maintain.
- **`search_tools` ranking**: simple keyword/substring scoring in v0.1. No embeddings or
  semantic ranking until there is measured need.

## Acceptance criteria

- [ ] Four base meta-tools specified and profile-gated.
- [ ] Three profiles (`readonly/coding/dangerous`) with approval semantics.
- [ ] Every backend carries provenance; no EE-derived security module is planned.
