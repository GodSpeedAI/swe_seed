# BAML Generation Model

The reference implementation is script-first and file-first. It does not require BAML at runtime.

When BAML is enabled, the authoritative typed fabricator contract remains
`.agent-harness/baml/baml_src/fabricator.baml`. BAML can generate structured intermediates, but the
operating surface remains Markdown, YAML, and JSON artifacts in the run directory.

This split keeps regeneration possible even when the runtime environment only has Python and the root
specs.
