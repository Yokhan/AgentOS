# Tool Registry

> Searchable index of reusable utilities across this project.
> **Check HERE before writing new code.** See `.claude/library/technical/atomic-reuse.md`.
>
> Maintained by: agents (manual), `scripts/scan-project.sh` (project-level scan), `scripts/audit-reuse.sh` (ongoing).

## Template-Level (available in ALL projects)

| Tool | Path | Purpose |
|------|------|---------|
| check-drift | scripts/check-drift.sh | Template health check (9 checks) |
| check-banlist | scripts/check-banlist.sh | Non-blocking phrase-signal scanner for contextual content review |
| session-metrics | scripts/session-metrics.sh | Session stats collector (daily log) |
| sync-template | scripts/sync-template.sh | Template sync with hash verification |
| bootstrap-mcp | scripts/bootstrap-mcp.sh | MCP server auto-setup |
| code-intelligence-tools | scripts/code-intelligence-tools.js | Route each task to a minimal tool sequence; validate, install, and health-check the pinned ten-tool stack |
| configure-codex-mcp | scripts/configure-codex-mcp.js | Merge the managed MCP block into project `.codex/config.toml` without overwriting project-owned Codex settings |
| test-codex-mcp-config | scripts/test-codex-mcp-config.js | Regression tests for safe Codex MCP add, update, idempotency, and conflict handling |
| audit-reuse | scripts/audit-reuse.sh | Duplicate detector, extraction candidates |
| downstream-census | scripts/downstream-census.sh | Migration matrix across downstream template repos |
| generate-project-spec | scripts/generate-project-spec.sh | Build PROJECT_SPEC.md from repo state |
| scan-project | scripts/scan-project.sh | Initial project scan, registry population |
| task-brief | scripts/task-brief.sh | Compact summary for tasks/current.md |
| codex-agent-policy | scripts/codex-agent-policy.js | Single SOT for Codex role models, reasoning effort, sandbox, and automatic fan-out limits |
| codex-route-task | scripts/codex-route-task.js | Deterministic Codex route selection for skills, subagents, fan-out decision, pipeline, risk, and orchestrator |
| validate-progressive-plan | scripts/validate-progressive-plan.js | Validates that every progressive implementation slice fulfills the product purpose end to end without stub-dependent or fabricated evidence |
| validate-change-strategy | scripts/validate-change-strategy.js | Validates evidence-bound destination/transition decisions, protected contracts, approval envelopes, compatibility profiles, and cleanup |
| validate-subagent-trace | scripts/validate-subagent-trace.js | Proves a real child thread used the required role/model and was awaited; parent markers do not count |
| test-codex-routing | scripts/test-codex-routing.js | Smoke tests for Codex route behavior and AgentOS detection |
| validate-agent-sot | scripts/validate-agent-sot.js | Validates local Agent SOT sources, AGENTS/CLAUDE links, and drift gates |
| validate-spec-kit | scripts/validate-spec-kit.js | Offline validation for the managed Spec Kit snapshot |
| validate-text-policy | scripts/validate-text-policy.js | Fails on invalid UTF-8, BOM, mixed line endings, mojibake, and unsafe shell OS assumptions |
| validate-writing-references | scripts/validate-writing-references.js | Validates template/project writing sources, profiles, external-tool access evidence, provenance, freshness, integrity, and editor links |
| writing-route-policy | scripts/lib/writing-route-policy.js | Shared cross-platform writing modes, Russian child profiles, external-tool states, technical/API/vendor specializations, editor roles, skills, and gates |
| sync-spec-kit | scripts/sync-spec-kit.sh | Fetch/update the managed GitHub Spec Kit snapshot |
| init-spec-kit | scripts/init-spec-kit.sh | Initialize Spec Kit in a project using the pinned snapshot ref |
| test-hooks | scripts/test-hooks.sh | Hook syntax validation |
| brain-search | scripts/brain-search.sh | Search brain/ knowledge base |

## Project-Level (auto-populated by scan-project.sh, updated by agents)

| Tool | Path | Purpose | Used by |
|------|------|---------|---------|
| blast-radius | scripts/blast-radius.sh | Basename/grep blast-radius fallback; not a semantic graph | agent/manual |
| codex-agent-policy | scripts/codex-agent-policy.js | Role/model/effort and adaptive fan-out policy used by router and validators | agent/runtime |
| codex-route-task | scripts/codex-route-task.js | Codex route contract generator for AGENTS.md route-first workflow | agent/manual |
| codex-hook-adapter | scripts/codex-hook-adapter.sh | codex-hook-adapter.sh — Translates Codex hook env vars to Claude hook | agent/manual |
| context-restore | scripts/context-restore.sh | context-restore.sh — Restore context after compaction or session start | agent/manual |
| import-graph | scripts/import-graph.sh | Relative-import grep fallback; prefer parser-backed graph for structural claims | agent/manual |
| measure-context | scripts/measure-context.sh | measure-context.sh — Measure auto-loaded context size + budget check | agent/manual |
| module-status | scripts/module-status.sh | Module Status Scanner — analyze project module health | agent/manual |
| plan-scaffold | scripts/plan-scaffold.sh | plan-scaffold.sh — Generate plan skeleton in tasks/current.md | agent/manual |
| research | scripts/research.sh | research.sh — Automated research protocol (replaces 6+ tool calls with | agent/manual |
| route-task | scripts/route-task.sh | route-task.sh — Dynamic keyword router (zero AI tokens) | agent/manual |
| scan-projects | scripts/scan-projects.sh | Scan directory for git repos and output JSON-friendly pipe-delimited dat | agent/manual |
| scan-repo | scripts/scan-repo.sh | scan-repo.sh — Scan a cloned/untrusted repo for security risks before | agent/manual |
| set-mode | scripts/set-mode.sh | set-mode.sh — manually set task mode (alternative to route-task.sh) | agent/manual |
| sync-agents | scripts/sync-agents.sh | sync-agents.sh — Validate CLAUDE.md and AGENTS.md reference the same s | agent/manual |
| sync-all | scripts/sync-all.sh | sync-all.sh — Sync template to all projects with .template-manifest.js | agent/manual |
| test-template | scripts/test-template.sh | test-template.sh — Smoke test for agent-project-template | agent/manual |
| update-template | scripts/update-template.sh | update-template.sh — Update project from newer template version | agent/manual |
| validate-agent-sot | scripts/validate-agent-sot.js | Ensures agent docs and source registry stay present and referenced | agent/manual |
| validate-spec-kit | scripts/validate-spec-kit.js | Ensures the local Spec Kit snapshot and manifest are complete | agent/manual |
| validate-text-policy | scripts/validate-text-policy.js | Enforces UTF-8/no-mojibake text policy and shell OS/temp helper usage | agent/manual |
| validate-writing-references | scripts/validate-writing-references.js | Enforces writing reference provenance, freshness, project overlay, and role integrity | agent/manual |
| sync-spec-kit | scripts/sync-spec-kit.sh | Updates `_reference/spec-kit/upstream` from GitHub Spec Kit | agent/manual |
| init-spec-kit | scripts/init-spec-kit.sh | Runs official Spec Kit CLI at the pinned manifest ref | agent/manual |
| validate-template | scripts/validate-template.sh | validate-template.sh — Pre-release validation for agent-project-templa | agent/manual |
| verify-check | scripts/verify-check.sh | verify-check.sh — Automated verification checklist | agent/manual |

## Helpers & Utilities (src/shared/ or lib/)

| Function | Path | Signature | Used by |
|----------|------|-----------|---------|
| _No shared utilities detected_ | | | |

## Candidates for Extraction (auto-detected by audit-reuse.sh)

| Function | Found in | Count | Recommendation |
|----------|----------|-------|----------------|
| _Run `bash scripts/audit-reuse.sh` to detect_ | | | |

## Design Tokens & Components (Figma projects only)

| Component | ID/Path | Variants | Used by |
|-----------|---------|----------|---------|
| _Populated by agents working with Figma MCP_ | | | |

---

_Last scan: 2026-04-21_
