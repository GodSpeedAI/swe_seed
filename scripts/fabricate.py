#!/usr/bin/env python3
"""Reference fabricate CLI for bounded product-to-prototype runs."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import re
import shutil
import sys
from pathlib import Path

from harness import _parse_yaml


ROOT = Path(__file__).resolve().parents[1]
FABRICATOR_ROOT = ROOT / ".fabricator"
FABRICATOR_CONFIG_PATH = FABRICATOR_ROOT / "config.yaml"
TEMPLATES_ROOT = FABRICATOR_ROOT / "templates"
RUNS_ROOT = FABRICATOR_ROOT / "runs"

REQUIRED_SEED_FIELDS = (
    "product_type",
    "user",
    "situation",
    "desired_outcome",
    "prototype_goal",
)

DEFAULT_REQUIRED_RUN_ARTIFACTS = (
    "PRODUCT_SEED.md",
    "JTBD.md",
    "JOB_HYPOTHESIS.md",
    "HYPOTHESIS.md",
    "ADR.md",
    "PRD.md",
    "SDS.md",
    "TDD.md",
    "CONTEXT_PACK.md",
    "AGENT_TASK.md",
    "EVAL_CHECKLIST.md",
    "EVAL_SPEC.yaml",
    "PROOF_RECORD.md",
    "REFLECTION.md",
    "ADAPTATION_DECISION.yaml",
    "EVAL_RESULT.json",
    "NO_SKILL_PROPOSED.md",
)

SKILL_DECISION_ARTIFACTS = ("SKILL_PROPOSAL.yaml", "NO_SKILL_PROPOSED.md")
YAML_PLAIN_SCALAR_KEYWORDS = {
    "y",
    "yes",
    "n",
    "no",
    "true",
    "false",
    "on",
    "off",
    "null",
    "~",
}

HTML5_COMPONENTS = (
    (
        "CMP-001",
        "BootstrapShell",
        "Initialize the single-file application, canvas, HUD, and deterministic state.",
        "REQ-001, REQ-004",
    ),
    (
        "CMP-002",
        "InputController",
        "Capture keyboard input and convert it into player movement without network dependencies.",
        "REQ-002",
    ),
    (
        "CMP-003",
        "WorldState",
        "Track player position, tokens, distractions, score, timer, and round state.",
        "REQ-002, REQ-003, REQ-004",
    ),
    (
        "CMP-004",
        "CollisionAndScoring",
        "Resolve token collection, distraction collisions, score changes, and win or loss thresholds.",
        "REQ-003, REQ-005",
    ),
    (
        "CMP-005",
        "RendererHUD",
        "Render the board, HUD, timer, state messaging, and restart affordance.",
        "REQ-004, REQ-005",
    ),
)

HTML5_REQUIREMENTS = (
    (
        "REQ-001",
        "Ubiquitous",
        "The system shall run locally from a single index.html file without a build step, backend, or network access.",
        "game opens locally",
    ),
    (
        "REQ-002",
        "Event-driven",
        "When the player presses movement keys, the system shall move the focus avatar within the play area.",
        "player can move",
    ),
    (
        "REQ-003",
        "Event-driven",
        "When the focus avatar collects a focus token or collides with a distraction, the system shall update score and lives immediately.",
        "score changes",
    ),
    (
        "REQ-004",
        "State-driven",
        "While a round is active, the system shall show score, lives, timer, and current state in the same document.",
        "score changes",
    ),
    (
        "REQ-005",
        "Event-driven",
        "When a win or loss condition is reached, the system shall show the outcome and allow the player to restart the round.",
        "win/loss condition works; restart works",
    ),
)

HTML5_SCENARIOS = (
    (
        "SCN-001",
        "REQ-001",
        "Given the player opens the local index.html file in a browser\nWhen the document initializes\nThen the game shell loads without external dependencies",
        "local boot",
    ),
    (
        "SCN-002",
        "REQ-002",
        "Given the round is active\nWhen the player presses arrow keys or WASD\nThen the focus avatar moves within the arena",
        "movement",
    ),
    (
        "SCN-003",
        "REQ-003",
        "Given the avatar overlaps a focus token or distraction\nWhen collision resolution runs\nThen score or lives update and the entity respawns",
        "score and collision",
    ),
    (
        "SCN-004",
        "REQ-004",
        "Given the round timer is running\nWhen each frame renders\nThen the HUD shows score, lives, timer, and state text",
        "hud state",
    ),
    (
        "SCN-005",
        "REQ-005",
        "Given the player wins, loses, or time expires\nWhen the round ends\nThen the outcome message appears and restart begins a fresh round",
        "restart flow",
    ),
)

HTML5_CHECKS = (
    (
        "CHK-001",
        "SCN-001",
        "Static HTML scan confirms a single local file with no network calls.",
        "static_html_scan",
    ),
    (
        "CHK-002",
        "SCN-002",
        "Static HTML scan confirms keyboard listeners and movement logic.",
        "static_html_scan",
    ),
    (
        "CHK-003",
        "SCN-003",
        "Static HTML scan confirms collision and scoring logic.",
        "static_html_scan",
    ),
    (
        "CHK-004",
        "SCN-004",
        "Static HTML scan confirms HUD and timer rendering.",
        "static_html_scan",
    ),
    (
        "CHK-005",
        "SCN-005",
        "Static HTML scan confirms end-state and restart behavior.",
        "static_html_scan",
    ),
)

PLACEHOLDER_RE = re.compile(r"\{\{\s*([a-z0-9_]+)\s*\}\}")


def fail(message: str) -> int:
    print(f"fabricate: {message}", file=sys.stderr)
    return 1


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def slugify(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "fabrication-run"


def load_mapping(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    try:
        loaded = json.loads(text)
    except json.JSONDecodeError:
        loaded = _parse_yaml(text)
    if not isinstance(loaded, dict):
        raise ValueError(f"{path} must contain an object")
    return loaded


def load_fabricator_config() -> dict:
    try:
        return load_mapping(FABRICATOR_CONFIG_PATH)
    except (OSError, ValueError, json.JSONDecodeError):
        return {}


def dump_yaml(value: object, indent: int = 0) -> str:
    pad = "  " * indent
    if isinstance(value, dict):
        lines: list[str] = []
        for key, item in value.items():
            if isinstance(item, (dict, list)):
                lines.append(f"{pad}{key}:")
                lines.append(dump_yaml(item, indent + 1))
            else:
                lines.append(f"{pad}{key}: {yaml_scalar(item)}")
        return "\n".join(lines)
    if isinstance(value, list):
        lines = []
        for item in value:
            if isinstance(item, (dict, list)):
                nested_lines = dump_yaml(item, indent + 1).splitlines()
                lines.append(f"{pad}- {nested_lines[0].strip()}")
                lines.extend(nested_lines[1:])
            else:
                lines.append(f"{pad}- {yaml_scalar(item)}")
        return "\n".join(lines)
    return f"{pad}{yaml_scalar(value)}"


def yaml_scalar(value: object) -> str:
    if value is None:
        return '""'
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    text = str(value)
    if text.lower() in YAML_PLAIN_SCALAR_KEYWORDS:
        escaped = text.replace('"', '\\"')
        return f'"{escaped}"'
    if not text or any(char in text for char in ':#[]{}\n\"'):
        escaped = text.replace('"', '\\"')
        return f'"{escaped}"'
    return text


def ensure_directory(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def write_text(path: Path, content: str) -> None:
    ensure_directory(path.parent)
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def render_template(template_name: str, values: dict[str, str]) -> str:
    template = (TEMPLATES_ROOT / template_name).read_text(encoding="utf-8")
    missing = sorted(set(PLACEHOLDER_RE.findall(template)) - set(values))
    if missing:
        raise ValueError(f"template {template_name} missing values: {', '.join(missing)}")
    return PLACEHOLDER_RE.sub(lambda match: values[match.group(1)], template)


def bullet_list(items: list[str]) -> str:
    return "\n".join(f"- {item}" for item in items) if items else "- none"


def normalize_seed(seed: dict) -> dict:
    missing = [field for field in REQUIRED_SEED_FIELDS if not str(seed.get(field, "")).strip()]
    if missing:
        raise ValueError(f"seed missing required fields: {', '.join(missing)}")

    constraints = [str(item).strip() for item in seed.get("constraints", []) if str(item).strip()]
    non_goals = [str(item).strip() for item in seed.get("non_goals", []) if str(item).strip()]
    success_conditions = [
        str(item).strip() for item in seed.get("success_conditions", []) if str(item).strip()
    ]
    proof_expectations = [
        str(item).strip() for item in seed.get("proof_expectations", []) if str(item).strip()
    ]

    run_id = str(seed.get("run_id", "")).strip() or slugify(str(seed["prototype_goal"]))
    normalized = {
        "run_id": run_id,
        "product_type": str(seed["product_type"]).strip(),
        "user": str(seed["user"]).strip(),
        "situation": str(seed["situation"]).strip(),
        "desired_outcome": str(seed["desired_outcome"]).strip(),
        "prototype_goal": str(seed["prototype_goal"]).strip(),
        "constraints": constraints,
        "non_goals": non_goals,
        "success_conditions": success_conditions,
        "proof_expectations": proof_expectations,
    }
    return normalized


def run_paths(run_id: str) -> dict[str, Path]:
    run_root = RUNS_ROOT / run_id
    return {
        "run_root": run_root,
        "input": run_root / "input.yaml",
        "generated": run_root / "generated",
        "prototype": run_root / "prototype",
        "proof": run_root / "proof",
        "handoff": run_root / "handoff",
        "trace": run_root / "trace.md",
    }


def append_trace(run_id: str, event: str, detail: str) -> None:
    trace_path = run_paths(run_id)["trace"]
    entry = f"- {utc_now()} | {event} | {detail}\n"
    ensure_directory(trace_path.parent)
    needs_header = not trace_path.exists() or trace_path.stat().st_size == 0
    with trace_path.open("a", encoding="utf-8") as handle:
        if needs_header:
            handle.write("# Fabrication Trace\n\n")
        handle.write(entry)


def configured_required_artifacts() -> tuple[str, ...]:
    config = load_fabricator_config()
    configured = config.get("required_generated_artifacts") if isinstance(config, dict) else None
    if isinstance(configured, list):
        normalized = [str(item).strip() for item in configured if str(item).strip()]
        if normalized:
            return tuple(normalized)
    return DEFAULT_REQUIRED_RUN_ARTIFACTS


def required_artifact_path(paths: dict[str, Path], artifact_name: str) -> Path:
    if artifact_name == "EVAL_RESULT.json":
        return paths["proof"] / artifact_name
    return paths["generated"] / artifact_name


def build_job_story(seed: dict) -> dict[str, str]:
    user = seed["user"]
    situation = seed["situation"]
    motivation = seed["prototype_goal"].rstrip(".")
    outcome = seed["desired_outcome"].rstrip(".")
    story = f"When {situation}, {user} wants to {motivation.lower()} so they can {outcome.lower()}."
    return {
        "job_id": "JOB-001",
        "job_story": story[0].upper() + story[1:],
        "job_user": user,
        "job_situation": situation,
        "job_motivation": motivation,
        "job_outcome": outcome,
    }


def build_requirements(seed: dict) -> list[dict[str, str]]:
    if seed["product_type"] == "single_page_html5_game":
        return [
            {
                "id": requirement_id,
                "pattern": pattern,
                "statement": statement,
                "source": source,
            }
            for requirement_id, pattern, statement, source in HTML5_REQUIREMENTS
        ]

    requirements: list[dict[str, str]] = []
    for index, condition in enumerate(seed["success_conditions"] or [seed["prototype_goal"]], start=1):
        requirements.append(
            {
                "id": f"REQ-{index:03d}",
                "pattern": "Ubiquitous",
                "statement": f"The system shall satisfy this success condition: {condition}.",
                "source": condition,
            }
        )
    return requirements


def build_components(seed: dict) -> list[dict[str, str]]:
    if seed["product_type"] != "single_page_html5_game":
        return [
            {
                "id": "CMP-001",
                "name": "PrimaryFlow",
                "description": "Own the minimum implementation flow described by the product seed.",
                "trace": ", ".join(requirement["id"] for requirement in build_requirements(seed)),
            }
        ]

    return [
        {"id": component_id, "name": name, "description": description, "trace": trace}
        for component_id, name, description, trace in HTML5_COMPONENTS
    ]


def build_scenarios(seed: dict) -> list[dict[str, str]]:
    if seed["product_type"] == "single_page_html5_game":
        return [
            {
                "id": scenario_id,
                "requirement_id": requirement_id,
                "gherkin": gherkin,
                "title": title,
            }
            for scenario_id, requirement_id, gherkin, title in HTML5_SCENARIOS
        ]

    scenarios: list[dict[str, str]] = []
    for requirement in build_requirements(seed):
        scenarios.append(
            {
                "id": requirement["id"].replace("REQ", "SCN"),
                "requirement_id": requirement["id"],
                "title": requirement["source"],
                "gherkin": (
                    f"Given the bounded prototype is ready\n"
                    f"When the requirement is exercised\n"
                    f"Then {requirement['statement'][0].lower() + requirement['statement'][1:]}"
                ),
            }
        )
    return scenarios


def build_checks(seed: dict) -> list[dict[str, str]]:
    if seed["product_type"] == "single_page_html5_game":
        return [
            {
                "id": check_id,
                "scenario_id": scenario_id,
                "description": description,
                "mode": mode,
            }
            for check_id, scenario_id, description, mode in HTML5_CHECKS
        ]

    checks: list[dict[str, str]] = []
    for scenario in build_scenarios(seed):
        checks.append(
            {
                "id": scenario["id"].replace("SCN", "CHK"),
                "scenario_id": scenario["id"],
                "description": f"Review {scenario['title']} against the generated prototype.",
                "mode": "manual_review",
            }
        )
    return checks


def build_sections(seed: dict) -> dict[str, str]:
    job = build_job_story(seed)
    requirements = build_requirements(seed)
    components = build_components(seed)
    scenarios = build_scenarios(seed)
    checks = build_checks(seed)

    requirement_sections = []
    for requirement in requirements:
        requirement_sections.append(
            "\n".join(
                [
                    f"### {requirement['id']}",
                    f"- Pattern: {requirement['pattern']}",
                    f"- Trace: {job['job_id']} -> ADR-001 -> {requirement['id']}",
                    f"- Source condition: {requirement['source']}",
                    f"Requirement: {requirement['statement']}",
                ]
            )
        )

    component_rows = [
        "| ID | Component | Responsibility | Trace |",
        "| --- | --- | --- | --- |",
    ]
    for component in components:
        component_rows.append(
            f"| {component['id']} | {component['name']} | {component['description']} | {component['trace']} |"
        )

    scenario_sections = []
    for scenario in scenarios:
        scenario_sections.append(
            "\n".join(
                [
                    f"### {scenario['id']}",
                    f"- Trace: {scenario['requirement_id']} -> {scenario['id']}",
                    "```gherkin",
                    f"Scenario: {scenario['title']}",
                    scenario["gherkin"],
                    "```",
                ]
            )
        )

    checklist_rows = []
    for check in checks:
        checklist_rows.append(
            f"- {check['id']}: {check['description']} ({check['mode']}; traces {check['scenario_id']})"
        )

    eval_spec = {
        "run_id": seed["run_id"],
        "eval_id": "EVAL-001",
        "product_type": seed["product_type"],
        "checks": [
            {
                "id": check["id"],
                "scenario_id": check["scenario_id"],
                "mode": check["mode"],
                "description": check["description"],
            }
            for check in checks
        ],
        "proof_expectations": seed["proof_expectations"],
    }

    initial_result = {
        "result_id": "RESULT-001",
        "status": "pending",
        "checks": [{"id": check["id"], "status": "pending"} for check in checks],
    }

    return {
        **job,
        "constraints_bullets": bullet_list(seed["constraints"]),
        "non_goals_bullets": bullet_list(seed["non_goals"]),
        "success_conditions_bullets": bullet_list(seed["success_conditions"]),
        "proof_expectations_bullets": bullet_list(seed["proof_expectations"]),
        "requirement_sections": "\n\n".join(requirement_sections),
        "component_table": "\n".join(component_rows),
        "scenario_sections": "\n\n".join(scenario_sections),
        "checklist_bullets": "\n".join(checklist_rows) or "- no checks recorded",
        "eval_spec_yaml": json.dumps(eval_spec, indent=2),
        "proof_entries": bullet_list([f"PROOF-001 pending {check['id']}" for check in checks]),
        "reflection_points": bullet_list(
            [
                "What intent survived the artifact chain without distortion?",
                "What proof is still missing or weak?",
                "What lesson is reusable and backed by evidence?",
            ]
        ),
        "adaptation_yaml": dump_yaml(
            {
                "decision_id": "ADAPT-001",
                "status": "blocked",
                "reason": "Await proof execution and reviewed reflection before adapting learning.",
                "requires": [
                    "proof/EVAL_RESULT.json",
                    "generated/PROOF_RECORD.md",
                    "generated/REFLECTION.md",
                ],
            }
        ),
        "initial_eval_result_yaml": dump_yaml(initial_result),
        "requirement_ids": ", ".join(requirement["id"] for requirement in requirements),
        "scenario_ids": ", ".join(scenario["id"] for scenario in scenarios),
        "check_ids": ", ".join(check["id"] for check in checks),
    }


def build_template_values(seed: dict) -> dict[str, str]:
    sections = build_sections(seed)
    return {
        **sections,
        "run_id": seed["run_id"],
        "generated_at": utc_now(),
        "product_type": seed["product_type"],
        "user": seed["user"],
        "situation": seed["situation"],
        "desired_outcome": seed["desired_outcome"],
        "prototype_goal": seed["prototype_goal"],
        "proof_status": "pending",
    }


def render_prototype(seed: dict) -> str:
    if seed["product_type"] != "single_page_html5_game":
        return (
            "<!doctype html>\n"
            "<html lang=\"en\">\n"
            "<meta charset=\"utf-8\">\n"
            "<title>Prototype placeholder</title>\n"
            "<body>\n"
            f"<main><h1>{seed['prototype_goal']}</h1><p>{seed['desired_outcome']}</p></main>\n"
            "</body>\n"
            "</html>\n"
        )

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Focus Runner</title>
  <style>
    :root {{
      color-scheme: light;
      --ink: #10243d;
      --sky: #f2f7fb;
      --surface: #fffdf6;
      --accent: #0d9488;
      --danger: #d9485f;
      --gold: #e3a008;
    }}
    * {{ box-sizing: border-box; }}
    body {{ margin: 0; font-family: Georgia, 'Times New Roman', serif; background: radial-gradient(circle at top, #fff8d8, var(--sky) 55%, #d8e8f2); color: var(--ink); }}
    main {{ min-height: 100vh; display: grid; place-items: center; padding: 24px; }}
    .shell {{ width: min(920px, 100%); background: rgba(255, 253, 246, 0.92); border: 3px solid rgba(16, 36, 61, 0.2); border-radius: 24px; box-shadow: 0 24px 80px rgba(16, 36, 61, 0.18); overflow: hidden; }}
    .header {{ display: flex; justify-content: space-between; gap: 16px; padding: 20px 24px; border-bottom: 1px solid rgba(16, 36, 61, 0.15); background: linear-gradient(135deg, rgba(13, 148, 136, 0.12), rgba(227, 160, 8, 0.12)); }}
    .header h1 {{ margin: 0; font-size: 2rem; }}
    .header p {{ margin: 6px 0 0; max-width: 52ch; }}
    .hud {{ display: flex; gap: 16px; flex-wrap: wrap; font-size: 1rem; align-items: center; }}
    .chip {{ padding: 8px 12px; border-radius: 999px; background: rgba(16, 36, 61, 0.08); }}
    canvas {{ display: block; width: 100%; height: auto; background: linear-gradient(180deg, rgba(255,255,255,0.7), rgba(216,232,242,0.9)); }}
    .footer {{ display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 16px 24px 24px; }}
    button {{ border: 0; border-radius: 999px; padding: 12px 18px; background: var(--accent); color: white; font: inherit; cursor: pointer; }}
    button:hover {{ filter: brightness(1.05); }}
    .hint {{ opacity: 0.8; }}
  </style>
</head>
<body>
  <main>
    <section class="shell">
      <div class="header">
        <div>
          <h1>Focus Runner</h1>
          <p>{seed['desired_outcome']}</p>
        </div>
        <div class="hud" aria-label="Round status">
          <span class="chip">Score: <strong id="score">0</strong></span>
          <span class="chip">Lives: <strong id="lives">3</strong></span>
          <span class="chip">Time: <strong id="timer">45</strong>s</span>
          <span class="chip">State: <strong id="state">Ready</strong></span>
        </div>
      </div>
      <canvas id="game" width="900" height="520" aria-label="Focus runner play area"></canvas>
      <div class="footer">
        <div class="hint">Move with arrow keys or WASD. Collect gold focus orbs and avoid the red distractions.</div>
        <button id="restart" type="button">Restart Round</button>
      </div>
    </section>
  </main>
  <script>
    const canvas = document.getElementById('game');
    const ctx = canvas.getContext('2d');
    const scoreNode = document.getElementById('score');
    const livesNode = document.getElementById('lives');
    const timerNode = document.getElementById('timer');
    const stateNode = document.getElementById('state');
    const restartButton = document.getElementById('restart');
    const keys = new Set();
    const arena = {{ width: canvas.width, height: canvas.height }};
    const playerSpeed = 240;
    let lastFrame = performance.now();
    let roundOver = false;

    const state = {{
      score: 0,
      lives: 3,
      timeLeft: 45,
      player: {{ x: 120, y: 260, size: 22 }},
      focus: createToken(1),
      distractions: [createToken(-1), createToken(-1), createToken(-1)],
      message: 'Ready',
    }};

    function random(min, max) {{
      return Math.random() * (max - min) + min;
    }}

    function createToken(weight) {{
      return {{
        weight,
        radius: weight > 0 ? 14 : 16,
        x: random(80, arena.width - 80),
        y: random(80, arena.height - 80),
        driftX: random(-90, 90),
        driftY: random(-90, 90),
      }};
    }}

    function restartGame() {{
      state.score = 0;
      state.lives = 3;
      state.timeLeft = 45;
      state.player.x = 120;
      state.player.y = arena.height / 2;
      state.focus = createToken(1);
      state.distractions = [createToken(-1), createToken(-1), createToken(-1)];
      state.message = 'Running';
      roundOver = false;
      syncHud();
    }}

    function syncHud() {{
      scoreNode.textContent = String(state.score);
      livesNode.textContent = String(state.lives);
      timerNode.textContent = String(Math.max(0, Math.ceil(state.timeLeft)));
      stateNode.textContent = state.message;
    }}

    function moveEntity(entity, deltaSeconds) {{
      entity.x += entity.driftX * deltaSeconds;
      entity.y += entity.driftY * deltaSeconds;
      if (entity.x < 30 || entity.x > arena.width - 30) entity.driftX *= -1;
      if (entity.y < 30 || entity.y > arena.height - 30) entity.driftY *= -1;
      entity.x = Math.max(30, Math.min(arena.width - 30, entity.x));
      entity.y = Math.max(30, Math.min(arena.height - 30, entity.y));
    }}

    function handleMovement(deltaSeconds) {{
      let dx = 0;
      let dy = 0;
      if (keys.has('ArrowLeft') || keys.has('a')) dx -= 1;
      if (keys.has('ArrowRight') || keys.has('d')) dx += 1;
      if (keys.has('ArrowUp') || keys.has('w')) dy -= 1;
      if (keys.has('ArrowDown') || keys.has('s')) dy += 1;
      const length = Math.hypot(dx, dy) || 1;
      state.player.x += (dx / length) * playerSpeed * deltaSeconds;
      state.player.y += (dy / length) * playerSpeed * deltaSeconds;
      state.player.x = Math.max(24, Math.min(arena.width - 24, state.player.x));
      state.player.y = Math.max(24, Math.min(arena.height - 24, state.player.y));
    }}

    function overlaps(entity, radius) {{
      const distance = Math.hypot(state.player.x - entity.x, state.player.y - entity.y);
      return distance <= state.player.size + radius;
    }}

    function resolveCollisions() {{
      if (overlaps(state.focus, state.focus.radius)) {{
        state.score += 10;
        state.focus = createToken(1);
      }}
      state.distractions = state.distractions.map((distraction) => {{
        if (overlaps(distraction, distraction.radius)) {{
          state.lives -= 1;
          return createToken(-1);
        }}
        return distraction;
      }});
      if (state.score >= 60) {{
        roundOver = true;
        state.message = 'Won';
      }} else if (state.lives <= 0) {{
        roundOver = true;
        state.message = 'Lost';
      }}
    }}

    function drawArena() {{
      ctx.clearRect(0, 0, arena.width, arena.height);
      ctx.fillStyle = 'rgba(255,255,255,0.8)';
      ctx.fillRect(0, 0, arena.width, arena.height);
      ctx.fillStyle = '#d6efe8';
      for (let index = 0; index < 24; index += 1) {{
        ctx.fillRect(index * 40, 0, 2, arena.height);
      }}
      ctx.fillStyle = '#0d9488';
      ctx.beginPath();
      ctx.arc(state.player.x, state.player.y, state.player.size, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#e3a008';
      ctx.beginPath();
      ctx.arc(state.focus.x, state.focus.y, state.focus.radius, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#d9485f';
      state.distractions.forEach((distraction) => {{
        ctx.beginPath();
        ctx.arc(distraction.x, distraction.y, distraction.radius, 0, Math.PI * 2);
        ctx.fill();
      }});
      ctx.fillStyle = '#10243d';
      ctx.font = '24px Georgia';
      ctx.fillText(roundOver ? `Round ${'{'}state.message{'}'}` : 'Collect focus, avoid distractions', 24, 36);
    }}

    function tick(frameTime) {{
      const deltaSeconds = Math.min(0.05, (frameTime - lastFrame) / 1000);
      lastFrame = frameTime;
      if (!roundOver) {{
        state.message = 'Running';
        state.timeLeft -= deltaSeconds;
        handleMovement(deltaSeconds);
        moveEntity(state.focus, deltaSeconds);
        state.distractions.forEach((entity) => moveEntity(entity, deltaSeconds));
        resolveCollisions();
        if (state.timeLeft <= 0) {{
          roundOver = true;
          state.message = state.score >= 40 ? 'Won' : 'Lost';
        }}
      }}
      syncHud();
      drawArena();
      requestAnimationFrame(tick);
    }}

    document.addEventListener('keydown', (event) => {{
      keys.add(event.key);
    }});
    document.addEventListener('keyup', (event) => {{
      keys.delete(event.key);
    }});
    restartButton.addEventListener('click', restartGame);

    restartGame();
    requestAnimationFrame(tick);
  </script>
</body>
</html>
"""


def write_generated_artifacts(seed: dict) -> None:
    paths = run_paths(seed["run_id"])
    values = build_template_values(seed)
    ensure_directory(paths["generated"])
    ensure_directory(paths["prototype"])
    ensure_directory(paths["proof"])

    artifact_map = {
        "PRODUCT_SEED.md": "PRODUCT_SEED.md.j2",
        "JTBD.md": "JTBD.md.j2",
        "JOB_HYPOTHESIS.md": "JOB_HYPOTHESIS.md.j2",
        "HYPOTHESIS.md": "HYPOTHESIS.md.j2",
        "ADR.md": "ADR.md.j2",
        "PRD.md": "PRD.md.j2",
        "SDS.md": "SDS.md.j2",
        "TDD.md": "TDD.md.j2",
        "CONTEXT_PACK.md": "CONTEXT_PACK.md.j2",
        "AGENT_TASK.md": "AGENT_TASK.md.j2",
        "EVAL_CHECKLIST.md": "EVAL_CHECKLIST.md.j2",
        "PROOF_RECORD.md": "PROOF_RECORD.md.j2",
        "REFLECTION.md": "REFLECTION.md.j2",
        "NO_SKILL_PROPOSED.md": "NO_SKILL_PROPOSED.md.j2",
        "ADAPTATION_DECISION.yaml": "ADAPTATION_DECISION.yaml.j2",
        "EVAL_SPEC.yaml": "EVAL_SPEC.yaml.j2",
    }
    for artifact_name, template_name in artifact_map.items():
        write_text(paths["generated"] / artifact_name, render_template(template_name, values))

    write_text(paths["prototype"] / "index.html", render_prototype(seed))
    append_trace(seed["run_id"], "generate", "rendered artifact packet and prototype")


def load_seed_for_run(run_id: str) -> dict:
    seed_path = run_paths(run_id)["input"]
    if not seed_path.is_file():
        raise FileNotFoundError(f"missing seed input: {seed_path}")
    return normalize_seed(load_mapping(seed_path))


def static_proof_checks(prototype_path: Path) -> list[dict[str, str]]:
    html = prototype_path.read_text(encoding="utf-8")
    checks = [
        ("CHK-001", all(token in html for token in ("<canvas", "requestAnimationFrame", "Focus Runner")) and "fetch(" not in html and "http://" not in html and "https://" not in html, "single local file with no network calls"),
        ("CHK-002", "document.addEventListener('keydown'" in html and "handleMovement" in html, "keyboard listeners and movement logic"),
        ("CHK-003", "resolveCollisions" in html and "state.score += 10" in html and "state.lives -= 1" in html, "collision and scoring logic"),
        ("CHK-004", all(token in html for token in ("id=\"score\"", "id=\"timer\"", "id=\"state\"")), "HUD and timer rendering"),
        ("CHK-005", "restartGame" in html and "id=\"restart\"" in html and "Won" in html and "Lost" in html, "end-state and restart flow"),
    ]
    return [
        {
            "id": check_id,
            "status": "pass" if passed else "fail",
            "description": description,
        }
        for check_id, passed, description in checks
    ]


def write_proof_outputs(seed: dict) -> dict:
    paths = run_paths(seed["run_id"])
    proof_checks = static_proof_checks(paths["prototype"] / "index.html")
    status = "pass" if all(check["status"] == "pass" for check in proof_checks) else "fail"
    result = {
        "result_id": "RESULT-001",
        "run_id": seed["run_id"],
        "status": status,
        "proof_mode": "static_html_scan",
        "checks": proof_checks,
    }
    write_text(paths["proof"] / "EVAL_RESULT.json", json.dumps(result, indent=2))

    values = build_template_values(seed)
    values["proof_status"] = status
    values["proof_entries"] = bullet_list(
        [f"PROOF-001 {check['id']} {check['status']} {check['description']}" for check in proof_checks]
    )
    values["reflection_points"] = bullet_list(
        [
            f"Observed result: {status}",
            "Lesson candidate: single-file local-first delivery keeps proof reproducible.",
            "Risk: proof is static analysis and should be paired with manual play verification for release claims.",
        ]
    )
    values["adaptation_yaml"] = dump_yaml(
        {
            "decision_id": "ADAPT-001",
            "status": "candidate" if status == "pass" else "blocked",
            "reason": (
                "Static proof passed; human review may promote this learning candidate."
                if status == "pass"
                else "Proof failed; do not adapt learning until the prototype is repaired."
            ),
            "proof_result": "proof/EVAL_RESULT.json",
        }
    )
    write_text(paths["generated"] / "PROOF_RECORD.md", render_template("PROOF_RECORD.md.j2", values))
    write_text(paths["generated"] / "REFLECTION.md", render_template("REFLECTION.md.j2", values))
    write_text(
        paths["generated"] / "ADAPTATION_DECISION.yaml",
        render_template("ADAPTATION_DECISION.yaml.j2", values),
    )
    append_trace(seed["run_id"], "proof", f"recorded {status} static proof")
    return result


def validate_run(run_id: str) -> dict:
    paths = run_paths(run_id)
    errors: list[str] = []
    warnings: list[str] = []

    if not paths["input"].is_file():
        return {"run_id": run_id, "status": "fail", "errors": ["missing input.yaml"], "warnings": []}

    generated_root = paths["generated"]
    required_artifacts = configured_required_artifacts()
    for artifact_name in required_artifacts:
        if artifact_name in SKILL_DECISION_ARTIFACTS:
            continue
        if not required_artifact_path(paths, artifact_name).is_file():
            errors.append(f"missing generated artifact: {artifact_name}")

    if any(artifact in required_artifacts for artifact in SKILL_DECISION_ARTIFACTS) and not any(
        (generated_root / artifact).is_file() for artifact in SKILL_DECISION_ARTIFACTS
    ):
        errors.append("missing skill decision record")

    if not (paths["prototype"] / "index.html").is_file():
        errors.append("missing prototype/index.html")

    if errors:
        return {"run_id": run_id, "status": "fail", "errors": errors, "warnings": warnings}

    prd_text = (generated_root / "PRD.md").read_text(encoding="utf-8")
    requirement_ids = re.findall(r"^### (REQ-\d{3})$", prd_text, flags=re.MULTILINE)
    if not requirement_ids:
        errors.append("PRD.md must contain EARS requirement sections with stable REQ IDs")
    for requirement_id in requirement_ids:
        pattern = re.compile(
            rf"^### {requirement_id}$.*?^- Pattern: .*?$.*?^Requirement: .*?shall .*?$",
            re.MULTILINE | re.DOTALL,
        )
        if not pattern.search(prd_text):
            errors.append(f"PRD requirement {requirement_id} is missing pattern or shall statement")

    tdd_text = (generated_root / "TDD.md").read_text(encoding="utf-8")
    for requirement_id in requirement_ids:
        if requirement_id not in tdd_text:
            errors.append(f"TDD.md is missing trace to {requirement_id}")

    sds_text = (generated_root / "SDS.md").read_text(encoding="utf-8")
    for requirement_id in requirement_ids:
        if requirement_id not in sds_text:
            errors.append(f"SDS.md is missing trace to {requirement_id}")

    context_text = (generated_root / "CONTEXT_PACK.md").read_text(encoding="utf-8")
    for heading in ("## Constraints", "## Non-Goals", "## Linked Specs"):
        if heading not in context_text:
            errors.append(f"CONTEXT_PACK.md missing heading: {heading}")

    agent_task_text = (generated_root / "AGENT_TASK.md").read_text(encoding="utf-8")
    for heading in ("## Allowed Actions", "## Forbidden Actions", "## Done When"):
        if heading not in agent_task_text:
            errors.append(f"AGENT_TASK.md missing heading: {heading}")

    proof_text = (generated_root / "PROOF_RECORD.md").read_text(encoding="utf-8")
    if "Status:" not in proof_text:
        errors.append("PROOF_RECORD.md must include a Status field")

    reflection_text = (generated_root / "REFLECTION.md").read_text(encoding="utf-8")
    if "## Evidence" not in reflection_text:
        errors.append("REFLECTION.md must include evidence prompts")

    hypothesis_text = (generated_root / "JOB_HYPOTHESIS.md").read_text(encoding="utf-8")
    mirror_text = (generated_root / "HYPOTHESIS.md").read_text(encoding="utf-8")
    if "HYP-001" not in hypothesis_text or "HYP-001" not in mirror_text:
        errors.append("JOB_HYPOTHESIS.md and HYPOTHESIS.md must preserve the same stable hypothesis ID")

    eval_spec: dict | None = None
    try:
        eval_spec = load_mapping(generated_root / "EVAL_SPEC.yaml")
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        errors.append(f"EVAL_SPEC.yaml parse error: {exc}")
    except Exception as exc:
        errors.append(f"EVAL_SPEC.yaml parse error: {exc}")
    if isinstance(eval_spec, dict):
        checks = eval_spec.get("checks")
        if not isinstance(checks, list) or not checks:
            errors.append("EVAL_SPEC.yaml must contain non-empty checks")
        elif len(checks) < len(requirement_ids):
            errors.append("EVAL_SPEC.yaml must cover every requirement with at least one check")

    if not (paths["proof"] / "EVAL_RESULT.json").is_file():
        warnings.append("proof not yet executed")
    if not (paths["handoff"] / "AGENT_TASK.md").is_file():
        warnings.append("handoff not yet prepared")

    status = "pass" if not errors else "fail"
    return {"run_id": run_id, "status": status, "errors": errors, "warnings": warnings}


def command_new(args: argparse.Namespace) -> int:
    seed_path = Path(args.seed).resolve()
    if not seed_path.is_file():
        return fail(f"seed path does not exist: {args.seed}")
    seed = normalize_seed(load_mapping(seed_path))
    paths = run_paths(seed["run_id"])
    ensure_directory(paths["run_root"])
    shutil.copyfile(seed_path, paths["input"])
    append_trace(seed["run_id"], "new", f"seed copied from {seed_path}")
    print(f"created run {seed['run_id']} at {paths['run_root'].relative_to(ROOT)}")
    return 0


def command_generate(args: argparse.Namespace) -> int:
    try:
        seed = load_seed_for_run(args.run_id)
    except (FileNotFoundError, ValueError) as exc:
        return fail(str(exc))
    write_generated_artifacts(seed)
    print(f"generated artifact packet for {seed['run_id']}")
    return 0


def command_validate(args: argparse.Namespace) -> int:
    report = validate_run(args.run_id)
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


def command_handoff(args: argparse.Namespace) -> int:
    try:
        seed = load_seed_for_run(args.run_id)
    except (FileNotFoundError, ValueError) as exc:
        return fail(str(exc))
    paths = run_paths(seed["run_id"])
    ensure_directory(paths["handoff"])
    handoff_artifacts = ("AGENT_TASK.md", "CONTEXT_PACK.md", "PRD.md", "SDS.md", "TDD.md", "EVAL_SPEC.yaml")
    missing = [name for name in handoff_artifacts if not (paths["generated"] / name).is_file()]
    if missing:
        missing_list = ", ".join(missing)
        return fail(
            f"missing generated handoff artifacts: {missing_list}; run 'fabricate generate {seed['run_id']}' first"
        )
    for name in handoff_artifacts:
        shutil.copyfile(paths["generated"] / name, paths["handoff"] / name)
    manifest = {
        "run_id": seed["run_id"],
        "handoff_files": sorted(path.name for path in paths["handoff"].iterdir()),
        "prototype_path": str((paths["prototype"] / "index.html").relative_to(ROOT)),
    }
    write_text(paths["handoff"] / "MANIFEST.json", json.dumps(manifest, indent=2))
    append_trace(seed["run_id"], "handoff", "prepared bounded handoff packet")
    print(f"prepared handoff files: {(paths['handoff'] / 'AGENT_TASK.md').relative_to(ROOT)}")
    return 0


def command_proof(args: argparse.Namespace) -> int:
    try:
        seed = load_seed_for_run(args.run_id)
    except (FileNotFoundError, ValueError) as exc:
        return fail(str(exc))
    result = write_proof_outputs(seed)
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "pass" else 1


def command_reflect(args: argparse.Namespace) -> int:
    try:
        seed = load_seed_for_run(args.run_id)
    except (FileNotFoundError, ValueError) as exc:
        return fail(str(exc))
    values = build_template_values(seed)
    reflection_path = run_paths(seed["run_id"])["generated"] / "REFLECTION.md"
    write_text(reflection_path, render_template("REFLECTION.md.j2", values))
    append_trace(seed["run_id"], "reflect", "refreshed reflection template")
    print(f"refreshed {reflection_path.relative_to(ROOT)}")
    return 0


def command_status(args: argparse.Namespace) -> int:
    report = validate_run(args.run_id)
    paths = run_paths(args.run_id)
    next_action = "generate"
    if report["status"] == "pass":
        if not (paths["handoff"] / "AGENT_TASK.md").is_file():
            next_action = "handoff"
        elif not (paths["proof"] / "EVAL_RESULT.json").is_file():
            next_action = "proof"
        else:
            next_action = "done"
    status = {
        "run_id": args.run_id,
        "validation": report["status"],
        "warnings": report["warnings"],
        "next_action": next_action,
    }
    print(json.dumps(status, indent=2))
    return 0 if report["status"] == "pass" else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="fabricate")
    subparsers = parser.add_subparsers(dest="command", required=True)

    new_parser = subparsers.add_parser("new", help="create a fabrication run from a bounded seed")
    new_parser.add_argument("seed")
    new_parser.set_defaults(handler=command_new)

    for name, handler, help_text in (
        ("generate", command_generate, "generate the artifact packet and prototype for a run"),
        ("validate", command_validate, "validate artifact completeness and traceability"),
        ("handoff", command_handoff, "prepare a bounded agent handoff packet"),
        ("proof", command_proof, "run proof checks for the generated prototype"),
        ("reflect", command_reflect, "refresh or create reflection artifacts"),
        ("status", command_status, "report run status and next action"),
    ):
        subparser = subparsers.add_parser(name, help=help_text)
        subparser.add_argument("run_id")
        subparser.set_defaults(handler=handler)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    raise SystemExit(main())