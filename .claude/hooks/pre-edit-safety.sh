#!/bin/bash
# Pre-Edit Safety Hook — branch protection + secrets detection + archive protection

if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would check branch and secrets"
  exit 0
fi

# Block edits on main/master
if git rev-parse --git-dir >/dev/null 2>&1; then
  branch=$(git branch --show-current 2>/dev/null || true)
  if [ "$branch" = "main" ] || [ "$branch" = "master" ]; then
    echo '{"block":true,"message":"Cannot edit on main/master. Create a feature branch first."}' >&2
    exit 2
  fi
fi

# Secrets detection (extended patterns)
if [ -n "$FILE_PATH" ] && [ -f "$FILE_PATH" ]; then
  if grep -qE "(sk-[a-zA-Z0-9]{20,}|ghp_[a-zA-Z0-9]{36}|PRIVATE.KEY|-----BEGIN.*(RSA|EC|DSA|OPENSSH)|AKIA[0-9A-Z]{16}|xoxb-|sk_live_|rk_live_|whsec_|AIza)" "$FILE_PATH" 2>/dev/null; then
    echo '{"block":true,"message":"Potential secret/key detected in file! Remove before writing."}' >&2
    exit 2
  fi
fi

# Prevent editing archived files
case "$FILE_PATH" in
  */_archive/*|*/_archive_deprecated/*)
    echo '{"block":true,"message":"Cannot edit archived files."}' >&2
    exit 2
    ;;
esac

exit 0
