#!/usr/bin/env bash
# Runs a live Codex subagent smoke test. This consumes Codex quota.
# Usage: CODEX_LIVE_TEST=1 bash scripts/test-codex-subagents-live.sh
#        bash scripts/test-codex-subagents-live.sh --yes

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_DIR="$ROOT_DIR/scripts"
[ -f "$SCRIPT_DIR/lib/platform.sh" ] && source "$SCRIPT_DIR/lib/platform.sh"
MODEL="${CODEX_LIVE_MODEL:-gpt-5.6-terra}"
ROLE="${CODEX_LIVE_ROLE:-pr_explorer}"
CHILD_MODEL="${CODEX_LIVE_CHILD_MODEL:-gpt-5.6-terra}"
OUTPUT_FILE=""
ERROR_FILE=""

usage() {
  cat <<'EOF'
Usage: scripts/test-codex-subagents-live.sh --yes

Runs a live Codex CLI smoke test that spawns the repo-scoped pr_explorer agent.
This consumes Codex quota. Set CODEX_LIVE_TEST=1 instead of --yes if desired.

Optional:
  CODEX_LIVE_BIN=path          Override Codex executable for version checks
  CODEX_LIVE_MODEL=model-name   Override model, default gpt-5.6-terra
  CODEX_LIVE_ROLE=role-name     Override child role, default pr_explorer
  CODEX_LIVE_CHILD_MODEL=model  Required child model evidence, default gpt-5.6-terra
EOF
}

find_codex() {
  if [ -n "${CODEX_LIVE_BIN:-}" ] && [ -x "${CODEX_LIVE_BIN}" ]; then
    printf '%s\n' "$CODEX_LIVE_BIN"
    return 0
  fi

  case "$(_detect_os)" in
    windows)
      if command -v codex.cmd >/dev/null 2>&1; then
        command -v codex.cmd
        return 0
      fi
      ;;
  esac

  if command -v codex >/dev/null 2>&1; then
    command -v codex
    return 0
  fi

  if command -v codex.cmd >/dev/null 2>&1; then
    command -v codex.cmd
    return 0
  fi

  if [ -n "${APPDATA:-}" ]; then
    local candidate="$APPDATA/npm/codex.cmd"
    if [ -f "$candidate" ]; then
      if command -v cygpath >/dev/null 2>&1; then
        cygpath "$candidate"
      else
        printf '%s\n' "$candidate"
      fi
      return 0
    fi
  fi

  return 1
}

cleanup() {
  [ -n "$OUTPUT_FILE" ] && rm -f "$OUTPUT_FILE"
  [ -n "$ERROR_FILE" ] && rm -f "$ERROR_FILE"
}

main() {
  if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    usage
    exit 0
  fi

  if [ "${CODEX_LIVE_TEST:-}" != "1" ] && [ "${1:-}" != "--yes" ]; then
    echo "SKIP: live Codex subagent test consumes quota."
    echo "Run with --yes or CODEX_LIVE_TEST=1."
    exit 0
  fi

  cd "$ROOT_DIR"

  if [ ! -f ".codex/agents/pr-explorer.toml" ]; then
    echo "ERROR: .codex/agents/pr-explorer.toml not found"
    exit 1
  fi

  local codex_bin
  codex_bin="$(find_codex || true)"
  if [ -z "$codex_bin" ]; then
    echo "ERROR: codex CLI not found in PATH"
    exit 1
  fi

  OUTPUT_FILE="$(_temp_file codex-live-agent)"
  ERROR_FILE="$(_temp_file codex-live-agent-err)"
  trap cleanup EXIT

  local prompt
  prompt="You MUST call the collab spawn_agent tool with agent_type $ROLE before any other tool. Ask that child to inspect its own project agent TOML and report the configured name. Then call wait with the returned child thread id. Do not use shell, do not read the TOML in the parent, and do not substitute a parent-authored answer. If spawn_agent fails, report the failure without guessing."

  echo "Running live Codex subagent smoke with model: $MODEL"
  "$codex_bin" --version
  # Collab spawn resolves the persisted parent thread; --ephemeral makes that
  # parent unavailable and produces a false runtime failure.
  if ! "$codex_bin" -c 'model_reasoning_effort="medium"' exec --json -s read-only -m "$MODEL" "$prompt" >"$OUTPUT_FILE" 2>"$ERROR_FILE"; then
    echo "ERROR: codex exec failed"
    tail -40 "$ERROR_FILE" 2>/dev/null || true
    exit 1
  fi

  if ! node scripts/validate-subagent-trace.js \
    --file "$OUTPUT_FILE" \
    --expected-role "$ROLE" \
    --expected-model "$CHILD_MODEL"; then
    echo "ERROR: live trace does not prove a genuine configured child"
    tail -40 "$ERROR_FILE" 2>/dev/null || true
    tail -40 "$OUTPUT_FILE" 2>/dev/null || true
    exit 1
  fi

  echo "PASS: Codex trace proves $ROLE on $CHILD_MODEL with a distinct child thread"
}

main "$@"
