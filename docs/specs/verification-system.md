# Verification System

The harness distinguishes work performed, proof gathered, and completion claimed.

Default proof command:

```bash
just ci
```

Implementation-like job types require traceability from requirement to files changed to verification output. Skipped checks must be justified.

Production-readiness verification also requires:

- route decision records for auditable route selection,
- trace records for material implementation work,
- negative conformance evals for known failure modes,
- route conflict evals for semantic router ambiguity,
- context-plan checks for context budget behavior,
- prose constraints that prevent completion language before proof.

Generated trace JSON belongs under `.agent-harness/traces/records/` or `.agent-harness/traces/route-decisions/` and must not contain secrets.
