# 0009 — License and Clean-Room / Provenance Boundaries

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** The
> runtime provenance carrier is **ArtifactMetadata** + **SourceRef**/**TraceabilityLink** (with
> `SeedArtifactStatus`/`ReviewRequirement`); `ProvenanceRecord` maps onto it. Keep the hashing +
> clean-room rules as written. Boundary/provenance validation runs in [0018](0018-layer-boundary-governance.md).

## Purpose

Record the license situation of both reference repos, define the clean-room boundary rules
SWE_Seed development must follow, and specify the runtime `ProvenanceRecord` + traceability
formats that keep SWE_Seed free of derivative-code risk. Conservative by mandate.

## Non-goals

- Not legal advice; the human-review points here gate any commercial use.
- Not a license for either reference repo's code.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `LICENSE` | head | MIT, "Copyright (c) 2025 (author redacted)" | Core ideas extractable; no code copied |
| gateway prior-art | `LICENSE-EE.md` | full | PolyForm Noncommercial 1.0.0 for listed EE files; commercial use needs separate license | EE files **out-of-bounds**, not even opened |
| gateway prior-art | `Cargo.toml:8` | line | `license = "MIT"` (crate metadata) | Crate-level MIT; EE carve-out is file-level SPDX |
| multi-host runtime prior-art | `LICENSE.md` | full | "Sustainable Use License v1.0" — internal/noncommercial only, keep notices, source-available | **No code reuse**; architecture observation only |

### License determinations

- **gateway prior-art core**: MIT (permissive). Patterns/ideas may inform SWE_Seed; no code
  copied. **Status: clear.**
- **gateway prior-art EE files**: PolyForm Noncommercial. Coverage list (from `LICENSE-EE.md`):
  `src/security/firewall/`, `src/security/agent_identity.rs`, `data_flow.rs`,
  `message_signing.rs`, `policy.rs`, `response_inspect.rs`, `response_scanner.rs`,
  `scope_collision.rs`, `tool_integrity.rs`, `src/cost_accounting/`, `src/key_server/`,
  `src/transparency_log/`. **Status: restricted — do not inspect or pattern-extract.**
- **multi-host runtime prior-art**: Sustainable Use License (noncommercial, source-available, not
  OSI/permissive). If SWE_Seed is distributed commercially, reuse is prohibited.
  **Status: AMBIGUOUS for SWE_Seed's intended distribution → human review required.**
- **Dual-licensed / restricted?** gateway prior-art is genuinely dual-licensed (MIT + EE).
  multi-host runtime prior-art is single but restrictive. Both require care.

## SWE_Seed requirements

### Clean-room boundary rules
1. Extract ideas/contracts/decompositions only; never expression (see
   `docs/specs/clean-room-boundaries.md`).
2. EE files are never opened for extraction; SWE_Seed security derives from public
   OWASP/MCP-spec material.
3. No reference repo is vendored, submoduled, or made a dependency in shipped artifacts.
4. Every extracted pattern has a traceability row with `Code copied? = No`.

### Prohibited copying rules
- No verbatim/near-verbatim code, config schemas, test fixtures, prose, branding, or
  persona/agent names from either repo.
- No line-by-line transliteration into Rust.

### Acceptable pattern extraction rules
- Architecture shapes, lifecycle orders, status enums (re-expressed), profile *concepts*,
  module decompositions cited as evidence — all permitted with a traceability row.

### ProvenanceRecord format (runtime)

```json
{
  "capability_id": "serena",
  "source_id": "oraios-serena",
  "source_kind": "git",
  "source_location": "https://github.com/oraios/serena",
  "source_ref": "v1.2.3",
  "source_hash": "sha256:…",
  "license_tag": "MIT",
  "license_status": "clear|ambiguous|restricted",
  "inspected_at": "2026-06-26",
  "scan_ref": ".swe-seed/scans/serena/abc.json",
  "code_copied": false,
  "notes": ""
}
```

Stored at `.swe-seed/provenance/<capability-id>.json`. `code_copied` must be `false`;
`license_status` other than `clear` requires a recorded human approval before activation.

### Traceability matrix format (design-time)

Per the committed traceability matrix under `docs/specs/provenance/`:

```
| Source repo | Source path | Pattern observed | SWE_Seed requirement | Code copied? | License concern? | Notes |
```

Every row: `Code copied? = No`.

### Implementation checklist to avoid derivative-code risk
- [ ] No SWE_Seed file pasted from a reference repo (diff review).
- [ ] No SWE_Seed module reproduces an EE file's logic (cross-check EE list).
- [ ] No reference branding/persona/agent names in SWE_Seed.
- [ ] Every capability ships a complete `ProvenanceRecord`.
- [ ] Traceability matrix has zero `Code copied? = Yes` rows.
- [ ] Legal review of SWE_Seed distribution vs. multi-host runtime prior-art noncommercial terms.
- [ ] Upstream commit SHAs re-pinned for both repos.

## CLI behavior, if applicable

`swe-seed provenance list`, `swe-seed provenance verify` (asserts every active capability
has a complete record + `code_copied=false`), `swe-seed provenance show <id>`.

## Generated files, if applicable

`.swe-seed/provenance/*.json` (runtime), `docs/specs/provenance/*.md` (design-time, this set).

## Rust module boundaries

`swe_seed::provenance` with `mod record`, `mod hash`, `mod verify`. Hashing is independent
(std/`sha2`), not derived from `capability/hash.rs`.

## Security and provenance considerations

This spec *is* the provenance safeguard. Misclassifying a license is the top risk;
mitigated by `license_status` gating + mandatory human review for non-`clear` statuses.

## Tests

- `provenance verify` fails if any active capability lacks source_hash/license_tag.
- `code_copied=true` is rejected at registration.
- EE exclusion: a guard test asserts no SWE_Seed module path mirrors the EE file list.

## Reference repos removed (strengthened clean-room)

The reference repositories are **deleted from the working tree** after analysis. SWE_Seed
is built solely from `docs/specs/`, which are self-contained. Removing the sources makes
copying physically impossible and removes any derivative-code surface. The license summaries
above and the evidence tables in specs 0002–0011 stand as **frozen historical record** of
what was inspected on 2026-06-26; they are not build inputs and require no access to the
(now absent) repos.

## Decisions

The three items below are **human-gated** (not auto-decidable):

- **Commercial distribution** — *blocking:* assume SWE_Seed **is** distributed commercially
  and hold the conservative clean-room rules. Cheapest to relax later, expensive to
  retrofit. Requires explicit sign-off to change.
- **Upstream SHAs** — *one-time action:* re-pin true upstream commit SHAs in
  `reference-sources.md` before these specs are used as legal evidence. (Now that repos are
  deleted, capture the SHAs from the remote/history.)
- **Seed-source license tags** — *per-source gate:* verify serena, context7, agent-skills,
  and SkillSpector licenses before each is activated. One-time check per source; a `clear`
  `license_status` is required to ship that integration.

## Acceptance criteria

- [ ] Both repos' license situations summarized with exact files inspected.
- [ ] Ambiguity (multi-host runtime prior-art) explicitly flagged for human review.
- [ ] Provenance + traceability formats defined; EE exclusion enforced by a test.
