# Fabrication Runs

Each run lives at `.fabricator/runs/<run_id>/` and is intentionally disposable runtime state.

Expected layout:

- `input.yaml`: bounded product seed copied from the seed source.
- `generated/`: semantic-chain artifacts, eval spec, proof record, reflection, and adaptation decision.
- `prototype/`: generated prototype files for the run.
- `proof/`: proof outputs such as `EVAL_RESULT.json`.
- `handoff/`: bounded agent handoff packet derived from generated artifacts.
- `trace.md`: file-first trace of generation, validation, proof, and reflection events.

Run directories are ignored by git so the repository keeps only the stable scaffold, templates,
schemas, docs, and command surface.
