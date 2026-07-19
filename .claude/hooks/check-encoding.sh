#!/usr/bin/env bash
# PostToolUse - Text Policy Guard
# Fails on invalid UTF-8, BOM, mixed line endings, mojibake, and unsafe shell OS assumptions.

set -euo pipefail

[ "${TEST_MODE:-}" = "1" ] && echo "check-encoding: OK (test mode)" && exit 0

TOOL="${TOOL_NAME:-}"
case "$TOOL" in
  Write|Edit|MultiEdit|file_write|file_edit) ;;
  *) exit 0 ;;
esac

FILE="${FILE_PATH:-}"
[ -z "$FILE" ] && exit 0
[ ! -f "$FILE" ] && exit 0

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
VALIDATOR="$ROOT/scripts/validate-text-policy.js"

if command -v node >/dev/null 2>&1 && [ -f "$VALIDATOR" ]; then
  REL_FILE="$(
    node -e "
const path = require('path');
const root = path.resolve(process.argv[1]);
const file = path.resolve(process.argv[2]);
const rel = path.relative(root, file).split(path.sep).join('/');
console.log(rel && !rel.startsWith('..') ? rel : file);
" "$ROOT" "$FILE"
  )"
  cd "$ROOT"
  node scripts/validate-text-policy.js --path "$REL_FILE"
  exit $?
fi

ERRORS=0

if command -v xxd >/dev/null 2>&1 && head -c 3 "$FILE" 2>/dev/null | xxd -p | grep -q "efbbbf"; then
  echo "ERROR [ENCODING] File has UTF-8 BOM: $FILE" >&2
  ERRORS=$((ERRORS + 1))
fi

if command -v iconv >/dev/null 2>&1; then
  if ! iconv -f UTF-8 -t UTF-8 "$FILE" >/dev/null 2>&1; then
    echo "ERROR [ENCODING] File is not valid UTF-8: $FILE" >&2
    ERRORS=$((ERRORS + 1))
  fi
fi

if command -v awk >/dev/null 2>&1; then
  if awk 'BEGIN { has_crlf=0; has_lf=0 } /\r$/ { has_crlf=1; next } { has_lf=1 } END { exit !(has_crlf && has_lf) }' "$FILE" 2>/dev/null; then
    echo "ERROR [ENCODING] Mixed CRLF and LF line endings: $FILE" >&2
    ERRORS=$((ERRORS + 1))
  fi
fi

exit "$ERRORS"
