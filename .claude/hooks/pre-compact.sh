#!/bin/bash
# Pre-Compaction Hook — save state before context compression

if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would save pre-compaction state"
  exit 0
fi

# Save compaction snapshot for context recovery
SNAPSHOT="tasks/.compaction-snapshot.md"
echo "## Pre-Compaction Snapshot — $(date -Iseconds 2>/dev/null || date +%Y-%m-%dT%H:%M:%S)" > "$SNAPSHOT"
echo "" >> "$SNAPSHOT"

# Current task context
echo "### Current Task" >> "$SNAPSHOT"
if [ -f tasks/current.md ]; then
  head -30 tasks/current.md >> "$SNAPSHOT"
fi
echo "" >> "$SNAPSHOT"

# Modified files
echo "### Modified Files" >> "$SNAPSHOT"
git diff --name-only 2>/dev/null >> "$SNAPSHOT"
git diff --cached --name-only 2>/dev/null >> "$SNAPSHOT"
echo "" >> "$SNAPSHOT"

# Recent lessons
echo "### Recent Lessons" >> "$SNAPSHOT"
if [ -f tasks/lessons.md ]; then
  tail -15 tasks/lessons.md >> "$SNAPSHOT"
fi

# Active rules (for route restoration after compaction)
echo "" >> "$SNAPSHOT"
echo "### Active Rules" >> "$SNAPSHOT"
if [ -f tasks/.active-rules ]; then
  cat tasks/.active-rules >> "$SNAPSHOT"
else
  echo "No active rules set. Run route-task.sh after compaction." >> "$SNAPSHOT"
fi

# Research cache preservation hint
echo "" >> "$SNAPSHOT"
echo "### Research Cache" >> "$SNAPSHOT"
if [ -f tasks/.research-cache.md ] && [ -s tasks/.research-cache.md ]; then
  # Include active entries count
  ACTIVE=$(grep -cE "^## \[[0-9]{4}-[0-9]{2}-[0-9]{2}\]" tasks/.research-cache.md 2>/dev/null) || ACTIVE=0
  echo "Research cache has $ACTIVE entries. Read tasks/.research-cache.md for full context." >> "$SNAPSHOT"
else
  echo "No research cache entries." >> "$SNAPSHOT"
fi

# Also save to daily log
if [ -d "brain/01-daily" ]; then
  d=$(date +%Y-%m-%d)
  f="brain/01-daily/${d}.md"
  if [ ! -f "$f" ]; then
    echo "# Session Log: ${d}" > "$f"
  fi
  echo "" >> "$f"
  echo "## Pre-compaction save: $(date +%H:%M)" >> "$f"
  changed=$(git diff --name-only HEAD 2>/dev/null || true)
  if [ -n "$changed" ]; then
    echo "Changed files:" >> "$f"
    echo "$changed" >> "$f"
  fi
fi

echo "Snapshot saved to $SNAPSHOT. After compaction, read tasks/.compaction-snapshot.md to restore context."

exit 0
