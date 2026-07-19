# Release Checklist

Use this checklist before calling the template production-ready or cutting a release tag.

Last reviewed: 2026-07-19 for published template `4.9.0`. Validation run `29672263414` passed all six Linux/Windows validation, bootstrap, full-toolchain, Codex MCP, and AgentOS jobs on commit `e00b714505ac4b1efb05c29e60e148a8ee7b2c85`. Release run `29672457240` revalidated the exact tag, built the archive, and published the non-draft, non-prerelease GitHub Release. The live Codex subagent probe is reported separately because it consumes quota; static markers never count as runtime proof.

## Validation Gate

- [ ] `bash scripts/validate-template.sh`
- [ ] `bash scripts/check-drift.sh`
- [ ] `bash scripts/test-hooks.sh`
- [ ] `bash scripts/test-template.sh`
- [ ] `bash scripts/sync-agents.sh`
- [ ] `node scripts/test-codex-routing.js`
- [ ] `node scripts/test-codex-agent-policy.js`
- [ ] `node scripts/validate-codex-skills.js`
- [ ] `node scripts/validate-codex-agents.js`
- [ ] `node scripts/validate-production-standard.js`
- [ ] `node scripts/validate-design-policy.js`
- [ ] `node scripts/test-design-policy.js`
- [ ] `node scripts/validate-agent-sot.js`
- [ ] `node scripts/validate-spec-kit.js`
- [ ] `node scripts/validate-text-policy.js`
- [ ] `bash scripts/generate-project-spec.sh --write`
- [ ] `bash scripts/scan-project.sh --report`

## Bootstrap Gate

- [ ] `bash setup.sh <smoke-project>` creates a clean project with `scripts/task-brief.sh`, starter task files, and no maintainer debug/audit leakage
- [ ] `cmd /c "(echo <smoke-project>) | setup.bat"` creates the same shipped surface on Windows
- [ ] Generated projects pass `bash scripts/test-hooks.sh`
- [ ] Generated projects pass `bash scripts/bootstrap-mcp.sh --dry-run`
- [ ] Linux and Windows runners pass `bash scripts/bootstrap-mcp.sh --install --tool-profile=full` and the matching `--check`
- [ ] Codex `0.125.0` loads `context-router`, `engram`, and `codebase-memory-mcp` from trusted project `.codex/config.toml`
- [ ] `node scripts/test-codex-routing.js` proves AgentOS remains the task-graph owner when `.agent-os` is present
- [ ] Generated projects pass `bash scripts/sync-template.sh <template-root> --dry-run`
- [ ] Generated projects can preview a pinned release sync with `bash scripts/sync-template.sh --from-git --ref <tag> --dry-run` when a `template` remote is configured

## Migration Gate

- [ ] `bash scripts/downstream-census.sh --brief <project-dir ...>` classifies representative downstream repos
- [ ] At least 3 real downstream repos have been evaluated for the current target version
- [ ] Clean and manual-merge paths are documented in `docs/MIGRATION_MATRIX.md`
- [ ] Any legacy local sync-script breakage is reproducible via the template-owned `sync-template.sh --project-dir` path

## Trust Gate

- [ ] No local-only state ships to fresh projects (`.claude/settings.local.json`, debug logs, audit history, dependency artifacts)
- [ ] `PROJECT_SPEC.md` and `_reference/tool-registry.md` can be regenerated from scripts instead of placeholders
- [ ] Session-start uses compact summaries, not raw markdown dumps
- [ ] No project-level Codex defaults override IDE/user-level model or effort settings
- [ ] `scripts/codex-agent-policy.js` is the only role/model/effort SOT; all template agent TOMLs match it and no profile exceeds `xhigh`
- [ ] Router output includes an observable fan-out status/reason/profile set; direct XS questions and explicit user opt-out do not spawn agents
- [ ] Automatic fan-out remains read-only first, uses `max_depth = 1`, and write delegation requires exact non-overlapping ownership
- [ ] No mojibake, replacement characters, mixed line endings, raw `uname`, raw `/tmp`, or raw `mktemp` outside `scripts/lib/platform.sh`
- [ ] Fetched manifest paths are passed to Node as data, never interpolated into generated JavaScript or shell hashing commands
- [ ] Pinned semver refs fetch `refs/tags/<tag>`; dry-run does not add remotes or rewrite project files/manifests
- [ ] `docs/PRODUCT_BOUNDARY.md`, `docs/SAFE_DEFAULTS.md`, and `docs/SUPPORTED_ENVIRONMENTS.md` match the shipped contract
- [ ] `_reference/spec-kit/manifest.json` matches the intended stable Spec Kit ref; run `bash scripts/sync-spec-kit.sh --check` before cutting the release
- [ ] `tasks/goal.md` and `templates/project-starter/tasks/goal.md` carry the v4 goal-loop contract
- [ ] Root `DESIGN.md` and `design-policy.ignore` are project-owned in generated projects
- [ ] Design policy findings include rule id, file, evidence, impact, next action, and ignore/baseline tuning guidance
- [ ] Product/business outcome priority is present in shared rules, agent entrypoints, skills, routing, and validators
- [ ] Client-executor accountability, anti-sycophancy, and no-fake-completion evidence gates are present in shared rules, agent entrypoints, skills, routing, and validators
- [ ] Progressive JPEG delivery, implementation gates, final-plan object readiness, progressive layer replacement, `PROGRESSIVE_STATUS` project-slice reporting, 1% production-function behavior, and plan/inventory/depth/cleanup/status verification order are present in shared rules, AGENTS/CLAUDE, Codex planning/feature/design skills, writing/testing rules, and production-standard validator checks
- [ ] Router output includes `planContract`, `productionBar`, `languagePolicy`, `qualityGates`, and `fanout`
- [ ] Codex routing uses exact patterns plus semantic intent scoring, reports exact/semantic matches, and has regression coverage for meaning-based routes
- [ ] Design-system work has token, composition trace, Storybook/equivalent, and rendered-geometry gates
- [ ] Design work has durable design context, command modes, hardening evidence, and deterministic design-policy checks
- [ ] Design work has product/brand register gates, command-mode reference coverage, critique ordering, and KPI-aware routing smoke
- [ ] Design work has a concrete screen anatomy/root-frame contract in shared rules, Codex design skills, and release smoke tests
- [ ] `sync-template.sh --from-git --ref <tag> --dry-run` fetches the ref and shows a real sync preview without modifying the downstream project

## Release Decision

- [ ] README/setup flow matches shipped behavior
- [ ] README and CLAUDE release-facing counts match the shipped filesystem
- [ ] CI workflow covers validation scripts plus Linux/Windows bootstrap smoke
- [ ] GitHub workflows and CI templates use Node 24-compatible actions; release/validation jobs disable unnecessary setup-node package-manager cache
- [ ] Manual release input is passed through `env`, validated as `vX.Y.Z`, checked out before validation, and asserted to match `HEAD`
- [ ] Published release assets are never silently replaced by workflow reruns
- [ ] Remaining manual-merge cases are acceptable and documented
- [ ] Release notes mention any unsupported or review-required upgrade paths
- [ ] Git tag uses `vX.Y.Z`; downstream instructions reference `scripts/sync-template.sh --from-git --ref <tag>`
- [ ] README, SETUP_GUIDE, and docs/TEMPLATE_RELEASES show the current stable release tag, link to `/releases/latest`, and warn that `main` is for canary/template development only
- [ ] AGENTS, CLAUDE, `/update-template`, and Codex sync skill point to the canonical source/downstream update protocol
- [ ] AgentOS rollout notes state whether AgentOS is the orchestrator or the project uses Codex parent orchestration
