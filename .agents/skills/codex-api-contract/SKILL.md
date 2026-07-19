---
name: codex-api-contract
description: "Validate or create API contracts, OpenAPI docs, endpoint behavior, request/response schemas, status codes, error shapes, pagination, rate limits, and backwards compatibility."
---

# Codex API Contract

When a public contract change depends on an unresolved destination or transition,
use `$codex-change-strategy` before changing the contract. This
skill then verifies the selected API path and compatibility evidence.

Read `.claude/skills/api-contract/SKILL.md` and `docs/API_CONTRACTS.md` when present.

## Process

1. Treat the contract as the source of truth.
2. Compare implementation, tests, generated types, and docs.
3. Identify breaking versus non-breaking changes.
4. Prefer typed schemas and Structured Outputs where applicable.
5. Add contract tests for public API changes.
6. Update `docs/API_CONTRACTS.md` or `docs/openapi.yaml` when behavior changes.
