# Domain: Software Development — Evidence-Based Guard

## NEVER Recommend (Anti-patterns)
1. **Magic Numbers** — Unexplained literals destroy readability and refactoring safety. Instead: named constants, enums, config.
2. **Swallowing Exceptions** — Empty catch blocks hide ~20% of errors, creating silent failures. Instead: log with stack trace, catch specific.
3. **God Object** — One class doing everything; 0.55 correlation with bug count. Instead: decompose by single responsibility.
4. **Premature Optimization** — Optimizing without profiling wastes effort on non-bottlenecks 97% of the time (Knuth). Instead: profile first.
5. **Copy-Paste Programming** — Duplicated code means duplicated bugs. Instead: extract shared logic after 2+ duplications.
6. **Stringly-Typed Code** — Using strings where enums/types belong causes runtime errors. Instead: typed enums, discriminated unions.
7. **Cargo Cult Architecture** — Blindly copying microservices/patterns from FAANG without matching scale needs. Instead: start monolith, split when proven.
8. **No Tests Before Refactoring** — Refactoring without test coverage introduces silent regressions. Instead: write characterization tests first.
9. **Reinventing Crypto/Auth** — Custom security code has near-100% vulnerability rate. Instead: use battle-tested libraries (bcrypt, OAuth).
10. **Monolithic Deployments at Scale** — Single deployable with 50+ devs creates merge hell and 2-week release cycles. Instead: modular architecture with independent deployment.
11. **Testing Implementation, Not Behavior** — Tests coupled to internal structure break on every refactor. DETECT: test mocks >3 internal methods. Instead: test public API inputs/outputs only.
12. **Hardcoded Secrets in Source** — API keys, passwords in code leak via git history. DETECT: grep for "password=", "secret=", base64 blobs. Instead: env vars, vault, .env in .gitignore.
13. **N+1 Query Problem** — Fetching related records in a loop; 100 items = 101 DB calls. DETECT: ORM logs showing repeated SELECT per row. Instead: eager loading, batch queries, DataLoader pattern.
14. **Unbounded Queries** — SELECT * without LIMIT on user-facing paths; one large table = OOM or timeout. Instead: always paginate, set max page size, use cursor-based pagination.
15. **Snapshot Test Overuse** — Snapshots of large objects become "approve all" rubber stamps hiding real regressions. Instead: assert on specific fields that matter; snapshots only for serialized outputs.

## ALWAYS Apply (Principles)
1. **YAGNI** — Don't build it until you need it. Unused code is maintenance debt.
2. **Strangler Fig Migration** — Incrementally replace legacy systems; never big-bang rewrite.
3. **Immutability by Default** — Mutable shared state is the #1 source of concurrency bugs. Default to const/final.
4. **Fail Fast** — Validate inputs at boundaries; crash early with clear errors, not late with corruption.
5. **Vertical Slice Architecture** — Organize by feature, not by layer. Each slice is independently deployable/testable.
6. **Trunk-Based Development** — Short-lived branches (<1 day) with feature flags reduce merge conflicts 80%+.
7. **Dependency Inversion** — Depend on abstractions, not concretions. Core logic must never import infrastructure.
8. **Observability Triad** — Logs + metrics + traces from day one. You cannot fix what you cannot see.
9. **Database Migrations as Code** — Schema changes versioned and reversible. Never manual DDL in production.
10. **Property-Based Testing** — For pure functions with wide input space, generate random inputs. Catches edge cases unit tests miss.
11. **Input Validation at Boundaries** — Sanitize and validate all external input (user, API, file) at the system edge. Interior code trusts boundary-validated data.
12. **Feature Flags for All Releases** — Ship behind flags; enables gradual rollout, instant rollback, and A/B testing without redeployment.
13. **Dependency Pinning + Audit** — Pin exact versions in lock files. Run npm audit / pip audit / cargo audit in CI. Never use * or latest.
14. **Connection Pooling + Timeouts** — Every external call (DB, HTTP, queue) must have a connection pool, timeout, and retry-with-backoff. Unbounded connections = cascading failure.
15. **Test Pyramid** — Many unit tests (fast, isolated), fewer integration tests (real dependencies), minimal E2E tests (full stack). Inverted pyramids are slow and brittle.

## For Details
See `brain/03-knowledge/domains/programming-and-architecture.md` for full 50+50 practices with sources.
