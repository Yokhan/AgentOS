#!/usr/bin/env bash
# codex-hook-adapter.sh — Translates Codex hook env vars to Claude hook format
# Usage: bash scripts/codex-hook-adapter.sh <event-type>
# Events: session-start, pre-tool-use, post-tool-use, session-stop
set -euo pipefail

normalize_drive_path() {
  local path="$1"
  case "$path" in
    /[A-Z]/*)
      printf '/%s%s\n' "$(printf '%s' "${path:1:1}" | tr 'A-Z' 'a-z')" "${path:2}"
      ;;
    *)
      printf '%s\n' "$path"
      ;;
  esac
}

SCRIPT_DIR="$(normalize_drive_path "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)")"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_DIR/.claude/hooks"
EVENT="${1:-}"

if [[ -z "$EVENT" ]]; then
  echo "Usage: codex-hook-adapter.sh <event-type>" >&2
  exit 1
fi

# Translate Codex hook JSON stdin to Claude-compatible env vars.
# Keep env fallbacks for older adapters and local tests.
HOOK_INPUT_JSON=""
if [[ ! -t 0 ]]; then
  HOOK_INPUT_JSON="$(cat 2>/dev/null || true)"
fi

json_value() {
  local key="$1"
  HOOK_INPUT_JSON="$HOOK_INPUT_JSON" node -e '
const key = process.argv[1];
let data = {};
try { data = JSON.parse(process.env.HOOK_INPUT_JSON || "{}"); } catch {}
const input = data.tool_input || data.toolInput || {};
const values = {
  tool_name: data.tool_name || data.toolName || data.tool || data.name || "",
  tool_input: JSON.stringify(input || {}),
  file_path: data.file_path || data.filePath || input.file_path || input.filePath || input.path || "",
  command: data.command || input.command || input.cmd || input.input || ""
};
process.stdout.write(String(values[key] || ""));
' "$key" 2>/dev/null || true
}

export TOOL_NAME="${CODEX_TOOL_NAME:-${TOOL_NAME:-$(json_value tool_name)}}"
export TOOL_INPUT="${CODEX_TOOL_INPUT:-${TOOL_INPUT:-$(json_value tool_input)}}"
export FILE_PATH="${CODEX_FILE_PATH:-${FILE_PATH:-$(json_value file_path)}}"
export COMMAND="${CODEX_COMMAND:-${COMMAND:-$(json_value command)}}"

ensure_codex_route_before_write() {
  if [[ "${TEST_MODE:-}" = "1" ]]; then
    return 0
  fi

  local route_state="$PROJECT_DIR/tasks/.active-codex-route.json"
  if [[ -f "$route_state" ]]; then
    return 0
  fi

  cat >&2 <<'JSON'
{"block":true,"message":"Codex route state missing. Run: node scripts/codex-route-task.js \"<user request>\" --summary --write-state"}
JSON
  exit 2
}

# Route to appropriate Claude hook scripts based on event type
case "$EVENT" in
  session-start)
    if [[ -x "$HOOKS_DIR/session-start.sh" ]]; then
      bash "$HOOKS_DIR/session-start.sh"
    fi
    ;;

  pre-tool-use)
    # Run security hooks: deny sensitive paths + bash safety
    if [[ -x "$HOOKS_DIR/deny-sensitive-paths.sh" ]]; then
      bash "$HOOKS_DIR/deny-sensitive-paths.sh" || exit $?
    fi
    # For shell/bash commands, run pre-bash-safety
    if [[ "$TOOL_NAME" == "shell" || "$TOOL_NAME" == "bash" || "$TOOL_NAME" == "Bash" ]]; then
      if [[ -x "$HOOKS_DIR/pre-bash-safety.sh" ]]; then
        bash "$HOOKS_DIR/pre-bash-safety.sh" || exit $?
      fi
    fi
    # For file edit/write, run pre-edit-safety (branch protection + secret scanning)
    if [[ "$TOOL_NAME" == "file_edit" || "$TOOL_NAME" == "file_write" || "$TOOL_NAME" == "Edit" || "$TOOL_NAME" == "Write" || "$TOOL_NAME" == "apply_patch" ]]; then
      ensure_codex_route_before_write
      if [[ -x "$HOOKS_DIR/pre-edit-safety.sh" ]]; then
        bash "$HOOKS_DIR/pre-edit-safety.sh" || exit $?
      fi
    fi
    ;;

  post-tool-use)
    # Run prompt injection defender
    if [[ -x "$HOOKS_DIR/prompt-injection-defender.sh" ]]; then
      bash "$HOOKS_DIR/prompt-injection-defender.sh" || true
    fi
    # Run encoding check for file operations
    if [[ "$TOOL_NAME" == "file_write" || "$TOOL_NAME" == "file_edit" || "$TOOL_NAME" == "Write" || "$TOOL_NAME" == "Edit" || "$TOOL_NAME" == "apply_patch" ]]; then
      if [[ -x "$HOOKS_DIR/check-encoding.sh" ]]; then
        bash "$HOOKS_DIR/check-encoding.sh" || true
      fi
      if [[ -f "$PROJECT_DIR/scripts/validate-design-policy.js" ]]; then
        node "$PROJECT_DIR/scripts/validate-design-policy.js" --hook || true
      fi
    fi
    # Run session audit logger
    if [[ -x "$HOOKS_DIR/session-audit.sh" ]]; then
      bash "$HOOKS_DIR/session-audit.sh" || true
    fi
    ;;

  session-stop)
    if [[ -x "$HOOKS_DIR/session-stop.sh" ]]; then
      bash "$HOOKS_DIR/session-stop.sh"
    fi
    ;;

  *)
    echo "Unknown event: $EVENT" >&2
    echo "Valid events: session-start, pre-tool-use, post-tool-use, session-stop" >&2
    exit 1
    ;;
esac
