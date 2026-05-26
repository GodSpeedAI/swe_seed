# Fabrication Layer

The fabrication layer turns one bounded product seed into a runnable prototype packet with linked
specs, a bounded agent task, proof artifacts, and post-run learning artifacts.

Use it when the project needs a small product-to-prototype workflow that another agent can
reconstruct from the root specs without relying on hidden prompts or unstated file shapes.

Reference flow:

1. Create a bounded seed.
2. Run `fabricate new <seed>`.
3. Run `fabricate generate <run_id>`.
4. Run `fabricate validate <run_id>`.
5. Run `fabricate handoff <run_id>`.
6. Run `fabricate proof <run_id>`.
7. Run `fabricate reflect <run_id>`.

See the command contract and proof-command map in
`docs/fabrication-layer/references/command-contract.md`.
