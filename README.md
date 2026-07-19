# Agent Project Template v4

[![Template Version](https://img.shields.io/badge/template-v4.9.0-blue)](.)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)](CONTRIBUTING.md)

Self-deploying AI-agent optimized project template with MCP-based dynamic rule routing, a pinned stack-aware code-intelligence toolchain, Codex repo-scoped skills and subagents, persistent memory, autonomous work loops, self-improvement, and merge-safe sync. Token savings are benchmarked per project instead of assumed globally.

> **Подробная инструкция на русском:** [SETUP_GUIDE.md](SETUP_GUIDE.md) — пошаговая настройка, MCP-серверы, Zed, troubleshooting.
> Product boundary: [docs/PRODUCT_BOUNDARY.md](docs/PRODUCT_BOUNDARY.md) • Safe defaults: [docs/SAFE_DEFAULTS.md](docs/SAFE_DEFAULTS.md) • Supported environments: [docs/SUPPORTED_ENVIRONMENTS.md](docs/SUPPORTED_ENVIRONMENTS.md) • Codex fan-out: [docs/CODEX_FANOUT_PATTERNS.md](docs/CODEX_FANOUT_PATTERNS.md) • Template releases: [docs/TEMPLATE_RELEASES.md](docs/TEMPLATE_RELEASES.md)

## If An Agent Only Has This GitHub Link

Follow [the canonical update protocol](docs/TEMPLATE_RELEASES.md#canonical-agent-update-protocol):

1. Classify source, generated downstream, or legacy downstream; never sync the source into itself.
2. Read installed version from `.template-manifest.json`.
3. Explicit user/AgentOS tag wins; otherwise verify the exact stable tag at <https://github.com/Yokhan/agent-project-template/releases/latest>. Current stable tag: `v4.9.0`.
4. Verify `git remote get-url template`; never silently replace a conflict.
5. Run pinned dry-run, then apply the same tag. Bare `--from-git` is canary-only.
6. Use the target release checkout's script with `--project-dir` when local sync is stale.
7. Verify manifest version, diff, overlays, conflicts, and checks before success.

Create a new project from the stable tag:

```bash
git clone --branch v4.9.0 --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template
cd agent-project-template
bash setup.sh my-project
```

Update an existing generated project from the stable tag:

```bash
template_url="$(git remote get-url template 2>/dev/null || true)"
[ -n "$template_url" ] || git remote add template https://github.com/Yokhan/agent-project-template.git
[ -z "$template_url" ] || [ "$template_url" = "https://github.com/Yokhan/agent-project-template.git" ] || { echo "template remote conflict: $template_url"; exit 1; }
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run
bash scripts/sync-template.sh --from-git --ref v4.9.0
```

`main` is for template development and explicit canary rollout only. Release archives are useful for inspection or offline transfer; agent-managed projects should prefer git tag sync.

## Quick Start

The commands below target stable release `v4.9.0`.

```bash
git clone --branch v4.9.0 --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template
cd agent-project-template
bash setup.sh my-project
cd my-project
bash scripts/bootstrap-mcp.sh --install --tool-profile=full
```

Open and trust the generated project in Codex. Restart Codex after bootstrap,
then run `codex mcp list`: `context-router`, `engram`, and
`codebase-memory-mcp` must be listed. The remaining eight tools stay on-demand
CLI/LSP capabilities and do not inflate the permanent MCP surface.

`.codex/config.toml` is the active Codex MCP configuration. `.mcp.json` remains
only as a compatibility payload for Claude Code; it is not how Codex discovers
project MCP servers.

**Windows**: run `setup.bat`. It detects the Windows environment and prepares the context-router with native `npm.cmd`; do not substitute Unix bootstrap commands in PowerShell.

`README.md` and `SETUP_GUIDE.md` stay template-owned after bootstrap. Put project-specific onboarding or architecture details into `AGENTS.md`, `PROJECT_SPEC.md`, `ecosystem.md`, and `docs/`.

Optional Spec Kit setup is shipped but inert by default:

```bash
# Validate local managed snapshot
node scripts/validate-spec-kit.js

# Initialize Spec Kit in the current project using the pinned upstream ref
bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .
```

## Project Creation Modes

- **Generated project**: `setup.sh my-project` or `setup.bat` creates the clean payload that should ship to real projects.
- **Template maintainer workspace**: stay in this repository only when improving the template itself.
- **Optional orchestrator workspace**: `bash setup.sh --orchestrator my-orchestrator` creates a multi-project coordination workspace without template-maintainer artifacts.

`setup.*` copies the tracked project-facing allowlist only. It intentionally leaves behind template-maintainer files such as `n8n/`, `templates/`, local fixtures, debug artifacts, and the setup entrypoints themselves.

## Updating Existing Projects

When the template improves (new rules, agents, skills, hooks), update your project:

```bash
# Preview changes (no modifications)
bash scripts/sync-template.sh /path/to/agent-project-template --dry-run

# Apply updates
bash scripts/sync-template.sh /path/to/agent-project-template
```

Or use the Claude Code command: `/update-template /path/to/template`

If you are operating from the template repo instead of inside the child project, use:

```bash
bash /path/to/agent-project-template/scripts/sync-template.sh /path/to/agent-project-template --project-dir /path/to/my-project --dry-run
```

### Pinned release updates (git-based)
If the template is hosted in a git repository, prefer release tags for normal project rollout:
```bash
# Check the latest release tag first
# https://github.com/Yokhan/agent-project-template/releases/latest

# Preview the pinned release
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run

# Apply the pinned release
bash scripts/sync-template.sh --from-git --ref v4.9.0
```
Projects created from a git-hosted template automatically have a `template` remote configured. The SessionStart hook reminds you when updates haven't been checked in 7+ days.

### Canary updates from `main`
Use branch-based sync only for template development, early rollout, or canary projects where untagged changes are intentional:

```bash
bash scripts/sync-template.sh --from-git --canary --ref main --dry-run
bash scripts/sync-template.sh --from-git --canary --ref main
```

AgentOS can orchestrate when and where a tag is applied, but the template release still comes from this repository. If AgentOS artifacts are present, Codex treats them as the source task graph and uses template routing only as the worker execution contract.

### Updating older projects (created before v2.2.0)
```bash
# 1. Copy sync script into your project
cp /path/to/agent-project-template/scripts/sync-template.sh my-project/scripts/

# 2. Bootstrap — generates .template-manifest.json from current state
cd my-project
bash scripts/sync-template.sh /path/to/agent-project-template --bootstrap

# 3. Sync — applies template updates
bash scripts/sync-template.sh /path/to/agent-project-template

# Optional: add git remote for future auto-updates
git remote add template https://github.com/Yokhan/agent-project-template.git
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run
bash scripts/sync-template.sh --from-git --ref v4.9.0
```

**What gets updated**: Template infrastructure (`.agents/`, `.claude/`, `.codex/`, scripts, MCP helper sources, AGENTS.md, onboarding docs)
**What's preserved**: Your code (`src/`), project docs, `brain/`, `tasks/`, `CLAUDE.md`, `PROJECT_SPEC.md`, `ecosystem.md`, and all `project-*` files
**Convention**: Template files are read-only baseline. Project customizations go to `project-*` prefixed files (e.g., `rules/project-no-mock-db.md`).

Writing references follow the same boundary: template source/property defaults
are synced from `.claude/library/technical/writing-reference-registry.json`, while
approved project voice, samples, rights, and replacements live in the preserved
`brain/03-knowledge/writing/reference-registry.json`.
The registry separates target-language/editorial authority from domain standards:
Russian output loads the Russian writing profile, while English technical or
regulatory sources can constrain correctness without shaping Russian prose.
Russian communication and explanatory work load dedicated child profiles for
business correspondence and explanation/persuasion. Paid writing services are
tracked separately as external tools. The template declares the Glavred API
capability as `not-configured`; it does not include an adapter or subscription.
Agents may apply public principles manually but cannot claim a provider check,
score, or warning list without a real artifact-specific response. Eligible routes
also report `not-run` until such a response exists.

## Extending the Template for Your Domain

The template is a **baseline**. Every project adds domain-specific infrastructure on top.

### Convention: `project-*` prefix

All project-specific files use the `project-` prefix. Template sync **never touches** these files.

| Type | Template (synced) | Project (preserved) |
|------|-------------------|---------------------|
| Rules | `rules/architecture.md` | `rules/project-kiro-system.md` |
| Commands | `commands/implement.md` | `commands/project-00-research.md` |
| Skills | `skills/debug/SKILL.md` | `skills/project-kiro-drafting/SKILL.md` |
| Codex Skills | `.agents/skills/codex-debug/SKILL.md` | `.agents/skills/project-kiro-drafting/SKILL.md` |
| Codex Agents | `.codex/agents/reviewer.toml` | `.codex/agents/project-kiro-reviewer.toml` |
| Agents | `agents/reviewer.md` | `agents/project-kiro-writer.md` |
| Hooks | `settings.json` (template) | `settings.local.json` (project) |

### Adding project hooks

Put project-specific hooks in `.claude/settings.local.json` (not `settings.json`):
```json
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "Edit",
      "hooks": [{"type": "command", "command": "bash core/scripts/validate.sh", "timeout": 10}]
    }]
  }
}
```
Claude Code merges both files. Project hooks run alongside template hooks.

### Adding a domain pipeline

For complex domain workflows (literary production, game design, data science):

1. **Domain rules** → `.claude/rules/project-[domain]-*.md` (enforcement, methodology)
2. **Domain commands** → `.claude/commands/project-[phase]-*.md` (pipeline steps)
3. **Domain skills** → `.claude/skills/project-[domain]-*/SKILL.md` (specialist knowledge)
4. **Codex domain skills** → `.agents/skills/project-[domain]-*/SKILL.md` (Codex-native specialist knowledge)
5. **Domain agents** → `.claude/agents/project-[domain]-*.md` (sub-agents)
6. **Domain scripts** → `core/scripts/` (validators, generators — NOT in template's `scripts/`)
7. **Domain docs** → `core/docs/` (methodology, reference material)
8. **Domain config** → `core/config.yaml` (universal project configuration)

### Progressive disclosure for domain docs

Follow the 5-level loading pattern:
1. **Critical** (every session): `.claude/rules/project-*.md`
2. **Project** (once per project): `core/config.yaml`, status dashboard
3. **Task** (per task): relevant domain docs from `core/docs/`
4. **Methodology** (on demand): methodology files, editorial boards
5. **Reference** (deep search): cheat sheets, examples, full indices

### Template updates preserve your extensions

When you run `/update-template` or `bash scripts/sync-template.sh`:
- Template files → **updated** to the explicitly resolved target tag
- `project-*` files → **untouched**
- `settings.local.json` → **untouched**
- `core/` directory → **untouched** (not tracked by template)

## What's Included

### Generated project includes these

| Category | Count | Details |
|----------|-------|---------|
| **Rules** | 32 | Shared library rules, four-mode writing profiles, technical-writing overlay, editorial board, change strategy, plus router entrypoint |
| **Hooks** | 12 | session-start/stop, pre-compact, format, post-edit, pre-edit-safety, verify-gate, security, audit, and encoding checks |
| **Claude Skills** | 33 | Core, development, quality, domain review, integrations, four-mode writing, and technical-writing generation/review |
| **Codex Skills** | 46 | Pipeline, route-first orchestration, evidence-backed change strategy, truthful progressive JPEG, four-mode and technical writing, subagent orchestration, design/Figma, audit/debug/security, setup, domain review, template ops, integrations, migrations, and OpenAI model guidance |
| **Codex Subagents** | 12 | Luna bounded discovery/log/summarization, Terra research/testing/isolated implementation, and Sol judgment-heavy specialists; adaptive fan-out preserves project orchestration ownership |
| **Agents** | 12 | protocol plus implementer, reviewer, researcher, test-engineer, security-auditor, writer, technical-writer, simplifier, documenter, devops, and profiler |
| **Commands** | 23 | setup, implementation, review, release, audit-tools, sync, sprint, rollback, mode switching, and maintenance commands |
| **Scripts** | 62 | validation, adaptive writing/reference routing, change-strategy validation, Codex MCP merge tests, provenance checks, agent policy, progressive plan/status and subagent-trace gates, design checks, drift checks, bootstrap, sync, scanning, task brief, hooks, Spec Kit setup, and release smoke |
| **Spec Kit** | snapshot | managed upstream snapshot, freshness check, and pinned init flow |
| **Pipelines** | 3 | feature, bugfix, security-patch |
| **Brain** | Obsidian vault | session logs, decisions, knowledge base |
| **Memory** | tasks/ | lessons.md, current.md, .research-cache.md, post-mortems/ |

### After Claude setup (/setup-project)
- Project initialized for your stack (TypeScript/Python/Go/Rust/etc.)
- Formatter, linter, test framework configured
- Scaffolding templates for your stack
- Reference implementation
- Documentation filled in
- Optional: Memory MCP, Telegram, Beads, Obsidian MCP

## Architecture

Based on AI-agent spec v3.1 + patterns from 20+ production repositories:

```
Three-tier context infrastructure:

Tier 1 (Hot Memory)     — CLAUDE.md + .claude/rules/router.md + tasks/lessons.md    (every session)
Tier 2 (Specialists)    — .claude/skills/ + .agents/skills/ + agents/        (on demand)
Tier 3 (Cold Memory)    — docs/ + brain/                                    (by request)
```

### Key Principles
- **Sinks, not Pipes** — components complete work, no cascading side effects
- **Working Memory Cliff** — files < 250 lines, tasks < 30 changes
- **Self-Improvement Loop** — every mistake → lessons.md → promote to rules
- **Autonomous Work** — /sprint with Ralph Loop + circuit breaker
- **Change Review > Code Review** — review intent and impact, not style

## Commands

| Command | What it does |
|---------|-------------|
| `/setup-project` | Configure project for your stack |
| `/implement` | Plan → Annotate → Implement (Boris Tane workflow) |
| `/commit` | Smart commit with 11-point deploy check |
| `/review` | Change review via isolated reviewer agent |
| `/refactor` | Safe refactoring via git worktree |
| `/sprint` | Autonomous work loop with circuit breaker |
| `/brain-sync` | Sync knowledge to Obsidian vault |
| `/weekly` | Retrospective + self-improvement promotion |
| `/status` | Project health dashboard |
| `/rollback` | Safe git revert workflow |
| `/onboard` | New developer onboarding |
| `/update-template` | Sync project with newer template version |

## Obsidian Brain

The `brain/` directory is an Obsidian vault:

```
brain/
├── 00-inbox/       — raw notes from agents
├── 01-daily/       — session logs (auto-generated by hooks)
├── 02-projects/    — project context, goals, blockers
├── 03-knowledge/   — patterns, solutions, lessons learned
├── 04-decisions/   — architectural decision records
└── templates/      — note templates
```

## Optional Integrations

| Integration | What | Required? |
|-------------|------|-----------|
| **Engram** | Persistent memory (SQLite+FTS5, zero-dep Go binary) | **Yes** |
| **Telegram** | Remote control from phone | No |
| **Beads** | Git-native task tracker | No |
| **Obsidian MCP** | Direct vault access via MCP | No |
| **codebase-memory-mcp** | Parser-backed code graph, call paths, routes, and change impact | Auto for code projects |
| **Ten-tool code-intelligence workflow** | Pinned graph, memory, search, refactor, policy, handoff, boundary, and secret tools | `full` installs all; routers select the sequence per task |

Default setup: `bash scripts/bootstrap-mcp.sh --install --tool-profile=full`
Lower-disk opt-in: `bash scripts/bootstrap-mcp.sh --install --tool-profile=auto`
For Zed AI chat: add `--zed`.
Selection, evidence, and benchmark gates: [docs/CODE_INTELLIGENCE_TOOLCHAIN.md](docs/CODE_INTELLIGENCE_TOOLCHAIN.md).
See `integrations/*/README.md` for details.

## Upgrading from v2.x to v3.0

```bash
# 1. Preview changes
bash scripts/sync-template.sh /path/to/agent-project-template --dry-run

# 2. Apply (with conflict detection)
bash scripts/sync-template.sh /path/to/agent-project-template

# 3. Review any CONFLICT files (*.template-new)
# 4. Run validation
bash scripts/check-drift.sh
```

**New in v3.0**: sync now detects conflicts (files modified locally AND in template) instead of silently overwriting. See `*.template-new` files for template version, resolve manually.

## Changelog

| Version | Key Changes |
|---------|------------|
| **4.9.0** | Minor release: makes the ten-tool code-intelligence workflow installable and health-checked as one pinned stack; adds a parser-backed graph, bounded zero-index fallback, task routing, safe Codex MCP merge, trusted-project guidance, Codex 0.125.0 config smoke, and AgentOS ownership regression coverage |
| **4.8.0** | Minor release: adds an evidence-backed Change Strategy Gate that evaluates architecture fitness during reading, compares repair/replacement/retirement destinations and transitions, binds decisions to structured triggers, derives compatibility checks from protected contracts, blocks fake alternatives and malformed CLI input, and verifies the behavior through setup and sync payload tests |
| **4.7.0** | Minor release: adds purpose-first literary, marketing, informational, communication, and technical-writing workflows; Russian editorial profiles and provenance; authority-isolated reference routing; and a fail-closed external-tool truth contract that keeps paid providers such as Glavred explicitly not configured and not run until real adapter evidence exists |
| **4.6.2** | Patch release: adds Luna support roles, one-wave cost-aware fan-out, genuine child-trace validation, and a progressive JPEG planner with an explicit anti-falsification gate |
| **4.6.1** | Patch release: adds the canonical agent update protocol, exact-tag preview/apply verification, safe path hashing, dry-run regression coverage, and release workflow tag/commit binding |
| **4.6.0** | Minor release: adds role-specific GPT-5.6 Sol/Terra subagents, an `xhigh` reasoning ceiling, automatic beneficial fan-out, product/systems reviewers, and semantic suppression for reference diagrams and release pages |
| **4.5.3** | Patch release: adds the progressive layer replacement pipeline plus `PROGRESSIVE_STATUS` project-slice reporting, so superseded wrong iterations are retired and changed working docs cannot close out with stale status headers |
| **4.5.2** | Patch release: clarifies progressive JPEG as final-plan-gated object readiness, where a 1% object has the full planned shape and already performs its smallest honest production function |
| **4.5.1** | Patch release: makes progressive JPEG an implementation gate, so known final product capabilities use an end-state skeleton with 1% callable hooks/stubs/contracts instead of legacy harness proof |
| **4.5.0** | Minor release: adds semantic intent scoring to Codex routing so routes trigger from task meaning, not only exact keywords; includes marketer/GTM, Sun Tzu/stratagem, TRIZ, SOT, and systemic-error regression coverage |
| **4.4.2** | Patch release: makes Ilyakhov/progressive JPEG client-control behavior mandatory in hot planning, status, replan, closeout, and Codex skill paths, with validator coverage |
| **4.4.1** | Patch release: makes the GitHub README/release entrypoint agent-safe with latest-release lookup, pinned tag install/sync commands, and explicit `main` versus release-tag guidance |
| **4.4.0** | Minor release: adds the client-executor accountability contract, anti-sycophancy rules, no-fake-completion evidence gates, research note, Codex skill wiring, and route/validator regression coverage |
| **4.3.4** | Patch release: makes production-standard validation downstream-aware so generated projects do not need source-only `templates/project-starter/*` files while the template source still checks them |
| **4.3.3** | Patch release: includes design-policy test fixtures in template sync delivery so downstream `test-design-policy` works without manual fixture copying |
| **4.3.2** | Patch release: makes screen anatomy/root-frame contracts mandatory in design rules and skills, adds regression coverage for those contracts, and makes `sync-template.sh --from-git --dry-run` show the real sync preview |
| **4.3.1** | Patch release: fixes README/CLAUDE documentation drift and adds regression coverage so release-facing counts and command lists match the shipped template surface |
| **4.3.0** | Minor release: design pipeline and skill upgrade with register-aware product/brand decision gates, detailed design command-mode reference, critique ordering that treats validators as evidence, and router regression coverage for conversion/KPI tasks |
| **4.2.0** | Minor release: production design QA upgrade with root `DESIGN.md` starter context, project-owned `design-policy.ignore`, design workflow command modes, hard design-policy validator, default Codex design-policy hook notifications, and browser/visual hardening gates |
| **4.1.1** | Patch release: GitHub workflows and CI templates now use Node 24-compatible actions and disable unnecessary setup-node package-manager cache in template validation/release jobs |
| **4.1.0** | Minor release: product-user experience and app-specific business outcomes now outrank technical perfection in plans, improvements, routing, skills, and validators |
| **4.0.3** | Patch release: fail-hard text/platform policy, no-mojibake gate, Windows-safe shell helpers, generalized UI Subtraction Gate, and faster sync regression smoke |
| **4.0.2** | Patch release: keeps source-only starter/bootstrap files out of generated-project sync and adds regression coverage for that boundary |
| **4.0.1** | Patch release: keeps the template release workflow source-only, ships validation workflow to downstream projects, and validates downstream-safe bootstrap/sync behavior |
| **4.0.0** | Production Product Standard, goal-like planning contract, design-system/product UX workflows, cross-project lesson promotion, router gates, and validators |
| **3.7.0** | Codex-native skills, subagent fan-out, validators, OpenAI model guidance, and setup/sync delivery for the Codex execution layer |
| **3.6.0** | Production-ready bootstrap contract, tracked-only payload, living PROJECT_SPEC/tool registry, AgentOS compatibility, and release hardening |
| **3.5.0** | Dual-agent support for Claude Code + Codex, Codex project config, validation and recovery hardening |
| **3.2.1** | MCP Context Router, depth=brief/normal/full, rules cache, Russian keywords, and the original estimated context-reduction claim; current releases require project-local measurement |
| 3.1.x | Dynamic task router, rules moved to .claude/library/, 7 mode commands, 6 runtime helpers |
| **3.0.0** | Merge-safe sync (conflict detection), cross-platform lib, 25 rules, 29 skills, 10 agents, audit-reuse system, design pipeline, validate-template.sh |
| 2.8.0 | Atomic reuse protocol, tool registry, design pipeline (domain-design.md) |
| 2.7.0 | Deep analysis rule, ecosystem map, research cache, session metrics, post-mortems |
| 2.5.0 | Agent routing, task queue, pipelines, graduated verification, circuit breaker |
| 2.4.0 | Template sync system, manifest-based hash verification |
| 2.0.0 | Initial release: 19 rules, 9 agents, 22 skills, hooks, brain vault |

## Sources

Built on research from 20+ repositories and papers:
- [christianestay/claude-code-base-project](https://github.com/christianestay/claude-code-base-project) — 4 agents, 12 skills, self-improvement
- [TheDecipherist/claude-code-mastery-starter-kit](https://github.com/TheDecipherist/claude-code-mastery-project-starter-kit) — 11 rules, MDD workflow
- [frankbria/ralph-claude-code](https://github.com/frankbria/ralph-claude-code) — autonomous work loop
- [damienlaine/agentic-sprint](https://github.com/damienlaine/agentic-sprint) — spec-driven sprints
- [Gentleman-Programming/engram](https://github.com/Gentleman-Programming/engram) — zero-dep memory
- [4 autonomous hooks](https://dev.to/yurukusa/4-hooks-that-let-claude-code-run-autonomously-with-zero-babysitting-1748)
- Ian Bull — Sinks Not Pipes, Working Memory Cliff, Planning Bottleneck
- Vasilopoulos — Codified Context Infrastructure (108K-line project)
- Steve Yegge — Beads (git-native agent memory)
- Anthropic official docs — skills, rules, hooks, memory
- ai-agent-spec-v3-final.md — foundation specification
