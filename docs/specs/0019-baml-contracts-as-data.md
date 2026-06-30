# 0019 — BAML Contracts as Data

## Purpose

Define how the Rust rewrite treats the `.baml` files as the **canonical data-model schema**
without ever invoking an LLM at runtime. Establishes the single source of truth for every
struct/enum used across specs 0013–0018 and the parity guarantee.

## Non-goals

- **No LLM runtime.** The `.baml` `Generate*`/`Propose*` functions are offline-generation
  contracts, not runtime dependencies. The Rust binary never calls them.
- Not porting `baml-py` or the generated `baml_client/`.
- Not authoring `.baml`; the rewrite consumes the existing contracts.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `.agent-harness/baml/baml_src/{swe_seed,harness,fabricator}.baml` | Canonical enums/classes/functions for all three layers |
| `scripts/harness.py:245-247,785-787` | Harness treats `.baml` as data: lists them as source paths and includes `.baml` in validated file extensions; never imports `baml_client` |
| `pyproject.toml` | `baml-py` listed but unused by the stdlib-only CLIs |

## SWE_Seed requirements

1. **Canonical schema**: every Rust struct/enum used in specs 0013–0018 corresponds 1:1 to a
   `.baml` `class`/`enum` (same name, same fields, same variants). `.baml` wins on any
   discrepancy.
2. **Hand-written structs + parity test** (chosen over codegen for v0.1, YAGNI): Rust types
   are hand-written; a `baml_parity` test parses each `.baml` `class`/`enum` and asserts the
   Rust type has matching field names/types and enum variants. Drift fails CI.
3. **Generation functions are contracts, not code**: `Generate*`/`Propose*`/`Validate*`
   functions document the offline pipeline's inputs/outputs. The Rust binary validates and
   renders the *resulting artifacts* (files), it does not execute generation.
4. **Rendering**: `.baml`-defined artifacts are rendered to/validated from files (TOML/JSON)
   as the Python harness does — `render-skills`, `fabricate`, etc.
5. `.baml` files remain committed and unchanged by the rewrite; they are inputs.

## Data model

A minimal `.baml` reader sufficient for parity: parse `enum Name { Variant ... }` and
`class Name { field type ... }`. No need to evaluate `function` bodies.

```
baml class/enum  ──parse──►  BamlType { name, kind, fields[], variants[] }
                                   │
                          assert_parity(rust_type)  ──fail CI on drift──
```

- **Rust**: `swe_seed::contracts` (`baml_parse.rs`, `parity.rs`). The actual domain structs
  live in their layer modules (`eval`, `trace`, `context`, `learning`, `fabricator`, `seed`).
- **Validation**: every domain struct registered for parity; unregistered `.baml` class →
  test warns (coverage gap).

## CLI behavior

N/A (internal). Optionally `swe-seed contracts check` runs the parity assertion locally.

## Generated files

None. `.baml` files are read-only inputs.

## Rust module boundaries

`swe_seed::contracts` is a dev/test-facing module; domain modules depend on nothing here at
runtime (parity is a test).

## Security and provenance considerations

Keeping `.baml` as the schema source prevents data-model drift between the offline generation
pipeline and the Rust runtime. No LLM credentials or network at runtime.

## Tests

- `baml_parity`: every domain struct/enum matches its `.baml` counterpart (field + variant
  names). Renaming a field in Rust without updating `.baml` → fail.
- Coverage: every `class`/`enum` in the three `.baml` files is either mapped or explicitly
  marked out-of-scope.

## Open questions

- If `.baml` contracts churn frequently post-v0.1, switch to a `.baml`→Rust generator?
  (Revisit then; hand-written + parity is correct now.)

## Acceptance criteria

- [ ] All specs 0013–0018 structs map 1:1 to `.baml`; parity test green.
- [ ] No LLM runtime; `baml-py`/`baml_client` not a dependency of the Rust binary.
- [ ] `.baml` files unchanged and treated as canonical schema.
