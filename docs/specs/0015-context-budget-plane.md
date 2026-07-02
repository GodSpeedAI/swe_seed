# 0015 — Context Budget Plane

## Purpose

Define the context-management plane: `ContextBudget`/`ContextPack`, the `context-plan`
command, and the `budget-policy.yaml` raw-output/containment discipline. This is the
existing project's answer to context flooding and supersedes the vaguer ContextBuild notes
in spec 0005.

## Non-goals

- Not a retrieval engine; v0.1 selects/excludes files and emits salient facts, not embeddings.
- Not the external Context Kernel (0011) — that is an optional federation source; this plane
  is local and always works.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `harness.baml` `ContextBudget{max_context_size,required_files,optional_files,excluded_files,freshness_requirements,relevance_rules,summarization_rules}`, `ContextPack{route_card_id,budget_id,included_files,excluded_files,salient_facts,stale_context_warnings}` | Budget + pack contracts |
| `scripts/harness.py:1797` `context_plan` | `context-plan <task>` command |
| `.agent-harness/context/budget-policy.yaml` | `raw_output_policy{max_default_lines:200, prefer/avoid}`, `tool_output_containment{high_volume_tools, containment_action: summarize_before_context}`, `session_continuity` |

## SWE_Seed requirements

1. **`context-plan <task>`** selects a `ContextBudget` for the routed job and produces a
   `ContextPack`: which files to include/exclude, salient facts, and stale-context warnings.
2. **Budget policy** (`budget-policy.yaml`) is enforced as defaults: raw output capped
   (default 200 lines), prefer counts/path-lists/focused-excerpts/JSON summaries, avoid full
   logs/dir-dumps/secret-bearing output.
3. **Containment**: high-volume tools (search, file read, shell, web fetch, test logs) are
   `summarize_before_context` — the durable artifact goes to traces, only the summary enters
   context. (This mirrors the project's context-mode discipline; the Rust binary applies it
   to its own output, e.g. `doctor`/`eval` summaries.)
4. **Freshness/relevance/summarization rules** from the budget gate what enters a pack;
   stale context is warned, not silently included.
5. ContextPack is tied to a `route_card_id` (0014) so a session can be reconstructed.

## Data model

`ContextBudget`, `ContextPack` (canonical, 0019). `budget-policy.yaml` parsed into a
`BudgetPolicy` struct.

```toml
# ContextPack (rendered)
id = "pack-test-001"
route_card_id = "test"
budget_id = "budget-default"
included_files = ["AGENTS.md", ".agent-harness/routes/test.json"]
excluded_files = ["**/node_modules/**"]
salient_facts = ["proof command is `just ci`"]
stale_context_warnings = []
```

- **Rust**: `swe_seed::context` (`budget.rs`, `pack.rs`, `policy.rs`).
- **Validation**: required files exist; excluded never included; size within `max_context_size`.

## CLI behavior

```
swe-seed context-plan <task>     # → ContextPack for the routed job
```

## Generated files

`ContextPack` records (gitignored generated, per budget policy). `budget-policy.yaml` is a
committed config consumed read-only.

## Rust module boundaries

`swe_seed::context`; consumed by `route`/`trace` (0014) and applied to CLI output formatting.

## Security and provenance considerations

`avoid` list includes secret-bearing output; the policy is the first line against leaking
secrets into context (paired with redaction in 0005).

## Tests

- `context-plan` excludes `excluded_files` and includes `required_files`.
- Output exceeding `max_default_lines` is summarized, not dumped.
- A stale required file raises a `stale_context_warnings` entry.
- `budget-policy.yaml` parses to `BudgetPolicy` (parity test vs current file).

## Open questions

- Adopt `serde_yaml` vs port the hand-rolled YAML parser? (Recommend `serde_yaml` + a parity
  test against existing config files.)

## Acceptance criteria

- [ ] `context-plan` produces valid `ContextPack`s respecting the budget.
- [ ] Raw-output/containment policy enforced on the binary's own output.
- [ ] Stale context warned; secrets excluded.
