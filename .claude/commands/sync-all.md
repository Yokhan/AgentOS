---
name: sync-all
description: "Sync template to all projects that have .template-manifest.json. Scans ~/Documents by default."
---

# /sync-all [directory]

Syncs the current template to all projects found in the specified directory (default: ~/Documents).

## Steps

1. **Scan** for projects with `.template-manifest.json`:
   ```bash
   bash scripts/sync-all.sh [directory]
   ```

2. **For each project found**:
   - Read `.template-manifest.json` → current template version
   - Compare with this template's version
   - If outdated: run `sync-template.sh`
   - Report: updated / current / conflict / failed

3. **Present summary table**:
   ```
   Sync Results:
   ✓ ProjectA: v2.3.0 → v2.4.0 (updated)
   ✓ ProjectB: v2.4.0 (already current)
   ⚠ ProjectC: v2.2.0 → v2.4.0 (conflicts in 2 files)
   ✗ ProjectD: failed (not a git repo)

   Summary: 1 updated, 1 current, 1 conflict, 1 failed
   ```

## Options
- `--dry-run` — show what would change without modifying
- Default directory: `~/Documents`
- Skips: the template itself, archived projects
