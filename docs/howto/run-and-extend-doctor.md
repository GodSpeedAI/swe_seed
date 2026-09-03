# How-To: Run & Extend Doctor Checks

This guide explains how to execute system-wide diagnostic checks using `swe-seed doctor` and how to register custom health checks in the engine.

---

## 1. Goal

Maintain full visibility over environment health, harness integrity, and host projection drift, and extend the doctor suite with project-specific verification rules.

---

## 2. Prerequisites

- Working Rust development environment.

---

## 3. Running Doctor Diagnostics

### Everyday Developer Health Check
Run the developer-level check verifying basic tools:

```bash
just doctor
```

### Comprehensive Harness & Drift Audit
Run the full harness audit including host projection drift checks:

```bash
just harness-doctor
```

Behind the scenes, this executes:
```bash
cargo run -q -p swe-seed -- doctor
```

### Emitting Machine-Readable JSON (for CI)
```bash
cargo run -q -p swe-seed -- doctor --json
```

---

## 4. Extending Doctor with a Custom Check

### Step 1: Define the Check Function
Open `crates/swe-seed-core/src/doctor/check.rs`. Add your custom check:

```rust
use std::path::Path;
use crate::doctor::report::CheckResult;

pub fn check_custom_license_file(root: &Path) -> CheckResult {
    let license_path = root.join("LICENSE");
    if license_path.is_file() {
        CheckResult::ok("license_file", "LICENSE file is present")
    } else {
        CheckResult::warning("license_file", "No LICENSE file found at repository root")
    }
}
```

### Step 2: Register the Check in the Doctor Runner
Open `crates/swe-seed-core/src/doctor/mod.rs`. In `run_doctor()`:

```rust
// Add your check to the execution sequence
results.push(check::check_custom_license_file(root));
```

### Step 3: Verify the Custom Check
Run the doctor test suite:

```bash
cargo test -p swe-seed-core --lib doctor
```

Then run `doctor` to see your new check execute:

```bash
cargo run -q -p swe-seed -- doctor
```

---

## 5. Common Failure Symptoms

- **Check Reported as Failed**: Address the specific diagnostic message emitted. If a required file is missing, restore it; if drift is detected, run `swe-seed sync --host <host>`.
