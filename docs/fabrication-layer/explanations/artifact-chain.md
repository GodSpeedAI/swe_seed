# Artifact Chain

The fabrication chain is the constrained product-side half of the broader semantic specification
chain.

Reference order:

`PRODUCT_SEED.md` -> `JTBD.md` -> `JOB_HYPOTHESIS.md` -> `ADR.md` -> `PRD.md` -> `SDS.md` ->
`TDD.md` -> `EVAL_SPEC.yaml` -> `AGENT_TASK.md` -> `PROOF_RECORD.md` -> `REFLECTION.md`

Every downstream artifact should be reviewable against an upstream artifact or an explicit waiver.
The reference validator checks that requirement IDs trace through SDS, TDD, eval, and the agent
task surfaces.
