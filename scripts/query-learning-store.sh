#!/usr/bin/env bash
set -euo pipefail

mode="${1:-summaries}"
limit="${2:-10}"
status="${3:-any}"
job_type="${4:-any}"
db_path="${5:-}"

if [[ -z "$db_path" ]]; then
  db_path=".agent-harness/traces/learning-store.sqlite3"
fi

if [[ ! "$limit" =~ ^[0-9]+$ ]]; then
  echo "limit must be a non-negative integer" >&2
  exit 1
fi

if [[ ! -f "$db_path" ]]; then
  bash scripts/sync-learning-store.sh "$db_path" >/dev/null
fi

python - "$db_path" "$mode" "$limit" "$status" "$job_type" <<'PY'
from __future__ import annotations

import json
import sqlite3
import sys
from pathlib import Path


db_path = Path(sys.argv[1])
mode = sys.argv[2]
limit = int(sys.argv[3])
status = sys.argv[4]
job_type = sys.argv[5]

valid_modes = {
    "summaries",
    "risks",
    "memory-updates",
    "skill-updates",
    "harness-updates",
}
if mode not in valid_modes:
    raise SystemExit(
        "mode must be one of summaries, risks, memory-updates, skill-updates, harness-updates"
    )

conn = sqlite3.connect(db_path)
conn.row_factory = sqlite3.Row

filters: list[str] = []
params: list[object] = []
if status != "any":
    filters.append("lr.verification_status = ?")
    params.append(status)
if job_type != "any":
    filters.append("tr.job_type = ?")
    params.append(job_type)
where_clause = f"WHERE {' AND '.join(filters)}" if filters else ""

base_query = f"""
    SELECT
        tr.trace_id,
        tr.task,
        tr.job_type,
        tr.route_card,
        tr.unresolved_risks_json,
        tr.trace_record,
        tr.synced_at,
        lr.summary,
        lr.verification_status,
        lr.candidate_memory_updates_json,
        lr.candidate_skill_updates_json,
        lr.candidate_harness_updates_json
    FROM trace_records tr
    JOIN learning_reviews lr ON lr.trace_id = tr.trace_id
    {where_clause}
    ORDER BY tr.synced_at DESC, tr.trace_id DESC
    LIMIT ?
"""
rows = conn.execute(base_query, [*params, limit]).fetchall()
conn.close()


def parse_json(value: str) -> object:
    return json.loads(value) if value else []


results: list[dict[str, object]]
if mode == "summaries":
    results = [
        {
            "trace_id": row["trace_id"],
            "task": row["task"],
            "job_type": row["job_type"],
            "verification_status": row["verification_status"],
            "summary": row["summary"],
            "trace_record": row["trace_record"],
            "synced_at": row["synced_at"],
        }
        for row in rows
    ]
elif mode == "risks":
    results = [
        {
            "trace_id": row["trace_id"],
            "job_type": row["job_type"],
            "verification_status": row["verification_status"],
            "unresolved_risks": parse_json(row["unresolved_risks_json"]),
            "trace_record": row["trace_record"],
        }
        for row in rows
        if parse_json(row["unresolved_risks_json"])
    ]
else:
    column_by_mode = {
        "memory-updates": "candidate_memory_updates_json",
        "skill-updates": "candidate_skill_updates_json",
        "harness-updates": "candidate_harness_updates_json",
    }
    column_name = column_by_mode[mode]
    results = []
    for row in rows:
        items = parse_json(row[column_name])
        if not items:
            continue
        results.append(
            {
                "trace_id": row["trace_id"],
                "task": row["task"],
                "job_type": row["job_type"],
                "verification_status": row["verification_status"],
                "items": items,
                "trace_record": row["trace_record"],
            }
        )

print(
    json.dumps(
        {
            "learning_store_db": str(db_path),
            "mode": mode,
            "filters": {
                "limit": limit,
                "verification_status": status,
                "job_type": job_type,
            },
            "results": results,
            "result_count": len(results),
            "source_of_truth": "filesystem artifacts under .agent-harness/",
        },
        indent=2,
    )
)
PY
