# Analysis Protocol — Extract Principles, Go Deep

## Two Rules in One

### Rule 1: Extract, Never Copy (for external material)
When given ANY external material (doc, project, codebase, example):
NEVER copy structure. ALWAYS extract principles first.

**Protocol**: Decompose (WHAT) → Interrogate (WHY each part exists) → Abstract (PRINCIPLES) → Evaluate (do they APPLY here?) → Synthesize (design for OUR context).

**Red flags**: "Let's do it the same way" → WHY? "The example has X so we need X" → Does OUR context need X?

**Litmus test**: Can you explain each element's PURPOSE without referencing the source? If no → you're copying.

### Rule 2: No Surface-Level Analysis (for audits/reports)
When analyzing ANY system — go deep. HTTP 200 is not "working". "Looks fine" is not analysis.

**Minimum depth levels**:
- **Level 0 (INSUFFICIENT)**: HTTP 200, compiles, tests pass. THIS IS NOT ANALYSIS.
- **Level 1 (MINIMUM)**: User can complete primary action end-to-end. Verified by doing it.
- **Level 2 (ADEQUATE)**: + error states handled, edge cases work, performance acceptable.
- **Level 3 (THOROUGH)**: + evaluated from user/business perspective.

Level 0 is NEVER acceptable. Always find at least one improvement (zero findings = insufficient).

**Three Whys**: For every finding, ask WHY three times to reach root cause.

**Anti-patterns**: "API returns 200" → should be "API returns valid user object with all fields". "Tests pass" → should be "42 tests covering auth, errors, edges. Coverage 87%".

## Enforcement

Before submitting ANY analysis:
1. Did I actually TEST what I'm claiming works? (not assume — test)
2. Would a skeptical user accept this evidence?
3. Did I find at least ONE issue?
4. Am I reporting OBSERVATIONS or EXPECTATIONS?

See also: `.claude/library/meta/critical-thinking.md`
