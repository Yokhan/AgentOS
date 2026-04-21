#!/bin/bash
# Module Status Scanner — analyze project module health
# Usage: bash scripts/module-status.sh /path/to/project
# Output: name|status|file_count|total_lines|issues

PROJECT="${1:-.}"
[ -d "$PROJECT" ] || { echo "ERROR: $PROJECT not found"; exit 1; }
cd "$PROJECT" || exit 1

# Find module directories: src/features/*, src/modules/*, app/*, or top-level code dirs
FOUND=0
for base in src/features src/modules app src lib; do
  [ -d "$base" ] || continue
  for mod_dir in "$base"/*/; do
    [ -d "$mod_dir" ] || continue
    mod_name=$(basename "$mod_dir")
    [ "$mod_name" = "node_modules" ] || [ "$mod_name" = ".git" ] || [ "$mod_name" = "dist" ] && continue

    files=0
    total_lines=0
    issues=""
    has_tests=false

    while IFS= read -r f; do
      [ -f "$f" ] || continue
      files=$((files + 1))
      lines=$(wc -l < "$f" 2>/dev/null | tr -d ' ')
      total_lines=$((total_lines + lines))
      if [ "$lines" -gt 375 ]; then
        issues="${issues}${f##*/}:${lines}lines "
      fi
      case "$f" in *.test.*|*.spec.*) has_tests=true ;; esac
    done < <(find "$mod_dir" -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.go" -o -name "*.rs" -o -name "*.vue" \) 2>/dev/null)

    # Determine status
    status="ok"
    if [ "$files" -eq 0 ]; then
      status="error"
      issues="no files"
    elif [ -n "$issues" ]; then
      status="warning"
    fi
    if [ "$has_tests" = false ] && [ "$files" -gt 0 ]; then
      issues="${issues}no tests "
    fi

    echo "$mod_name|$status|$files|$total_lines|$issues"
    FOUND=$((FOUND + 1))
  done
done

# If no modules found, report top-level files
if [ "$FOUND" -eq 0 ]; then
  echo "root|info|0|0|no module structure detected"
fi
