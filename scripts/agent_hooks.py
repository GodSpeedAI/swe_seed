#!/usr/bin/env python3
from __future__ import annotations

import argparse
import gzip
import json
import sqlite3
import sys
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable
from xml.etree import ElementTree as ET


ROOT = Path.cwd()
OBS_ROOT = ROOT / ".agent-hooks"
LOG_DIR = OBS_ROOT / "logs"
PAYLOAD_ROOT = OBS_ROOT / "payloads"
ARTIFACT_ROOT = OBS_ROOT / "artifacts"
INDEX_DB = OBS_ROOT / "index" / "hooks.rusql"
VECTOR_ROOT = OBS_ROOT / "index" / "vectors"
SCHEMA_VERSION = "1.0"
COMPACT_MIN_SIZE_BYTES = 131072
REDACT_KEYS = ("secret", "token", "password", "api_key", "authorization", "cookie")


@dataclass
class EventRecord:
    envelope: dict[str, Any]
    log_path: str


def utc_now() -> datetime:
    return datetime.now(timezone.utc).replace(microsecond=0)


def timestamp_string(moment: datetime | None = None) -> str:
    return (moment or utc_now()).isoformat().replace("+00:00", "Z")


def date_string(moment: datetime | None = None) -> str:
    return (moment or utc_now()).date().isoformat()


def ensure_layout() -> None:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    PAYLOAD_ROOT.mkdir(parents=True, exist_ok=True)
    ARTIFACT_ROOT.mkdir(parents=True, exist_ok=True)
    INDEX_DB.parent.mkdir(parents=True, exist_ok=True)
    VECTOR_ROOT.mkdir(parents=True, exist_ok=True)


def redact(value: Any) -> Any:
    if isinstance(value, dict):
        redacted: dict[str, Any] = {}
        for key, item in value.items():
            lowered = key.lower()
            if any(fragment in lowered for fragment in REDACT_KEYS):
                redacted[key] = "[REDACTED]"
            else:
                redacted[key] = redact(item)
        return redacted
    if isinstance(value, list):
        return [redact(item) for item in value]
    return value


def write_json(path: Path, payload: Any) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return str(path.relative_to(ROOT))


def write_text(path: Path, content: str) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return str(path.relative_to(ROOT))


def current_log_path(moment: datetime | None = None) -> Path:
    return LOG_DIR / f"events-{date_string(moment)}.jsonl"


def open_log_file(path: Path):
    if path.suffix == ".gz":
        return gzip.open(path, "rt", encoding="utf-8")
    return path.open("r", encoding="utf-8")


def iter_log_paths() -> list[Path]:
    ensure_layout()
    return sorted(list(LOG_DIR.glob("events-*.jsonl")) + list(LOG_DIR.glob("events-*.jsonl.gz")))


def iter_events() -> Iterable[EventRecord]:
    for log_path in iter_log_paths():
        with open_log_file(log_path) as handle:
            for line in handle:
                line = line.strip()
                if not line:
                    continue
                yield EventRecord(envelope=json.loads(line), log_path=str(log_path.relative_to(ROOT)))


def append_event(envelope: dict[str, Any]) -> str:
    ensure_layout()
    log_path = current_log_path()
    with log_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(envelope, ensure_ascii=False) + "\n")
    return str(log_path.relative_to(ROOT))


def find_event(event_id: str) -> EventRecord:
    for record in iter_events():
        if record.envelope.get("event_id") == event_id:
            return record
    raise SystemExit(f"event not found: {event_id}")


def read_ref(ref: str | None, kind: str) -> Any:
    if not ref:
        return None
    path = ROOT / ref
    if not path.exists():
        return None
    if kind == "json":
        return json.loads(path.read_text(encoding="utf-8"))
    return path.read_text(encoding="utf-8")


def build_capture_envelope(payload: dict[str, Any], args: argparse.Namespace) -> dict[str, Any]:
    moment = utc_now()
    event_id = str(uuid.uuid4())
    trace_id = payload.get("trace_id") or payload.get("session_id") or event_id
    span_id = payload.get("span_id") or event_id
    session_id = payload.get("session_id") or trace_id
    turn_id = payload.get("turn_id") or event_id
    native_payload = redact(payload)
    normalized_payload = {
        key: native_payload.get(key)
        for key in (
            "native_event",
            "event",
            "session_id",
            "turn_id",
            "profile",
            "hook_id",
            "script",
            "status",
            "duration_ms",
            "exit_code",
        )
        if key in native_payload
    }
    normalized_payload["attributes"] = redact(payload.get("attributes", {}))
    normalized_payload["message"] = payload.get("message")
    result_payload = redact(payload.get("result", {}))
    stdout_text = str(payload.get("stdout", ""))
    stderr_text = str(payload.get("stderr", ""))
    dated_payload_dir = PAYLOAD_ROOT / date_string(moment)
    dated_artifact_dir = ARTIFACT_ROOT / date_string(moment)
    native_ref = write_json(dated_payload_dir / f"{event_id}.native.json", native_payload)
    normalized_ref = write_json(dated_payload_dir / f"{event_id}.normalized.json", normalized_payload)
    result_ref = write_json(dated_payload_dir / f"{event_id}.result.json", result_payload)
    stdout_ref = write_text(dated_artifact_dir / f"{event_id}.stdout.txt", stdout_text)
    stderr_ref = write_text(dated_artifact_dir / f"{event_id}.stderr.txt", stderr_text)

    return {
        "schema_version": SCHEMA_VERSION,
        "event_id": event_id,
        "trace_id": trace_id,
        "span_id": span_id,
        "parent_span_id": payload.get("parent_span_id"),
        "timestamp": timestamp_string(moment),
        "agent": args.agent,
        "agent_version": args.agent_version,
        "native_event": payload.get("native_event", payload.get("event", "unknown")),
        "event": payload.get("event", payload.get("native_event", "unknown")),
        "session_id": session_id,
        "turn_id": turn_id,
        "cwd": payload.get("cwd", str(ROOT)),
        "repo_root": payload.get("repo_root", str(ROOT)),
        "profile": payload.get("profile", "default"),
        "hook_id": payload.get("hook_id", "manual-capture"),
        "script": payload.get("script", args.script or "scripts/agent-hooks"),
        "status": payload.get("status", "ok"),
        "duration_ms": int(payload.get("duration_ms", 0)),
        "exit_code": int(payload.get("exit_code", 0)),
        "stdout_ref": stdout_ref,
        "stderr_ref": stderr_ref,
        "native_payload_ref": native_ref,
        "normalized_payload_ref": normalized_ref,
        "result_ref": result_ref,
    }


def command_capture(args: argparse.Namespace) -> None:
    raw = sys.stdin.read().strip()
    if not raw:
        raise SystemExit("capture requires JSON on stdin")
    payload = json.loads(raw)
    envelope = build_capture_envelope(payload, args)
    log_path = append_event(envelope)
    print(json.dumps({"event_id": envelope["event_id"], "log_path": log_path, "event": envelope}, indent=2))


def filter_events(session_id: str | None = None, limit: int | None = None) -> list[EventRecord]:
    records = list(iter_events())
    if session_id is not None:
        records = [record for record in records if record.envelope.get("session_id") == session_id]
    records.sort(key=lambda record: (record.envelope.get("timestamp", ""), record.envelope.get("event_id", "")), reverse=True)
    if limit is not None:
        records = records[:limit]
    return records


def command_trace(args: argparse.Namespace) -> None:
    session_id = args.session
    limit = 10 if args.last else None
    records = filter_events(session_id=session_id, limit=limit)
    print(
        json.dumps(
            {
                "session_id": session_id,
                "count": len(records),
                "events": [record.envelope for record in records],
            },
            indent=2,
        )
    )


def inspect_payloads(envelope: dict[str, Any]) -> dict[str, Any]:
    return {
        "native_payload": read_ref(envelope.get("native_payload_ref"), "json"),
        "normalized_payload": read_ref(envelope.get("normalized_payload_ref"), "json"),
        "result_payload": read_ref(envelope.get("result_ref"), "json"),
        "stdout": read_ref(envelope.get("stdout_ref"), "text"),
        "stderr": read_ref(envelope.get("stderr_ref"), "text"),
    }


def command_inspect(args: argparse.Namespace) -> None:
    record = find_event(args.event)
    print(json.dumps({"event": record.envelope, **inspect_payloads(record.envelope)}, indent=2))


def command_replay(args: argparse.Namespace) -> None:
    record = find_event(args.event)
    payloads = inspect_payloads(record.envelope)
    print(
        json.dumps(
            {
                "event_id": record.envelope.get("event_id"),
                "replay_input": payloads["normalized_payload"],
                "native_payload": payloads["native_payload"],
                "result_payload": payloads["result_payload"],
                "stdout": payloads["stdout"],
                "stderr": payloads["stderr"],
            },
            indent=2,
        )
    )


def command_doctor(args: argparse.Namespace) -> None:
    if not args.observability:
        raise SystemExit("doctor currently supports only --observability")
    ensure_layout()
    records = list(iter_events())
    status = "ok" if (OBS_ROOT / "config.yaml").exists() else "missing-config"
    latest_event = records[-1].envelope.get("event_id") if records else None
    print(
        json.dumps(
            {
                "status": status,
                "config": str((OBS_ROOT / "config.yaml").relative_to(ROOT)),
                "log_files": len(iter_log_paths()),
                "event_count": len(records),
                "latest_event_id": latest_event,
                "index_db": str(INDEX_DB.relative_to(ROOT)),
                "vector_index_root": str(VECTOR_ROOT.relative_to(ROOT)),
            },
            indent=2,
        )
    )


def command_compact_logs(args: argparse.Namespace) -> None:
    compacted: list[str] = []
    today_name = current_log_path().name
    for log_path in sorted(LOG_DIR.glob("events-*.jsonl")):
        if log_path.name == today_name:
            continue
        if log_path.stat().st_size < COMPACT_MIN_SIZE_BYTES:
            continue
        gz_path = log_path.with_suffix(log_path.suffix + ".gz")
        with log_path.open("rb") as source, gzip.open(gz_path, "wb") as target:
            target.write(source.read())
        log_path.unlink()
        compacted.append(str(gz_path.relative_to(ROOT)))
    print(json.dumps({"compacted_files": compacted, "count": len(compacted)}, indent=2))


def command_index_rebuild(args: argparse.Namespace) -> None:
    ensure_layout()
    conn = sqlite3.connect(INDEX_DB)
    conn.execute("DROP TABLE IF EXISTS events")
    conn.execute(
        """
        CREATE TABLE events (
            event_id TEXT PRIMARY KEY,
            trace_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            event TEXT NOT NULL,
            hook_id TEXT NOT NULL,
            status TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            log_path TEXT NOT NULL,
            raw_event_json TEXT NOT NULL
        )
        """
    )
    count = 0
    for record in iter_events():
        envelope = record.envelope
        conn.execute(
            """
            INSERT OR REPLACE INTO events (
                event_id,
                trace_id,
                session_id,
                event,
                hook_id,
                status,
                timestamp,
                log_path,
                raw_event_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                envelope.get("event_id", ""),
                envelope.get("trace_id", ""),
                envelope.get("session_id", ""),
                envelope.get("event", ""),
                envelope.get("hook_id", ""),
                envelope.get("status", ""),
                envelope.get("timestamp", ""),
                record.log_path,
                json.dumps(envelope, ensure_ascii=False),
            ),
        )
        count += 1
    conn.commit()
    conn.close()
    print(json.dumps({"index_db": str(INDEX_DB.relative_to(ROOT)), "indexed_events": count}, indent=2))


def maybe_write_output(content: str, output: str | None) -> None:
    if output:
        output_path = ROOT / output
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(content, encoding="utf-8")
    print(content)


def command_export_otel(args: argparse.Namespace) -> None:
    records = [record.envelope for record in iter_events()]
    otel_payload = {
        "resource": {
            "attributes": {
                "service.name": "swe-seed-dev-harness",
                "service.namespace": "dev-harness",
                "service.version": "0.2.0",
                "repo.root": str(ROOT),
            }
        },
        "scopeLogs": [
            {
                "scope": {"name": "agent-hooks"},
                "logRecords": [
                    {
                        "timeUnixNano": envelope.get("timestamp"),
                        "severityText": envelope.get("status", "ok"),
                        "traceId": envelope.get("trace_id"),
                        "spanId": envelope.get("span_id"),
                        "body": {"stringValue": envelope.get("event")},
                        "attributes": [
                            {"key": "event_id", "value": {"stringValue": envelope.get("event_id")}},
                            {"key": "hook_id", "value": {"stringValue": envelope.get("hook_id")}},
                            {"key": "session_id", "value": {"stringValue": envelope.get("session_id")}},
                        ],
                    }
                    for envelope in records
                ],
            }
        ],
    }
    maybe_write_output(json.dumps(otel_payload, indent=2), args.output)


def command_export_junit(args: argparse.Namespace) -> None:
    records = [record.envelope for record in iter_events()]
    suite = ET.Element("testsuite", name="agent-hooks-observability", tests=str(len(records)))
    failures = 0
    for envelope in records:
        case = ET.SubElement(
            suite,
            "testcase",
            classname=envelope.get("hook_id", "agent-hooks"),
            name=envelope.get("event_id", "unknown-event"),
            time=str(envelope.get("duration_ms", 0) / 1000),
        )
        if envelope.get("status") != "ok":
            failures += 1
            failure = ET.SubElement(case, "failure", message=envelope.get("event", "failed-event"))
            failure.text = f"status={envelope.get('status')} exit_code={envelope.get('exit_code')}"
    suite.set("failures", str(failures))
    maybe_write_output(ET.tostring(suite, encoding="unicode"), args.output)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="agent-hooks")
    subparsers = parser.add_subparsers(dest="command", required=True)

    capture = subparsers.add_parser("capture")
    capture.add_argument("--agent", default="unknown")
    capture.add_argument("--agent-version", default="unknown")
    capture.add_argument("--script", default="")
    capture.set_defaults(func=command_capture)

    trace = subparsers.add_parser("trace")
    trace_group = trace.add_mutually_exclusive_group(required=True)
    trace_group.add_argument("--last", action="store_true")
    trace_group.add_argument("--session")
    trace.set_defaults(func=command_trace)

    inspect_command = subparsers.add_parser("inspect")
    inspect_command.add_argument("--event", required=True)
    inspect_command.set_defaults(func=command_inspect)

    replay = subparsers.add_parser("replay")
    replay.add_argument("--event", required=True)
    replay.set_defaults(func=command_replay)

    doctor = subparsers.add_parser("doctor")
    doctor.add_argument("--observability", action="store_true")
    doctor.set_defaults(func=command_doctor)

    compact_logs = subparsers.add_parser("compact-logs")
    compact_logs.set_defaults(func=command_compact_logs)

    index = subparsers.add_parser("index")
    index_subparsers = index.add_subparsers(dest="index_command", required=True)
    index_rebuild = index_subparsers.add_parser("rebuild")
    index_rebuild.set_defaults(func=command_index_rebuild)

    export = subparsers.add_parser("export")
    export_subparsers = export.add_subparsers(dest="export_command", required=True)
    export_otel = export_subparsers.add_parser("otel")
    export_otel.add_argument("--output", default="")
    export_otel.set_defaults(func=command_export_otel)
    export_junit = export_subparsers.add_parser("junit")
    export_junit.add_argument("--output", default="")
    export_junit.set_defaults(func=command_export_junit)

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
