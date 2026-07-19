# /audit-tools — Audit Code Reuse and Tool Registry

Run the reuse audit to detect duplicate code, extraction candidates, and stale registry entries.

## Steps

1. Run the audit script:
```bash
bash scripts/audit-reuse.sh
```

2. Review the output:
   - **PROMOTE**: functions imported from 3+ files → extract to shared/
   - **DUPLICATE**: same function defined in 2+ files → consolidate
   - **UNREGISTERED**: shared utilities not in tool-registry → register
   - **STALE**: registry entries pointing to deleted files → remove

3. Update `_reference/tool-registry.md` based on findings

4. If promotions are found, create extraction plan in `tasks/current.md`

## Quick Mode (session start)
```bash
bash scripts/audit-reuse.sh --quick
```

## Report Only (no changes)
```bash
bash scripts/audit-reuse.sh --report
```
