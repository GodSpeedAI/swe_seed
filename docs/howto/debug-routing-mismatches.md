# How-To: Debug Routing Mismatches

This guide explains how to diagnose why a prompt matched an unintended RouteCard and how to refine card definitions to achieve accurate routing.

---

## 1. Goal

Ensure prompts reliably select the intended RouteCard (e.g. ensuring a performance profiling request selects `benchmark` rather than `test`).

---

## 2. Prerequisites

- SWE_SEED repository.
- Failing prompt example.

---

## 3. Procedure

### Step 1: Observe the Actual Routing Decision
Execute the router with the target prompt:

```bash
cargo run -q -p swe-seed -- route "verify checkout memory usage under load"
```

Suppose this outputs:
```json
{
  "job_type": "test",
  "route_card": ".agent-harness/routes/test.json"
}
```
If the desired route was `research` or a custom performance route, this represents a mismatch.

### Step 2: Identify Why the Misclassification Occurred
Inspect both the selected card (`.agent-harness/routes/test.json`) and the intended card:
1. **Semantic Triggers**: Notice if words in your prompt (e.g. `"verify"`) appear in the competing card's `semantic_triggers`.
2. **Positive Examples**: Check if the competing card has a positive example closely resembling your prompt.

### Step 3: Add Disambiguating Rules
Edit the relevant route cards in `.agent-harness/routes/`:

1. **Add a Negative Example to the False-Positive Card**:
   In `.agent-harness/routes/test.json`:
   ```json
   "negative_examples": [
     "verify checkout memory usage under load",
     "profile latency across requests"
   ]
   ```
2. **Add a Positive Example to the Intended Card**:
   In the intended route card:
   ```json
   "positive_examples": [
     "verify checkout memory usage under load"
   ]
   ```

### Step 4: Verify the Routing Change
Re-run the router:

```bash
cargo run -q -p swe-seed -- route "verify checkout memory usage under load"
```

Verify that the intended `job_type` is now selected.

### Step 5: Guard Against Regressions
Run the router golden test suite:

```bash
cargo test -p swe-seed --test route_golden
```

If golden fixtures changed, update the test expectation fixture to document the deliberate shift.
