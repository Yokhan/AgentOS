#!/usr/bin/env bash
# brain-search.sh — Search Obsidian vault without MCP
# Usage: ./scripts/brain-search.sh "search term" [directory]

set -euo pipefail

QUERY="${1:-}"
SEARCH_DIR="${2:-brain}"

if [ -z "$QUERY" ]; then
    echo "Usage: $0 'search term' [directory]"
    echo "Examples:"
    echo "  $0 'authentication'          # search all brain/"
    echo "  $0 'migration' brain/04-decisions  # search specific dir"
    exit 1
fi

echo "Searching for: '$QUERY' in $SEARCH_DIR/"
echo "================================================"

# Search with context (2 lines before/after)
grep -r --include="*.md" -l "$QUERY" "$SEARCH_DIR" 2>/dev/null | while read -r file; do
    echo ""
    echo "📄 $file"
    echo "---"
    grep -n --color=always -A 2 -B 1 "$QUERY" "$file" 2>/dev/null | head -20
done

echo ""
echo "================================================"
echo "Files found: $(grep -r --include="*.md" -l "$QUERY" "$SEARCH_DIR" 2>/dev/null | wc -l | tr -d ' ')"
