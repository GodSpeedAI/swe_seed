# Syntelligent Infrastructure Canonicalization Audit - Resolved Fable Addendum

> Scope: SWE_SEED local scratch report, derived from the cross-repo
> Syntelligent Infrastructure audit and owner answers supplied on 2026-07-01.
> `.agents/reports/` is non-authoritative working memory. Promote any normative
> decision to the relevant repo's committed specs, AGENTS.md, or service docs
> before treating it as binding.

## Decision Summary

The through-line is now explicit: prefer the smallest mechanism that already has
a working precedent elsewhere in the stack. Use edgeai's spool pattern, the
ledger's contract gate, and Context Kernel's MCP surface before introducing new
infrastructure.

The resolved stack keeps SWE_SEED as a local development harness with a narrow
federation exit ramp. It does not make SWE_SEED a NATS participant. It keeps
Context Kernel as MCP-only. It treats settlements, capability updates, and
governance decisions as the outbound facts that deserve durable transport.

## Architecture Resolutions

**NATS is staged, not the core-loop transport.** The dev-harness hot path stays
in-process or HTTP: route, context, authority, proof. That keeps latency low and
keeps the harness simple. NATS is for outbound facts after the loop has produced
something durable: settlements, capability updates, and governance decisions.
The ledger already enforces contracts on that boundary, and edgeai shows the
spool-and-publish pattern working. SWE_SEED should expose a small "publish
result" exit ramp instead of joining NATS directly.

**Context Kernel uses MCP only.** CK is already a single binary serving stdio/SSE
with auth boundaries. Adding NATS would create a second personality without a
real consumer. The build gap is to wire `AclContextAgent` into MCP dispatch and
have SWE_SEED call it as a tool.

**Evidence settlement runs with the harness session.** `EvidenceRecorded` to
`handle_evidence_recorded` should run in the same process as the harness session,
with godspeed_agent imported as a library or invoked as a subprocess. The result
is then published to NATS as a settlement. A separate settlement microservice is
premature while there is only one producer.

## Naming Resolutions

**"Harness" belongs to SWE_SEED.** SWE_SEED already describes itself as the
development harness. godspeed_agent should describe itself as a settlement
runtime or navigation runtime, which matches its code: settlement classification,
capability updates, developmental memory, and navigation.

**Retire "epistemic envelope" as a separate term.** Its useful job is artifact
status plus provenance. That is the provenance block of the v1/CEP-0008 envelope.
One envelope vocabulary with real provenance is better than two partially
overlapping envelope vocabularies.

**Keep one settlement concept.** Human judgment and runtime settlement should use
the same word deliberately. SOPs should state the relationship directly: the
runtime event is the machine-recorded form of the human judgment.

## Implementation Resolutions

**Ed25519 key distribution stays file-based.** Use files on disk with env-var
paths, one keypair per repo, and rotation by redeployment. A key registry is
enterprise machinery for a later phase. The mutual-trust model in `signing.md`
already assumes static peer public keys.

**GSA JSONL forwards to the memory ledger one way.** The local JSONL log remains
the write-ahead buffer. A publisher drains it to NATS when reachable, mirroring
edgeai's spool pattern. This gives offline durability without creating two
sources of truth or a bidirectional sync problem.

**SEA in-stack services are a small canonical set.** Treat these as the
canonical platform services: `policy-gateway`, `simulation`, `ifl_service`,
`sea-kernel`, `workbench-bff`, plus the loop, messaging, and governance libs.
Mark finance, healthcare, pet-training, and similar areas as adjacent products
in `services/AGENTS.md`. This prevents audits from using "SEA" ambiguously for
both the platform and the product portfolio.

## Business And Product Resolutions

**edgeai is a standalone product.** It embeds the stack's client contracts, but
its README sells a private Jetson appliance. Positioning it only as a deployment
of the stack would fight the product's own framing.

**DomainForge is an independent OSS product.** SEA is a reference consumer, not
the owner of the product identity. DomainForge has its own Apache license, CI,
multi-language bindings, and no SEA-specific concepts such as `sea_file_hash`.

## Benchmark And Evaluation Resolutions

**Start with one enforced benchmark number.** The CEP settlement-benchmark score
must not regress week over week. This ratchet is enforceable before a large live
corpus exists. Anti-collapse caps remain hard failures. Absolute thresholds
should wait until live corpus history exists.

**Humans consume benchmark results first.** The first consumer is a report read
by the founder or architect. The adapt loop is second. Wiring scores directly
into capability gates before scorers have real data would automate on fiction.

## Resulting Work Packages

1. Add SWE_SEED's publish-result exit ramp for settlements, capability updates,
   and governance decisions, without adding NATS to the hot path.
2. Wire Context Kernel's `AclContextAgent` into MCP dispatch and call it from
   SWE_SEED as a tool.
3. Run godspeed_agent settlement in-process or as a subprocess from the harness
   session, then publish the settlement result.
4. Rename godspeed_agent prose from harness language to settlement runtime or
   navigation runtime.
5. Collapse epistemic-envelope references into the v1/CEP-0008 provenance block.
6. Document file-based Ed25519 keys, repo-local keypairs, env-var paths, and
   redeploy rotation.
7. Add one-way JSONL-to-NATS forwarding for GSA using the edgeai spool pattern.
8. Add a SEA service-boundary note that distinguishes platform services from
   adjacent products.
9. Publish CEP benchmark reports for human review before using them as capability
   gates.

## Closed Questions

The following questions are closed by this addendum:

- NATS role in the core loop: staged outbound transport only.
- Context Kernel transport: MCP only.
- Settlement execution location: same process or subprocess as the harness
  session.
- Ownership of the word harness: SWE_SEED.
- Epistemic envelope status: retired as a separate vocabulary.
- Settlement vocabulary: one concept, with runtime settlement as recorded human
  judgment.
- Signing key distribution: file paths plus env vars, one keypair per repo.
- GSA ledger sync: one-way forwarding from JSONL to NATS.
- SEA in-stack scope: small canonical platform set plus adjacent products.
- edgeai positioning: standalone product embedding stack contracts.
- DomainForge positioning: independent OSS product with SEA as reference
  consumer.
- Benchmark threshold: week-over-week CEP settlement-score ratchet plus hard
  anti-collapse caps.
- Benchmark consumer: human report first, adapt-loop automation second.
