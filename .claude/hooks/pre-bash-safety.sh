#!/usr/bin/env bash
# PreToolUse — Enhanced dangerous command blocking for Bash
# Extends the inline grep in settings.json with more patterns and better logging.
# Replaces the inline command in settings.json.

[ "${TEST_MODE:-}" = "1" ] && echo "pre-bash-safety: OK (test mode)" && exit 0

CMD="${COMMAND:-}"
[ -z "$CMD" ] && exit 0

BLOCKED=""

# === Destructive filesystem operations ===
if echo "$CMD" | grep -qE "rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+|--force\s+)?(\/|~|\\\$HOME|\.\.)" ; then
  BLOCKED="Destructive rm on root/home/parent directory"
elif echo "$CMD" | grep -qE "rm\s+-[a-zA-Z]*r[a-zA-Z]*f|rm\s+-[a-zA-Z]*f[a-zA-Z]*r" ; then
  # rm -rf without specific safe target
  if ! echo "$CMD" | grep -qE "rm\s+-rf\s+(node_modules|dist|build|\.cache|__pycache__|\.next|\.nuxt|target|tmp|\.tmp)" ; then
    BLOCKED="Recursive force delete (rm -rf) on non-standard target"
  fi
fi

# === Git destructive operations ===
if echo "$CMD" | grep -qE "git\s+push\s+.*--force|git\s+push\s+-f\b" ; then
  BLOCKED="Force push"
elif echo "$CMD" | grep -qE "git\s+reset\s+--hard" ; then
  BLOCKED="Hard reset"
elif echo "$CMD" | grep -qE "git\s+clean\s+-[a-zA-Z]*f" ; then
  BLOCKED="Git clean (force delete untracked files)"
elif echo "$CMD" | grep -qE "git\s+branch\s+-D\s+main|git\s+branch\s+-D\s+master" ; then
  BLOCKED="Delete main/master branch"
fi

# === Database destructive operations ===
if echo "$CMD" | grep -qiE "DROP\s+(TABLE|DATABASE|SCHEMA|INDEX)|TRUNCATE\s+TABLE|DELETE\s+FROM\s+\w+\s*$" ; then
  BLOCKED="Destructive database operation"
fi

# === Pipe-to-shell (supply chain attack vector) ===
if echo "$CMD" | grep -qE "curl\s.*\|\s*(ba)?sh|wget\s.*\|\s*(ba)?sh|\|\s*bash\s+-c" ; then
  BLOCKED="Pipe-to-shell execution (supply chain risk)"
fi

# === System-wide changes ===
if echo "$CMD" | grep -qE "chmod\s+(-R\s+)?777|chmod\s+.*u\+s|chown\s+-R\s+root" ; then
  BLOCKED="Dangerous permission change"
elif echo "$CMD" | grep -qE "docker\s+system\s+prune\s+-a|docker\s+rm\s+-f\s+\$\(" ; then
  BLOCKED="Docker mass cleanup"
elif echo "$CMD" | grep -qE "pkill\s+-9|killall\s+-9|kill\s+-9\s+-1" ; then
  BLOCKED="Mass process kill"
fi

# === Secret exfiltration ===
if echo "$CMD" | grep -qiE "curl\s.*(-d|--data).*(\\\$\(cat|password|secret|token|key)|wget\s.*(-O\s*-|--output-document).*\|" ; then
  BLOCKED="Possible secret exfiltration via HTTP"
fi

# === npm/pip install from unknown sources ===
if echo "$CMD" | grep -qE "npm\s+install\s+https?://|pip\s+install\s+https?://" ; then
  BLOCKED="Package install from URL (verify source first)"
fi

if [ -n "$BLOCKED" ]; then
  echo "{\"block\":true,\"message\":\"🛡️ BLOCKED: $BLOCKED — Review command: $CMD\"}" >&2

  # Log to audit
  if [ -d "tasks" ]; then
    mkdir -p "tasks/audit"
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%S")
    echo "{\"ts\":\"$TIMESTAMP\",\"type\":\"command_blocked\",\"reason\":\"$BLOCKED\",\"cmd\":\"$(echo "$CMD" | head -c 200 | sed 's/"/\\"/g')\"}" >> "tasks/audit/security.jsonl"
  fi

  exit 2
fi

exit 0
