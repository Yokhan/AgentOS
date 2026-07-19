#!/usr/bin/env bash
# test-hooks.sh — Validate all Claude Code hooks work on the current platform
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
[ -f "$SCRIPT_DIR/lib/platform.sh" ] && source "$SCRIPT_DIR/lib/platform.sh"

echo "Testing hooks on $(_detect_os)..."
echo ""

errors=0
total=0

for hook in .claude/hooks/*.sh; do
  if [ ! -f "$hook" ]; then
    echo "  No hook files found in .claude/hooks/"
    exit 1
  fi
  total=$((total + 1))
  printf "  %-40s" "$hook"

  # Syntax check first
  if ! bash -n "$hook" 2>/dev/null; then
    echo "FAIL (syntax error)"
    errors=$((errors + 1))
    continue
  fi

  # Dry-run with TEST_MODE
  output=$(TEST_MODE=1 bash "$hook" 2>&1)
  status=$?
  if [ $status -ne 0 ]; then
    echo "FAIL (exit $status)"
    errors=$((errors + 1))
  else
    echo "OK"
  fi
done

echo ""
if [ $errors -gt 0 ]; then
  echo "FAILED: $errors/$total hooks broken"
  exit 1
else
  echo "ALL $total HOOKS OK"
fi
