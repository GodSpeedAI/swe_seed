# Reference Sources

> **HUMAN-ONLY LEGAL RECORD — NOT A BUILD INPUT.** This file names the (now-deleted)
> reference repositories for clean-room/provenance audit. Implementing agents must build
> SWE_Seed from `.agents/specs/` only and must **not** fetch, clone, or browse any project
> named here. The names exist solely for human legal review.

Evidence inventory for the two read-only reference repositories studied to design
SWE_Seed's centralization layer. These repos are **pattern sources only**. No code
was copied, vendored, or adopted as a dependency.

## reference/mcp-gateway

| Field | Value |
|---|---|
| Repo name | mcp-gateway |
| Local path | `reference/mcp-gateway` |
| Upstream | <https://github.com/MikkoParkkola/mcp-gateway> |
| Commit SHA | Not an independent git repo — vendored into SWE_Seed working tree. `git -C reference/mcp-gateway rev-parse HEAD` returns the **parent** SWE_Seed SHA `2d5b4c1732ab51bafa7ac83983ec0d5bb858fb5c`. No nested `.git`. **Open question: record true upstream SHA before relying on line citations.** |
| Declared version | `2.19.0` (`Cargo.toml:3`); ARCHITECTURE.md header references `v2.7.0` (stale) |
| Language | Rust, edition 2024, `rust-version = 1.95` (`Cargo.toml:4-5`) |
| Date inspected | 2026-06-26 |
| License files inspected | `LICENSE` (MIT, "Copyright (c) 2025 Mikko Parkkola"); `LICENSE-EE.md` (PolyForm Noncommercial 1.0.0, EE files) |
| Docs inspected | `ARCHITECTURE.md`, `docs/SECURITY_AUDIT.md` (name only), `docs/OWASP_AGENTIC_AI_COMPLIANCE.md` (name only), `Cargo.toml` |
| Source dirs inspected | `src/capability/`, `src/gateway/meta_mcp/`, `src/discovery/`, `src/config_reload/`, `src/projection/`, `src/commands/` (incl. `doctor.rs`, `config_export/`), `src/skills/`, `src/backend/` (names) |

**License posture:** Dual-licensed. Core = MIT (permissive). EE files marked with SPDX
`PolyForm-Noncommercial-1.0.0` are commercial-restricted. EE coverage (per
`LICENSE-EE.md`): `src/security/firewall/`, `src/security/agent_identity.rs`,
`data_flow.rs`, `message_signing.rs`, `policy.rs`, `response_inspect.rs`,
`response_scanner.rs`, `scope_collision.rs`, `tool_integrity.rs`, `src/cost_accounting/`,
`src/key_server/`, `src/transparency_log/`. **SWE_Seed must not read or pattern-extract
from EE files** (see `clean-room-boundaries.md`).

## reference/oh-my-openagent

| Field | Value |
|---|---|
| Repo name | oh-my-openagent (package id: `oh-my-opencode`) |
| Local path | `reference/oh-my-openagent` |
| Commit SHA | Not an independent git repo — same situation as above; returns parent SHA `2d5b4c1`. **Open question: record true upstream SHA.** |
| Language | TypeScript (Bun), monorepo of `packages/*` |
| Date inspected | 2026-06-26 |
| License files inspected | `LICENSE.md` — **Sustainable Use License v1.0** (n8n-style source-available, noncommercial/internal-use only) |
| Docs inspected | `CLAUDE.md`, `docs/AGENTS.md` (name), `docs/manifesto.md` (name only — NOT extracted, persona/branding) |
| Packages inspected (names + selected `src/` listings) | `claude-code-compat-core` (loaders), `skills-loader-core` (hooks/types/features), `rules-engine`, `agents-md-core`, `mcp-client-core`, `omo-codex`, `omo-opencode` |

**License posture:** Source-available but **restrictive**. The Sustainable Use License
limits use to "internal business purposes or non-commercial or personal use," prohibits
removing notices, and is **not OSI-approved / not a permissive license**. Copying any
portion into SWE_Seed (which may be distributed commercially) is **prohibited**. Treat as
**architecture observation only**. Mark commercial-distribution status of SWE_Seed as a
required human-review point.

## Status: removed

Both reference directories are **deleted from the working tree** after this analysis
(2026-06-26). SWE_Seed is built solely from `.agents/specs/`. This file is a frozen record
of what was inspected; it does not require the repos to exist. To re-pin true upstream
commit SHAs (a human action), recover them from the upstream remotes/history listed above —
not from the deleted local copies.

## Notes

- Neither reference directory is a real nested git checkout in this working tree, so
  per-line citations are pinned to file content as inspected on 2026-06-26, not to an
  upstream commit. A human should re-pin against upstream SHAs before these specs are
  used as legal/provenance evidence.
- `docs/manifesto.md`, persona/agent names, README marketing copy, and the EE security
  module internals were deliberately **not** opened for extraction.
