---
name: modify-api
description: "Safely modify an existing API endpoint: update contract, implementation, tests, and documentation"
---

# Modify API Skill

## Process

### 1. Understand the Change
- What endpoint is being modified?
- What's the current behavior?
- What should the new behavior be?
- Is this a breaking change?

### 2. Review Current State
- Read the endpoint's module (find via entry point)
- Read the API contract in `docs/API_CONTRACTS.md`
- Read existing tests
- Check who calls this endpoint (use CodeGraphContext if available)

### 2b. Backward Compatibility Check
Before any changes, list all fields/parameters being removed or renamed:
1. Search codebase for all usages: `grep -r "fieldName" src/`
2. Check if any external clients may use this (check docs, CHANGELOG)
3. If used externally:
   - Keep old field/parameter for at least 1 version
   - Add deprecation warning in response/logs
   - Document removal timeline in API_CONTRACTS.md
   - Create entry in brain/templates/api-changelog.md

### 3. Update Contract First
- Update types/interfaces FIRST
- If breaking change: discuss migration strategy with user
- Update `docs/API_CONTRACTS.md`

### 4. Update Implementation
- Modify service/handler to match new contract
- Keep changes minimal and focused
- Preserve backward compatibility if possible

### 5. Update Tests
- Update existing tests for new behavior
- Add tests for new cases
- Ensure old edge cases still covered

### 6. Verify
- Typecheck passes (contract changes propagate)
- All tests pass
- Linter passes
- Log the API change in `brain/04-decisions/`

### 7. Document
- Update API_CONTRACTS.md
- If breaking: note in DECISIONS.md with migration guide
