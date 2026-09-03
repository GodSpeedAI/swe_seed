# Explanation: Why Three Layers?

This document explains why SWE_SEED is structured into three distinct layers (`SweSeed` -> `Harness` -> `Fabricator`) and why ownership flows strictly downward.

---

## 1. The Core Question

Earlier iterations and specifications attempted to combine repository governance, capability registries, execution harnesses, and prototype generation into a single monolithic layer. Why did specification [0012](specs/0012-existing-harness-reconciliation.md) explicitly mandate the three-layer hierarchy:

```
SweSeed     (Outer: governance, capability assembly, boundaries)
  └─ Harness   (Middle: routing, proof, context, hooks, traces)
       └─ Fabricator (Inner: bounded product-to-prototype pipeline)
```

---

## 2. Confirmed Design Rationale

### A. Preventing Governance Leakage
Each layer has a fundamentally different scope and lifecycle:
- **SweSeed**: Operates at the **organization and repository repository scope**. It governs which tools, skills, and MCP servers are permitted, ensures license compliance, and validates architectural boundaries.
- **Harness**: Operates at the **task and session scope**. It routes requests, limits context, logs tool calls, records traces, and gates merges.
- **Fabricator**: Operates at the **product feature scope**. It converts an ambiguous user need into a 10-node specification chain and hands off a prototype task.

If these concerns are merged:
- Prototyping logic (like EARS requirements or Gherkin parsing) would pollute low-level routing and hook interception.
- Capability governance (like license checks or boundary audits) would slow down everyday bugfix routing.

### B. Downward-Only Governance
By enforcing that ownership and dependencies flow downward only:
1. **The Harness remains independent of the Fabricator**: You can use SWE_SEED purely as a coding-agent harness for existing codebases without adopting the Fabricator prototype pipeline.
2. **SweSeed remains independent of runtime tools**: The governance layer can validate boundaries and assembly manifests without running tests or invoking agents.
3. **No circular dependencies**: An inner layer never governs an outer layer concern, preventing architectural spaghetti.

---

## 3. Invariants Protected

1. **Inner Layer Independence**: An inner layer cannot import or mutate an outer layer's configuration.
2. **Standalone Integrity**: Disabling or removing the Fabricator layer leaves the Harness layer 100% operational. Disabling the Harness layer leaves the SweSeed governance manifest intact.

---

## 4. Source Trail

- `SWE_SEED_SPEC_v0.2.0.md`: Root governance contract.
- `HARNESS_SPEC.md`: Root harness contract.
- `FABRICATOR_SPEC_v0.1.0.md`: Root fabricator contract.
- `.agent-harness/baml/baml_src/swe_seed.baml`: `LayerName` enum `{ SweSeed, Harness, Fabricator }`.
- `crates/swe-seed-core/src/seed/boundary.rs`: `validate_layer_boundaries()`.
- `docs/specs/0012-existing-harness-reconciliation.md`: Reconciliation bridge.
- `docs/specs/0018-layer-boundary-governance.md`: Boundary governance specification.
