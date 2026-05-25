#!/usr/bin/env bash
set -euo pipefail

db_path="${1:-}"
if [[ -z "$db_path" ]]; then
  db_path=".agent-harness/traces/learning-store.sqlite3"
fi

if [[ ! -f "$db_path" ]]; then
  bash scripts/sync-learning-store.sh "$db_path" >/dev/null
fi

python - "$db_path" <<'PY'
from __future__ import annotations

import json
import sqlite3
import sys
from pathlib import Path


db_path = Path(sys.argv[1])
conn = sqlite3.connect(db_path)

counts = {
    "learning_reviews": conn.execute("SELECT COUNT(*) FROM learning_reviews").fetchone()[0],
    "verified_reviews": conn.execute(
        "SELECT COUNT(*) FROM learning_reviews WHERE verification_status = 'verified'"
    ).fetchone()[0],
    "distinct_job_types": conn.execute(
        "SELECT COUNT(DISTINCT job_type) FROM trace_records"
    ).fetchone()[0],
    "risk_carrying_reviews": conn.execute(
        "SELECT COUNT(*) FROM trace_records WHERE unresolved_risks_json != '[]'"
    ).fetchone()[0],
}
conn.close()

criteria = {
    "minimum_learning_reviews": 25,
    "minimum_verified_reviews": 15,
    "minimum_distinct_job_types": 3,
    "must_outperform": [
        "filename or grep search on top candidate recall",
        "structured rusql-style metadata queries on exact-match recovery",
    ],
}

thresholds_met = (
    counts["learning_reviews"] >= criteria["minimum_learning_reviews"]
    and counts["verified_reviews"] >= criteria["minimum_verified_reviews"]
    and counts["distinct_job_types"] >= criteria["minimum_distinct_job_types"]
)

if not thresholds_met:
    vector_readiness = "defer"
    recommendation = (
        "Keep structured recall only. Grow the distilled learning corpus and reuse the optional SQLite mirror before evaluating ruvector."
    )
elif counts["risk_carrying_reviews"] == 0:
    vector_readiness = "benchmark"
    recommendation = (
        "Corpus size is sufficient to benchmark semantic retrieval, but benchmark it against exact and structured search before adopting ruvector."
    )
else:
    vector_readiness = "benchmark"
    recommendation = (
        "Structured recall shows enough corpus and friction to justify a retrieval benchmark. Add ruvector only if it beats exact and structured search on representative tasks."
    )

print(
    json.dumps(
        {
            "learning_store_db": str(db_path),
            "vector_readiness": vector_readiness,
            "counts": counts,
            "criteria": criteria,
            "recommendation": recommendation,
        },
        indent=2,
    )
)
PY
