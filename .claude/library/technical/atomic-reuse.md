# Atomic Reuse Protocol

## The Rule

Before writing ANY new utility, helper, component, or function:
1. **Search the project** — grep/glob for similar functionality
2. **Check the registry** — read `_reference/tool-registry.md`
3. **Check canonical examples** — read `_reference/README.md`
4. If found → **REUSE**. If not → create, then **REGISTER**.

This rule exists because agents repeatedly write code that already exists elsewhere in the project.
That's wasted tokens, duplicated logic, and divergent behavior.

## Atomic Hierarchy

Build bottom-up. Never top-down.

```
Level 0: TOKENS / TYPES     — type definitions, constants, config values
Level 1: ATOMS / UTILITIES   — pure functions, formatters, validators (1 file, 1 job)
Level 2: MOLECULES / SERVICES — combine atoms (auth.service, api.client)
Level 3: ORGANISMS / FEATURES — combine molecules (checkout flow, dashboard)
Level 4: TEMPLATES / PAGES    — layout + organisms
Level 5: SCREENS / APPS       — templates + data + routing
```

**Build order**: always Level 0 → Level 5. Never skip levels.
**Reuse direction**: higher levels import from lower. Never the reverse.

## Search Before Create (mandatory)

Before creating a new file:

```
1. glob: **/*{keyword}*  — does a file with similar name exist?
2. grep: "function {name}" or "export.*{name}" — is this already defined?
3. tool-registry: _reference/tool-registry.md — is it registered?
4. reference: _reference/README.md — is there a canonical pattern?
```

If ANY of these finds a match → read it first, then decide:
- **Exact match** → import and use it
- **Close match** → extend it (add parameter, refactor) instead of creating new
- **No match** → create new, register in tool-registry

## The "3 Uses" Rule

**Thresholds:**
- **2 uses** → consider extraction to shared/ (agent judgment call)
- **3+ uses** → mandatory extraction to shared/ (no exceptions)

**How this is detected** (agents don't need to remember):
- `bash scripts/audit-reuse.sh` scans import patterns across the project
- Outputs: "PROMOTE: formatDate() imported in 4 files → extract to shared/utils"
- Updates `_reference/tool-registry.md` automatically
- Run by: `/weekly`, session-start (lightweight), manually

**Agent responsibility**: after creating any new utility:
1. Register it in `_reference/tool-registry.md` (name, path, purpose)
2. If already used in 2+ places → consider extracting to shared/ now
3. At 3+ uses → extract immediately, this is not optional

## Anti-Patterns

| Wrong (agent wrote new code) | Right (agent reused) |
|------------------------------|----------------------|
| Created `formatCurrency()` in component | Found `shared/format.ts:formatCurrency()` via grep |
| Wrote custom `debounce()` | Found lodash/debounce already in deps |
| Created new API client wrapper | Found `src/shared/api.client.ts` in tool-registry |
| Duplicated validation logic | Found `shared/validators.ts` in registry |
| Wrote CSS from scratch | Found design tokens in tool-registry (Figma projects) |

## Tool Registry (`_reference/tool-registry.md`)

The registry has 4 sections:
1. **Template-Level** — scripts available in ALL projects (from template)
2. **Project-Level** — project-specific tools (filled by agents + scan-project.sh)
3. **Helpers & Utilities** — shared functions in src/ (filled by audit-reuse.sh)
4. **Design Tokens** — Figma components/tokens (filled by agents in design projects)

### Keeping the registry current
- `bash scripts/scan-project.sh` — initial population (deploy/update)
- `bash scripts/audit-reuse.sh` — ongoing maintenance (finds promotions, stale entries)
- Agent manual updates — after creating any new shared utility
- `/weekly` — runs audit-reuse.sh as part of retrospective

## Cross-Project Reuse

Some utilities are useful across ALL projects. These live in the template:
- `scripts/*.sh` — template-level tools (synced via sync-template.sh)
- `.claude/rules/*.md` — template-level rules
- `.claude/skills/*/SKILL.md` — template-level skills

If you discover a utility that would benefit multiple projects:
1. Note it in `tasks/lessons.md` with tag `[TEMPLATE-CANDIDATE]`
2. During `/weekly` or template update — promote to template's `scripts/`

## File Size Discipline

- One file = one responsibility
- Max 375 lines per source file
- If a file approaches 375 lines → SPLIT before it exceeds, not after
- Prefer many small files over few large ones — easier to search, reuse, and test

## Reference

See also:
- `_reference/tool-registry.md` — the registry itself
- `_reference/README.md` — canonical implementation patterns
- `.claude/library/process/research-first.md` — research protocol (includes registry check)
- `scripts/audit-reuse.sh` — automated duplicate detection
- `scripts/scan-project.sh` — initial project scan
