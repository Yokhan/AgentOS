---
name: test-rules
description: "Run regression tests against rule definitions. Verify that domain rules (health, science, software) correctly prevent anti-patterns. Trigger: /test-rules or as part of /benchmark."
---

# Rule Regression Tests

Verify that rules actually work by presenting test scenarios and checking agent behavior.

## When to Use
- After updating domain rules
- As part of `/benchmark`
- During `/weekly` or `/retrospective` to verify rule health

## Process

1. **Load test cases** from `tests/rules/*.test.md`
2. **For each test**:
   a. Read the input scenario
   b. Evaluate: what would the agent recommend given this input?
   c. Compare against expected behavior
   d. Score: PASS (matches expected) / FAIL (violates rule)
3. **Report**: X/Y tests passed, list failures

## Test Case Format

```markdown
## Test: [name]
Input: "[what user asks or scenario description]"
Expected: [what the agent should do/not do]
Rule tested: [which NEVER/ALWAYS item from which rule file]
```

## Running Tests

For each test case:
1. Read the input as if a user asked it
2. Mentally evaluate: given all loaded rules, what would I recommend?
3. Check: does my response match the Expected behavior?
4. If FAIL: the rule may need strengthening or the agent needs better enforcement

## Report Format

```
Rule Test Results — [date]
┌──────────────────────┬───────┬────────┬────────┐
│ Rule File            │ Tests │ Passed │ Failed │
├──────────────────────┼───────┼────────┼────────┤
│ domain-health        │ 5     │ 5      │ 0      │
│ domain-software      │ 4     │ 3      │ 1      │
│ domain-science       │ 3     │ 3      │ 0      │
└──────────────────────┴───────┴────────┴────────┘
Total: 12/12 passed (100%)

Failed tests:
- domain-software/Test: God Object — agent did not flag 500-line class
```

## Creating Project-Specific Tests

Projects create `tests/rules/project-*.test.md` for their project-specific rules.
These are preserved during template sync.
