# Artifact Schemas

The reference schema files live in `.fabricator/schemas/`.

Key schema groups:

- seed and discovery: `product-seed.schema.yaml`, `jtbd.schema.yaml`, `job-hypothesis.schema.yaml`
- product specs: `adr.schema.yaml`, `prd.schema.yaml`, `sds.schema.yaml`, `tdd-plan.schema.yaml`
- execution context: `context-pack.schema.yaml`, `agent-task.schema.yaml`
- proof and learning: `eval-spec.schema.yaml`, `eval-result.schema.yaml`,
  `proof-record.schema.yaml`, `reflection.schema.yaml`, `adaptation-decision.schema.yaml`,
  `skill-proposal.schema.yaml`

The schemas are small on purpose. They describe the stable fields an agent needs to regenerate the
packet and validator behavior.
