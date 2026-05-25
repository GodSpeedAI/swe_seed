#!/usr/bin/env python3
"""Minimal dependency-free CLI for the agentic SWE harness scaffold."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import re
import shutil
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HARNESS_ROOT = ROOT / ".agent-harness"
TRACE_RECORDS_ROOT = HARNESS_ROOT / "traces" / "records"
ROUTE_DECISIONS_ROOT = HARNESS_ROOT / "traces" / "route-decisions"

REQUIRED_SKILL_FIELDS = {
    "id",
    "version",
    "category",
    "jtbd",
    "description",
    "triggers",
    "procedure",
    "evidence_required",
    "forbidden_behaviors",
    "outputs",
    "success_criteria",
}

REQUIRED_JOB_TYPES = {
    "research",
    "spec",
    "implementation",
    "bugfix",
    "refactor",
    "test",
    "review",
    "release",
    "documentation",
    "harness_improvement",
    "skill_authoring",
}

REQUIRED_PATHS = [
    "AGENTS.md",
    "HARNESS_SPEC.md",
    ".github/copilot-instructions.md",
    ".agent-harness/config.yaml",
    ".agent-harness/context/README.md",
    ".agent-harness/context/budget-policy.yaml",
    ".agent-harness/context/context-mode-normalization.md",
    ".agent-harness/hooks/hook-router.sh",
    ".agent-harness/imports/9arm-skills-normalization.md",
    ".agent-harness/imports/anthropic-skills-skill-creator-normalization.md",
    ".agent-harness/imports/hermes-agent-normalization.md",
    ".agent-harness/traces/README.md",
    ".agent-harness/traces/route-decisions/README.md",
    ".agent-harness/traces/traceability-template.yaml",
    ".agent-harness/reflections/reflection-template.yaml",
    ".agent-harness/reflections/learning-review-template.yaml",
    ".agent-harness/reflections/harness-improvement-proposals.md",
    ".agent-harness/evals/negative-conformance.md",
    ".agent-harness/evals/route-conflicts.md",
    "docs/specs/agentic-swe-harness.md",
    "docs/specs/skill-ir.md",
    "docs/specs/verification-system.md",
    "docs/specs/memory-system.md",
    "docs/specs/hook-strategy.md",
]

REQUIRED_MEMORY = [
    "repo-map.md",
    "decisions.md",
    "open-questions.md",
    "failure-patterns.md",
    "successful-patterns.md",
    "glossary.md",
    "constraints.md",
]

MEMORY_MIN_WORDS = 80
MEMORY_REQUIRED_PHRASES = ["Use this when", "Keep in mind"]
BEHAVIOR_SHAPING_PHRASES = [
    "behavior shaping",
    "agent tendency",
    "outcome production",
]

REQUIRED_PLAYBOOKS = [
    "00-orient-and-route.md",
    "10-frame-outcome.md",
    "20-change-with-proof.md",
    "30-debug-from-symptom.md",
    "40-review-for-risk.md",
    "50-capture-learning.md",
]
PLAYBOOK_MIN_WORDS = 120
PLAYBOOK_REQUIRED_PHRASES = ["Use this when", "Composes with", "Stop when"]
HOOK_EVENTS = [
    "session.start",
    "prompt.submit",
    "tool.pre",
    "tool.post",
    "turn.stop",
    "session.end",
]
REFLECTION_MIN_WORDS = 100
REFLECTION_REQUIRED_PHRASES = ["Use this when", "Do not use this for", "Evidence required"]
PROPOSAL_REQUIRED_PHRASES = ["Use this when", "Proposal states", "Not enough evidence"]
LEARNING_REVIEW_REQUIRED_PHRASES = [
    "candidate_memory_updates",
    "candidate_skill_updates",
    "candidate_harness_updates",
    "provenance",
]
EVAL_MIN_CASES = 6
EVAL_REQUIRED_PHRASES = ["Eval Case", "Command", "Expected", "Why it matters"]
NEGATIVE_EVAL_REQUIRED_PHRASES = ["Failure Eval", "Breakage", "Expected failure", "Why it matters"]
ROUTE_CONFLICT_REQUIRED_PHRASES = ["Conflict Eval", "Prompt", "Expected route", "Precedence rule"]
TRACE_REQUIRED_PHRASES = ["Use this when", "Route decision", "Trace record", "Do not store secrets"]
CONTEXT_REQUIRED_PHRASES = [
    "Use this when",
    "context budget",
    "tool-output containment",
    "think in code",
    "session continuity",
]
CONTEXT_AGENTS_PHRASES = [
    "context budget",
    "tool-output containment",
    "think in code",
    "session continuity",
]
CONTEXT_NORMALIZATION_PHRASES = [
    "Source mechanism",
    "Imported invariant",
    "Local artifact",
    "Validation",
]
RENDER_TARGET_MIN_WORDS = 100
RENDER_TARGET_REQUIRED_PHRASES = [
    "Use this when",
    "What to do",
    "Evidence required",
    "Forbidden behavior",
    "Done when",
]
CLAUDE_SKILL_REQUIRED_PHRASES = ["---", "name:", "description:", "#"]
SKILL_RESOURCE_KINDS = {"script", "reference", "asset"}
CANONICAL_RENDER_RULE = (
    "Skill IR JSON is canonical. Render targets are generated projections and must not be edited directly. "
    "If two paths need the same bytes, one must be a symlink."
)
NINEARM_DEBUG_PHRASES = [
    "reliable reproduction",
    "fail path",
    "disprove",
    "breadcrumb ledger",
]
NINEARM_REVIEW_PHRASES = [
    "simpler alternative",
    "trace the actual path",
    "claim vs verification",
]
NINEARM_LEARNING_PHRASES = [
    "post-mortem",
    "root cause",
    "validation coverage",
]

REQUIRED_ROUTE_FIELDS = {
    "id",
    "job_type",
    "purpose",
    "semantic_triggers",
    "positive_examples",
    "negative_examples",
    "required_context",
    "required_skills",
    "work_loop",
    "required_artifacts",
    "proof",
    "done_when",
    "failure_modes",
    "fallback_policy",
}
ROUTE_MIN_ITEMS = {
    "semantic_triggers": 4,
    "positive_examples": 2,
    "negative_examples": 2,
    "required_context": 3,
    "work_loop": 5,
    "required_artifacts": 3,
    "done_when": 3,
    "failure_modes": 2,
}


def fail(message: str) -> int:
    print(f"harness: {message}", file=sys.stderr)
    return 1


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def load_skill(path: Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def skill_paths() -> list[Path]:
    return sorted((HARNESS_ROOT / "skills").glob("**/*.json"))


def route_paths() -> list[Path]:
    return sorted((HARNESS_ROOT / "routes").glob("*.json"))


def load_route(path: Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def slugify(text: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return slug[:48] or "trace"


def tokenize(text: str) -> set[str]:
    return {token for token in re.findall(r"[a-z0-9_]+", text.lower()) if len(token) > 1}


def validate_skill(path: Path) -> list[str]:
    errors: list[str] = []
    try:
        skill = load_skill(path)
    except json.JSONDecodeError as exc:
        return [f"{rel(path)} is invalid JSON: {exc}"]

    missing = sorted(REQUIRED_SKILL_FIELDS - set(skill))
    if missing:
        errors.append(f"{rel(path)} missing fields: {', '.join(missing)}")

    if not isinstance(skill.get("description"), str) or not skill.get("description", "").strip():
        errors.append(f"{rel(path)} description must be a non-empty string")

    trigger_contexts = skill.get("trigger_contexts")
    if trigger_contexts is not None:
        if not isinstance(trigger_contexts, list) or not trigger_contexts:
            errors.append(f"{rel(path)} trigger_contexts must be a non-empty list when present")
        elif any(not isinstance(item, str) or not item.strip() for item in trigger_contexts):
            errors.append(f"{rel(path)} trigger_contexts entries must be non-empty strings")

    bundled_resources = skill.get("bundled_resources")
    if bundled_resources is not None:
        if not isinstance(bundled_resources, list) or not bundled_resources:
            errors.append(f"{rel(path)} bundled_resources must be a non-empty list when present")
        else:
            for index, resource in enumerate(bundled_resources, start=1):
                if not isinstance(resource, dict):
                    errors.append(f"{rel(path)} bundled_resources[{index}] must be an object")
                    continue
                missing_resource_fields = {
                    field for field in ("kind", "path", "purpose", "when") if field not in resource
                }
                if missing_resource_fields:
                    fields = ", ".join(sorted(missing_resource_fields))
                    errors.append(
                        f"{rel(path)} bundled_resources[{index}] missing fields: {fields}"
                    )
                kind = resource.get("kind")
                if kind not in SKILL_RESOURCE_KINDS:
                    errors.append(
                        f"{rel(path)} bundled_resources[{index}] kind must be one of: "
                        + ", ".join(sorted(SKILL_RESOURCE_KINDS))
                    )
                for field in ("path", "purpose", "when"):
                    if not isinstance(resource.get(field), str) or not resource.get(field, "").strip():
                        errors.append(
                            f"{rel(path)} bundled_resources[{index}] {field} must be a non-empty string"
                        )

    evaluation_prompts = skill.get("evaluation_prompts")
    if evaluation_prompts is not None:
        if not isinstance(evaluation_prompts, list) or not evaluation_prompts:
            errors.append(f"{rel(path)} evaluation_prompts must be a non-empty list when present")
        else:
            for index, prompt in enumerate(evaluation_prompts, start=1):
                if not isinstance(prompt, dict):
                    errors.append(f"{rel(path)} evaluation_prompts[{index}] must be an object")
                    continue
                missing_prompt_fields = {
                    field for field in ("id", "prompt", "checks") if field not in prompt
                }
                if missing_prompt_fields:
                    fields = ", ".join(sorted(missing_prompt_fields))
                    errors.append(
                        f"{rel(path)} evaluation_prompts[{index}] missing fields: {fields}"
                    )
                if not isinstance(prompt.get("id"), str) or not prompt.get("id", "").strip():
                    errors.append(
                        f"{rel(path)} evaluation_prompts[{index}] id must be a non-empty string"
                    )
                if not isinstance(prompt.get("prompt"), str) or not prompt.get("prompt", "").strip():
                    errors.append(
                        f"{rel(path)} evaluation_prompts[{index}] prompt must be a non-empty string"
                    )
                checks = prompt.get("checks")
                if not isinstance(checks, list) or not checks:
                    errors.append(
                        f"{rel(path)} evaluation_prompts[{index}] checks must be a non-empty list"
                    )
                elif any(not isinstance(item, str) or not item.strip() for item in checks):
                    errors.append(
                        f"{rel(path)} evaluation_prompts[{index}] checks entries must be non-empty strings"
                    )

    if skill.get("status") not in {
        "draft",
        "active",
        "deprecated",
        "experimental",
        "imported",
        "candidate",
    }:
        errors.append(f"{rel(path)} has invalid status: {skill.get('status')}")

    skill_id = skill.get("id", "")
    if skill_id != skill_id.lower() or "_" in skill_id or " " in skill_id:
        errors.append(f"{rel(path)} id must be lowercase kebab-case")

    return errors


def validate_route(path: Path) -> list[str]:
    errors: list[str] = []
    try:
        route_card = load_route(path)
    except json.JSONDecodeError as exc:
        return [f"{rel(path)} is invalid JSON: {exc}"]

    missing = sorted(REQUIRED_ROUTE_FIELDS - set(route_card))
    if missing:
        errors.append(f"{rel(path)} missing fields: {', '.join(missing)}")

    route_id = route_card.get("id", "")
    if route_id != path.stem:
        errors.append(f"{rel(path)} id must match filename")

    if route_card.get("job_type") not in REQUIRED_JOB_TYPES:
        errors.append(f"{rel(path)} job_type is not a required job type")

    for field in [
        "semantic_triggers",
        "positive_examples",
        "required_context",
        "work_loop",
        "required_artifacts",
        "proof",
        "done_when",
    ]:
        if not route_card.get(field):
            errors.append(f"{rel(path)} {field} must not be empty")

    for field, minimum in ROUTE_MIN_ITEMS.items():
        if len(route_card.get(field, [])) < minimum:
            errors.append(f"{rel(path)} {field} must have at least {minimum} items")

    if not any("playbooks" in item for item in route_card.get("required_context", [])):
        errors.append(f"{rel(path)} required_context must include a relevant playbook")

    if not any(
        "read the output" in item.lower() or "read proof" in item.lower()
        for item in route_card.get("work_loop", [])
    ):
        errors.append(f"{rel(path)} work_loop must include reading proof output")

    return errors


def render_skill(skill: dict) -> dict[Path, str]:
    skill_ref = f"{skill['id']}@{skill['version']}"
    header = (
        f"Generated from Skill IR: {skill_ref}\n"
        "Do not edit this generated file directly unless this repository intentionally allows "
        "generated-surface edits.\n"
        "Update the Skill IR source instead.\n\n"
    )

    evidence = "\n".join(f"- {item}" for item in skill["evidence_required"])
    forbidden = "\n".join(f"- {item}" for item in skill["forbidden_behaviors"])
    procedure = "\n".join(
        f"{index}. {item}" for index, item in enumerate(skill["procedure"], start=1)
    )
    checklist = "\n".join(f"- [ ] {item}" for item in skill["procedure"])
    success = "\n".join(f"- {item}" for item in skill["success_criteria"])
    triggers = ", ".join(skill["triggers"])
    title = skill["id"].replace("-", " ").title()
    claude_description = skill["description"]

    if skill.get("trigger_contexts"):
        contexts = ", ".join(skill["trigger_contexts"])
        claude_description += f" Use this skill when the request mentions {contexts}."

    bundled_resources = ""
    if skill.get("bundled_resources"):
        resource_lines = []
        for resource in skill["bundled_resources"]:
            resource_lines.append(
                f"- `{resource['path']}` ({resource['kind']}): {resource['purpose']} Use when {resource['when']}."
            )
        bundled_resources = "\n\n## Bundled resources\n\n" + "\n".join(resource_lines)

    evaluation_prompts = ""
    if skill.get("evaluation_prompts"):
        prompt_lines = []
        for prompt in skill["evaluation_prompts"]:
            checks = "; ".join(prompt["checks"])
            prompt_lines.append(
                f"- `{prompt['id']}`: {prompt['prompt']} Check for: {checks}."
            )
        evaluation_prompts = "\n\n## Evaluation prompts\n\n" + "\n".join(prompt_lines)

    return {
        HARNESS_ROOT / "render-targets" / "copilot" / f"{skill['id']}.instructions.md": (
            header
            + f"# {title}\n\n"
            + "## Use this when\n\n"
            + f"Use this when the request or observed state matches these triggers: {triggers}.\n\n"
            + f"Job to be done: {skill['jtbd']}.\n\n"
            + "## What to do\n\n"
            + procedure
            + "\n\n"
            + "## Evidence required\n\n"
            + evidence
            + bundled_resources
            + evaluation_prompts
            + "\n\n## Forbidden behavior\n\n"
            + forbidden
            + "\n\n## Done when\n\n"
            + success
            + "\n\nIf evidence is missing, do not claim completion. State the gap and the next proof command.\n"
        ),
        HARNESS_ROOT / "render-targets" / "hooks" / f"{skill['id']}.prompt.md": (
            header
            + f"# {title} Hook Prompt\n\n"
            + "## Use this when\n\n"
            + f"Use this hook prompt when lifecycle context indicates: {triggers}.\n\n"
            + "## What to do\n\n"
            + f"Require the agent to pursue this outcome: {skill['jtbd']}.\n\n"
            + procedure
            + "\n\n"
            + "## Evidence required\n\n"
            + evidence
            + bundled_resources
            + evaluation_prompts
            + "\n\n## Forbidden behavior\n\n"
            + forbidden
            + "\n\n## Done when\n\n"
            + success
            + "\n\nHook behavior should warn or block only when the agent is about to skip evidence, guess at root cause, or claim completion without proof.\n"
        ),
        HARNESS_ROOT / "render-targets" / "checklists" / f"{skill['id']}.md": (
            header
            + f"# {title} Checklist\n\n"
            + "## Use this when\n\n"
            + f"Use this checklist when triggers match: {triggers}.\n\n"
            + "## What to do\n\n"
            + checklist
            + "\n\n## Evidence required\n\n"
            + evidence
            + bundled_resources
            + evaluation_prompts
            + "\n\n## Forbidden behavior\n\n"
            + forbidden
            + "\n\n## Done when\n\n"
            + success
            + "\n\nBefore final response, every checked item must be backed by observed evidence or a documented skipped-check reason.\n"
        ),
        HARNESS_ROOT / "render-targets" / "claude" / skill["category"] / skill["id"] / "SKILL.md": (
            "---\n"
            + f"name: {skill['id']}\n"
            + f"description: {claude_description}\n"
            + "---\n\n"
            + header
            + f"# {title}\n\n"
            + "## Use this when\n\n"
            + f"Use this when triggers match: {triggers}.\n\n"
            + f"Job to be done: {skill['jtbd']}.\n\n"
            + "## What to do\n\n"
            + procedure
            + "\n\n"
            + "## Evidence required\n\n"
            + evidence
            + bundled_resources
            + evaluation_prompts
            + "\n\n## Forbidden behavior\n\n"
            + forbidden
            + "\n\n## Done when\n\n"
            + success
            + "\n\nThis generated Claude-style skill follows the 9arm-skills shape: one skill directory, one SKILL.md, YAML frontmatter, and behavior-preserving body generated from Skill IR.\n"
        ),
    }


def validate() -> int:
    errors: list[str] = []

    for required in REQUIRED_PATHS:
        if not (ROOT / required).is_file():
            errors.append(f"missing required file: {required}")

    for dirname in [
        "skills",
        "render-targets",
        "hooks",
        "memory",
        "context",
        "traces",
        "evals",
        "reflections",
        "playbooks",
        "imports",
    ]:
        if not (HARNESS_ROOT / dirname).is_dir():
            errors.append(f"missing required directory: .agent-harness/{dirname}")

    if not (HARNESS_ROOT / "routes").is_dir():
        errors.append("missing required directory: .agent-harness/routes")

    for memory in REQUIRED_MEMORY:
        memory_path = HARNESS_ROOT / "memory" / memory
        if not memory_path.is_file():
            errors.append(f"missing memory artifact: .agent-harness/memory/{memory}")
        else:
            text = memory_path.read_text(encoding="utf-8")
            word_count = len(re.findall(r"\b\w+\b", text))
            if word_count < MEMORY_MIN_WORDS:
                errors.append(f"memory artifact too thin: .agent-harness/memory/{memory}")
            for phrase in MEMORY_REQUIRED_PHRASES:
                if phrase not in text:
                    errors.append(f".agent-harness/memory/{memory} missing phrase: {phrase}")

    successful_patterns = HARNESS_ROOT / "memory" / "successful-patterns.md"
    if successful_patterns.is_file():
        text = successful_patterns.read_text(encoding="utf-8").lower()
        for phrase in BEHAVIOR_SHAPING_PHRASES:
            if phrase not in text:
                errors.append(f"successful patterns missing behavior-shaping phrase: {phrase}")

    constraints = HARNESS_ROOT / "memory" / "constraints.md"
    if constraints.is_file():
        text = constraints.read_text(encoding="utf-8").lower()
        for phrase in ["no value if it does not improve outcome", "cheap lever"]:
            if phrase not in text:
                errors.append(f"constraints missing strategic constraint phrase: {phrase}")

    agents = ROOT / "AGENTS.md"
    if agents.is_file():
        text = agents.read_text(encoding="utf-8").lower()
        for phrase in BEHAVIOR_SHAPING_PHRASES:
            if phrase not in text:
                errors.append(f"AGENTS.md missing behavior-shaping phrase: {phrase}")
        for phrase in ["avoid em dashes", "no praise before verification", "avoid should work"]:
            if phrase not in text:
                errors.append(f"AGENTS.md missing prose constraint phrase: {phrase}")
        for phrase in CONTEXT_AGENTS_PHRASES:
            if phrase not in text:
                errors.append(f"AGENTS.md missing context discipline phrase: {phrase}")

    hook_router = HARNESS_ROOT / "hooks" / "hook-router.sh"
    if hook_router.is_file():
        hook_text = hook_router.read_text(encoding="utf-8")
        for event in HOOK_EVENTS:
            if event not in hook_text:
                errors.append(f"hook router missing event: {event}")
        for phrase in ["purpose:", "action:", "boundary:"]:
            if phrase not in hook_text:
                errors.append(f"hook router missing phrase: {phrase}")

    playbook_root = HARNESS_ROOT / "playbooks"
    playbook_files = sorted(
        path.name for path in playbook_root.glob("*.md") if path.name != "README.md"
    )
    if playbook_files != REQUIRED_PLAYBOOKS:
        errors.append(f"playbooks must be exactly: {', '.join(REQUIRED_PLAYBOOKS)}")

    for playbook in REQUIRED_PLAYBOOKS:
        playbook_path = playbook_root / playbook
        if not playbook_path.is_file():
            errors.append(f"missing playbook: .agent-harness/playbooks/{playbook}")
            continue
        text = playbook_path.read_text(encoding="utf-8")
        word_count = len(re.findall(r"\b\w+\b", text))
        if word_count < PLAYBOOK_MIN_WORDS:
            errors.append(f"playbook too thin: .agent-harness/playbooks/{playbook}")
        for phrase in PLAYBOOK_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f".agent-harness/playbooks/{playbook} missing phrase: {phrase}")

    debug_playbook = playbook_root / "30-debug-from-symptom.md"
    if debug_playbook.is_file():
        text = debug_playbook.read_text(encoding="utf-8").lower()
        for phrase in NINEARM_DEBUG_PHRASES:
            if phrase not in text:
                errors.append(f"debug playbook missing 9arm process phrase: {phrase}")

    review_playbook = playbook_root / "40-review-for-risk.md"
    if review_playbook.is_file():
        text = review_playbook.read_text(encoding="utf-8").lower()
        for phrase in NINEARM_REVIEW_PHRASES:
            if phrase not in text:
                errors.append(f"review playbook missing 9arm process phrase: {phrase}")

    learning_playbook = playbook_root / "50-capture-learning.md"
    if learning_playbook.is_file():
        text = learning_playbook.read_text(encoding="utf-8").lower()
        for phrase in NINEARM_LEARNING_PHRASES:
            if phrase not in text:
                errors.append(f"learning playbook missing 9arm process phrase: {phrase}")

    reflection_template = HARNESS_ROOT / "reflections" / "reflection-template.yaml"
    if reflection_template.is_file():
        text = reflection_template.read_text(encoding="utf-8")
        word_count = len(re.findall(r"\b\w+\b", text))
        if word_count < REFLECTION_MIN_WORDS:
            errors.append("reflection template too thin")
        for phrase in REFLECTION_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"reflection template missing phrase: {phrase}")

    learning_review_template = HARNESS_ROOT / "reflections" / "learning-review-template.yaml"
    if learning_review_template.is_file():
        text = learning_review_template.read_text(encoding="utf-8")
        word_count = len(re.findall(r"\b\w+\b", text))
        if word_count < REFLECTION_MIN_WORDS:
            errors.append("learning review template too thin")
        for phrase in LEARNING_REVIEW_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"learning review template missing phrase: {phrase}")

    proposals = HARNESS_ROOT / "reflections" / "harness-improvement-proposals.md"
    if proposals.is_file():
        text = proposals.read_text(encoding="utf-8")
        word_count = len(re.findall(r"\b\w+\b", text))
        if word_count < REFLECTION_MIN_WORDS:
            errors.append("harness improvement proposal ledger too thin")
        for phrase in PROPOSAL_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"harness improvement proposal ledger missing phrase: {phrase}")

    eval_file = HARNESS_ROOT / "evals" / "core-conformance.md"
    if eval_file.is_file():
        text = eval_file.read_text(encoding="utf-8")
        case_count = text.count("## Eval Case")
        if case_count < EVAL_MIN_CASES:
            errors.append(f"core conformance evals need at least {EVAL_MIN_CASES} cases")
        for phrase in EVAL_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"core conformance evals missing phrase: {phrase}")

    negative_eval_file = HARNESS_ROOT / "evals" / "negative-conformance.md"
    if negative_eval_file.is_file():
        text = negative_eval_file.read_text(encoding="utf-8")
        for phrase in NEGATIVE_EVAL_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"negative conformance evals missing phrase: {phrase}")

    route_conflict_file = HARNESS_ROOT / "evals" / "route-conflicts.md"
    if route_conflict_file.is_file():
        text = route_conflict_file.read_text(encoding="utf-8")
        for phrase in ROUTE_CONFLICT_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"route conflict evals missing phrase: {phrase}")

    trace_readme = HARNESS_ROOT / "traces" / "README.md"
    if trace_readme.is_file():
        text = trace_readme.read_text(encoding="utf-8")
        for phrase in TRACE_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"trace README missing phrase: {phrase}")

    imports_note = HARNESS_ROOT / "imports" / "9arm-skills-normalization.md"
    if imports_note.is_file():
        text = imports_note.read_text(encoding="utf-8")
        for phrase in ["Source skill", "Imported invariant", "Local artifact", "Validation"]:
            if phrase not in text:
                errors.append(f"9arm normalization note missing phrase: {phrase}")

    context_readme = HARNESS_ROOT / "context" / "README.md"
    if context_readme.is_file():
        text = context_readme.read_text(encoding="utf-8")
        for phrase in CONTEXT_REQUIRED_PHRASES:
            if phrase not in text:
                errors.append(f"context README missing phrase: {phrase}")

    context_normalization = HARNESS_ROOT / "context" / "context-mode-normalization.md"
    if context_normalization.is_file():
        text = context_normalization.read_text(encoding="utf-8")
        for phrase in CONTEXT_NORMALIZATION_PHRASES:
            if phrase not in text:
                errors.append(f"context-mode normalization missing phrase: {phrase}")

    context_budget = HARNESS_ROOT / "context" / "budget-policy.yaml"
    if context_budget.is_file():
        text = context_budget.read_text(encoding="utf-8")
        for phrase in [
            "raw_output_policy",
            "summarize_before_context",
            "script_bulk_analysis",
            "route_required_context_first",
        ]:
            if phrase not in text:
                errors.append(f"context budget policy missing phrase: {phrase}")

    agents = ROOT / "AGENTS.md"
    if agents.is_file():
        text = agents.read_text(encoding="utf-8")
        for job_type in REQUIRED_JOB_TYPES:
            if job_type not in text:
                errors.append(f"AGENTS.md missing job type: {job_type}")

    seen_ids: set[str] = set()
    for path in skill_paths():
        errors.extend(validate_skill(path))
        try:
            skill_id = load_skill(path)["id"]
        except (json.JSONDecodeError, KeyError):
            continue
        if skill_id in seen_ids:
            errors.append(f"duplicate skill id: {skill_id}")
        seen_ids.add(skill_id)

    if "debug-discipline" not in seen_ids:
        errors.append("missing required active skill: debug-discipline")
    else:
        for path in skill_paths():
            skill = load_skill(path)
            if skill.get("id") == "debug-discipline":
                debug_text = json.dumps(skill).lower()
                for phrase in NINEARM_DEBUG_PHRASES:
                    if phrase not in debug_text:
                        errors.append(f"debug-discipline missing 9arm process phrase: {phrase}")
                break

    seen_routes: set[str] = set()
    for path in route_paths():
        errors.extend(validate_route(path))
        try:
            route_card = load_route(path)
        except json.JSONDecodeError:
            continue
        route_id = route_card.get("id")
        if route_id in seen_routes:
            errors.append(f"duplicate route id: {route_id}")
        seen_routes.add(route_id)

    for job_type in REQUIRED_JOB_TYPES:
        if job_type not in seen_routes:
            errors.append(f"missing route card for job type: {job_type}")

    for path in skill_paths():
        rendered = render_skill(load_skill(path))
        for target, expected in rendered.items():
            if not target.is_file():
                errors.append(f"missing rendered artifact: {rel(target)}")
            elif target.read_text(encoding="utf-8") != expected:
                errors.append(f"stale rendered artifact: {rel(target)}")
            else:
                text = target.read_text(encoding="utf-8")
                word_count = len(re.findall(r"\b\w+\b", text))
                if word_count < RENDER_TARGET_MIN_WORDS:
                    errors.append(f"rendered artifact too thin: {rel(target)}")
                for phrase in RENDER_TARGET_REQUIRED_PHRASES:
                    if phrase not in text:
                        errors.append(f"{rel(target)} missing phrase: {phrase}")
                if target.name == "SKILL.md":
                    for phrase in CLAUDE_SKILL_REQUIRED_PHRASES:
                        if phrase not in text:
                            errors.append(f"{rel(target)} missing Claude skill phrase: {phrase}")

    render_files = [
        path for path in (HARNESS_ROOT / "render-targets").glob("**/*") if path.is_file()
    ]
    regular_by_content: dict[str, Path] = {}
    for path in render_files:
        if path.is_symlink():
            continue
        text = path.read_text(encoding="utf-8")
        previous = regular_by_content.get(text)
        if previous is not None:
            errors.append(
                f"duplicate render target content must use symlink: {rel(previous)} and {rel(path)}"
            )
        regular_by_content[text] = path

    for path in render_files:
        if path.suffix == ".md":
            text = path.read_text(encoding="utf-8")
            if (
                "Generated from Skill IR:" in text
                and "Update the Skill IR source instead." not in text
            ):
                errors.append(
                    f"generated render target missing canonical-source notice: {rel(path)}"
                )

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    print("Harness validation passed")
    return 0


def doctor() -> int:
    missing = [command for command in ("git", "just", "python") if shutil.which(command) is None]
    if missing:
        return fail(f"missing required commands: {', '.join(missing)}")

    if not (ROOT / ".github/copilot-instructions.md").is_file():
        return fail("missing Copilot entry instruction")

    if not (HARNESS_ROOT / "traces").is_dir():
        return fail("trace directory is not writable")

    print("Harness doctor passed")
    return 0


def render_skills() -> int:
    paths = skill_paths()
    if not paths:
        return fail("no Skill IR files found")

    for path in paths:
        for target, content in render_skill(load_skill(path)).items():
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
            print(f"rendered {rel(target)}")

    return 0


def score_route(task: str, route_card: dict) -> int:
    task_text = task.lower()
    task_tokens = tokenize(task)
    score = 0

    for phrase in route_card["semantic_triggers"]:
        phrase_lower = phrase.lower()
        if phrase_lower in task_text:
            score += 8
        score += len(task_tokens & tokenize(phrase)) * 2

    for example in route_card["positive_examples"]:
        overlap = task_tokens & tokenize(example)
        if example.lower() in task_text:
            score += 3
        elif len(overlap) >= 2:
            score += len(overlap)

    for example in route_card["negative_examples"]:
        overlap = task_tokens & tokenize(example)
        if example.lower() in task_text:
            score -= 4
        elif len(overlap) >= 2:
            score -= len(overlap) * 2

    if route_card["job_type"] in task_text:
        score += 5

    return score


def infer_bootstrap_route(task: str, cards: list[dict]) -> dict | None:
    task_text = task.lower()
    task_tokens = tokenize(task)
    build_verbs = {"build", "create", "make", "add", "start", "scaffold"}
    clarification_nouns = {"spec", "requirements", "contract", "acceptance", "criteria", "plan"}
    direct_change_terms = {
        "fix",
        "debug",
        "review",
        "audit",
        "refactor",
        "release",
        "document",
        "docs",
        "test",
        "tests",
        "harness",
        "router",
        "skill",
    }

    if not (task_tokens & build_verbs):
        return None
    if task_tokens & clarification_nouns:
        return None
    if task_tokens & direct_change_terms:
        return None
    if re.search(r"\.(py|ts|tsx|js|jsx|md|json|yaml|yml|sh)\b", task_text):
        return None

    for card in cards:
        if card["job_type"] == "spec":
            return card
    return None


def build_route_result(task: str) -> dict:
    cards = [load_route(path) for path in route_paths()]
    if not cards:
        raise RuntimeError("no route cards found")

    ranked = sorted(cards, key=lambda card: (score_route(task, card), card["id"]), reverse=True)
    selected = ranked[0]
    selected_score = score_route(task, selected)
    if selected_score <= 0:
        bootstrap_card = infer_bootstrap_route(task, cards)
        if bootstrap_card is not None:
            selected = bootstrap_card
            selected_score = 1
    confidence = "medium" if selected_score > 0 else "low"
    assumption = (
        None if selected_score > 0 else "No strong semantic match; selected safest default route."
    )

    result = {
        "job_type": selected["job_type"],
        "route_card": rel(HARNESS_ROOT / "routes" / f"{selected['id']}.json"),
        "confidence": confidence,
        "assumption": assumption,
        "required_context": selected["required_context"],
        "required_skills": selected["required_skills"],
        "work_loop": selected["work_loop"],
        "required_artifacts": selected["required_artifacts"],
        "proof": selected["proof"],
        "done_when": selected["done_when"],
        "next_action": f"Read required context, then execute work_loop[0]: {selected['work_loop'][0]}",
    }
    return result


def write_route_decision(task: str, result: dict) -> Path:
    ROUTE_DECISIONS_ROOT.mkdir(parents=True, exist_ok=True)
    trace_id = f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}-{slugify(task)}"
    decision = {
        "trace_id": trace_id,
        "created_at": utc_now(),
        "task": task,
        "route": result,
        "decision_basis": "deterministic token overlap against route-card triggers, examples, and job type",
    }
    path = ROUTE_DECISIONS_ROOT / f"{trace_id}.json"
    path.write_text(json.dumps(decision, indent=2) + "\n", encoding="utf-8")
    return path


def route(task: str, record: bool = False) -> int:
    try:
        result = build_route_result(task)
    except RuntimeError as exc:
        return fail(str(exc))

    if record:
        result["route_decision_record"] = rel(write_route_decision(task, result))
    print(json.dumps(result, indent=2))
    return 0


def inspect(item: str) -> int:
    matches: dict[str, object] = {
        "query": item,
        "skills": [],
        "routes": [],
        "render_targets": [],
        "evals": [],
        "memory": [],
        "context": [],
    }

    query = item.lower()

    for path in skill_paths():
        skill = load_skill(path)
        text = json.dumps(skill).lower()
        if query in skill.get("id", "").lower() or query in text:
            matches["skills"].append(
                {"id": skill.get("id"), "path": rel(path), "status": skill.get("status")}
            )
            for target in render_skill(skill):
                matches["render_targets"].append(rel(target))

    for path in route_paths():
        route_card = load_route(path)
        text = json.dumps(route_card).lower()
        if (
            query in route_card.get("id", "").lower()
            or query in route_card.get("job_type", "").lower()
            or query in text
        ):
            matches["routes"].append(
                {
                    "id": route_card.get("id"),
                    "job_type": route_card.get("job_type"),
                    "path": rel(path),
                    "proof": route_card.get("proof", []),
                    "next_action": route_card.get("work_loop", [""])[0],
                }
            )

    for root_name in ("evals", "memory", "context"):
        for path in sorted((HARNESS_ROOT / root_name).glob("*.md")):
            text = path.read_text(encoding="utf-8").lower()
            if query in path.name.lower() or query in text:
                matches[root_name].append(rel(path))

    if not any(
        matches[key] for key in ("skills", "routes", "render_targets", "evals", "memory", "context")
    ):
        return fail(f"no harness item matched: {item}")

    print(json.dumps(matches, indent=2))
    return 0


def context_plan(task: str) -> int:
    try:
        route_result = build_route_result(task)
    except RuntimeError as exc:
        return fail(str(exc))

    context_files = [
        ".agent-harness/context/README.md",
        ".agent-harness/context/budget-policy.yaml",
        *route_result["required_context"],
    ]
    deduped_context = list(dict.fromkeys(context_files))
    result = {
        "task": task,
        "route": {
            "job_type": route_result["job_type"],
            "route_card": route_result["route_card"],
            "confidence": route_result["confidence"],
        },
        "context_budget": {
            "read_order": deduped_context,
            "raw_output_policy": "Do not paste large raw command output into the conversation. Summarize the finding and keep full output in a file or trace note when it matters.",
            "tool_output_containment": "Prefer commands that emit counts, paths, JSON fields, or focused excerpts over broad file dumps.",
            "think_in_code": "For bulk analysis, write or run a small script or pipeline and bring back only the computed result.",
            "session_continuity": "Use trace records for durable decisions, proof, unresolved risks, and restart context.",
        },
        "next_action": "Read context in order, then execute the selected route work loop.",
    }
    print(json.dumps(result, indent=2))
    return 0


def resolve_trace_path(trace: str) -> Path:
    candidate = Path(trace)
    if candidate.is_file():
        return candidate.resolve()
    if not trace.endswith(".json"):
        trace = f"{trace}.json"
    return TRACE_RECORDS_ROOT / trace


def trace_start(task: str) -> int:
    try:
        route_result = build_route_result(task)
    except RuntimeError as exc:
        return fail(str(exc))

    TRACE_RECORDS_ROOT.mkdir(parents=True, exist_ok=True)
    trace_id = f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}-{slugify(task)}"
    route_decision_path = write_route_decision(task, route_result)
    record = {
        "trace_id": trace_id,
        "created_at": utc_now(),
        "task": task,
        "route_decision_record": rel(route_decision_path),
        "route": route_result,
        "events": [
            {
                "at": utc_now(),
                "type": "trace.start",
                "note": "Trace record created before implementation work.",
            }
        ],
        "verification": [],
        "unresolved_risks": [],
        "completion_claim": None,
    }
    path = TRACE_RECORDS_ROOT / f"{trace_id}.json"
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                "trace_id": trace_id,
                "trace_record": rel(path),
                "route_decision_record": rel(route_decision_path),
            },
            indent=2,
        )
    )
    return 0


def trace_append(trace: str, note: str) -> int:
    path = resolve_trace_path(trace)
    if not path.is_file():
        return fail(f"trace record not found: {trace}")
    record = json.loads(path.read_text(encoding="utf-8"))
    record.setdefault("events", []).append({"at": utc_now(), "type": "trace.note", "note": note})
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"trace_record": rel(path), "events": len(record["events"])}, indent=2))
    return 0


def trace_checkpoint(
    trace: str,
    stage: str,
    summary: str,
    next_action: str | None,
    artifacts: list[str],
    unresolved_risks: list[str],
) -> int:
    path = resolve_trace_path(trace)
    if not path.is_file():
        return fail(f"trace record not found: {trace}")

    record = json.loads(path.read_text(encoding="utf-8"))
    checkpoint = {
        "at": utc_now(),
        "type": "trace.checkpoint",
        "stage": stage,
        "summary": summary,
        "next_action": next_action,
        "artifacts": artifacts,
        "unresolved_risks": unresolved_risks,
    }
    record.setdefault("events", []).append(checkpoint)
    if unresolved_risks:
        record["unresolved_risks"] = unresolved_risks
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"trace_record": rel(path), "checkpoint": checkpoint}, indent=2))
    return 0


def trace_resume(trace: str) -> int:
    path = resolve_trace_path(trace)
    if not path.is_file():
        return fail(f"trace record not found: {trace}")

    record = json.loads(path.read_text(encoding="utf-8"))
    checkpoints = [
        event
        for event in record.get("events", [])
        if isinstance(event, dict) and event.get("type") == "trace.checkpoint"
    ]
    latest_checkpoint = checkpoints[-1] if checkpoints else None
    print(
        json.dumps(
            {
                "trace_id": record.get("trace_id"),
                "task": record.get("task"),
                "route": record.get("route", {}),
                "latest_checkpoint": latest_checkpoint,
                "unresolved_risks": record.get("unresolved_risks", []),
                "completion_claim": record.get("completion_claim"),
            },
            indent=2,
        )
    )
    return 0


def trace_finish(trace: str, claim: str, command: str | None, result: str | None) -> int:
    path = resolve_trace_path(trace)
    if not path.is_file():
        return fail(f"trace record not found: {trace}")
    record = json.loads(path.read_text(encoding="utf-8"))
    record["completion_claim"] = {"at": utc_now(), "claim": claim}
    if command:
        record.setdefault("verification", []).append(
            {"at": utc_now(), "command": command, "result": result or "not recorded"}
        )
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {"trace_record": rel(path), "completion_claim": record["completion_claim"]}, indent=2
        )
    )
    return 0


def classify_verification_status(record: dict) -> str:
    verification = record.get("verification", [])
    unresolved = record.get("unresolved_risks", [])
    if not verification:
        return "missing"
    if unresolved:
        return "partial"

    verification_text = " ".join(
        str(item.get("result", "")) for item in verification if isinstance(item, dict)
    ).lower()
    success_markers = ("exit 0", "passed", "success", "all checks passed")
    return "verified" if any(marker in verification_text for marker in success_markers) else "partial"


def trace_distill(trace: str) -> int:
    path = resolve_trace_path(trace)
    if not path.is_file():
        return fail(f"trace record not found: {trace}")

    record = json.loads(path.read_text(encoding="utf-8"))
    route = record.get("route", {})
    job_type = route.get("job_type", "unknown")
    route_card = route.get("route_card", "")
    latest_checkpoint = next(
        (
            event
            for event in reversed(record.get("events", []))
            if isinstance(event, dict) and event.get("type") == "trace.checkpoint"
        ),
        None,
    )
    completion_claim = record.get("completion_claim") or {}
    summary = (
        (latest_checkpoint or {}).get("summary")
        or completion_claim.get("claim")
        or record.get("task", "")
    )
    verification_status = classify_verification_status(record)
    unresolved_risks = record.get("unresolved_risks", [])
    checkpoint_artifacts = (latest_checkpoint or {}).get("artifacts", [])
    checkpoint_next_action = (latest_checkpoint or {}).get("next_action")

    evidence = [
        f"trace:{record.get('trace_id', '')}",
        f"route:{route_card}",
    ]
    evidence.extend(f"artifact:{artifact}" for artifact in checkpoint_artifacts)

    if verification_status == "verified" and not unresolved_risks:
        candidate_memory_updates = [
            {
                "destination": "successful-patterns",
                "note": f"{job_type} trace reached proof-backed completion: {summary}",
                "rationale": "Preserve the validated pattern without storing the whole transcript.",
                "evidence": evidence,
            }
        ]
    else:
        destination = "open-questions" if unresolved_risks else "failure-patterns"
        candidate_memory_updates = [
            {
                "destination": destination,
                "note": f"{job_type} trace ended with incomplete proof or unresolved risk: {summary}",
                "rationale": "Carry forward the blocker so the next agent sees the gap without rereading the transcript.",
                "evidence": evidence + [f"risk:{risk}" for risk in unresolved_risks],
            }
        ]

    required_skills = route.get("required_skills", [])
    if required_skills:
        candidate_skill_updates = [
            {
                "priority": 1,
                "action": "patch_existing_skill",
                "target": required_skills[0],
                "summary": f"Patch the governing skill first if this trace exposed a reusable lesson: {summary}",
                "rationale": "Prefer updating the active governing skill before creating a new one.",
                "evidence": evidence,
            }
        ]
    else:
        candidate_skill_updates = [
            {
                "priority": 2,
                "action": "add_support_file",
                "target": "existing umbrella skill or future skill-authoring target",
                "summary": f"If this trace revealed reusable detail, prefer a reference, template, or script before a new standalone skill: {summary}",
                "rationale": "Support files capture durable operational detail with less library sprawl than creating a new skill.",
                "evidence": evidence,
            }
        ]

    candidate_harness_updates = []
    if job_type == "harness_improvement" or any(
        artifact.startswith(".agent-harness/") or artifact == "HARNESS_SPEC.md"
        for artifact in checkpoint_artifacts
    ):
        candidate_harness_updates.append(
            {
                "target": "spec" if "HARNESS_SPEC.md" in checkpoint_artifacts else "validation",
                "summary": f"Review whether the trace lesson should be promoted into the harness contract: {summary}",
                "rationale": "Harness-facing work should convert repeated friction into a validated contract rather than a one-off note.",
                "validation": [
                    {
                        "command": "python scripts/harness.py validate",
                        "expected": "passes with the promoted contract",
                    },
                    {
                        "command": "bash tests/validate-harness.sh",
                        "expected": "smoke checks cover the new behavior",
                    },
                ],
            }
        )

    learning_review = {
        "trace_id": record.get("trace_id"),
        "route": {"job_type": job_type, "route_card": route_card},
        "summary": summary,
        "verification_status": verification_status,
        "candidate_memory_updates": candidate_memory_updates,
        "candidate_skill_updates": candidate_skill_updates,
        "candidate_harness_updates": candidate_harness_updates,
        "compression_handoff": {
            "preserve_for_resume": [
                summary,
                *([checkpoint_next_action] if checkpoint_next_action else []),
                *[f"risk: {risk}" for risk in unresolved_risks],
            ]
        },
        "provenance": {
            "generated_by": "trace.distill",
            "generated_at": utc_now(),
            "trace_record": rel(path),
            "route_decision_record": record.get("route_decision_record"),
        },
    }
    print(json.dumps({"learning_review": learning_review}, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(prog="harness")
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("validate")
    subparsers.add_parser("doctor")
    subparsers.add_parser("render-skills")
    route_parser = subparsers.add_parser("route")
    route_parser.add_argument(
        "--record", action="store_true", help="write a route decision ledger entry"
    )
    route_parser.add_argument("task")
    inspect_parser = subparsers.add_parser("inspect")
    inspect_parser.add_argument("item")
    context_plan_parser = subparsers.add_parser("context-plan")
    context_plan_parser.add_argument("task")
    trace_parser = subparsers.add_parser("trace")
    trace_subparsers = trace_parser.add_subparsers(dest="trace_command", required=True)
    trace_start_parser = trace_subparsers.add_parser("start")
    trace_start_parser.add_argument("task")
    trace_append_parser = trace_subparsers.add_parser("append")
    trace_append_parser.add_argument("trace")
    trace_append_parser.add_argument("note")
    trace_checkpoint_parser = trace_subparsers.add_parser("checkpoint")
    trace_checkpoint_parser.add_argument("trace")
    trace_checkpoint_parser.add_argument("--stage", required=True)
    trace_checkpoint_parser.add_argument("--summary", required=True)
    trace_checkpoint_parser.add_argument("--next-action")
    trace_checkpoint_parser.add_argument("--artifact", action="append", default=[])
    trace_checkpoint_parser.add_argument("--risk", action="append", default=[])
    trace_resume_parser = trace_subparsers.add_parser("resume")
    trace_resume_parser.add_argument("trace")
    trace_distill_parser = trace_subparsers.add_parser("distill")
    trace_distill_parser.add_argument("trace")
    trace_finish_parser = trace_subparsers.add_parser("finish")
    trace_finish_parser.add_argument("trace")
    trace_finish_parser.add_argument("--claim", required=True)
    trace_finish_parser.add_argument("--command", dest="verification_command")
    trace_finish_parser.add_argument("--result")

    args = parser.parse_args()
    if args.command == "validate":
        return validate()
    if args.command == "doctor":
        return doctor()
    if args.command == "render-skills":
        return render_skills()
    if args.command == "route":
        return route(args.task, record=args.record)
    if args.command == "inspect":
        return inspect(args.item)
    if args.command == "context-plan":
        return context_plan(args.task)
    if args.command == "trace":
        if args.trace_command == "start":
            return trace_start(args.task)
        if args.trace_command == "append":
            return trace_append(args.trace, args.note)
        if args.trace_command == "checkpoint":
            return trace_checkpoint(
                args.trace,
                args.stage,
                args.summary,
                args.next_action,
                args.artifact,
                args.risk,
            )
        if args.trace_command == "resume":
            return trace_resume(args.trace)
        if args.trace_command == "distill":
            return trace_distill(args.trace)
        if args.trace_command == "finish":
            return trace_finish(args.trace, args.claim, args.verification_command, args.result)

    return fail(f"unknown command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
