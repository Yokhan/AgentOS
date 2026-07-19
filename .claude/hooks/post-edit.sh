#!/bin/bash
# Post-Edit Hook — file size check, syntax validation, deep import detection
# Receives $FILE_PATH from Claude Code environment

if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would check file size and syntax"
  exit 0
fi

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

# File size check (Working Memory Cliff — 375 lines for 1M context models)
if [ -f "$FILE_PATH" ]; then
  lines=$(wc -l < "$FILE_PATH" 2>/dev/null || echo 0)
  lines=$(echo "$lines" | tr -d ' ')
  if [ "$lines" -gt 375 ] 2>/dev/null; then
    echo "WARNING: $FILE_PATH has $lines lines (limit 375, Working Memory Cliff). Consider splitting."
  fi
fi

# CLAUDE.md size check (hard limit 300 for 1M context)
case "$FILE_PATH" in
  *CLAUDE.md)
    lines=$(wc -l < "$FILE_PATH" 2>/dev/null || echo 0)
    lines=$(echo "$lines" | tr -d ' ')
    if [ "$lines" -gt 300 ] 2>/dev/null; then
      echo "WARNING: CLAUDE.md has $lines lines (limit 300). Consider trimming."
    fi
    ;;
esac

# Syntax validation
case "$FILE_PATH" in
  *.json)
    node -e "JSON.parse(require('fs').readFileSync('$FILE_PATH','utf8'))" 2>/dev/null || echo "WARNING: Invalid JSON in $FILE_PATH"
    ;;
  *.sh)
    bash -n "$FILE_PATH" 2>&1 || true
    ;;
esac

# Deep import detection
if [ -f "$FILE_PATH" ]; then
  grep -nE "from [\"'][^\"/]+(/[^\"/]+)+\.(service|data|types|adapter)[\"']" "$FILE_PATH" 2>/dev/null && \
    echo "WARNING: Deep import detected in $FILE_PATH - import through index only" || true
fi

# Sensitive path warning
case "$FILE_PATH" in
  *.env|*.log|*/node_modules/*)
    echo "WARNING: Writing to potentially sensitive path: $FILE_PATH"
    ;;
esac

exit 0
