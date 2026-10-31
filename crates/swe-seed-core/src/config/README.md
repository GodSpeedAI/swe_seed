# config

## Overview

Typed YAML configuration parsing for the SWE_Seed harness (spec 0019, Phase 0).
Pure serde data structs with no behavior: one struct group per config file
(budget policy, agent hooks, fabricator, strategy), all loaded through a single
generic `load_yaml` entry point. Structs are lenient via `#[serde(default)]` so
incidental fields do not break parsing, while the four top-level config structs
use `#[serde(deny_unknown_fields)]`.

## Interface

- `load_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T>`: reads the
  file and parses it with `serde_yaml` [mod.rs:load_yaml]
- `BudgetPolicyConfig` (top-level, `deny_unknown_fields`) with nested
  `BudgetDefaults`, `RawOutputPolicy`, `ToolOutputContainment`, `DurableStorage`,
  `SessionContinuity` [mod.rs:BudgetPolicyConfig], [mod.rs:BudgetDefaults],
  [mod.rs:RawOutputPolicy], [mod.rs:ToolOutputContainment],
  [mod.rs:DurableStorage], [mod.rs:SessionContinuity]
- `HooksConfig` (top-level, `deny_unknown_fields`) with nested `HooksPaths`
  (`root`, `logs`, `payloads`, `artifacts`, `index_db`, `vector_index_root`),
  `HooksLogging`, `RedactionConfig`, `HooksSecurity` (`stdin_max_bytes`),
  `HooksSettings` (`timeout_seconds`), `HooksExports` (`otel_format`,
  `junit_suite_name`) [mod.rs:HooksConfig], [mod.rs:HooksPaths],
  [mod.rs:HooksLogging], [mod.rs:RedactionConfig], [mod.rs:HooksSecurity],
  [mod.rs:HooksSettings], [mod.rs:HooksExports]
- `FabricatorConfig` (top-level, `deny_unknown_fields`: `template_root`,
  `schema_root`, `run_root`, `generated_dir_name`, `prototype_dir_name`,
  `proof_dir_name`, `handoff_dir_name`, `required_generated_artifacts`, ...) with
  nested `FabricatorDefaults` [mod.rs:FabricatorConfig],
  [mod.rs:FabricatorDefaults]
- `StrategyConfig` (top-level, `deny_unknown_fields`: `layer_mode`,
  `python_runtime`, `paths: BTreeMap<String, String>`,
  `required_example_artifacts`) with nested `StrategyPython`
  [mod.rs:StrategyConfig], [mod.rs:StrategyPython]

## Invariants

- All four top-level config structs (`BudgetPolicyConfig`, `HooksConfig`,
  `FabricatorConfig`, `StrategyConfig`) reject unknown fields via
  `#[serde(deny_unknown_fields)]`; every field is additionally
  `#[serde(default)]`, so empty files parse successfully
  [mod.rs:BudgetPolicyConfig], [mod.rs:HooksConfig],
  [mod.rs:FabricatorConfig], [mod.rs:StrategyConfig]
- The module is pure data: no behavior is attached to any config struct
  [mod.rs:BudgetPolicyConfig]

## Side Effects

- `load_yaml` reads the file at `path` from disk via `std::fs::read`
  [mod.rs:load_yaml]

## Verification

- Run: `cargo test -p swe-seed-core`
