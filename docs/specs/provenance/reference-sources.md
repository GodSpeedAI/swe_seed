# Reference Sources

This is a human legal/provenance record, not a build input.

The reference repositories inspected during the original analysis have been removed from the
working tree. SWE_Seed implementation must proceed from `docs/specs/` and the root contracts,
not from re-opening those repositories.

## Sources

| Source | License posture | Use allowed in SWE_Seed |
|---|---|---|
| gateway prior-art core | MIT, with separate EE carve-outs | High-level ideas and public contracts only; no code copied |
| gateway prior-art EE files | PolyForm Noncommercial | Do not open or extract patterns |
| multi-host runtime prior-art | Sustainable Use License, noncommercial/source-available | Architecture observation only; no code, config, prose, names, or schemas copied |

## Required Human Review

- Re-pin true upstream commit SHAs before relying on historical line citations.
- Confirm SWE_Seed distribution model against the noncommercial source-available reference.
- Confirm every traceability row says `Code copied? = No`.
