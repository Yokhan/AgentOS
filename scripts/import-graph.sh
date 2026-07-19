#!/usr/bin/env bash
# import-graph.sh - Heuristic relative-import fallback, not a semantic code graph
# Usage: bash scripts/import-graph.sh [directory] [--top N]
#
# Shows which files are imported most frequently across the codebase.
# Hot files = high blast radius = change carefully.
# Heuristic text fallback; this is not a semantic dependency graph.

set -euo pipefail

echo "NOTE: heuristic grep fallback; use codebase-memory-mcp for structural or blast-radius claims." >&2

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

SEARCH_DIR="${1:-.}"
TOP_N=20
RESULTS_FILE="$(_temp_file import-graph-results)"

cleanup() {
  rm -f "$RESULTS_FILE"
}
trap cleanup EXIT

if [ "${2:-}" = "--top" ] && [ -n "${3:-}" ]; then
  TOP_N="$3"
fi

echo "=== IMPORT GRAPH: $SEARCH_DIR ==="
echo ""

SRC_FILES=$(find "$SEARCH_DIR" -type f \( \
  -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" \
  -o -name "*.py" -o -name "*.go" -o -name "*.rs" -o -name "*.vue" \
  -o -name "*.svelte" \
  \) -not -path "*/node_modules/*" -not -path "*/.git/*" -not -path "*/dist/*" \
  -not -path "*/build/*" -not -path "*/__pycache__/*" \
  2>/dev/null)

[ -z "$SRC_FILES" ] && echo "No source files found in $SEARCH_DIR" && exit 0

TOTAL_FILES=$(echo "$SRC_FILES" | wc -l | tr -d ' ')
echo "Scanning $TOTAL_FILES source files..."
echo ""

echo "$SRC_FILES" | xargs grep -hE \
  "from\s+['\"]|import\s+['\"]|require\s*\(['\"]" \
  2>/dev/null | \
  sed -E "s/.*from\s+['\"]([^'\"]+)['\"].*/\1/; s/.*import\s+['\"]([^'\"]+)['\"].*/\1/; s/.*require\s*\(['\"]([^'\"]+)['\"].*/\1/" | \
  grep -E "^\.\.?/" | \
  sed -E 's/\.(ts|tsx|js|jsx|vue|svelte|py)$//; s/\/index$//' | \
  sort | uniq -c | sort -rn | head -"$TOP_N" > "$RESULTS_FILE" 2>/dev/null || true

if [ ! -s "$RESULTS_FILE" ]; then
  echo "No local imports found."
  exit 0
fi

echo "TOP $TOP_N MOST IMPORTED FILES (hot files):"
printf '%s\n' "---------------------------------------------"
printf "%-6s  %s\n" "REFS" "MODULE"
printf '%s\n' "---------------------------------------------"

while read -r count module; do
  if [ "$count" -ge 20 ]; then
    RISK="CRITICAL"
  elif [ "$count" -ge 10 ]; then
    RISK="HIGH"
  elif [ "$count" -ge 5 ]; then
    RISK="MEDIUM"
  else
    RISK="LOW"
  fi
  printf "%-6s  %-50s  [%s]\n" "$count" "$module" "$RISK"
done < "$RESULTS_FILE"

echo ""
printf '%s\n' "---------------------------------------------"

ORPHAN_COUNT=0
echo "$SRC_FILES" | while read -r f; do
  BASENAME=$(basename "$f" | sed 's/\.[^.]*$//')
  case "$f" in
    *.test.*|*.spec.*|*config*|*main.*|*index.*|*App.*) continue ;;
  esac
  REFS=$(echo "$SRC_FILES" | xargs grep -l "$BASENAME" 2>/dev/null | grep -v "$f" | wc -l | tr -d ' ')
  if [ "$REFS" -eq 0 ]; then
    ORPHAN_COUNT=$((ORPHAN_COUNT + 1))
  fi
done 2>/dev/null

echo ""
echo "Use 'bash scripts/blast-radius.sh <file>' to analyze a specific hot file."
