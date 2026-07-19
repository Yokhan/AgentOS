#!/usr/bin/env bash
# PostToolUse — Prompt Injection Defender
# Scans tool output for injection attempts. Warns but does NOT block.
# Inspired by lasso-security/claude-hooks — regex-based, zero cost, instant.
#
# Receives tool output via stdin or TOOL_OUTPUT env var.
# Exit 0 always (warn-only mode). Prints alerts to stdout for Claude to see.

[ "${TEST_MODE:-}" = "1" ] && echo "prompt-injection-defender: OK (test mode)" && exit 0

# Get tool output — from stdin if piped, else from env
if [ -t 0 ]; then
  CONTENT="${TOOL_OUTPUT:-}"
else
  CONTENT=$(cat)
fi

[ -z "$CONTENT" ] && exit 0

# Load patterns from YAML-like file if exists, else use built-in patterns
PATTERNS_FILE="$(dirname "$0")/security/patterns.yaml"

ALERTS=""

# === Category 1: Instruction Override ===
if echo "$CONTENT" | grep -qiE "(ignore (all |any )?(previous|prior|above|earlier) (instructions|prompts|rules|guidelines)|disregard (your|all|any) (instructions|rules|guidelines|prompts)|forget (everything|all|your) (you|about|instructions)|override (system|your) (prompt|instructions|rules)|new (instructions|rules|directives):)"; then
  ALERTS="${ALERTS}\n⚠️  [CRITICAL] Instruction Override attempt detected"
fi

# === Category 2: Role-Playing / DAN / Jailbreak ===
if echo "$CONTENT" | grep -qiE "(you are now |act as (a |an )?(unrestricted|unfiltered|evil|DAN)|pretend (you are|to be)|jailbreak|DAN mode|Developer Mode|you have been (freed|liberated|unlocked)|bypass (your|all|safety) (restrictions|filters|rules)|do anything now)"; then
  ALERTS="${ALERTS}\n⚠️  [CRITICAL] Role-Playing/Jailbreak attempt detected"
fi

# === Category 3: Encoding / Obfuscation ===
# Base64 encoded instructions (long base64 blocks that decode to text)
if echo "$CONTENT" | grep -qE "[A-Za-z0-9+/]{40,}={0,2}" | head -1 && echo "$CONTENT" | grep -qiE "(base64|decode|eval|execute)"; then
  ALERTS="${ALERTS}\n⚠️  [HIGH] Possible encoded/obfuscated payload detected"
fi
# Unicode homoglyphs and zero-width characters
if command -v xxd >/dev/null 2>&1 && printf '%s' "$CONTENT" | xxd -p -c 999999 2>/dev/null | grep -qiE '(e2808b|e2808c|e2808d|efbbbf|c2ad)'; then
  ALERTS="${ALERTS}\n⚠️  [HIGH] Zero-width/invisible characters detected (possible steganography)"
fi

# === Category 4: Context Manipulation ===
if echo "$CONTENT" | grep -qiE "(SYSTEM:|ASSISTANT:|Human:|<system>|<\|im_start\|>|<\|endoftext\|>|\[INST\]|<<SYS>>|<s>|</s>|anthropic|from Claude|from Anthropic|official (update|instruction|policy)|new (system|safety) (message|update|policy))"; then
  ALERTS="${ALERTS}\n⚠️  [HIGH] Context Manipulation — fake system/role markers detected"
fi

# === Category 5: Instruction Smuggling ===
# Hidden in HTML comments, code comments, or markdown
if echo "$CONTENT" | grep -qE "(<!--.*\b(ignore|override|execute|sudo|admin)\b.*-->|/\*.*\b(ignore|override|execute|sudo|admin)\b.*/\*|#.*\b(IMPORTANT|OVERRIDE|SECRET INSTRUCTION|HIDDEN)\b)"; then
  ALERTS="${ALERTS}\n⚠️  [MEDIUM] Instruction Smuggling — hidden directives in comments"
fi

# === Category 6: Data Exfiltration ===
if echo "$CONTENT" | grep -qiE "(curl .*(env|ssh|key|secret|password|token|credential)|wget .*(env|ssh|key|secret)|send (to|this to|data to)|exfiltrate|upload.*secret|post.*(api.key|password|token))"; then
  ALERTS="${ALERTS}\n⚠️  [CRITICAL] Data Exfiltration attempt detected"
fi

# === Category 7: Privilege Escalation ===
if echo "$CONTENT" | grep -qiE "(sudo (rm|chmod|chown|bash|sh|python|node|curl|wget)|chmod (777|u\+s)|as root|with admin|escalat|privilege)"; then
  ALERTS="${ALERTS}\n⚠️  [HIGH] Privilege Escalation pattern detected"
fi

# Output alerts if any found
if [ -n "$ALERTS" ]; then
  echo ""
  echo "🛡️ PROMPT INJECTION DEFENDER — ALERTS:"
  echo -e "$ALERTS"
  echo ""
  echo "Source tool: ${TOOL_NAME:-unknown}"
  echo "Action: Content flagged for review. Proceed with caution."
  echo "---"

  # Log to session audit if available
  AUDIT_DIR="tasks/audit"
  if [ -d "tasks" ]; then
    mkdir -p "$AUDIT_DIR"
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%S")
    echo "{\"ts\":\"$TIMESTAMP\",\"type\":\"injection_alert\",\"tool\":\"${TOOL_NAME:-unknown}\",\"alerts\":\"$(echo -e "$ALERTS" | tr '\n' '|' | sed 's/"/\\"/g')\"}" >> "$AUDIT_DIR/security.jsonl"
  fi
fi

exit 0
