# Testing Rules

## Test Location
- Tests colocated with source code: `module.test.ts`, `test_module.py`, `module_test.go`
- Integration tests in `tests/integration/`
- E2E tests in `tests/e2e/`

## Test Independence
- Each test is fully independent. No shared mutable state between tests.
- Use factory functions for test data, not hardcoded objects.
- Setup/teardown per test, not per suite (unless expensive resources).

## Test Structure
- Arrange → Act → Assert (AAA pattern)
- One assertion concept per test (multiple asserts OK if testing same concept)
- Test name describes the scenario: `should_return_error_when_input_is_empty`

## What to Test
- Pure functions: input/output pairs, edge cases, error cases
- For pure functions with wide input space: property-based tests with fixed seed
- Async code: test both success and error paths
- Public API only. Don't test private internals.

## What NOT to Test
- Framework code (React renders, Express routing)
- Simple getters/setters
- Type system (that's what typecheck is for)
- Stale implementation paths whose only requirement is "must not ship". Delete
  the stale path instead, and test the intended public contract plus absence of
  obsolete routes/flags only when that absence is a product or safety contract.

## Test Execution
- ALWAYS run relevant tests after making changes
- Fix failing tests before committing
- Never commit with known test failures

## Coverage Thresholds
- `core/` — 90% minimum.
- `features/` — 80% minimum.
- `adapters/` — 60% minimum (IO is hard to unit test).
- `shared/` utilities — 95% minimum.
- Property-based tests for pure functions with >3 parameters.

## Snapshot Tests
- Use ONLY for serialized outputs (JSON responses, HTML templates, CLI output).
- Never for business logic.
- Always review snapshot diffs — never run `--update-all` blindly.
- Snapshots must be deterministic (no dates, UUIDs, random values).

## Testing Pyramid Strategy
- **Many unit tests** (~70%): fast, isolated, test pure logic. Run in <10s. Cheap to write and maintain.
- **Fewer integration tests** (~20%): test module boundaries, DB queries, API contracts. Slower but catch wiring bugs.
- **Few E2E tests** (~10%): test critical user journeys only. Expensive, slow, flaky-prone.
- WHY this ratio: unit tests give fast feedback loops (seconds, not minutes). Integration tests catch what units miss (serialization, config, wiring). E2E tests prove the system works but are too slow/flaky to be the primary safety net.
- Invert the pyramid = slow CI, flaky builds, developer frustration, ignored test suites.

## Flaky Test Policy
- A flaky test is WORSE than no test — it erodes trust in the entire suite.
- When a test flakes: fix root cause within 48 hours or delete it.
- Never `@skip` / `.skip()` permanently. Skipped tests are invisible broken windows.
- Common flaky causes: time-dependent logic, shared state, network calls, race conditions.
- Fix pattern: make deterministic (inject clocks, isolate state, mock network, add proper waits).

## Test Naming Convention
- Format: `should_[expected behavior]_when_[condition]`
- Examples: `should_return_404_when_user_not_found`, `should_retry_when_connection_timeout`
- The name IS the specification. A reader should understand the requirement without reading the test body.
- Group related tests with `describe` blocks named after the unit under test.

## Factory / Fixture Pattern
- Never use raw object literals scattered across tests. Create factory functions.
- `createUser({ name: 'Alice' })` with sensible defaults for all other fields.
- Factories centralized per module: `auth.fixtures.ts`, `test_auth_factory.py`.
- Benefits: one place to update when schema changes, readable tests, less duplication.
- For complex graphs (user + org + permissions), use builder pattern: `UserBuilder().withOrg(org).withRole('admin').build()`

## Mocks vs Real Dependencies
- **Mock external services**: APIs, payment gateways, email providers — you don't control them.
- **Use real internal dependencies**: your own modules, in-memory DB, local queues — they're fast and accurate.
- Mocking internal code couples tests to implementation. Refactor breaks tests even when behavior is unchanged.
- Exception: mock internal dependencies that are genuinely slow (>100ms) or have side effects (file I/O).
- Rule: if you mock more than 2 things in a test, the design probably needs refactoring, not more mocks.

## Contract Testing for API Boundaries
- When modules communicate via API (REST, gRPC, messages): consumer-driven contract tests.
- Consumer defines expected request/response shape. Provider verifies it passes.
- Tools: Pact, Spring Cloud Contract, or custom schema validation.
- Contract tests catch breaking changes BEFORE deployment, not after production incidents.
- Especially critical for microservices and external API consumers.

## Performance Test Thresholds
- Define SLA for critical paths: API response time, page load, batch processing duration.
- Codify thresholds in tests: `expect(responseTime).toBeLessThan(200)` // ms
- Run performance tests in CI on every merge to main (not just before release).
- Fail the build on regression: if P95 latency increases >20%, investigate before merging.
- Track trends over time — slow degradation is invisible without historical data.
