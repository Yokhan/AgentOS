---
name: codex-modify-api
description: "Safely modify an existing API endpoint by updating the contract, implementation, tests, docs, and compatibility notes. Trigger on endpoint changes or API behavior requests."
---

# Codex Modify API

Use with `$codex-api-contract`.

## Process

1. Read the current contract and endpoint implementation.
2. Identify consumers and compatibility risk.
3. Update the contract first.
4. Update implementation through module boundaries.
5. Add tests for success, validation, auth, and errors.
6. Run API-specific checks and document behavior changes.
