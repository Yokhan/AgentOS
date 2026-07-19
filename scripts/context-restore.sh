#!/bin/bash
# context-restore.sh — Restore context after compaction or session start
# Usage: bash scripts/context-restore.sh
# Replaces 5+ Read calls with 1 bash call

echo "=== CONTEXT RESTORE ==="
echo ""

# Mode
if [ -f "tasks/.mode" ]; then
  MODE=$(grep "^MODE=" tasks/.mode 2>/dev/null | cut -d= -f2)
  echo "MODE: $MODE"
else
  echo "MODE: (not set — run /mode-* or route-task.sh)"
fi

# Active rules
if [ -f "tasks/.active-rules" ]; then
  TASK=$(grep "^TASK=" tasks/.active-rules 2>/dev/null | cut -d= -f2)
  MODES=$(grep "^MODES=" tasks/.active-rules 2>/dev/null | cut -d= -f2)
  AGENT=$(grep "^AGENT=" tasks/.active-rules 2>/dev/null | cut -d= -f2)
  COUNT=$(grep "^COUNT=" tasks/.active-rules 2>/dev/null | cut -d= -f2)
  echo "ACTIVE TASK: $TASK"
  echo "ACTIVE MODES: $MODES"
  echo "AGENT: $AGENT"
  echo "RULES LOADED: $COUNT"
  echo ""
  echo "Rule files to re-read:"
  grep "^\.claude/library/" tasks/.active-rules 2>/dev/null | while read -r f; do echo "  $f"; done
else
  echo "ACTIVE TASK: (none — run route-task.sh)"
fi

# Current task handoff
echo ""
if [ -f "tasks/current.md" ]; then
  echo "--- HANDOFF (tasks/current.md) ---"
  if [ -f "scripts/task-brief.sh" ]; then
    bash scripts/task-brief.sh --brief 2>/dev/null || head -30 tasks/current.md 2>/dev/null
  else
    head -30 tasks/current.md 2>/dev/null
  fi
else
  echo "HANDOFF: (no tasks/current.md)"
fi

# Compaction snapshot
echo ""
if [ -f "tasks/.compaction-snapshot.md" ]; then
  echo "--- COMPACTION SNAPSHOT ---"
  head -20 tasks/.compaction-snapshot.md 2>/dev/null
fi

# Recent git
echo ""
echo "--- RECENT GIT ---"
if git rev-parse --git-dir > /dev/null 2>&1; then
  git log --oneline -5 2>/dev/null || echo "(empty)"
  echo ""
  uncommitted=$(git diff --name-only HEAD 2>/dev/null | head -10)
  if [ -n "$uncommitted" ]; then
    echo "UNCOMMITTED:"
    echo "$uncommitted"
  fi
else
  echo "(not a git repo)"
fi

# Recent lessons
echo ""
echo "--- RECENT LESSONS ---"
if [ -f "tasks/lessons.md" ]; then
  tail -15 tasks/lessons.md 2>/dev/null
else
  echo "(no lessons)"
fi

# Research cache
echo ""
if [ -f "tasks/.research-cache.md" ]; then
  ACTIVE_ENTRIES=$(grep -cE "^## \[[0-9]{4}-[0-9]{2}-[0-9]{2}\]" tasks/.research-cache.md 2>/dev/null) || ACTIVE_ENTRIES=0
  echo "RESEARCH CACHE: $ACTIVE_ENTRIES entries"
fi
