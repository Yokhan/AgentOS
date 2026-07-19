#!/usr/bin/env bash
# blast-radius.sh — Heuristic basename/grep fallback, not semantic impact analysis
# Usage: bash scripts/blast-radius.sh <file_path> [--depth N]
# Heuristic text fallback; the parser-backed graph is the primary impact source.
#
# Given a file, finds all files that directly or transitively import it.
# Shows: affected files, routes, tests, and estimated blast radius level.

set -euo pipefail

echo "NOTE: heuristic grep fallback; use codebase-memory-mcp detect_changes for structural impact." >&2

TARGET="${1:?Usage: blast-radius.sh <file_path> [--depth N]}"
MAX_DEPTH="${3:-5}"

if [ "$2" = "--depth" ] 2>/dev/null && [ -n "${3:-}" ]; then
  MAX_DEPTH="$3"
fi

# Normalize target
TARGET_BASE=$(basename "$TARGET" | sed 's/\.[^.]*$//')
TARGET_DIR=$(dirname "$TARGET")

echo "=== BLAST RADIUS: $TARGET ==="
echo ""

# Find all source files
SRC_EXTENSIONS="ts,tsx,js,jsx,py,go,rs,vue,svelte,rb,java,kt,cs"
SRC_FILES=$(find . -type f \( \
  -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" \
  -o -name "*.py" -o -name "*.go" -o -name "*.rs" -o -name "*.vue" \
  -o -name "*.svelte" -o -name "*.rb" -o -name "*.java" -o -name "*.kt" \
  -o -name "*.cs" \
  \) -not -path "*/node_modules/*" -not -path "*/.git/*" -not -path "*/dist/*" \
  -not -path "*/build/*" -not -path "*/__pycache__/*" -not -path "*/.next/*" \
  2>/dev/null)

[ -z "$SRC_FILES" ] && echo "No source files found." && exit 0

# BFS: find all files importing the target (direct + transitive)
declare -A VISITED
declare -A DEPTH_MAP
QUEUE=("$TARGET")
VISITED["$TARGET"]=1
DEPTH_MAP["$TARGET"]=0

DIRECT_IMPORTERS=()
ALL_AFFECTED=()

while [ ${#QUEUE[@]} -gt 0 ]; do
  CURRENT="${QUEUE[0]}"
  QUEUE=("${QUEUE[@]:1}")

  CURRENT_DEPTH=${DEPTH_MAP["$CURRENT"]:-0}
  [ "$CURRENT_DEPTH" -ge "$MAX_DEPTH" ] && continue

  CURRENT_BASE=$(basename "$CURRENT" | sed 's/\.[^.]*$//')

  # Find files importing current file
  IMPORTERS=$(echo "$SRC_FILES" | xargs grep -l \
    -e "from.*['\"].*${CURRENT_BASE}['\"]" \
    -e "import.*['\"].*${CURRENT_BASE}['\"]" \
    -e "require.*['\"].*${CURRENT_BASE}['\"]" \
    -e "from ${CURRENT_BASE}" \
    2>/dev/null || true)

  for IMP in $IMPORTERS; do
    if [ -z "${VISITED[$IMP]:-}" ]; then
      VISITED["$IMP"]=1
      DEPTH_MAP["$IMP"]=$((CURRENT_DEPTH + 1))
      QUEUE+=("$IMP")
      ALL_AFFECTED+=("$IMP")

      if [ "$CURRENT" = "$TARGET" ]; then
        DIRECT_IMPORTERS+=("$IMP")
      fi
    fi
  done
done

# Categorize results
TESTS=()
ROUTES=()
COMPONENTS=()
OTHER=()

for f in "${ALL_AFFECTED[@]}"; do
  case "$f" in
    *.test.*|*.spec.*|*__test__*|*_test.go|*_test.py)
      TESTS+=("$f") ;;
    */route*|*/api/*|*/pages/*|*/app/*)
      ROUTES+=("$f") ;;
    */component*|*/Component*|*.vue|*.svelte)
      COMPONENTS+=("$f") ;;
    *)
      OTHER+=("$f") ;;
  esac
done

TOTAL=${#ALL_AFFECTED[@]}

# Output
echo "Direct importers (depth 1): ${#DIRECT_IMPORTERS[@]}"
for f in "${DIRECT_IMPORTERS[@]}"; do
  echo "  $f"
done

echo ""
echo "Total affected (depth $MAX_DEPTH): $TOTAL"

if [ ${#ROUTES[@]} -gt 0 ]; then
  echo ""
  echo "Routes/APIs affected: ${#ROUTES[@]}"
  for f in "${ROUTES[@]}"; do echo "  $f"; done
fi

if [ ${#COMPONENTS[@]} -gt 0 ]; then
  echo ""
  echo "Components affected: ${#COMPONENTS[@]}"
  for f in "${COMPONENTS[@]}"; do echo "  $f"; done
fi

if [ ${#TESTS[@]} -gt 0 ]; then
  echo ""
  echo "Tests to run: ${#TESTS[@]}"
  for f in "${TESTS[@]}"; do echo "  $f"; done
fi

if [ ${#OTHER[@]} -gt 0 ]; then
  echo ""
  echo "Other files: ${#OTHER[@]}"
  for f in "${OTHER[@]}"; do echo "  $f"; done
fi

# Blast radius level
echo ""
if [ "$TOTAL" -eq 0 ]; then
  echo "BLAST RADIUS: NONE — file appears to be a leaf (no importers)"
elif [ "$TOTAL" -le 3 ]; then
  echo "BLAST RADIUS: LOW ($TOTAL files)"
elif [ "$TOTAL" -le 10 ]; then
  echo "BLAST RADIUS: MEDIUM ($TOTAL files) — review carefully"
elif [ "$TOTAL" -le 25 ]; then
  echo "BLAST RADIUS: HIGH ($TOTAL files) — consider incremental changes"
else
  echo "BLAST RADIUS: CRITICAL ($TOTAL files) — this is a core dependency!"
fi
