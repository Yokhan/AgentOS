---
name: setup-project
description: "Full project setup: detect stack, install deps, configure tooling, create scaffolding templates, reference implementation, and update CLAUDE.md"
---

# Setup Project Skill

You are configuring a fresh project from the agent-project-template. Follow these phases exactly.

## Phase 1: Gather Information

Ask the user (use AskUserQuestion or chat):

1. **Stack**: What programming language(s) and framework(s)?
   - TypeScript (Node.js): React/Vue/Svelte + Express/Fastify/Nest
   - Python: FastAPI/Django/Flask + data tools
   - Go: standard library / Gin / Echo
   - Rust: Actix / Axum / Rocket
   - Other: ask for details

2. **Project type**: What kind of project?
   - Web app (frontend + backend)
   - API service (backend only)
   - CLI tool
   - Library/package
   - Monorepo (multiple packages)

3. **Name**: Project name (for package.json / pyproject.toml / go.mod etc.)

4. **Integrations**: Which optional integrations?
   - Memory-MCP (persistent semantic memory across sessions)
   - CodeGraphContext (code graph for token optimization)
   - Telegram (remote control from phone)

## Phase 2: Initialize Project

Based on the stack, run the appropriate init command:

### TypeScript/Node.js
```bash
npm init -y  # or pnpm init
npx tsc --init --strict --esModuleInterop --resolveJsonImports
```
Install: typescript, prettier, eslint, vitest (or jest)

### Python
```bash
python -m venv .venv
pip install ruff pytest mypy
```
Create: pyproject.toml, .python-version

### Go
```bash
go mod init [module-name]
```
Install: golangci-lint

### Rust
```bash
cargo init
```

## Phase 3: Create Source Structure

Create directories based on project type:

```
src/
├── shared/          # Types, utilities, validators
├── core/            # Domain logic (NO IO)
└── features/        # Vertical slices
    └── [first-feature]/
        ├── index.*          # Public entry point
        ├── [name].service.* # Business logic
        ├── [name].types.*   # Types
        ├── [name].test.*    # Tests
        └── [name].data.*    # Config/data
```

For Python: use `__init__.py` as entry points.
For Go: use `package` directories.
For Rust: use `mod.rs` as entry points.

### Phase 3b: Scan and populate tool registry

Run project scanner to auto-detect existing utilities:
```bash
bash scripts/scan-project.sh
```

This populates `_reference/tool-registry.md` with:
- Shared utilities found in `shared/`, `utils/`, `helpers/`, `common/`, `lib/`
- Scripts found in `scripts/`, `bin/`, `tools/`
- Components found in `components/`, `ui/` (React/Vue/Svelte)
- Design tokens if `design-tokens/` or Figma config exists

For existing projects with code: this is critical — the registry enables agents to find and reuse existing code instead of writing new.

## Phase 4: Configure Tooling

### Formatter
- TypeScript: prettier (create .prettierrc)
- Python: ruff format (configure in pyproject.toml)
- Go: gofmt (built-in)
- Rust: rustfmt (create rustfmt.toml)

### Linter
- TypeScript: eslint (create eslint.config.js)
- Python: ruff check (configure in pyproject.toml)
- Go: golangci-lint (create .golangci.yml)
- Rust: clippy (built-in)

### Test framework
- TypeScript: vitest or jest
- Python: pytest
- Go: built-in testing
- Rust: built-in #[test]

### Update hooks in .claude/settings.json
Replace the placeholder auto-format hook with the real formatter command for the chosen stack.

### Git Hooks Setup
Configure pre-commit hooks to enforce typecheck + lint + test before every commit:
- **Node.js**: `npm install --save-dev husky && npx husky init` → add pre-commit hook
- **Python**: `pip install pre-commit` → create `.pre-commit-config.yaml` with ruff + mypy
- **Go**: Add `pre-commit` target to Makefile that runs `go vet ./... && golangci-lint run`
- **Rust**: Add `pre-commit` target to Makefile that runs `cargo clippy && cargo test`

## Phase 5: Create Scaffolding Templates

Create `templates/` with stack-appropriate templates:

### templates/feature/
A template for new feature modules matching the project's patterns.

### templates/api-endpoint/ (if API project)
A template for new API endpoints.

### templates/component/ (if frontend project)
A template for new UI components.

Each template should include:
- Entry point file
- Service/logic file
- Types file
- Test file
- Data file (if applicable)

## Phase 6: Create Reference Implementation

Create ONE complete feature (e.g., `src/features/health/`) as a canonical example:
- Full vertical slice with all patterns
- Working tests
- Proper typing
- Data-oriented design (data.* + service.*)

Update `_reference/README.md` to point to this implementation.

## Phase 7: Generate Documentation

Rename `docs/*.md.template` to `docs/*.md` and fill in:
- ARCHITECTURE.md: modules, dependencies, boundaries
- DATA_DESIGN.md: data structures overview
- API_CONTRACTS.md: API endpoints (if applicable)
- DECISIONS.md: first ADR about stack choice

## Phase 8: Update CLAUDE.md

Replace the `## Status` section: change `NEW_PROJECT` to `CONFIGURED`.
Fill in:
- Stack section with actual technologies
- Build & Test section with actual commands
- Any stack-specific rules

## Phase 9: Install Integrations (if selected)

For each selected integration, read the corresponding README in `integrations/` and follow the setup instructions:

- **Memory-MCP**: Read `integrations/memory-mcp/README.md`, install and configure
- **CodeGraphContext**: Read `integrations/codegraph/README.md`, install and index
- **Telegram**: Read `integrations/telegram/README.md`, guide user through bot creation

## Phase 10: Agent Specialization (v2.5)

Based on detected stack from Phase 1, generate project-specific agent overlays:

**If TypeScript + React/Vue/Svelte detected:**
→ Create `.claude/agents/project-frontend-implementer.md` with:
  - React/Vue/Svelte component conventions, hooks/composables patterns
  - State management approach (Zustand/Pinia/Redux detected from package.json)
  - CSS approach (Tailwind/CSS modules/styled-components detected from config)

**If TypeScript + Express/Fastify/Nest detected:**
→ Create `.claude/agents/project-backend-implementer.md` with:
  - API route patterns, middleware conventions
  - ORM/DB patterns (Prisma/TypeORM/Drizzle detected from package.json)
  - Error handling and validation approach

**If Python + FastAPI/Django/Flask detected:**
→ Create `.claude/agents/project-python-implementer.md` with:
  - Framework-specific patterns (dependency injection, middleware, etc.)
  - ORM patterns (SQLAlchemy/Django ORM/Tortoise)

**If Godot/GDScript detected (project.godot exists):**
→ Create `.claude/agents/project-godot-implementer.md` with:
  - Signal-based communication, scene composition
  - Resource preloading, node lifecycle

**If Supabase detected (.env contains SUPABASE or package.json has @supabase):**
→ Create `.claude/agents/project-supabase-implementer.md` with:
  - RLS policy patterns, service role usage
  - Migration conventions, realtime subscriptions

**Template for generated overlay:**
```markdown
---
name: project-{stack}-implementer
extends: implementer
model: sonnet
description: "Implementer specialized for {detected stack}"
---
# {Stack} Implementer
Extends base implementer. Stack: {versions detected}.
## Conventions
- {extracted from _reference/ or package.json}
## Key Files
- {entry points, configs, env vars}
## Patterns
- {from detected frameworks}
```

These overlays evolve via `/weekly` — if lessons.md contains 3+ errors about a technology, the overlay gets enriched with prevention rules.

## Phase 11: Verify

Run the full verification chain (was Phase 10 in v2.4):
1. Typecheck passes
2. Linter passes
3. Tests pass (at least the health check reference impl)
4. Suggest initial commit

Report what was set up and what the user should do next.
