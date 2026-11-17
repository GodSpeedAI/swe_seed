#!/usr/bin/env bash
# Clean-room smoke test for an installed swe-seed binary.
# Usage: scripts/native-smoke.sh /path/to/swe-seed [--tamper]
#
# Runs in a fresh temporary project outside any checkout with SWE_SEED_ROOT
# unset: version -> init (dry-run, apply, rerun, customized conflict) ->
# route -> context-plan -> trace lifecycle -> gate --verify.
# --tamper additionally forges a ledger claim and requires the gate to fail.
# Any failed assertion exits non-zero.
set -euo pipefail

BIN="${1:?usage: native-smoke.sh /path/to/swe-seed [--tamper]}"
TAMPER="${2:-}"
BIN="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
unset SWE_SEED_ROOT

fail() { echo "SMOKE FAIL: $*" >&2; exit 1; }
expect_contains() { grep -q -- "$2" <<<"$1" || fail "expected '$2' in output of: $3"; }

cd "$WORK"
mkdir my-app && cd my-app

echo "== version"
V="$("$BIN" --version)"
expect_contains "$V" "swe-seed" "--version"

echo "== init dry-run writes nothing"
"$BIN" init --dry-run >/dev/null
[ ! -d .agent-harness ] || fail "dry-run created .agent-harness"

echo "== init apply"
OUT="$("$BIN" init)"
expect_contains "$OUT" "summary: create=" "init"

echo "== init rerun is a no-op"
OUT="$("$BIN" init)"
expect_contains "$OUT" "create=0" "init rerun"
expect_contains "$OUT" "conflict=0" "init rerun"

echo "== customized route is preserved as conflict"
printf '{"custom":true}\n' > .agent-harness/routes/bugfix.json
OUT="$("$BIN" init)"
expect_contains "$OUT" "conflict" "init after customization"
grep -q '"custom":true' .agent-harness/routes/bugfix.json || fail "customized route was overwritten"
rm .agent-harness/routes/bugfix.json
"$BIN" init >/dev/null
expect_contains "$("$BIN" route "fix a failing unit test")" '"job_type": "bugfix"' "route after restore"

echo "== route + context-plan"
OUT="$("$BIN" route "fix a failing unit test")"
expect_contains "$OUT" '"job_type": "bugfix"' "route"
OUT="$("$BIN" context-plan "fix a failing unit test")"
expect_contains "$OUT" '"job_type": "bugfix"' "context-plan"

echo "== trace lifecycle + gate"
TRACE="$("$BIN" trace start "fix a failing unit test" | sed -n 's/.*"trace_id": "\([^"]*\)".*/\1/p')"
[ -n "$TRACE" ] || fail "trace start returned no trace_id"
"$BIN" trace checkpoint "$TRACE" --stage verify --summary "smoke" >/dev/null
"$BIN" trace finish "$TRACE" --claim "smoke ok" --result pass >/dev/null
"$BIN" gate "$TRACE" --verify >/dev/null || fail "gate rejected a valid trace"

if [ "$TAMPER" = "--tamper" ]; then
  echo "== forged claim must fail the gate"
  python3 -I -c "
import sqlite3
c = sqlite3.connect('.agent-harness/traces/ledger.sqlite3')
c.execute(\"update chain_entries set payload_json='{\\\"claim\\\":\\\"forged\\\"}' where event_type='TraceFinished'\")
c.commit()"
  if "$BIN" gate "$TRACE" --verify >/dev/null 2>&1; then
    fail "gate accepted a forged ledger claim"
  fi
fi

echo "SMOKE PASS: $BIN"
