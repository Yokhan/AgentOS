# Code Style Rules (Stack-Agnostic)

## Immutability
- Prefer returning new objects over mutating existing ones.
- Use const/final/let (not var/mutable) by default.
- If mutation is necessary, document why.

## Type Safety
- No `any` (TypeScript), no `object` (Python), no `interface{}` (Go old-style).
- Use `unknown` + type guards, generics, or proper typing.
- Define explicit types for all public function signatures.
- Go: avoid `any` without type constraints (use generics). Rust: avoid `Box<dyn Any>`, prefer typed enums.

## Naming
- Functions: verb + noun (getUser, calculateScore, validateInput)
- Booleans: is/has/can/should prefix (isActive, hasPermission)
- Constants: UPPER_SNAKE_CASE
- Files: kebab-case or snake_case (match language convention)

## Functions
- Single responsibility: one function does one thing.
- Max 30 lines per function. Longer = split into helpers.
- Pure functions preferred: same input = same output, no side effects.
- Error handling at boundaries, not deep inside.

## Logging & Observability
- Use structured logging (JSON format).
- Log levels: debug (dev only), info (events), warn (recoverable issues), error (failures).
- Include correlation IDs.
- Never log PII (emails, passwords, tokens).

## Async Patterns
- async/await by default.
- Streams only for large data volumes (>10MB or continuous).
- Never use callbacks (except legacy API adapters).
- Avoid nested async without explanation.

## Configuration
- All config via environment variables (12-factor).
- Validate and type config at startup.
- No hardcoded URLs, ports, or credentials.
- Use typed config objects, never raw `process.env.X` scattered in business logic.

## Comments
- Don't comment WHAT (the code shows that). Comment WHY.
- No commented-out code. Delete it (git has history).
- TODO format: `// TODO(context): description`

## Error Message Quality
- Every error needs THREE components: machine-readable code + human message + correlation ID.
- Machine code: `AUTH_TOKEN_EXPIRED`, `VALIDATION_PHONE_INVALID` — parseable by code, stable across releases.
- Human message: "Your session has expired. Please log in again." — actionable, no jargon.
- Correlation ID: trace the error across services and logs. Include in every error response.
- Never expose stack traces, internal paths, or DB schemas in user-facing errors.

## Guard Clauses (Early Returns)
- Use early returns to handle edge cases at the top of the function.
- Each guard clause handles ONE condition and exits. This eliminates nesting.
- Bad: `if (valid) { if (authorized) { if (exists) { ...logic... } } }`
- Good: `if (!valid) return error; if (!authorized) return error; if (!exists) return error; ...logic...`
- Result: flat code, clear preconditions, easy to scan.

## Avoid Stringly-Typed Code
- Never use raw strings for values from a known set (status, role, category, event type).
- Use enums, discriminated unions, or const objects with `as const`.
- Stringly-typed code moves errors from compile-time to runtime. Runtime errors cost 10-100x more.
- Exception: data coming from external systems — validate and convert to typed values at the boundary.

## Magic Number Elimination
- No unexplained numeric literals in logic. Every number needs a name.
- Bad: `if (retries > 3)` / `setTimeout(fn, 86400000)`
- Good: `if (retries > MAX_RETRY_ATTEMPTS)` / `setTimeout(fn, ONE_DAY_MS)`
- Exception: universally obvious values (0, 1, -1, 100 for percentages) in clear context.
- Constants group by domain: `AUTH_CONSTANTS`, `RATE_LIMIT_CONSTANTS`, etc.

## Boolean Parameter Anti-Pattern
- Never: `createUser(data, true, false, true)` — impossible to read at call site.
- Instead: use an options object `createUser(data, { sendEmail: true, isAdmin: false, verify: true })`
- One boolean parameter is acceptable ONLY if the function name makes it obvious: `setVisible(true)`.
- Two or more booleans: always use options object or named arguments.

## Dead Code Deletion
- Delete dead code immediately. Don't comment it out "just in case."
- Git has full history — any code can be recovered from a previous commit.
- Commented-out code rots: it doesn't get updated when APIs change, creating false safety.
- If code might be needed soon, create a task/ticket referencing the commit hash instead.
