---
name: update-deps
description: "Safely update project dependencies: audit, update patch/minor/major, test, document. Trigger on: update dependencies, npm audit, security advisory."
---

# Dependency Update Skill

## When to use
Weekly/monthly maintenance, security advisories, or before major releases.

## Steps

### Step 1 — Audit
Run security audit:
- `npm audit` (Node.js)
- `pip audit` (Python)
- `cargo audit` (Rust)
- `govulncheck ./...` (Go)

List: critical vulnerabilities, high vulnerabilities, outdated packages.

### Step 2 — Categorize updates
- **Security patches**: update immediately regardless of semver
- **Patch** (1.2.3 → 1.2.4): safe, update in batch
- **Minor** (1.2.x → 1.3.x): review CHANGELOG, update in batch
- **Major** (1.x → 2.x): read migration guide, update one at a time

### Step 3 — Update patch/minor
```bash
npm update          # Node.js
pip install --upgrade [packages]  # Python
cargo update        # Rust
go get -u ./...     # Go
```
Run full test suite. If pass: commit as `chore: update dependencies (patch/minor)`.

### Step 4 — Update major (one at a time)
For each major update:
1. Read CHANGELOG/migration guide
2. Update single package
3. Fix any breaking changes
4. Run full test suite
5. Commit as `chore: upgrade [package] to v[X]`

### Step 5 — Document
- Add to `brain/03-knowledge/` if significant API changes were required
- Update `.env.example` if new environment variables are required
- Note any deprecated APIs still in use (create tasks to migrate)

## Rules
- Always commit lock files after updates
- Never update multiple major versions in one commit
- If tests fail after update: revert that package, investigate separately
