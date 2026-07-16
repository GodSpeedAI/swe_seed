# Open Questions

## Root Spec Files

Question: should the deleted root spec files be restored, or should the harness be changed to use the tracked `.agents/specs/` copies instead?

Recommendation: restore the root specs unless the deletion is intentional. The current harness config, tests, and docs still name `SWE_SEED_SPEC_v0.2.0.md`, `HARNESS_SPEC.md`, and `FABRICATOR_SPEC_v0.1.0.md` as root-level required files.

Impact: this blocks full route proof for the local-memory AGENTS update because `python scripts/harness.py validate`, `bash tests/validate-harness.sh`, and `just ci` fail before or during root spec validation.
