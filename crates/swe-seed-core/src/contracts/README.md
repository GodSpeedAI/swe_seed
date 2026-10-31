# contracts

## Overview

`.baml` contracts-as-data (spec 0019). Parses the canonical `.baml` schema files
into `BamlType` values and provides the `BamlParity` trait so hand-written Rust
structs and enums can assert 1:1 parity with their `.baml` counterparts in tests.
There is no LLM runtime here: `.baml` files are read-only inputs, and `function`
blocks are skipped as offline-generation contracts. The module also defines the
shared harness-layer contract types (`harness.rs`) that replaced module-local
validation aliases across the crate.

## Interface

- `pub enum BamlKind { Class, Enum }` [baml_parse.rs:BamlKind]
- `pub struct BamlType { name, source_file, kind, fields, field_types, variants }`
  with helpers `field_set()` / `variant_set()` [baml_parse.rs:BamlType],
  [baml_parse.rs:field_set], [baml_parse.rs:variant_set]
- `parse_baml_dir(dir: &Path) -> std::io::Result<Vec<BamlType>>`: parses all
  `*.baml` files under a directory, sorted, tagging each type with the file stem
  [baml_parse.rs:parse_baml_dir]
- `parse_baml_src(src: &str, source_file: &str) -> Vec<BamlType>`: parses
  `class Name { ... }` and `enum Name { ... }` blocks; skips `function` blocks
  [baml_parse.rs:parse_baml_src]
- `pub enum BamlShape { Class { fields, field_types }, Enum { variants } }`
  [parity.rs:BamlShape]
- `pub trait BamlParity { baml_name() -> &'static str; baml_shape() -> BamlShape }`
  [parity.rs:BamlParity]
- `SourceRef { path, summary }` (harness.baml `SourceRef`, `BamlParity`)
  [harness.rs:SourceRef]
- `ArtifactStatus { Draft, Candidate, Active, Deprecated, Rejected }` with
  `#[serde(rename_all = "PascalCase")]` (harness.baml `ArtifactStatus`,
  `BamlParity`) [harness.rs:ArtifactStatus]
- `ProofDisposition { Required, Optional, SkippedWithReason }` (harness.baml
  `ProofDisposition`, `BamlParity`) [harness.rs:ProofDisposition]
- `ValidationRequirement { check, blocking, evidence }` (harness.baml
  `ValidationRequirement`, `BamlParity`); the canonical validation type that
  module-local aliases MUST be replaced by [harness.rs:ValidationRequirement]
- `HarnessNeed { request, desired_outcome, constraints, non_goals,
proof_requirements }` (harness.baml `HarnessNeed`, `BamlParity`)
  [harness.rs:HarnessNeed]
- `HarnessADR { id, title, status, context, decision, alternatives,
consequences, rollback_plan, validation, sources }` (harness.baml `HarnessADR`,
  `BamlParity`) [harness.rs:HarnessADR]
- `RegenerationInput { harness_spec_path, approved_route_cards,
approved_skill_ir_files, eval_specs, hook_policies,
approved_learning_records, regression_cases }` (harness.baml
  `RegenerationInput`, `BamlParity`) [harness.rs:RegenerationInput]
- `RegenerationPlan { source_artifacts, render_targets, preserved_decisions,
blocking_diffs, validation }` (harness.baml `RegenerationPlan`, `BamlParity`)
  [harness.rs:RegenerationPlan]
- Re-exports: `HarnessValidationRequirement` is an alias of
  `ValidationRequirement` [mod.rs:HarnessValidationRequirement]

## Invariants

- `function` blocks in `.baml` sources are never parsed into runtime types: they
  are offline-generation contracts, not runtime code
  [baml_parse.rs:parse_baml_src]
- `parse_baml_dir` propagates filesystem failures instead of returning an empty
  schema set: an unreadable directory or file is a real error, not "zero types"
  [baml_parse.rs:parse_baml_dir]
- Brace matching is depth-counted only after `strip_strings_and_comments` removes
  raw strings (`#"..."#`), normal strings, and `//` comments, so scanning is
  unambiguous [baml_parse.rs:match_brace], [baml_parse.rs:strip_strings_and_comments]
- `BamlShape::Class` carries parallel `field_types` strings so type drift (not
  just field-name drift) fails parity [parity.rs:BamlShape]
- `ValidationRequirement` is the single canonical validation-requirement type for
  every harness class with a `validation` field; module-local aliases MUST NOT
  be reintroduced [harness.rs:ValidationRequirement]

## Side Effects

- `parse_baml_dir` reads the directory and every `*.baml` file in it via
  `std::fs::read_dir` / `std::fs::read_to_string` [baml_parse.rs:parse_baml_dir]

## Verification

- Run: `cargo test -p swe-seed-core`
