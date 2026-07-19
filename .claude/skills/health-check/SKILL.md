---
name: health-check
description: "Full project health assessment: drift, coverage, security, dependencies, code quality. Trigger on: health check, project status, weekly review."
---

# Project Health Check Skill

## When to use
Weekly review, before releases, when something feels off, after long absence.

## Steps

### Step 1 — Drift Check
```bash
bash scripts/check-drift.sh
```
Note any warnings about stale docs, CLAUDE.md size, file sizes, architecture violations.

### Step 2 — Test Coverage
Run coverage tool for project stack. Report:
- Overall coverage %
- By layer: core/, features/, adapters/, shared/
- Modules below threshold (90%/80%/60%/95%)

### Step 3 — Security Audit
```bash
npm audit 2>/dev/null || pip audit 2>/dev/null || cargo audit 2>/dev/null || echo "no audit tool"
```
Count: critical, high, medium vulnerabilities.

### Step 4 — Dependency Health
- Outdated packages count
- Packages with no releases in >2 years (abandonment risk)
- Lock file up to date?

### Step 5 — Code Quality Metrics
```bash
# Files > 375 lines
find src -type f \( -name "*.ts" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) 2>/dev/null \
  | xargs wc -l 2>/dev/null | awk '$1>250 && $2!="total" {print $2": "$1" lines"}' | head -10

# Open TODOs
grep -r "TODO" src/ 2>/dev/null | wc -l

# Lessons entries
grep -c "^###" tasks/lessons.md 2>/dev/null
```

### Step 6 — Report
Save to `brain/03-knowledge/health-YYYY-MM-DD.md`:
```
# Health Report [date]
Drift:        [PASS / N issues]
Coverage:     [X%] (target: core 90%, features 80%)
Security:     [N critical, N high, N medium]
Outdated:     [N packages]
Large files:  [N files > 375 lines]
Open TODOs:   [N]
Lessons:      [N entries] (promote if >50)
Overall:      GREEN / YELLOW / RED
```

**GREEN**: all checks pass
**YELLOW**: non-critical issues (stale docs, <10 TODOs, low severity vulns)
**RED**: critical vuln, coverage <60%, >20 large files, lessons >60 without promotion
