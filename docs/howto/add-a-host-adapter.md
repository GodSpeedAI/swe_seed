# How-To: Add a Host Adapter

This guide explains how to add support for a new AI coding assistant by implementing the `HostAdapter` trait in `swe-seed-core`.

---

## 1. Goal

Enable SWE_SEED to project its canonical doctrine, rules, and skills into a new tool format (e.g. Cursor, Windsurf, or Aider) while supporting drift detection and snapshot rollback.

---

## 2. Prerequisites

- Working Rust development environment.
- Knowledge of the target tool's configuration format and target paths.

---

## 3. Procedure

### Step 1: Create the Adapter File
Create `crates/swe-seed-core/src/adapters/<new_host>.rs`. Implement the `HostAdapter` trait:

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::adapters::capabilities::{EnforcementLevel, HostCapabilityMatrix};
use crate::adapters::host::HostAdapter;
use crate::adapters::marker::replace_managed_block;
use crate::adapters::projection::Projection;

pub struct NewHostAdapter;

impl HostAdapter for NewHostAdapter {
    fn name(&self) -> &'static str {
        "new_host"
    }

    fn target_files(&self, root: &Path) -> Vec<PathBuf> {
        vec![root.join(".new_host/rules.md")]
    }

    fn capability_matrix(&self) -> HostCapabilityMatrix {
        HostCapabilityMatrix {
            routing_hook: EnforcementLevel::Advisory,
            pre_tool_block: EnforcementLevel::Unsupported,
            post_tool_log: EnforcementLevel::Unsupported,
            token_budget: EnforcementLevel::Advisory,
        }
    }

    fn project(&self, root: &Path) -> Result<Vec<Projection>> {
        let content = format!(
            "<!-- BEGIN SWE_SEED MANAGED BLOCK -->\n# Rules for NewHost\nGenerated from canonical SWE_SEED specifications.\n<!-- END SWE_SEED MANAGED BLOCK -->\n"
        );
        Ok(vec![Projection {
            relative_path: PathBuf::from(".new_host/rules.md"),
            content,
        }])
    }
}
```

### Step 2: Register the Adapter
In `crates/swe-seed-core/src/adapters/mod.rs`:
1. Add `pub mod <new_host>;`.
2. Register `NewHostAdapter` in the adapter registry function:
   ```rust
   pub fn get_adapter(name: &str) -> Option<Box<dyn HostAdapter>> {
       match name {
           "claude" => Some(Box::new(claude::ClaudeAdapter)),
           // ...
           "new_host" => Some(Box::new(<new_host>::NewHostAdapter)),
           _ => None,
       }
   }
   ```

In `crates/swe-seed/src/host_cli.rs`:
Add the new host variant to `HostSelection` clap enum.

### Step 3: Run Tests
Verify compilation and test suite:

```bash
cargo test -p swe-seed-core --lib adapters
```

### Step 4: Validate with Dry Run
Test the projection without writing to disk:

```bash
cargo run -q -p swe-seed -- sync --host new_host --dry-run
```

---

## 4. Common Failure Symptoms

- **Managed Marker Mismatch**: Ensure your projection includes the exact begin and end comments matching `marker.rs`.
- **Drift Detection Errors**: Ensure trailing newlines in `project()` match the output written to disk.
