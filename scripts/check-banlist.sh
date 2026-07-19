#!/bin/bash
# Phrase-signal scanner — flag possible filler for contextual review
# Usage: bash scripts/check-banlist.sh [file_or_dir...]
# Default: scans staged files if no args given

# Default Russian diagnostic signals
BAN_RU=(
  "является"
  "представляет собой"
  "ключевой аспект"
  "стоит отметить"
  "важно понимать"
  "комплексный подход"
  "в современном мире"
  "безусловно"
  "зачастую"
  "Кроме того"
  "Более того"
  "Помимо этого"
  "Таким образом"
  "Подведём итог"
  "В заключение"
  "Резюмируя"
)

# Default English diagnostic signals
BAN_EN=(
  "Furthermore"
  "Moreover"
  "Additionally"
  "In conclusion"
  "It's worth noting"
  "It's important to understand"
  "In today's world"
  "Comprehensive approach"
  "Delve"
  "Navigate"
  "Landscape"
  "Embark"
  "Cutting-edge"
  "Game-changer"
  "Paradigm shift"
)

# Merge project-owned diagnostic signals if present
PROJECT_BANS=()
for banfile in ban-list.md BAN-LIST.md ban-list.txt; do
  if [ -f "$banfile" ]; then
    while IFS= read -r line; do
      # Skip empty lines and comments
      [[ -z "$line" || "$line" =~ ^# ]] && continue
      # Strip markdown list markers
      line=$(echo "$line" | sed 's/^[-*] //')
      PROJECT_BANS+=("$line")
    done < "$banfile"
    break
  fi
done

# Determine files to scan
FILES=()
if [ $# -gt 0 ]; then
  for arg in "$@"; do
    if [ -d "$arg" ]; then
      while IFS= read -r f; do
        FILES+=("$f")
      done < <(find "$arg" -type f \( -name "*.md" -o -name "*.txt" -o -name "*.html" \) 2>/dev/null)
    elif [ -f "$arg" ]; then
      FILES+=("$arg")
    fi
  done
else
  # Scan staged files (content files only)
  while IFS= read -r f; do
    case "$f" in
      *.md|*.txt|*.html|*.vue|*.jsx|*.tsx) FILES+=("$f") ;;
    esac
  done < <(git diff --cached --name-only 2>/dev/null)
fi

if [ ${#FILES[@]} -eq 0 ]; then
  echo "No files to scan."
  exit 0
fi

# Scan
VIOLATIONS=0
TOTAL_FILES=${#FILES[@]}

echo "=== Phrase Signal Scan: $TOTAL_FILES file(s) ==="

for file in "${FILES[@]}"; do
  [ -f "$file" ] || continue
  FILE_HITS=0

  # Skip non-content files (code, config, rules, agents)
  case "$file" in
    .claude/*|scripts/*|*.sh|*.json|*.yaml|*.yml|*.toml|*.lock) continue ;;
  esac

  # Check all ban lists
  for phrase in "${BAN_RU[@]}" "${BAN_EN[@]}" "${PROJECT_BANS[@]}"; do
    count=$(grep -ci "$phrase" "$file" 2>/dev/null || echo 0)
    if [ "$count" -gt 0 ]; then
      if [ "$FILE_HITS" -eq 0 ]; then
        echo ""
        echo "  $file:"
      fi
      # Show first occurrence with line number
      line_info=$(grep -ni "$phrase" "$file" 2>/dev/null | head -1)
      echo "    REVIEW: \"$phrase\" ($count occurrence(s)) — $line_info"
      FILE_HITS=$((FILE_HITS + count))
    fi
  done

  if [ "$FILE_HITS" -gt 0 ]; then
    VIOLATIONS=$((VIOLATIONS + FILE_HITS))
  fi
done

echo ""
if [ "$VIOLATIONS" -gt 0 ]; then
  echo "=== REVIEW: $VIOLATIONS phrase signal(s) found ==="
  echo "These are diagnostic signals, not forbidden words. Review each occurrence in context."
  exit 0
else
  echo "=== PASS: No phrase signals found ==="
  exit 0
fi
