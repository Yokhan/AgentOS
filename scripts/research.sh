#!/bin/bash
# research.sh — Automated research protocol (replaces 6+ tool calls with 1)
# Usage: bash scripts/research.sh <file_or_dir> [keywords]

TARGET="${1:-.}"
KEYWORDS="${2:-$(basename "$TARGET")}"

echo "=== RESEARCH: $TARGET ==="

# 1. Files in target
if [ -d "$TARGET" ]; then
  FILE_LIST=$(find "$TARGET" -maxdepth 2 -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.py" -o -name "*.go" -o -name "*.rs" -o -name "*.vue" -o -name "*.svelte" \) -not -name "*.test.*" -not -name "*.spec.*" 2>/dev/null | head -20)
elif [ -f "$TARGET" ]; then
  FILE_LIST="$TARGET"
else
  echo "Target not found: $TARGET"
  exit 1
fi

echo ""
echo "FILES:"
for f in $FILE_LIST; do
  lines=$(wc -l < "$f" 2>/dev/null | tr -d ' ')
  echo "  $f ($lines lines)"
done

# 2. Importers (who imports this?)
echo ""
echo "IMPORTERS:"
target_name=$(basename "$TARGET" | sed 's/\.[^.]*$//')
if [ -d src ] || [ -d lib ] || [ -d app ]; then
  importers=$(grep -rl "$target_name" src/ lib/ app/ 2>/dev/null | grep -v node_modules | head -10)
  if [ -n "$importers" ]; then
    echo "$importers" | while read -r f; do echo "  $f"; done
  else
    echo "  (none found)"
  fi
else
  echo "  (no src/lib/app directory)"
fi

# 3. Recent git changes
echo ""
echo "RECENT GIT:"
if git rev-parse --git-dir > /dev/null 2>&1; then
  git log --oneline -5 -- "$TARGET" 2>/dev/null || echo "  (no git history for this path)"
else
  echo "  (not a git repo)"
fi

# 4. Lessons
echo ""
echo "LESSONS:"
if [ -f tasks/lessons.md ]; then
  matches=$(grep -i "$KEYWORDS" tasks/lessons.md 2>/dev/null | head -5)
  if [ -n "$matches" ]; then
    echo "$matches"
  else
    echo "  (no relevant lessons)"
  fi
else
  echo "  (no tasks/lessons.md)"
fi

# 5. Tool registry
echo ""
echo "REGISTRY:"
if [ -f _reference/tool-registry.md ]; then
  matches=$(grep -i "$KEYWORDS" _reference/tool-registry.md 2>/dev/null | head -5)
  if [ -n "$matches" ]; then
    echo "$matches"
  else
    echo "  (no registry match for '$KEYWORDS')"
  fi
else
  echo "  (no tool registry)"
fi

# 6. Ecosystem
echo ""
echo "ECOSYSTEM:"
if [ -f ecosystem.md ]; then
  matches=$(grep -i "$KEYWORDS" ecosystem.md 2>/dev/null | head -3)
  if [ -n "$matches" ]; then
    echo "$matches"
  else
    echo "  (no ecosystem match)"
  fi
else
  echo "  (no ecosystem.md)"
fi

# 7. Research cache
echo ""
echo "CACHE:"
if [ -f tasks/.research-cache.md ]; then
  matches=$(grep -i "$KEYWORDS" tasks/.research-cache.md 2>/dev/null | head -3)
  if [ -n "$matches" ]; then
    echo "$matches"
  else
    echo "  (no cached research)"
  fi
else
  echo "  (no research cache)"
fi
