# 0001 — Reference Analysis Protocol

## Purpose

Define how SWE_Seed extracts architectural patterns from read-only reference repos
(`gateway prior-art (source removed)`, `multi-host runtime prior-art (source removed)`) without creating derivative code or
license exposure. This protocol governs every other spec in this directory.

## Non-goals

- Not a plan to vendor, fork, or depend on either reference repo.
- Not a license grant. Legal sign-off is a human gate (spec 0009).
- Not an excuse to transliterate reference source into Rust.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `LICENSE`, `LICENSE-EE.md` | full | Dual MIT + PolyForm Noncommercial EE | Permissive ideas OK; EE files out-of-bounds |
| multi-host runtime prior-art | `LICENSE.md` | full | Sustainable Use License (noncommercial) | Architecture observation only; no reuse |
| both | (vendored dirs) | `git rev-parse HEAD` → parent SHA | No nested `.git` | Citations pinned to content @2026-06-26; re-pin upstream SHA before legal use |

## SWE_Seed requirements

1. **Read-only**: never write into `reference/`. Enforced by convention + CI guard.
2. **Idea/expression split**: extract contracts, lifecycles, decompositions, names-as-
   evidence. Never copy expression (code, prose, branding).
3. **EE exclusion**: the gateway prior-art EE file list (spec 0009 / `docs/specs/clean-room-boundaries.md`)
   is never opened for extraction.
4. **Traceability**: every extracted pattern gets a row in
   `docs/specs/provenance/pattern-traceability.md` with `Code copied? = No`.
5. **Citation discipline**: cite `repo/path:line` where a concrete symbol was observed;
   otherwise cite `repo/path` (directory-level evidence).
6. **Uncertainty → stop**: ambiguous license or idea/expression calls become open
   questions + human-review points, not assumptions.

## Data model

`ProvenanceRecord` (see spec 0003) is the runtime artifact this protocol mirrors at
design time: source, path, hash, inspection date, license tag, copied=false.

## CLI behavior, if applicable

None. This is a process spec. Future `swe-seed provenance verify` (spec 0009) checks the
traceability matrix is complete.

## Generated files, if applicable

- `docs/specs/provenance/reference-sources.md`
- `docs/specs/clean-room-boundaries.md`
- `docs/specs/provenance/pattern-traceability.md`

## Rust module boundaries

N/A at protocol level. Downstream: `swe_seed::provenance` validates the matrix.

## Security and provenance considerations

The protocol *is* the provenance safeguard. License misclassification is the primary
risk; mitigated by conservative defaults and human review (spec 0009).

## Tests

- Doc test / CI: assert no file under `reference/` is modified in a SWE_Seed PR.
- CI: assert pattern-traceability.md has zero `Code copied? = Yes` rows.
- CI: assert no SWE_Seed source path matches the EE exclusion list as a copy target.

## Self-containment

The reference repositories (`gateway prior-art (source removed)`, `multi-host runtime prior-art (source removed)`) are
**removed from the repo** after this analysis. These specs are **authoritative and
self-contained**: an implementing agent builds SWE_Seed entirely from `docs/specs/`
without ever opening the reference repos. The "Reference evidence" tables in each spec are
**frozen historical provenance** — they document where an idea was observed (as of
2026-06-26, repos since deleted) and assert `Code copied? = No`; they are not build inputs.
Deleting the sources also makes accidental copying impossible.

## Decisions

- **Upstream SHAs** — *open, human action:* re-pin true upstream commit SHAs for both repos
  before these specs are cited as legal/provenance evidence. (Mechanical; do once.)
- **Commercial distribution** — *open, blocking human call:* see spec 0009. Default posture
  is to **assume commercial** and hold the conservative clean-room rules already specified.

## Acceptance criteria

- [ ] All three provenance files exist and are internally consistent.
- [ ] Every spec 0002–0010 cites evidence via this protocol's citation rules.
- [ ] No EE file is referenced as a source of code-level patterns.
