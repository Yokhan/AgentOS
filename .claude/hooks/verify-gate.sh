#!/usr/bin/env bash
# verify-gate.sh — Soft reminder to run VERIFY phase after tests pass
# PostToolUse hook on Bash: triggers after test commands

[ "${TEST_MODE:-}" = "1" ] && echo "verify-gate: OK (test mode)" && exit 0

CMD="${COMMAND:-}"

# Check if the command looks like a test runner
if echo "$CMD" | grep -qiE "(npm test|npx jest|pytest|cargo test|go test|mix test|ruby.*rspec|dotnet test|mvn test|gradle test)"; then
  echo "VERIFY REMINDER: Tests passed. Before committing, run the VERIFY phase:"
  echo "  1. Re-read original task  2. Diff review  3. Devil's advocate  4. Confidence check"
  echo "  See: .claude/library/process/self-verification.md"
fi

exit 0
