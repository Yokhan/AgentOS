#!/bin/bash
# measure-context.sh — Measure auto-loaded context size + budget check
# Usage: bash scripts/measure-context.sh [--budget 200]

BUDGET=210
if [ "$1" = "--budget" ] && [ -n "$2" ]; then BUDGET="$2"; fi

echo "=== Context Size Report ==="
echo ""

# CLAUDE.md
CLAUDE_LINES=0
if [ -f CLAUDE.md ]; then
  CLAUDE_LINES=$(wc -l < CLAUDE.md | tr -d ' ')
  echo "CLAUDE.md: $CLAUDE_LINES lines"
fi

# Rules (auto-loaded from .claude/rules/ — should be only router.md)
RULES_LINES=0
RULES_COUNT=0
for f in .claude/rules/*.md; do
  [ -f "$f" ] || continue
  lines=$(wc -l < "$f" | tr -d ' ')
  RULES_LINES=$((RULES_LINES + lines))
  RULES_COUNT=$((RULES_COUNT + 1))
  echo "  $(basename "$f"): $lines lines"
done
echo "Rules total: $RULES_COUNT files, $RULES_LINES lines"

# Total auto-loaded
TOTAL=$((CLAUDE_LINES + RULES_LINES))

# Approximate token estimation for local comparisons.
# Code: ~3 chars/token, prose: ~4 chars/token, blended: ~3.5 chars/token
# Lines average ~40 chars, so ~11 tokens/line for prose, ~13 for code
estimate_tokens() {
  local file="$1"
  local chars=$(wc -c < "$file" 2>/dev/null | tr -d ' ')
  # Detect if code or prose by extension
  case "$file" in
    *.md|*.txt|*.rst) echo $((chars / 4)) ;;  # prose: 4 chars/token
    *.ts|*.js|*.py|*.go|*.rs|*.sh) echo $((chars / 3)) ;;  # code: 3 chars/token
    *) echo $((chars * 2 / 7)) ;;  # blended: 3.5 chars/token
  esac
}

# Accurate token count for auto-loaded files
TOKENS_EST=0
if [ -f CLAUDE.md ]; then
  T=$(estimate_tokens CLAUDE.md)
  TOKENS_EST=$((TOKENS_EST + T))
fi
for f in .claude/rules/*.md; do
  [ -f "$f" ] || continue
  T=$(estimate_tokens "$f")
  TOKENS_EST=$((TOKENS_EST + T))
done

echo ""
echo "AUTO-LOADED TOTAL: $TOTAL lines (~$TOKENS_EST tokens estimated)"
echo "BUDGET: $BUDGET lines"

if [ "$TOTAL" -gt "$BUDGET" ]; then
  OVER=$((TOTAL - BUDGET))
  echo "⚠️  OVER BUDGET by $OVER lines!"
  echo "Action: check .claude/rules/ contains only router.md + project-* files"
  exit 1
else
  UNDER=$((BUDGET - TOTAL))
  echo "✅ WITHIN BUDGET ($UNDER lines remaining)"
fi

# Library stats (not auto-loaded)
echo ""
LIBRARY_LINES=0
LIBRARY_COUNT=0
if [ -d .claude/library ]; then
  for f in $(find .claude/library -name "*.md" 2>/dev/null); do
    lines=$(wc -l < "$f" | tr -d ' ')
    LIBRARY_LINES=$((LIBRARY_LINES + lines))
    LIBRARY_COUNT=$((LIBRARY_COUNT + 1))
  done
  echo "Library (on-demand): $LIBRARY_COUNT files, $LIBRARY_LINES lines"
fi

# Docs stats
DOCS_LINES=0
DOCS_COUNT=0
if [ -d .claude/docs ]; then
  for f in $(find .claude/docs -name "*.md" 2>/dev/null); do
    lines=$(wc -l < "$f" | tr -d ' ')
    DOCS_LINES=$((DOCS_LINES + lines))
    DOCS_COUNT=$((DOCS_COUNT + 1))
  done
  echo "Docs (archive): $DOCS_COUNT files, $DOCS_LINES lines"
fi

echo ""
echo "GRAND TOTAL (all rules): $((TOTAL + LIBRARY_LINES + DOCS_LINES)) lines"
echo "LOADED PER MESSAGE: $TOTAL lines ($(( TOTAL * 100 / (TOTAL + LIBRARY_LINES + DOCS_LINES + 1) ))% of total)"
