#!/bin/bash
# set-mode.sh — manually set task mode (alternative to route-task.sh)
# Usage: bash scripts/set-mode.sh code|design|review|research|write|fix|plan

MODE="${1:-}"
LIB=".claude/library"
ACTIVE="tasks/.active-rules"

if [ -z "$MODE" ]; then
  echo "Usage: bash scripts/set-mode.sh <mode>"
  echo "Modes: code design review research write fix plan"
  exit 1
fi

case "$MODE" in
  code)     RULES="process/research-first process/plan-first process/self-verification technical/architecture technical/code-style technical/error-handling technical/atomic-reuse" ;;
  design)   RULES="domain/domain-design-pipeline meta/analysis technical/writing process/research-first technical/atomic-reuse" ;;
  review)   RULES="meta/analysis meta/critical-thinking process/self-verification technical/testing" ;;
  research) RULES="meta/analysis meta/critical-thinking meta/strategic-thinking process/research-first" ;;
  write)    RULES="technical/writing domain/domain-guards meta/critical-thinking" ;;
  fix)      RULES="technical/error-handling technical/testing meta/analysis process/research-first process/self-verification" ;;
  plan)     RULES="meta/strategic-thinking process/plan-first conflict/conflict-resolution process/research-first" ;;
  *)        echo "Unknown mode: $MODE"; echo "Available: code design review research write fix plan"; exit 1 ;;
esac

# Always include context-first
RULES="process/context-first $RULES"

# Build file list and count
FILES=""
LINE_COUNT=0
FILE_COUNT=0
for r in $RULES; do
  f="$LIB/$r.md"
  if [ -f "$f" ]; then
    FILES="$FILES $f"
    lines=$(wc -l < "$f" | tr -d ' ')
    LINE_COUNT=$((LINE_COUNT + lines))
    FILE_COUNT=$((FILE_COUNT + 1))
  fi
done

# Save
mkdir -p tasks
{
  echo "TASK=manual mode: $MODE"
  echo "MODES=$MODE"
  echo "AGENT="
  echo "COUNT=$FILE_COUNT files, $LINE_COUNT lines"
  echo "ROUTED_AT=$(date -u +%Y-%m-%dT%H:%M 2>/dev/null || date +%Y-%m-%dT%H:%M)"
  echo "---"
  for f in $FILES; do echo "$f"; done
} > "$ACTIVE"

echo "=== Mode: $MODE ==="
echo "Rules: $FILE_COUNT files, ~$LINE_COUNT lines"
echo "READ these files:"
for f in $FILES; do echo "  $f"; done
