# Error Handling Rules

## Strategy
- Use Result/Either pattern for expected failures (validation errors, not-found, etc.)
- Use exceptions/panics ONLY for truly unexpected states (programmer errors)
- Never throw in core/ modules — return typed errors
- Catch errors at system boundaries (HTTP handlers, CLI entry points, message consumers)

## Typed Errors
- Define explicit error types per domain (AuthError, ValidationError, NotFoundError)
- Include context: error code, message, original cause
- No raw strings as error messages in production code

## Error Logging
- Log at the boundary where you catch, not where you throw
- Include: correlation ID, stack trace (in debug mode), input context (sanitized)
- Never swallow errors silently (`catch {}` with no logging is forbidden)

## Recovery
- Always define recovery path: retry, fallback, fail-fast, or user notification
- Transient failures: retry with exponential backoff (max 3 attempts)
- Permanent failures: fail fast, log clearly, return typed error

## Error Categories
Two fundamentally different kinds of errors requiring different strategies:
- **Operational errors** (expected): network timeout, invalid input, file not found, rate limit hit.
  Handle gracefully: retry, fallback, inform user. The system continues running.
- **Programmer errors** (bugs): null reference, out-of-bounds, type mismatch, logic error.
  Crash fast: log everything, alert, restart process. Don't try to "handle" bugs — fix them.
- Treating bugs as operational errors = silent corruption. Treating operational errors as bugs = unnecessary crashes.

## Boundary Error Handling Pattern
- Catch and handle errors at system BOUNDARIES: HTTP handlers, CLI entry points, message consumers, scheduled jobs.
- Deep inside the code: propagate errors upward, don't catch them. Let the boundary decide.
- Each boundary has ONE error handler that: logs, maps to user-facing response, tracks metrics.
- Anti-pattern: catch-rethrow at every layer. This clutters code and often loses the original stack trace.
- If you must catch mid-stack, ADD context: `throw new ServiceError('Failed to process order', { cause: originalError, orderId })`.

## Error Propagation Rules
- Never catch-and-rethrow without adding meaningful context.
- Context to add: which operation failed, which entity was involved, what was the input.
- Preserve the original error as `cause` — stack trace from origin is essential for debugging.
- Never: `catch(e) { throw new Error('something went wrong') }` — this destroys all diagnostic information.
- Pattern: `catch(e) { throw new OrderProcessingError('Failed to process order ' + orderId, { cause: e }) }`

## User-Facing vs Developer-Facing Errors
- **User-facing**: friendly message, error code for support, suggested action. Never: stack traces, internal paths, SQL queries, server names.
- **Developer-facing** (logs): full stack trace, input context (sanitized), correlation ID, timestamp, service version.
- Map internal errors to HTTP/response codes at the boundary: ValidationError->400, AuthError->401, NotFound->404, internal->500.
- 500 errors always trigger alerts. 400-level errors are normal — track rate, don't alert on individuals.

## Circuit Breaker for External Services
- External calls (APIs, DBs, third-party services) WILL fail. Plan for it.
- Circuit breaker states: CLOSED (normal) -> OPEN (failing, stop calling) -> HALF-OPEN (test recovery).
- Thresholds: open after 5 failures in 60 seconds. Half-open after 30 seconds. Close after 3 successes.
- When circuit is open: return cached data, default value, or graceful degradation — never block waiting.
- Every external dependency needs a circuit breaker. No exceptions.

## Graceful Degradation Hierarchy
When a dependency fails, degrade through levels — never jump straight to "service unavailable":
1. **Full service** — everything works normally.
2. **Partial service** — non-critical features disabled (recommendations off, but checkout works).
3. **Cached/stale data** — serve last known good data with staleness indicator.
4. **Static fallback** — pre-generated static content, maintenance page with status info.
5. **Honest failure** — clear error page with ETA if known, status page link, alternative contact methods.
- Design degradation paths BEFORE incidents. During incidents you won't think clearly.

## Dead Letter Queue (DLQ)
- For async message processing: messages that fail after all retries go to a DLQ, never silently dropped.
- DLQ messages need: original payload, error details, retry count, timestamp, source queue.
- Monitor DLQ size — growing DLQ = systematic problem, not random failures.
- Process DLQ entries: fix the bug, replay messages. Don't let the DLQ become a graveyard.
- Alert when DLQ exceeds threshold (e.g., >100 messages or >1% of throughput).

## Error Monitoring and Alerting
- Track error RATE, not just error COUNT. 100 errors at 1M requests is fine. 100 errors at 1K requests is a crisis.
- Alert thresholds: error rate >1% = warning, >5% = critical. Adjust per endpoint based on baseline.
- Group errors by type — alert on NEW error types immediately (never-seen-before = likely deployment issue).
- Dashboards: error rate over time, top errors by frequency, error budget remaining.
- Error budget: if SLA is 99.9%, you have 43 minutes/month of downtime. Track consumption.
