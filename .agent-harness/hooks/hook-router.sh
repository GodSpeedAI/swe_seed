#!/usr/bin/env bash
set -euo pipefail

event="${1:-}"
shift || true

task=""
agent="unknown"
agent_version="unknown"
session_id=""
trace_id=""
span_id=""
parent_span_id=""
capture=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --task)
      task="${2:-}"
      shift 2
      ;;
    --agent)
      agent="${2:-unknown}"
      shift 2
      ;;
    --agent-version)
      agent_version="${2:-unknown}"
      shift 2
      ;;
    --session-id)
      session_id="${2:-}"
      shift 2
      ;;
    --trace-id)
      trace_id="${2:-}"
      shift 2
      ;;
    --span-id)
      span_id="${2:-}"
      shift 2
      ;;
    --parent-span-id)
      parent_span_id="${2:-}"
      shift 2
      ;;
    --capture)
      capture=true
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

emit() {
  local event_name="$1"
  local purpose="$2"
  local action="$3"
  local boundary="$4"

  printf 'event: %s\n' "$event_name"
  printf 'purpose: %s\n' "$purpose"
  printf 'action: %s\n' "$action"
  printf 'boundary: %s\n' "$boundary"
}

capture_event() {
  local event_name="$1"
  local purpose="$2"
  local action="$3"
  local boundary="$4"
  local route_preview="{}"

  if [[ "$event_name" == "prompt.submit" && -n "$task" ]]; then
    route_preview="$(cd "$repo_root" && python scripts/harness.py route "$task")"
  fi

  local effective_trace_id="$trace_id"
  local effective_session_id="$session_id"
  local effective_span_id="$span_id"

  if [[ -z "$effective_trace_id" ]]; then
    effective_trace_id="$effective_session_id"
  fi

  if [[ -z "$effective_session_id" ]]; then
    effective_session_id="$effective_trace_id"
  fi

  if [[ -z "$effective_span_id" ]]; then
    effective_span_id="$event_name"
  fi

  python - "$event_name" "$purpose" "$action" "$boundary" "$task" "$effective_session_id" "$effective_trace_id" "$effective_span_id" "$parent_span_id" "$route_preview" <<'PY' \
    | (cd "$repo_root" && scripts/agent-hooks capture --agent "$agent" --agent-version "$agent_version" --script ".agent-harness/hooks/hook-router.sh")
from __future__ import annotations

import json
import sys

event_name, purpose, action, boundary, task, session_id, trace_id, span_id, parent_span_id, route_preview = sys.argv[1:11]

payload = {
    "native_event": event_name,
    "event": event_name,
    "session_id": session_id,
    "trace_id": trace_id,
    "span_id": span_id,
    "parent_span_id": parent_span_id or None,
    "profile": "agent-harness-hook",
    "hook_id": event_name,
    "script": ".agent-harness/hooks/hook-router.sh",
    "status": "ok",
    "message": purpose,
    "result": {
        "purpose": purpose,
        "action": action,
        "boundary": boundary,
        "task": task or None,
        "route_preview": json.loads(route_preview),
    },
}

print(json.dumps(payload))
PY
}

emit_or_capture() {
  local event_name="$1"
  local purpose="$2"
  local action="$3"
  local boundary="$4"

  if [[ "$capture" == true ]]; then
    capture_event "$event_name" "$purpose" "$action" "$boundary"
  else
    emit "$event_name" "$purpose" "$action" "$boundary"
  fi
}

case "$event" in
  session.start)
    emit_or_capture \
      "session.start" \
      "Orient the agent before it spends attention." \
      "Read AGENTS.md, .agent-harness/context/README.md, .agent-harness/memory/repo-map.md, .agent-harness/memory/constraints.md, and the selected route card when known." \
      "Do not perform project edits from this hook."
    ;;
  prompt.submit)
    emit_or_capture \
      "prompt.submit" \
      "Convert user intent into an executable route plan." \
      "Run or mirror python scripts/harness.py route '<task>'; if the route lacks next action, treat the router as defective." \
      "Ask for clarification only when route choice materially changes the work."
    ;;
  tool.pre)
    emit_or_capture \
      "tool.pre" \
      "Prevent high-cost mistakes before tools run." \
      "Check workspace, command intent, context budget, secret risk, and destructive-operation risk before execution." \
      "Do not block ordinary read, validation, or formatting commands."
    ;;
  tool.post)
    emit_or_capture \
      "tool.post" \
      "Preserve evidence while it is still fresh." \
      "Record changed files, command result, proof relevance, and tool-output containment decisions in the trace draft when the command affects completion." \
      "Do not write secrets, raw environment dumps, or noisy logs into traces."
    ;;
  turn.stop)
    emit_or_capture \
      "turn.stop" \
      "Stop false completion claims." \
      "Compare the route card's proof and done_when fields against observed evidence before final response." \
      "If proof is missing, report the gap instead of claiming completion."
    ;;
  session.end)
    emit_or_capture \
      "session.end" \
      "Capture reusable learning without bloating memory." \
      "Write a reflection or improvement proposal only when there is evidence of repeated friction or a changed harness contract." \
      "Do not auto-apply material harness changes while learning.auto_apply is false."
    ;;
  *)
    echo "usage: $0 <session.start|prompt.submit|tool.pre|tool.post|turn.stop|session.end>" >&2
    exit 2
    ;;
esac
