# Architecture Rules

## Module Boundaries
- Every module has a single public entry point (index.ts / __init__.py / mod.rs / main.go)
- Import ONLY through the entry point. No deep imports into internal files.
- If you need something from a module, it must be exported from its entry point.
- Extract to `shared/` ONLY when used by ≥3 modules. Until then, duplication is acceptable. Premature abstraction is worse than duplication.

## Dependency Direction
- `shared/` depends on nothing
- `core/` depends only on `shared/`. NO IO (fetch, DB, filesystem, network).
- `features/` depends on `shared/` and `core/`
- `adapters/` / UI depends on `features/`
- Never create circular dependencies.

## File Size
- Maximum 375 lines per file (1M context models). For smaller context models, keep at 250.
- This is a cognitive limit for AI agents (Working Memory Cliff), not just style.
- If a file grows beyond 375 lines, split it by responsibility.
- Instruction files (skills, agents, rules) can be longer — up to 800 lines for domain-heavy skills.

## Module Structure (Vertical Slices)
Each feature is a self-contained module with all layers:
```
features/auth/
  ├── index.ts          # Public API (only this is importable)
  ├── auth.service.ts   # Business logic
  ├── auth.types.ts     # Types/contracts
  ├── auth.test.ts      # Tests (colocated)
  └── auth.data.ts      # Config/tables (Data-Oriented Design)
```

## Adapters Layer
- `adapters/[name].adapter.*` handles all IO: API calls, DB queries, filesystem operations.
- Adapters depend on `features/` types and interfaces only — never on services directly.
- Adapters are the ONLY layer allowed to perform IO. All other layers must be IO-free.

## Data-Oriented Design
- `data.*` — configurations, tables, lookup maps (easy to change)
- `processor.*` / `service.*` — pure functions: (input, config) => output (stable)
- `types.*` — contracts and interfaces

## When to Split a Module
Split when ANY of these heuristics fire:
- **>3 public exports** — the module is doing too many things. Each export is a promise to consumers.
- **>2 consumers** — multiple dependents = high change cost. Smaller modules reduce blast radius.
- **Mixed concerns** — if a module has both "user auth" and "email sending," those are separate domains.
- **Different change rates** — parts that change weekly shouldn't live with parts that change yearly.
- When in doubt, keep together. Premature splitting creates coupling through shared types.

## Monolith vs Microservice Decision
Default: **monolith**. Split ONLY when you have a concrete reason:
- **Independent deployment needed** — one team ships daily, another monthly. Different cadences = split.
- **Independent scaling needed** — one module handles 100x more traffic. Scaling everything together wastes resources.
- **Team boundary alignment** — Conway's Law is real. If two teams own it, two services make sense.
- **Regulatory isolation** — PCI/HIPAA compliance scope can be reduced by isolating the sensitive module.
- If none of these apply, a well-structured monolith outperforms microservices on complexity, latency, and debugging.
- Reference: Startup Genome data — 74% of premature scalers fail. This applies to architecture too.

## API Boundary Design
- **Internal APIs** (module-to-module): typed function calls, no serialization overhead. Change freely only after checking for actual consumers and project-owned contracts; repository age alone does not make internals protected.
- **External APIs** (exposed to consumers): versioned, stable, backward-compatible. Breaking changes = new version.
- Version strategy: URL prefix (`/v1/`, `/v2/`) for REST; field deprecation for GraphQL.
- Never expose internal models directly — use DTOs/response types at the boundary.
- API contracts live in `docs/API_CONTRACTS.md` and are the source of truth.

## Database Strategy
- **Shared DB** (default for monolith): simpler transactions, joins, consistency. Use schema namespaces per module.
- **DB per module**: required when modules need independent scaling, different storage engines, or separate team ownership.
- Trade-off: shared DB = easy consistency, hard independence. Separate DB = easy independence, hard consistency (eventual consistency, sagas).
- Rule: even with shared DB, modules access ONLY their own tables. Cross-module data goes through the module's public API.
- Migrations always versioned, reversible, and owned by the module that owns the table.

When destination and transition alternatives compete, use
`.claude/library/process/change-strategy-gate.md`. Preserve the boundary that
users or consumers depend on; do not preserve the internal implementation by
default.

## Cross-Cutting Concerns (auth, logging, validation)

Cross-cutting concerns don't fit clean vertical slices. Place them in:

- **`shared/middleware/`** — auth middleware, request logging, rate limiting, CORS
- **`shared/validators/`** — input validation, sanitization (used by multiple features)
- **`shared/observability/`** — structured logger, metrics collector, error reporter

Rules:
- Cross-cutting code lives in `shared/`, never inside a feature module
- Features CONSUME cross-cutting services via import, never duplicate them
- Each cross-cutting concern has a single owner file (e.g., `shared/middleware/auth.ts`)
- Register all cross-cutting utilities in `_reference/tool-registry.md`
- If a validation/middleware pattern is used by 3+ features → it's cross-cutting, extract it

## Safe Refactoring Protocol

Before refactoring ANY code:

1. **Write characterization tests first** — tests that capture CURRENT behavior, even if wrong
2. **Verify tests pass** — if they don't pass before refactoring, you can't trust them after
3. **Refactor** — make structural changes WITHOUT changing behavior
4. **Re-run tests** — all characterization tests must still pass
5. **Then fix behavior** — if behavior changes are needed, do them in a SEPARATE commit

Never refactor and change behavior in the same commit. The git history must show:
- Commit 1: "refactor: extract auth logic to shared/middleware" (tests pass, behavior unchanged)
- Commit 2: "fix: update auth middleware to check token expiry" (behavior change, tests updated)
