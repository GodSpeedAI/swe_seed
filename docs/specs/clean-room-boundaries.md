# Clean-Room Boundaries

This is the authoritative clean-room boundary text for SWE_Seed. Normative provenance
rules live under `docs/specs/`.

## What MAY Be Extracted

- Architectural shapes, interface contracts, lifecycle order, status enums, and module
  decomposition, re-expressed in SWE_Seed-native terms.
- Config-model concepts such as precedence, profile concepts, declarative registration,
  provenance records, scan gating, and approval flags.
- High-level security goals derived from public OWASP or MCP material.

## What MAY NOT Be Copied

- Source code, test fixtures, config schemas, branding, product names, persona names,
  README prose, CLI wording, help text, or other expression from reference repos.
- Any mcp-gateway Enterprise Edition files or patterns from those files.
- Anything from noncommercial source-available references beyond high-level architecture
  observation.

## Required Controls

- Do not vendor, submodule, or depend on reference repos in shipped artifacts.
- Do not transliterate reference files line-by-line into Rust.
- Treat ambiguous idea-versus-expression calls as expression and do not copy.
- Record every extracted pattern in the committed traceability matrix with
  `Code copied? = No`.
