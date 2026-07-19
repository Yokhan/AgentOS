#!/bin/bash
# Session Metrics — collect session stats for tracking improvement velocity
# Called by session-stop.sh or manually: bash scripts/session-metrics.sh
# Output: appends metrics to brain/01-daily/YYYY-MM-DD.md

if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would collect session metrics"
  exit 0
fi

d=$(date +%Y-%m-%d)
METRICS_FILE="brain/01-daily/${d}.md"

# Ensure file exists
if [ ! -d "brain/01-daily" ]; then
  echo "No brain/01-daily directory. Skipping metrics."
  exit 0
fi
if [ ! -f "$METRICS_FILE" ]; then
  echo "# Session Log: ${d}" > "$METRICS_FILE"
fi

echo "" >> "$METRICS_FILE"
echo "## Session Metrics — $(date +%H:%M)" >> "$METRICS_FILE"
echo "" >> "$METRICS_FILE"

# 1. Commits today
COMMITS=$(git log --oneline --since="${d} 00:00" 2>/dev/null | wc -l | tr -d ' ')
echo "- Commits today: $COMMITS" >> "$METRICS_FILE"

# 2. Files changed (unstaged + staged)
CHANGED=$(git diff --name-only HEAD 2>/dev/null | wc -l | tr -d ' ')
STAGED=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
echo "- Files changed (uncommitted): $CHANGED staged, $STAGED cached" >> "$METRICS_FILE"

# 3. Lines added/removed today (from commits)
if [ "$COMMITS" -gt 0 ]; then
  DIFF_STAT=$(git diff --shortstat "$(git log --since="${d} 00:00" --format=%H --reverse 2>/dev/null | head -1)^" HEAD 2>/dev/null || echo "unavailable")
  echo "- Diff stat (today's commits): $DIFF_STAT" >> "$METRICS_FILE"
fi

# 4. Lessons count
if [ -f tasks/lessons.md ]; then
  LESSONS=$(grep -c "^### " tasks/lessons.md 2>/dev/null) || LESSONS=0
  echo "- Lessons total: $LESSONS" >> "$METRICS_FILE"
fi

# 5. Research cache entries
if [ -f tasks/.research-cache.md ]; then
  CACHE_ENTRIES=$(grep -cE "^## \[[0-9]{4}-[0-9]{2}-[0-9]{2}\]" tasks/.research-cache.md 2>/dev/null) || CACHE_ENTRIES=0
  echo "- Research cache entries: $CACHE_ENTRIES" >> "$METRICS_FILE"
fi

# 6. Post-mortems count
if [ -d tasks/post-mortems ]; then
  PM_COUNT=$(find tasks/post-mortems -name "*.md" ! -name "TEMPLATE.md" 2>/dev/null | wc -l | tr -d ' ')
  echo "- Post-mortems: $PM_COUNT" >> "$METRICS_FILE"
fi

# 7. Rules count (template + project)
TEMPLATE_RULES=$(find .claude/rules -name "*.md" ! -name "project-*" 2>/dev/null | wc -l | tr -d ' ')
PROJECT_RULES=$(find .claude/rules -name "project-*.md" 2>/dev/null | wc -l | tr -d ' ')
echo "- Rules: $TEMPLATE_RULES template + $PROJECT_RULES project" >> "$METRICS_FILE"

# 8. File size violations
if [ -d src ]; then
  OVERSIZED=$(find src -type f \( -name "*.ts" -o -name "*.py" -o -name "*.rs" -o -name "*.go" -o -name "*.js" \) -exec sh -c 'test $(wc -l < "$1") -gt 375' _ {} \; -print 2>/dev/null | wc -l | tr -d ' ')
  echo "- Oversized files (>375 lines): $OVERSIZED" >> "$METRICS_FILE"
fi

echo "" >> "$METRICS_FILE"
echo "Metrics saved to $METRICS_FILE"
