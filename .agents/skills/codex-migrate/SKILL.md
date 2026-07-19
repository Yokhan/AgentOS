---
name: codex-migrate
description: "Plan and implement database, schema, data, or storage migrations with rollback, dry-run, backup, compatibility, and verification gates. Trigger on migrations, schema changes, or data moves."
---

# Codex Migrate

Migrations are high-risk shared-state changes. Do not start writes until the current schema, data shape, compatibility window, and rollback path are clear.

When the target architecture is unresolved, use `$codex-change-strategy` first
to choose the destination and transition. This skill owns safe execution of an
`expand-migrate-contract` transition only after the strategy and client approval
boundary are established.

## Workflow

1. Identify current and target schema/data contracts.
2. Determine forward-only, reversible, or expand-migrate-contract strategy.
3. Plan backup, rollback, and observability.
4. Implement migration logic in importable modules where possible.
5. Add tests for empty, partial, duplicate, and already-migrated states.
6. Run dry-run or local migration verification before closeout.

## Gates

- No destructive migration without an explicit rollback or recovery path.
- No hidden production defaults, credentials, or hardcoded endpoints.
- No broad data rewrite without a small-sample validation path.
- User checkpoint is required when data loss, downtime, or irreversible state is possible.
