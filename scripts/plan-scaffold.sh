#!/bin/bash
# plan-scaffold.sh — Generate plan skeleton in tasks/current.md
# Usage: bash scripts/plan-scaffold.sh "add OAuth flow to auth module"

TASK="${1:-}"
CURRENT="tasks/current.md"

if [ -z "$TASK" ]; then
  echo "Usage: bash scripts/plan-scaffold.sh \"<task description>\""
  exit 1
fi

# Estimate size by keyword heuristics
KEYWORDS=$(echo "$TASK" | tr ' ' '\n' | wc -l | tr -d ' ')
FILE_EST=0
SIZE="S"

# Scan for affected files
AFFECTED=""
for word in $(echo "$TASK" | tr ' ' '\n' | grep -iE "^[a-z]" | head -5); do
  found=$(find src/ lib/ app/ 2>/dev/null -type f -iname "*${word}*" | head -5)
  [ -n "$found" ] && AFFECTED="$AFFECTED $found"
done

if [ -n "$AFFECTED" ]; then
  FILE_EST=$(echo "$AFFECTED" | tr ' ' '\n' | sort -u | wc -l | tr -d ' ')
fi

# Size classification
if [ "$FILE_EST" -le 1 ] && [ "$KEYWORDS" -le 5 ]; then SIZE="XS"
elif [ "$FILE_EST" -le 2 ]; then SIZE="S"
elif [ "$FILE_EST" -le 7 ]; then SIZE="M"
elif [ "$FILE_EST" -le 15 ]; then SIZE="L"
else SIZE="XL"; fi

# Generate plan
mkdir -p tasks
{
  echo ""
  echo "## Plan — $TASK"
  echo ""
  echo "### Goal"
  echo "$TASK"
  echo ""
  echo "### Complexity Estimate"
  echo "- Size: $SIZE"
  echo "- Files to modify: ~$FILE_EST (estimated)"
  echo "- Files to create: [FILL IN]"
  echo "- Risk: [LOW/MEDIUM/HIGH — FILL IN]"
  echo ""
  echo "### File Architecture"
  if [ -n "$AFFECTED" ]; then
    echo '```'
    for f in $(echo "$AFFECTED" | tr ' ' '\n' | sort -u); do
      lines=$(wc -l < "$f" 2>/dev/null | tr -d ' ')
      echo "  $f  — [MODIFY] $lines lines"
    done
    echo '```'
  else
    echo "[No matching files found. Fill in manually.]"
  fi
  echo ""
  echo "### Implementation Order"
  echo "1. [FILL IN — types/interfaces first]"
  echo "2. [FILL IN — core logic]"
  echo "3. [FILL IN — tests]"
  echo ""
  echo "### Plan B (mandatory for M+ tasks)"
  echo "- Alternative: [FILL IN]"
  echo "- Trigger: [FILL IN — what failure signals Plan B]"
  echo ""
  echo "### Risks"
  echo "- [FILL IN]"
} >> "$CURRENT"

echo "=== Plan scaffold written to $CURRENT ==="
echo "Size: $SIZE, ~$FILE_EST files detected"
echo "Agent: fill in [FILL IN] sections, then implement."
