#!/usr/bin/env bash
# PostToolUse — Session Audit Logger
# Logs every tool invocation to tasks/audit/session-YYYY-MM-DD.jsonl
# Useful for debugging, post-mortems, and tracking agent behavior.
# Inspired by NOVA's session tracking.

[ "${TEST_MODE:-}" = "1" ] && echo "session-audit: OK (test mode)" && exit 0

AUDIT_DIR="tasks/audit"
[ ! -d "tasks" ] && exit 0  # Not in a project with tasks/
mkdir -p "$AUDIT_DIR"

DATE=$(date +"%Y-%m-%d" 2>/dev/null || echo "unknown")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%S")
LOG_FILE="$AUDIT_DIR/session-${DATE}.jsonl"

# Collect available context
TOOL="${TOOL_NAME:-unknown}"
FILE="${FILE_PATH:-}"
CMD="${COMMAND:-}"

# Truncate long values
[ ${#CMD} -gt 200 ] && CMD="${CMD:0:200}..."
[ ${#FILE} -gt 150 ] && FILE="${FILE:0:150}..."

# Build JSON entry (no jq dependency)
ENTRY="{\"ts\":\"$TIMESTAMP\",\"tool\":\"$TOOL\""

[ -n "$FILE" ] && ENTRY="$ENTRY,\"file\":\"$(echo "$FILE" | sed 's/\\/\\\\/g; s/"/\\"/g')\""
[ -n "$CMD" ] && ENTRY="$ENTRY,\"cmd\":\"$(echo "$CMD" | sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n' ' ')\""

ENTRY="$ENTRY}"

echo "$ENTRY" >> "$LOG_FILE"

# Rotate: keep last 7 days
find "$AUDIT_DIR" -name "session-*.jsonl" -mtime +7 -delete 2>/dev/null || true

exit 0
