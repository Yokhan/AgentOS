---
name: debug
description: "Systematic debugging approach: reproduce, isolate, fix, verify, document"
---

# Debug Skill

## Systematic Debugging Process

### 1. Reproduce
- Get exact steps to reproduce the bug
- Identify: input, expected output, actual output
- Find the minimal reproduction case

### 2. Isolate
- **Search memory globally**: `mem_search("{module_name} bug")` without project filter — similar bugs may have been solved in other projects. If Engram unavailable: grep `tasks/lessons.md` and `tasks/.memory-fallback.md`
- Trace the execution path from input to error
- Use CodeGraphContext (if available) to find callers/callees
- Narrow down to the specific module/function
- Check recent changes: `git log --oneline -20` and `git diff`
- Run a bounded negative check over the affected path and direct consumers for
  causal final-plan, SOT/owner, duplicate-state, obsolete-path, or
  protected-boundary mismatch. This is not a general architecture review.

### 3. Hypothesize
- Form a specific hypothesis about the cause
- Look for common patterns:
  - Off-by-one errors
  - Null/undefined handling
  - Async race conditions
  - Type coercion issues
  - Stale cache/state
  - Missing error handling

### 4. Fix
- If reading already proves a qualifying system mismatch, invoke the shared
  Change Strategy Gate before the first patch. Reroute once only when pipeline,
  risk, or approval authority changes.
- Make the smallest reasonable change toward the accepted final architecture
- Do not refactor unrelated code in the same change
- Prefer fixing the root cause over symptoms

### 5. Verify
- Write a test that reproduces the original bug (it should fail without the fix)
- Apply the fix -- test passes
- Run full test suite -- no regressions
- Run typecheck and linter

### 5b. Regression Check
After fixing the immediate module, also run integration tests:
- `npm test -- --testPathPattern=integration` (Node)
- `pytest tests/integration/` (Python)
- `go test ./tests/integration/...` (Go)
If integration tests fail, the fix may have introduced a regression -- investigate before committing.

### 6. Document
- Log the bug and fix in `brain/03-knowledge/` for future reference
- If this is a pattern that could recur, consider:
  - Adding a project rule to `.claude/rules/project-*.md`
  - Adding a test to prevent regression
  - Adding a type constraint to prevent at compile time

### 7. Commit
Use `fix: description` commit message.
Reference the issue if there is one.

---

## Systematic Debugging Methodology (Scientific Method)

Debugging IS the scientific method applied to software:

1. **Observe** -- Collect all evidence before touching code. Error messages, logs, stack traces, reproduction steps, environment details. Write them down. Do not rely on memory.
2. **Hypothesize** -- Form a SPECIFIC, falsifiable hypothesis: "The null pointer occurs because `user.profile` is not loaded when the cache misses." Vague hypotheses ("something is wrong with the database") are useless.
3. **Test** -- Design an experiment that can DISPROVE your hypothesis. If you can only confirm, you have confirmation bias. Add a log line, set a breakpoint, write a minimal test case.
4. **Conclude** -- Either the hypothesis was correct (fix it) or wrong (return to Observe with new data). Never skip this step -- explicitly state what you learned.

Key discipline: only change ONE variable per experiment. If you change two things and the bug disappears, you do not know which change fixed it (or if it is just masked).

## Common Bug Patterns by Category

### State Bugs
- Shared mutable state modified from multiple call sites
- Stale state after async operation completes (UI shows outdated data)
- State initialized in wrong order (dependency on initialization sequence)
- State not cleaned up on error path (partial state corruption)
- Fix pattern: make state immutable, use state machines, centralize mutations

### Race Conditions
- Time-of-check to time-of-use (TOCTOU) -- checking a condition then acting on it non-atomically
- Multiple async operations completing in unexpected order
- Database read-then-write without locking or optimistic concurrency
- Event handlers firing during state transitions
- Fix pattern: atomic operations, locks/mutexes, optimistic concurrency control, idempotent handlers

### Off-by-One Errors
- Loop iterates one too many or one too few times
- Array index at boundary (0 vs 1, length vs length-1)
- Fence-post errors (intervals: inclusive vs exclusive boundaries)
- Pagination: page 0 vs page 1, last page calculation
- Fix pattern: use half-open intervals consistently, prefer iterators over manual indexing

### Null / Undefined References
- Optional field accessed without null check
- API response missing expected field
- Database query returns no rows but code assumes one
- Chained property access on potentially null object (`user.address.city`)
- Fix pattern: optional chaining, null object pattern, typed Result/Option types, strict null checks

### Encoding and Serialization
- Character encoding mismatch (UTF-8 vs Latin-1, BOM issues)
- JSON serialization losing type info (Date becomes string, BigInt fails)
- URL encoding/decoding applied twice or not at all
- Binary data treated as text (corrupted by encoding layer)
- Fix pattern: normalize encoding at system boundaries, use typed serialization, test with non-ASCII data

### Type Coercion
- Implicit type conversion producing unexpected results ("5" + 3 = "53" in JS)
- Truthy/falsy checks on values like 0, empty string, empty array that behave unexpectedly
- Integer overflow or floating-point precision loss
- Fix pattern: strict equality, explicit conversion, BigInt for large numbers, decimal libraries for money

## Debugging Tools Awareness

### Breakpoints and Stepping
- **Conditional breakpoints**: break only when a condition is true -- essential for loops and high-frequency code
- **Logpoints**: emit a log message without stopping execution (VS Code, Chrome DevTools)
- **Watch expressions**: monitor variable values as you step through code
- When to use: when you need to observe state changes step by step in a specific execution path

### Logging
- Add temporary structured logs with correlation IDs to trace request flow
- Log at decision points (if/else branches) to see which path executes
- Include timestamps to detect timing issues
- Remove debug logging before committing (or use debug-level that is off in production)

### Git Bisect
- Binary search through commit history to find the commit that introduced a bug
- `git bisect start`, `git bisect bad` (current), `git bisect good <known-good-commit>`
- Automate with `git bisect run <test-script>` for reproducible failures
- When to use: "this used to work" bugs where you do not know when it broke

### Profilers
- CPU profilers: find hot functions consuming disproportionate time
- Memory profilers: find leaks (objects that grow unboundedly)
- Flame graphs: visualize call stack depth and time distribution
- When to use: performance bugs, memory growth, unexpected resource consumption

### Network Inspectors
- Browser DevTools Network tab: inspect request/response pairs, timing, headers
- `curl -v` / `httpie`: reproduce API calls outside the application
- Proxy tools (mitmproxy, Charles): intercept and modify traffic for testing
- When to use: API integration bugs, CORS issues, authentication failures, payload mismatches

## "5 Whys" Root Cause Analysis

When you find the immediate cause, keep asking WHY until you reach the systemic root:

```
Bug: User sees stale data after updating profile.
Why 1: The cache returns old data after the update.
Why 2: The cache invalidation does not run after profile updates.
Why 3: The update function does not call cache.invalidate().
Why 4: There is no convention requiring cache invalidation after writes.
Why 5: The caching layer has no automatic invalidation strategy.
Root cause: Missing write-through or event-driven cache invalidation pattern.
Fix: Implement cache invalidation at the data access layer, not per-caller.
```

Stop when you reach something you can change structurally (architecture, process, convention) rather than just patching the symptom. The fix should make the CATEGORY of bug impossible, not just this instance.

## When to STOP Debugging and Rewrite Instead

Sunk cost awareness is critical. Consider rewriting when:

- **Time investment exceeds value**: you have spent 2+ hours on a function that would take 30 minutes to rewrite from scratch with tests
- **Fix count is climbing**: a second repair failed against the same acceptance
  criterion. This is the mandatory fallback circuit breaker; architecture
  evidence found earlier must activate the gate without waiting for attempt two.
- **Understanding is lost**: nobody (including you) can confidently explain what the code is supposed to do -- you are pattern-matching, not reasoning
- **The bug reveals a design flaw**: the fix requires violating the module contract or adding special cases that make future bugs more likely
- **Test coverage is zero**: adding characterization tests to understand behavior takes longer than rewriting with TDD

The rewrite decision: "If I were starting fresh today knowing what I know now, would I write it this way?" If no, and the cost of rewriting is bounded, rewrite.

Before rewriting: write the tests FIRST (based on desired behavior), then rewrite to pass them.

## Regression Prevention Protocol

After every fix, before committing:

1. **Write a regression test** that reproduces the original bug -- it must FAIL without your fix applied
2. **Name it descriptively**: `test_profile_cache_invalidated_after_update` not `test_fix_123`
3. **Add it to the right test suite**: unit test if isolated, integration test if it crosses boundaries
4. **Document the bug-test link**: comment in the test referencing the bug/issue for future archaeologists
5. **Consider related edge cases**: if this boundary was wrong, what about adjacent boundaries?

If you fix a bug and do not add a test, you have not fixed it -- you have merely delayed its return.

## Cross-Reference

- `.claude/library/meta/strategic-thinking.md` -- "Center of gravity" principle: find the root cause (center of gravity), not the symptom (periphery). "Friction" principle: the simplest bug is harder than it looks.
- `.claude/library/technical/error-handling.md` -- Typed errors, logging at boundaries, recovery patterns
- `.claude/library/technical/testing.md` -- Test structure, coverage thresholds, property-based testing for edge cases
