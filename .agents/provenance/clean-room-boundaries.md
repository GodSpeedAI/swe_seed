# Clean-Room Boundaries

Rules governing how SWE_Seed may use the two reference repositories. Conservative by
design: when in doubt, do not copy.

## What MAY be extracted (ideas, not text)

- Architectural shapes: gateway-in-front-of-many-backends, meta-tool discovery surface,
  capability registry as normalized metadata, host-adapter projection, normalized hook
  lifecycle, doctor/drift checks.
- Interface *contracts* described in prose (inputs/outputs, lifecycle order, status
  enums) re-expressed in SWE_Seed-native Rust types.
- Config-model *concepts*: cascade/precedence ideas, profile names, capability YAML as a
  declarative registration idea.
- Security *concepts* named at a high level: hash-pinning, provenance records, scan
  gating, approval flags for risky tool groups, tool-integrity checking as a goal.
- The fact that certain features exist and roughly how they are decomposed (file/module
  names as evidence of decomposition), cited for traceability.

## What MAY NOT be copied

- Any source code, in whole or in part, from either repo (verbatim or lightly edited).
- Branding, product names, persona/agent names, mythology, manifesto prose, README copy,
  CLI wording, help text.
- mcp-gateway **Enterprise Edition** files (SPDX `PolyForm-Noncommercial-1.0.0`):
  `src/security/firewall/`, `src/security/agent_identity.rs`, `data_flow.rs`,
  `message_signing.rs`, `policy.rs`, `response_inspect.rs`, `response_scanner.rs`,
  `scope_collision.rs`, `tool_integrity.rs`, `src/cost_accounting/`, `src/key_server/`,
  `src/transparency_log/`. **Do not even open these for pattern extraction.** SWE_Seed's
  security concepts must be derived from public OWASP/MCP-spec material, not EE internals.
- Anything from oh-my-openagent beyond high-level architecture observation. Its
  Sustainable Use License is noncommercial; SWE_Seed may be distributed commercially, so
  no code, config, or prose may be reused.

## Prohibited implementation shortcuts

- Translating a reference file line-by-line into Rust ("transliteration") — this produces
  a derivative work. Re-derive from the contract/spec, then implement independently.
- Importing either repo as a crate/npm dependency, git submodule, or vendored tree in
  shipped artifacts.
- Copying test fixtures, capability YAML samples, or config schemas verbatim.

## How to handle uncertainty

- If a license status is ambiguous → mark **AMBIGUOUS — human review** and stop, do not
  proceed on assumption.
- If you cannot tell whether something is an idea vs. an expression → treat it as
  expression (do not copy).
- If a pattern only exists in an EE-licensed file → treat the pattern as **unavailable**
  and design from first principles / public specs instead.

## Reference repos deleted

After analysis, `reference/mcp-gateway` and `reference/oh-my-openagent` are **removed from
the repo**. This is the strongest clean-room control: an implementing agent builds SWE_Seed
from `.agents/specs/` alone and physically cannot copy from absent sources. The spec
"Reference evidence" tables remain as frozen provenance (observed 2026-06-26) and are not
build inputs. Re-adding either repo to the tree is prohibited without restarting this
clean-room protocol.

## Required human-review points

1. Confirm true upstream commit SHAs for both repos (currently unpinned).
2. Legal review of SWE_Seed's intended distribution model vs. oh-my-openagent's
   Sustainable Use License (noncommercial) — confirm no derivative exposure.
3. Confirm no SWE_Seed module reproduces mcp-gateway EE security logic.
4. Sign-off on `pattern-traceability.md` (every row `Code copied? = No`).
