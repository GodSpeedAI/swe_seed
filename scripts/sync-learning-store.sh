#!/usr/bin/env bash
set -euo pipefail

DEFAULT_DB_PATH=".agent-harness/traces/learning-store.sqlite3"
db_path="${1:-}"
if [[ -z "$db_path" ]]; then
  db_path="$DEFAULT_DB_PATH"
fi

python - "$db_path" <<'PY'
from __future__ import annotations

import json
import sqlite3
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


root = Path.cwd()
db_path = Path(sys.argv[1])
db_path.parent.mkdir(parents=True, exist_ok=True)
trace_root = root / ".agent-harness" / "traces" / "records"
trace_paths = sorted(trace_root.glob("*.json"))

conn = sqlite3.connect(db_path)
conn.execute("PRAGMA journal_mode=WAL")
conn.execute(
    """
    CREATE TABLE IF NOT EXISTS trace_records (
        trace_id TEXT PRIMARY KEY,
        task TEXT NOT NULL,
        job_type TEXT NOT NULL,
        route_card TEXT NOT NULL,
        verification_status TEXT NOT NULL,
        latest_summary TEXT NOT NULL,
        unresolved_risks_json TEXT NOT NULL,
        trace_record TEXT NOT NULL,
        route_decision_record TEXT,
        raw_trace_json TEXT NOT NULL,
        synced_at TEXT NOT NULL
    )
    """
)
conn.execute(
    """
    CREATE TABLE IF NOT EXISTS learning_reviews (
        trace_id TEXT PRIMARY KEY,
        summary TEXT NOT NULL,
        verification_status TEXT NOT NULL,
        candidate_memory_updates_json TEXT NOT NULL,
        candidate_skill_updates_json TEXT NOT NULL,
        candidate_harness_updates_json TEXT NOT NULL,
        compression_handoff_json TEXT NOT NULL,
        provenance_json TEXT NOT NULL,
        raw_learning_review_json TEXT NOT NULL,
        synced_at TEXT NOT NULL,
        FOREIGN KEY(trace_id) REFERENCES trace_records(trace_id)
    )
    """
)
conn.execute("CREATE INDEX IF NOT EXISTS idx_traces_job_type ON trace_records(job_type)")
conn.execute("CREATE INDEX IF NOT EXISTS idx_traces_created_at ON trace_records(synced_at)")
conn.execute("CREATE INDEX IF NOT EXISTS idx_learning_reviews_trace_id ON learning_reviews(trace_id)")
conn.execute("CREATE INDEX IF NOT EXISTS idx_learning_reviews_verification_status ON learning_reviews(verification_status)")

trace_count = 0
review_count = 0
synced_at = utc_now()

last_sync = None
try:
    cursor = conn.execute("SELECT MAX(synced_at) FROM trace_records")
    row = cursor.fetchone()
    if row and row[0]:
        last_sync = row[0]
except sqlite3.OperationalError:
    pass

if last_sync:
    print(f"Incremental sync from {last_sync}", file=sys.stderr)
else:
    print("Full sync", file=sys.stderr)

for trace_path in trace_paths:
    if last_sync:
        file_mtime = datetime.fromtimestamp(trace_path.stat().st_mtime, tz=timezone.utc).replace(microsecond=0).isoformat()
        if file_mtime <= last_sync:
            continue
    record = json.loads(trace_path.read_text(encoding="utf-8"))
    try:
        distill_output = subprocess.check_output(
            ["cargo", "run", "-q", "-p", "swe-seed", "--", "trace", "distill", str(trace_path)],
            cwd=root,
            text=True,
            timeout=60,
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
        print(f"Warning: distill failed for {trace_path}: {exc}", file=sys.stderr)
        continue
    learning_review = json.loads(distill_output)["learning_review"]

    conn.execute(
        """
        INSERT OR REPLACE INTO trace_records (
            trace_id,
            task,
            job_type,
            route_card,
            verification_status,
            latest_summary,
            unresolved_risks_json,
            trace_record,
            route_decision_record,
            raw_trace_json,
            synced_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            record.get("trace_id", ""),
            record.get("task", ""),
            learning_review.get("route", {}).get("job_type", "unknown"),
            learning_review.get("route", {}).get("route_card", ""),
            learning_review.get("verification_status", "missing"),
            learning_review.get("summary", ""),
            json.dumps(record.get("unresolved_risks", []), ensure_ascii=False),
            str(trace_path.relative_to(root)),
            record.get("route_decision_record"),
            json.dumps(record, ensure_ascii=False),
            synced_at,
        ),
    )
    conn.execute(
        """
        INSERT OR REPLACE INTO learning_reviews (
            trace_id,
            summary,
            verification_status,
            candidate_memory_updates_json,
            candidate_skill_updates_json,
            candidate_harness_updates_json,
            compression_handoff_json,
            provenance_json,
            raw_learning_review_json,
            synced_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            learning_review.get("trace_id", ""),
            learning_review.get("summary", ""),
            learning_review.get("verification_status", "missing"),
            json.dumps(learning_review.get("candidate_memory_updates", []), ensure_ascii=False),
            json.dumps(learning_review.get("candidate_skill_updates", []), ensure_ascii=False),
            json.dumps(learning_review.get("candidate_harness_updates", []), ensure_ascii=False),
            json.dumps(learning_review.get("compression_handoff", {}), ensure_ascii=False),
            json.dumps(learning_review.get("provenance", {}), ensure_ascii=False),
            json.dumps(learning_review, ensure_ascii=False),
            synced_at,
        ),
    )
    trace_count += 1
    review_count += 1

conn.commit()
conn.close()

print(
    json.dumps(
        {
            "backend": "sqlite3-stdlib-bootstrap",
            "preferred_optional_backend": "rusql",
            "learning_store_db": str(db_path),
            "trace_records_synced": trace_count,
            "learning_reviews_synced": review_count,
            "source_of_truth": "filesystem artifacts under .agent-harness/",
        },
        indent=2,
    )
)
PY
