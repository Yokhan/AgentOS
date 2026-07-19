#!/bin/bash
# Auto-formatter — runs appropriate formatter for file type
# Receives $FILE_PATH from Claude Code environment

if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would format file"
  exit 0
fi

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

case "$FILE_PATH" in
  *.ts|*.tsx|*.js|*.jsx|*.json|*.css|*.html|*.md)
    npx prettier --write "$FILE_PATH" 2>/dev/null || true
    ;;
  *.py)
    black "$FILE_PATH" 2>/dev/null || true
    ;;
  *.rs)
    rustfmt "$FILE_PATH" 2>/dev/null || true
    ;;
  *.go)
    gofmt -w "$FILE_PATH" 2>/dev/null || true
    ;;
esac

exit 0
