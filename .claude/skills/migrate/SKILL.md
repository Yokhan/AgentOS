---
name: migrate
description: "Database migration management -- generate, validate, apply, rollback. Trigger on: schema change, new table, migration, alter column."
---

# Database Migration Skill

## When to use
Schema changes: new tables, column additions/modifications, index changes, data migrations.

## Steps

### Step 1 -- Understand the change
- What schema change is needed?
- Is this additive (safe) or destructive (risky)?
- How many rows affected by data migration?

### Step 2 -- Generate migration
- Use project migration tool (Prisma, Alembic, golang-migrate, sqlx)
- Name format: `YYYYMMDD_HHMMSS_descriptive_name`
- Include both `up` and `down` migrations

### Step 3 -- Validate
- Is it idempotent? (safe to run twice)
- Is it backward-compatible? (old code still runs against new schema)
- Does it have a tested rollback path?
- For large tables (>100K rows): does it use online DDL or lock the table?

### Step 4 -- Dry run
- Run against dev/staging environment first
- Measure execution time
- Verify row counts and data integrity after

### Step 5 -- Document
- Add to `brain/04-decisions/migrations/YYYYMMDD-description.md`: why, what, impact, rollback plan
- Update `docs/DATA_DESIGN.md`

### Step 6 -- Apply
- Apply to target environment
- Verify with spot-check queries
- Monitor error rates for 5 minutes after

## Rules
- Never `DROP COLUMN` without deprecation period: null then stop writing then confirm unused then drop
- Never run migrations during peak traffic without online DDL support
- Always test rollback before applying to production
- Data migrations > 1M rows: run in batches with progress logging

---

## Migration Risk Assessment Framework

Before writing any migration, assess risk along these dimensions:

| Dimension | Low Risk | Medium Risk | High Risk |
|-----------|----------|-------------|-----------|
| **Data loss potential** | Additive only (new column/table) | Type change with safe cast | Column drop, table drop, data truncation |
| **Downtime requirement** | None (online DDL) | Brief lock (<5s on small tables) | Table lock on large table, full rewrite |
| **Rollback complexity** | Drop new column/table | Reverse data transform | Data lost, requires backup restore |
| **Blast radius** | Single feature, no dependents | Multiple features read the table | Shared table across all services |
| **Data volume** | <10K rows | 10K-1M rows | >1M rows |

**Risk score**: count High=3, Medium=2, Low=1 across all dimensions.
- Score 5-7: low risk, proceed normally
- Score 8-11: medium risk, require staging validation and explicit rollback test
- Score 12-15: high risk, require team review, off-peak scheduling, and rehearsal on production-sized dataset

## Common Migration Patterns

### Add Column (Safe)
```sql
ALTER TABLE users ADD COLUMN nickname VARCHAR(100) DEFAULT NULL;
```
- Safe in most databases (no table rewrite for nullable columns)
- Set DEFAULT NULL, not a value -- avoids table rewrite on large tables in older MySQL
- Deploy code that handles both with and without the column first

### Rename Column (Expand-Contract)
Do NOT use `ALTER TABLE RENAME COLUMN` directly -- it breaks running code.
```
Phase 1 (Expand):  Add new column, copy data, write to both
Phase 2 (Migrate): Backfill historical data, verify consistency
Phase 3 (Contract): Remove old column after all code uses new name
```
Each phase is a separate migration AND a separate deployment.

### Change Column Type
- Safe: widening (INT to BIGINT, VARCHAR(50) to VARCHAR(200))
- Unsafe: narrowing or changing kind (VARCHAR to INT, BIGINT to INT)
- For unsafe changes: create new column, backfill with CAST, swap, drop old
- Always validate: `SELECT COUNT(*) FROM table WHERE CAST(old_col AS new_type) IS NULL AND old_col IS NOT NULL`

### Split Table
- Create new table with subset of columns + foreign key back to original
- Backfill new table from original
- Update code to join or query new table
- After validation period: drop migrated columns from original (contract phase)

### Merge Tables
- Create combined table with all columns from both sources
- Backfill from both source tables (handle key conflicts!)
- Update code to use combined table
- Drop source tables only after full validation

## Zero-Downtime Migration Strategy (Expand-Contract Pattern)

The expand-contract pattern ensures no deployment requires simultaneous code + schema changes:

```
Timeline:
  --- Deploy 1: Code handles BOTH old and new schema ---
      (expand: new column exists but is not required)
  --- Migration: Apply schema change ---
      (data backfill, index creation, etc.)
  --- Deploy 2: Code uses ONLY new schema ---
      (contract: old column/table no longer referenced)
  --- Migration: Remove old schema artifacts ---
      (drop old column/table -- safe because nothing uses it)
```

Rules for zero-downtime:
1. Never add a NOT NULL column without a DEFAULT in the same migration
2. Never rename -- always add-new, migrate, remove-old
3. Never drop until code is deployed without references to the old schema
4. Index creation: use `CREATE INDEX CONCURRENTLY` (Postgres) or equivalent to avoid table locks

## Data Migration vs Schema Migration

These are fundamentally different operations. Never mix them in one migration file.

| Aspect | Schema Migration | Data Migration |
|--------|-----------------|----------------|
| **What changes** | Table structure, indexes, constraints | Row values, data transformation |
| **Reversibility** | Usually straightforward (add then drop) | Often lossy (transformed data may not round-trip) |
| **Duration** | Fast (DDL is usually instant or brief) | Slow (proportional to row count) |
| **Testing** | Structure verification | Value verification, data integrity checks |
| **Failure mode** | Immediate error, clean rollback | Partial completion, inconsistent state |

Best practice: separate migration files, run schema first, then data, with verification between them.

## Rollback Protocol

Every migration step must have a documented rollback. Before applying ANY migration:

### Pre-Migration Checklist
1. Rollback script exists and has been tested on staging
2. Database backup is current (point-in-time recovery enabled)
3. Rollback time estimate is known and acceptable
4. Rollback procedure is documented in the migration decision record

### Rollback Testing Protocol
```
1. Apply migration on staging (clean copy of production schema + sample data)
2. Run application smoke tests against migrated schema
3. Execute rollback script
4. Run application smoke tests against rolled-back schema
5. Verify data integrity: row counts match, no data corruption
6. Measure rollback duration -- multiply by 3x for production estimate
```

### Emergency Rollback Decision Tree
```
Migration failed mid-execution?
  Failure is in schema DDL: usually auto-rolled-back (transactional DDL)
  Failure is in data migration: check partial state
    Data migration is idempotent: fix issue, re-run
    Data migration is NOT idempotent: restore from backup
  Unknown state: DO NOT proceed. Assess manually, restore from backup if needed.
```

## Testing Migrations: Forward AND Backward

### Forward Testing
1. Apply migration to empty database -- schema is valid
2. Apply migration to production-like dataset -- data transforms correctly
3. Run application test suite against migrated schema -- app works
4. Check query performance on migrated schema -- no regressions from missing indexes

### Backward Testing (Often Neglected)
1. Apply migration, THEN apply rollback -- schema matches pre-migration state
2. Run application test suite against rolled-back schema -- app still works
3. Apply migration AGAIN after rollback -- idempotent, no errors
4. For data migrations: verify data survives the round-trip (migrate then rollback then re-migrate)

### Edge Case Testing
- Test with NULL values in every nullable column
- Test with maximum-length strings and boundary numeric values
- Test with empty tables (no rows to migrate)
- Test with foreign key references that could block DDL
- Test with concurrent read/write load (simulate production traffic)

## Cross-Reference

- `.claude/library/meta/strategic-thinking.md` -- "Friction" principle: expect that migrations will encounter unexpected difficulties (foreign keys, triggers, views that depend on columns). "Multiple contingencies": always have Plan B (rollback) and Plan C (restore from backup).
- `.claude/library/domain/domain-guards.md` -- "Database migrations as code": schema changes versioned and reversible. "Strangler Fig Migration": incrementally replace, never big-bang.
- `.claude/library/technical/error-handling.md` -- Recovery patterns for transient vs permanent failures during migration execution.
