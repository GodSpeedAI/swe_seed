# Tutorial: Authoring a New Skill

This tutorial guides you through creating a normalized skill in SWE_SEED, running security scans, and rendering it to host-native agent surfaces.

---

## 1. Prerequisites

- Development environment initialized (`just doctor`).
- Clean repository working tree.

---

## 2. Step 1: Create the Skill Definition

Skills reside in `.agent-harness/skills/`. Create a directory for your new skill:

```bash
mkdir -p .agent-harness/skills/json-safety
```

Create `.agent-harness/skills/json-safety/SKILL.md`:

```markdown
---
name: json-safety
version: 0.1.0
description: "Guidelines for safely deserializing untrusted JSON payloads without panics or memory exhaustion."
requires: []
tags: ["safety", "json", "rust"]
---

# JSON Safety

## Intent
Prevent denial-of-service or crash bugs when parsing untrusted JSON inputs.

## Rules
1. Never parse untrusted JSON directly into unbounded memory structures.
2. Always enforce byte size limits prior to deserialization.
3. Handle deserialization errors explicitly; avoid `.unwrap()`.
```

---

## 3. Step 2: Validate Harness Structure

Verify that the newly added skill conforms to structural rules:

```bash
just harness-validate
```

### Expected Output:
```text
Harness validation passed
```

The validator confirms that:
- The skill directory name matches the `name` field in the frontmatter.
- No duplicate skill IDs exist.
- Required frontmatter fields are present.

---

## 4. Step 3: Render Skills to Host Targets

Compile and project the newly added skill to all configured host targets:

```bash
just harness-render-skills
```

Behind the scenes, this executes:
```bash
cargo run -q -p swe-seed -- render-skills
```

### Expected Output:
```text
Rendering SkillIR to host targets...
  Rendered 1 skills into .agent-harness/render-targets/
  Updated host projections successfully.
```

---

## 5. Step 4: Verify Projected Output

Inspect the rendered output to ensure the canonical notice and managed block delimiters are preserved:

```bash
cat .agent-harness/render-targets/skills.md
```

You will see `json-safety` formatted into the canonical skill bundle, ready to be ingested by host tools (such as Claude Code or Antigravity).

---

## 6. Next Steps

- Consult the [Skill Ingestion Specification](../specs/0007-skill-ingestion-and-scan-gate.md) for automated scan gate requirements.
- Review [Host Adapters & Projection](../subsystems/host-adapters-and-projection.md) to understand how skills project into native tool formats.
