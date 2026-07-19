You are displaying the full project status dashboard in a single view.

## Output

Run and display all of the following:

### Git State
```bash
git branch --show-current          # current branch
git status --short                  # uncommitted changes
git log --oneline -1               # last commit
```

### Task State
- First line of `tasks/current.md` (current task)
- Lessons count: `grep -c "^###" tasks/lessons.md 2>/dev/null`
- Sprint state: check if `tasks/sprint-state.md` exists and when last modified

### Code Health
```bash
# Files > 375 lines (show top 5)
find src -type f \( -name "*.ts" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) 2>/dev/null \
  | xargs wc -l 2>/dev/null \
  | awk '$1>250 && $2!="total" {print $1" "$2}' \
  | sort -rn | head -5

# Open TODOs
grep -r "TODO" src/ 2>/dev/null | wc -l
```

### Recent Activity
Last 3 session log files: `ls -t brain/01-daily/*.md 2>/dev/null | head -3`

### Template Health
- Template version: `grep -oP '(?<=Template Version: )[\d.]+' CLAUDE.md 2>/dev/null`

## Format
Present as a compact, scannable summary. Use ✅ for healthy, ⚠️ for warning, ❌ for critical.
