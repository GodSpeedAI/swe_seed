#!/usr/bin/env python3
"""Self-contained file-first strategy layer runtime."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import re
import shutil
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
TEMPLATES_ROOT = ROOT / "templates"
CONFIG_PATH = ROOT / "config.yaml"

PLACEHOLDER_RE = re.compile(r"\{\{\s*([a-z0-9_]+)\s*\}\}")

TEST_TYPES = (
    "analytical_research",
    "ugc_social_listening",
    "jtbd_interview",
    "design_research",
    "competitor_analysis",
    "category_mapping",
    "purchase_behavior_research",
    "prototype_pilot",
    "sales_commitment",
    "cost_model",
    "capability_audit",
)

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


def fail(message: str) -> int:
    print(f"strategy: {message}", file=sys.stderr)
    return 1


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def slugify(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "strategy-item"


def ensure_directory(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def write_text(path: Path, content: str) -> None:
    ensure_directory(path.parent)
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def _parse_yaml(text: str) -> dict:
    lines = text.split("\n")
    pos = [0]

    def indent_of(idx: int) -> int:
        if idx >= len(lines):
            return -1
        line = lines[idx]
        stripped = line.lstrip()
        if not stripped or stripped.startswith("#"):
            return -1
        return len(line) - len(stripped)

    def skip_empty() -> None:
        while pos[0] < len(lines):
            stripped = lines[pos[0]].lstrip()
            if stripped and not stripped.startswith("#"):
                break
            pos[0] += 1

    def scalar(value: str) -> object:
        value = value.strip()
        if not value or value.startswith("#"):
            return ""
        if " #" in value:
            value = value[: value.index(" #")].strip()
        if len(value) >= 2 and (
            (value[0] == '"' and value[-1] == '"') or (value[0] == "'" and value[-1] == "'")
        ):
            return value[1:-1]
        if value.lower() == "true":
            return True
        if value.lower() == "false":
            return False
        try:
            return int(value)
        except ValueError:
            pass
        try:
            return float(value)
        except ValueError:
            pass
        return value

    def parse_mapping(min_indent: int) -> dict:
        result: dict = {}
        while pos[0] < len(lines):
            skip_empty()
            if pos[0] >= len(lines):
                break
            ind = indent_of(pos[0])
            if ind < min_indent:
                break
            line = lines[pos[0]].lstrip()
            if ":" not in line or line.startswith("-"):
                break
            key, _, value = line.partition(":")
            key = key.strip()
            value = value.strip()
            if value == "" or value.startswith("#"):
                pos[0] += 1
                skip_empty()
                if pos[0] < len(lines) and indent_of(pos[0]) > ind:
                    next_line = lines[pos[0]].lstrip()
                    if next_line.startswith("- "):
                        result[key] = parse_sequence(indent_of(pos[0]))
                    else:
                        result[key] = parse_mapping(indent_of(pos[0]))
                else:
                    result[key] = {}
            else:
                result[key] = scalar(value)
                pos[0] += 1
        return result

    def parse_sequence(min_indent: int) -> list:
        items: list = []
        while pos[0] < len(lines):
            skip_empty()
            if pos[0] >= len(lines):
                break
            ind = indent_of(pos[0])
            if ind < min_indent:
                break
            line = lines[pos[0]].lstrip()
            if not line.startswith("- "):
                break
            items.append(scalar(line[2:]))
            pos[0] += 1
        return items

    return parse_mapping(0)


def load_mapping(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    try:
        loaded = json.loads(text)
    except json.JSONDecodeError:
        loaded = _parse_yaml(text)
    if not isinstance(loaded, dict):
        raise ValueError(f"{path} must contain an object")
    return loaded


def load_config() -> dict:
    return load_mapping(CONFIG_PATH)


def yaml_scalar(value: object) -> str:
    if value is None:
        return '""'
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    text = str(value)
    if text.lower() in YAML_PLAIN_SCALAR_KEYWORDS:
        return f'"{text.replace("\"", "\\\"")}"'
    if not text or any(char in text for char in ':#[]{}\n"'):
        return f'"{text.replace("\"", "\\\"")}"'
    return text


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
        lines: list[str] = []
        for item in value:
            if isinstance(item, (dict, list)):
                nested = dump_yaml(item, indent + 1).splitlines()
                lines.append(f"{pad}- {nested[0].strip()}")
                lines.extend(nested[1:])
            else:
                lines.append(f"{pad}- {yaml_scalar(item)}")
        return "\n".join(lines)
    return f"{pad}{yaml_scalar(value)}"


def render_template(template_name: str, values: dict[str, str]) -> str:
    template = (TEMPLATES_ROOT / template_name).read_text(encoding="utf-8")
    missing = sorted(set(PLACEHOLDER_RE.findall(template)) - set(values))
    if missing:
        raise ValueError(f"template {template_name} missing values: {', '.join(missing)}")
    return PLACEHOLDER_RE.sub(lambda match: values[match.group(1)], template)


def path_for(kind: str, filename: str) -> Path:
    paths = load_config().get("paths", {})
    folder = str(paths.get(kind, kind)).strip()
    return ROOT / folder / filename


def ensure_scaffold() -> None:
    paths = load_config().get("paths", {})
    for folder in paths.values():
        ensure_directory(ROOT / str(folder))
    ensure_directory(ROOT / "schemas")
    ensure_directory(ROOT / "templates")


def markdown_list(items: list[str]) -> str:
    return "\n".join(f"- {item}" for item in items) if items else "- none"


def example_option_id_from_brief(brief_id: str) -> str:
    base = brief_id.removeprefix("brief.")
    return f"option.{base}"


def template_values(question: str = "", brief_id: str = "", option_id: str = "") -> dict[str, str]:
    slug = brief_id or option_id or slugify(question)
    strategic_question = question or "Which first strategic wedge should this option pursue?"
    option_name = option_id or f"option.{slug}"
    gap_id = f"gap.{slug}.001"
    prompt_id = f"prompt.{slug}"
    report_id = f"report.{slug}"
    test_id = f"test.{slug}.001"
    evidence_id = f"evidence.{slug}.001"
    decision_id = f"decision.{slug}"
    return {
        "created_at": utc_now(),
        "question": strategic_question,
        "brief_id": brief_id or f"brief.{slug}",
        "need_signal_id": f"need.{slug}",
        "option_id": option_name,
        "gap_id": gap_id,
        "prompt_id": prompt_id,
        "report_id": report_id,
        "test_id": test_id,
        "evidence_id": evidence_id,
        "decision_id": decision_id,
        "slug": slug,
        "tradeoffs": markdown_list([
            "Focus on one wedge instead of a broad market narrative.",
            "Prefer evidence gathering before fabrication commitment.",
        ]),
        "conditions": markdown_list([
            "Target users already spend money on comparable or adjacent solutions.",
            "The chosen segment feels the pain frequently enough to change behavior.",
        ]),
        "gaps": markdown_list([
            "Purchase behavior evidence for the selected segment.",
            "Evidence that the how-to-win choice is differentiated enough to matter.",
        ]),
        "research_questions": markdown_list([
            "What paid solutions already serve this job or an adjacent workflow?",
            "What evidence weakens the case that a new standalone strategy is worth pursuing?",
            "What next test would most reduce uncertainty?",
        ]),
    }


def option_data(option_id: str, question: str) -> dict:
    slug = option_id.removeprefix("option.")
    return {
        "strategy_option": {
            "id": option_id,
            "title": "Proof-backed agentic development harness wedge",
            "status": "research_needed",
            "strategic_question": question,
            "winning_aspiration": "Become the most trusted proof-backed workflow for agentic software delivery.",
            "where_to_play": {
                "segment": "AI-native software teams",
                "user_or_buyer": "technical leads and builder-operators",
                "use_case": "proof-backed AI coding workflow governance",
                "channel": "open-source plus consulting-led adoption",
                "geography": "remote-first English-speaking teams",
                "value_chain_stage": "development workflow orchestration",
                "constraints": ["stay lightweight", "no hosted dependency required"],
                "excluded_segments": ["large enterprise PMO transformation"],
            },
            "how_to_win": {
                "value_proposition": "Provide file-first proof, traceability, and bounded agent execution instead of generic AI coding assistance.",
                "differentiation_or_cost_position": "Higher trust through proof discipline rather than lowest-cost automation.",
                "unfair_advantage": "Deep understanding of harness, fabrication, and proof-first workflows.",
                "proof_of_value": "Users can point to faster recovery and fewer false completion claims.",
                "tradeoffs": [
                    "Narrow wedge over broad productivity messaging",
                    "Evidence-backed claims over growth theater",
                ],
                "alternatives_rejected": ["generic AI IDE assistant", "full enterprise transformation suite"],
            },
            "capabilities_required": [
                {
                    "capability": "proof-system design",
                    "importance": "high",
                    "current_strength": "strong",
                    "obtain_by": "build",
                    "evidence": "Current harness and fabrication implementation",
                }
            ],
            "management_systems_required": [
                {
                    "system": "decision review cadence",
                    "purpose": "reassess strategy with new evidence",
                    "metric": "evidence gap closure rate",
                    "cadence": "weekly",
                    "owner": "operator",
                    "evidence": "decision record review",
                }
            ],
            "conditions_that_must_be_true": [
                {
                    "id": f"condition.{slug}.purchase_behavior",
                    "condition": "Target users already spend money on comparable or adjacent reliability workflows.",
                    "category": "purchase_behavior",
                    "confidence": "low",
                    "evidence_currently_available": "Anecdotal knowledge of AI tooling spend",
                    "evidence_missing": "Direct or adjacent purchase evidence",
                    "test_required": "purchase behavior research",
                    "research_required": "yes",
                }
            ],
            "evidence_gaps": [
                {
                    "gap_id": f"gap.{slug}.001",
                    "missing_information": "Validated purchase behavior for the target wedge",
                    "affected_condition_id": f"condition.{slug}.purchase_behavior",
                    "decision_blocked": "commit",
                    "recommended_research_type": "purchase_behavior_research",
                    "required_evidence_level": 3,
                }
            ],
            "tests": [
                {
                    "id": f"test.{slug}.001",
                    "condition_id": f"condition.{slug}.purchase_behavior",
                    "method": "purchase_behavior_research",
                    "pass_criteria": "Direct or adjacent paid solutions demonstrate money already moves around the job",
                    "fail_criteria": "Only interest or free workaround evidence exists",
                    "owner": "operator",
                    "due_date": "TBD",
                }
            ],
            "evidence": [
                {
                    "source": "internal hypothesis",
                    "evidence_type": "analytical_inference",
                    "summary": "The workflow appears painful and recurring for AI-assisted development teams.",
                    "supports": f"condition.{slug}.purchase_behavior",
                    "weakens": "",
                    "confidence": "low",
                }
            ],
            "decision": {
                "status": "delegate_research",
                "reason": "Critical purchase behavior evidence is missing.",
                "next_action": f"Generate and run {f'prompt.{slug}'}",
            },
        }
    }


def write_option(option_id: str, question: str) -> Path:
    path = path_for("options", f"{option_id}.yaml")
    write_text(path, dump_yaml(option_data(option_id, question)))
    return path


def command_new(args: argparse.Namespace) -> int:
    ensure_scaffold()
    brief_id = f"brief.{slugify(args.question)}"
    values = template_values(question=args.question, brief_id=brief_id)
    path = path_for("briefs", f"{brief_id}.md")
    write_text(path, render_template("STRATEGY_BRIEF.md", values))
    print(f"created {path.relative_to(ROOT)}")
    return 0


def command_capture(args: argparse.Namespace) -> int:
    ensure_scaffold()
    source_path = Path(args.source)
    exact_language = (
        source_path.read_text(encoding="utf-8").strip() if source_path.is_file() else args.source.strip()
    )
    signal_id = f"need.{slugify(exact_language[:80])}"
    data = {
        "need_signal": {
            "id": signal_id,
            "captured_at": utc_now(),
            "exact_language": exact_language,
            "situation": "unknown",
            "struggle": "unknown",
            "current_workaround": "unknown",
            "desired_outcome": "unknown",
            "constraint": "unknown",
            "system_friction": "unknown",
            "emotional_intensity": "unknown",
            "frequency": "unknown",
            "cost": "unknown",
            "evidence_source": str(source_path) if source_path.is_file() else "direct-input",
            "confidence": "low",
        }
    }
    path = path_for("need_signals", f"{signal_id}.yaml")
    write_text(path, dump_yaml(data))
    print(f"captured {path.relative_to(ROOT)}")
    return 0


def load_brief_question(brief_id: str) -> str:
    path = path_for("briefs", f"{brief_id}.md")
    if not path.is_file():
        raise FileNotFoundError(path)
    text = path.read_text(encoding="utf-8")
    for line in text.splitlines():
        if line.startswith("- Strategic question: "):
            return line.split(": ", 1)[1].strip()
    return "Which strategic wedge should we pursue first?"


def command_generate_options(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        question = load_brief_question(args.brief_id)
    except FileNotFoundError:
        return fail(f"missing brief: {args.brief_id}")
    option_id = example_option_id_from_brief(args.brief_id)
    path = write_option(option_id, question)
    print(f"generated {path.relative_to(ROOT)}")
    return 0


def load_strategy_option(option_id: str) -> dict:
    path = path_for("options", f"{option_id}.yaml")
    if not path.is_file():
        raise FileNotFoundError(path)
    return load_mapping(path)


def command_identify_gaps(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        option = load_strategy_option(args.option_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid option: {args.option_id}")
    strategic_question = option["strategy_option"].get("strategic_question", "Unknown question")
    values = template_values(question=strategic_question, option_id=args.option_id)
    path = path_for("gaps", f"gap.{args.option_id.removeprefix('option.')}.001.md")
    write_text(path, render_template("EVIDENCE_GAP_REPORT.md", values))
    print(f"generated {path.relative_to(ROOT)}")
    return 0


def command_generate_research_prompts(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        option = load_strategy_option(args.option_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid option: {args.option_id}")
    strategic_question = option["strategy_option"].get("strategic_question", "Unknown question")
    values = template_values(question=strategic_question, option_id=args.option_id)
    path = path_for("research_prompts", f"prompt.{args.option_id.removeprefix('option.')}.md")
    write_text(path, render_template("RESEARCH_PROMPT_PACKET.md", values))
    print(f"generated {path.relative_to(ROOT)}")
    return 0


def command_ingest_research_report(args: argparse.Namespace) -> int:
    ensure_scaffold()
    source = Path(args.report_path).resolve()
    if not source.is_file():
        return fail(f"report path does not exist: {args.report_path}")
    report_id = f"report.{slugify(source.stem)}"
    report_target = path_for("research_reports", f"{report_id}.md")
    shutil.copyfile(source, report_target)
    values = template_values(question="Imported research report", option_id=f"option.{slugify(source.stem)}")
    values["report_id"] = report_id
    synthesis_target = path_for("research", f"synthesis.{slugify(source.stem)}.md")
    write_text(synthesis_target, render_template("RESEARCH_SYNTHESIS.md", values))
    print(f"ingested {report_target.relative_to(ROOT)}")
    return 0


def command_design_tests(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        option = load_strategy_option(args.option_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid option: {args.option_id}")
    option_root = option["strategy_option"]
    slug = args.option_id.removeprefix("option.")
    test_data = {
        "strategy_test": {
            "id": f"test.{slug}.001",
            "strategy_option_id": args.option_id,
            "condition_id": f"condition.{slug}.purchase_behavior",
            "method": "purchase_behavior_research",
            "pass_criteria": "Direct or adjacent paid solutions validate category spend.",
            "fail_criteria": "Only curiosity and unpaid workarounds are visible.",
            "owner": "operator",
            "due_date": "TBD",
        }
    }
    test_path = path_for("tests", f"test.{slug}.001.yaml")
    write_text(test_path, dump_yaml(test_data))
    eval_data = {
        "strategy_eval_spec": {
            "id": f"strategy-eval.{slug}",
            "strategy_option_id": args.option_id,
            "checks": [
                {
                    "id": "evidence.gaps_identified",
                    "question": "Have critical evidence gaps been identified before decision?",
                    "pass_criteria": "At least one critical gap is recorded.",
                    "evidence_required": "EvidenceGapReport",
                },
                {
                    "id": "research.prompts_generated",
                    "question": "Were research prompts generated for unresolved critical gaps?",
                    "pass_criteria": "Research prompt packet exists.",
                    "evidence_required": "ResearchPromptPacket",
                },
                {
                    "id": "decision.evidence_sufficiency",
                    "question": "Is the evidence strong enough for the proposed decision status?",
                    "pass_criteria": "Decision status is not commit while critical gaps remain.",
                    "evidence_required": "StrategyDecisionRecord",
                },
            ],
            "decision_status": option_root.get("decision", {}).get("status", "delegate_research"),
        }
    }
    eval_path = path_for("research", f"strategy-eval.{slug}.yaml")
    write_text(eval_path, dump_yaml(eval_data))
    print(f"generated {test_path.relative_to(ROOT)} and {eval_path.relative_to(ROOT)}")
    return 0


def load_strategy_test(test_id: str) -> dict:
    path = path_for("tests", f"{test_id}.yaml")
    if not path.is_file():
        raise FileNotFoundError(path)
    return load_mapping(path)


def command_record_evidence(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        test = load_strategy_test(args.test_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid test: {args.test_id}")
    test_root = test["strategy_test"]
    slug = args.test_id.removeprefix("test.")
    values = template_values(question="Recorded evidence", option_id=test_root.get("strategy_option_id", "option.unknown"))
    values["test_id"] = args.test_id
    values["evidence_id"] = f"evidence.{slug}"
    path = path_for("evidence", f"evidence.{slug}.md")
    write_text(path, render_template("STRATEGY_EVIDENCE.md", values))
    print(f"recorded {path.relative_to(ROOT)}")
    return 0


def command_evaluate(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        option = load_strategy_option(args.option_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid option: {args.option_id}")
    slug = args.option_id.removeprefix("option.")
    eval_path = path_for("research", f"strategy-eval.{slug}.yaml")
    if not eval_path.is_file():
        return fail(f"missing strategy eval for {args.option_id}; run design-tests first")
    report = {
        "option_id": args.option_id,
        "status": option["strategy_option"].get("decision", {}).get("status", "delegate_research"),
        "critical_gap_count": len(option["strategy_option"].get("evidence_gaps", [])),
        "next_action": option["strategy_option"].get("decision", {}).get("next_action", "review evidence"),
    }
    print(json.dumps(report, indent=2))
    return 0


def command_decide(args: argparse.Namespace) -> int:
    ensure_scaffold()
    try:
        option = load_strategy_option(args.option_id)
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return fail(f"missing or invalid option: {args.option_id}")
    values = template_values(
        question=option["strategy_option"].get("strategic_question", "Unknown question"),
        option_id=args.option_id,
    )
    path = path_for("decisions", f"decision.{args.option_id.removeprefix('option.')}.md")
    write_text(path, render_template("STRATEGY_DECISION_RECORD.md", values))
    print(f"generated {path.relative_to(ROOT)}")
    return 0


def command_status(args: argparse.Namespace) -> int:
    ensure_scaffold()
    slug = args.option_id.removeprefix("option.")
    status = {
        "option_id": args.option_id,
        "option_exists": path_for("options", f"{args.option_id}.yaml").is_file(),
        "gap_exists": path_for("gaps", f"gap.{slug}.001.md").is_file(),
        "prompt_exists": path_for("research_prompts", f"prompt.{slug}.md").is_file(),
        "test_exists": path_for("tests", f"test.{slug}.001.yaml").is_file(),
        "decision_exists": path_for("decisions", f"decision.{slug}.md").is_file(),
    }
    if not status["option_exists"]:
        status["next_action"] = "generate-options"
    elif not status["gap_exists"]:
        status["next_action"] = "identify-gaps"
    elif not status["prompt_exists"]:
        status["next_action"] = "generate-research-prompts"
    elif not status["test_exists"]:
        status["next_action"] = "design-tests"
    elif not status["decision_exists"]:
        status["next_action"] = "decide"
    else:
        status["next_action"] = "review"
    print(json.dumps(status, indent=2))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="strategy")
    subparsers = parser.add_subparsers(dest="command", required=True)

    new_parser = subparsers.add_parser("new", help="create a strategy brief from a strategic question")
    new_parser.add_argument("question")
    new_parser.set_defaults(handler=command_new)

    capture_parser = subparsers.add_parser("capture", help="capture a need signal from text or file")
    capture_parser.add_argument("source")
    capture_parser.set_defaults(handler=command_capture)

    for name, help_text, arg_name, handler in (
        ("generate-options", "generate a strategy option from a brief", "brief_id", command_generate_options),
        ("identify-gaps", "generate an evidence gap report for an option", "option_id", command_identify_gaps),
        ("generate-research-prompts", "generate research prompts for an option", "option_id", command_generate_research_prompts),
        ("design-tests", "generate strategy tests and eval for an option", "option_id", command_design_tests),
        ("evaluate", "evaluate a strategy option", "option_id", command_evaluate),
        ("decide", "create a strategy decision record", "option_id", command_decide),
        ("status", "report strategy artifact status", "option_id", command_status),
    ):
        subparser = subparsers.add_parser(name, help=help_text)
        subparser.add_argument(arg_name)
        subparser.set_defaults(handler=handler)

    ingest_parser = subparsers.add_parser("ingest-research-report", help="ingest an external research report")
    ingest_parser.add_argument("report_path")
    ingest_parser.set_defaults(handler=command_ingest_research_report)

    evidence_parser = subparsers.add_parser("record-evidence", help="record strategy evidence for a test")
    evidence_parser.add_argument("test_id")
    evidence_parser.set_defaults(handler=command_record_evidence)

    return parser


def main() -> int:
    ensure_scaffold()
    parser = build_parser()
    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    raise SystemExit(main())
