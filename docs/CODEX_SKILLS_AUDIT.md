# Codex Skills Audit And Upgrade Plan

Дата: 2026-05-19

## Цель

Сделать Codex-слой шаблона сопоставимым с Claude-слоем: не копировать `.claude/skills` один в один, а перенести релевантные workflow, pipeline и дизайн-инструкции в Codex-native формат skills.

## Краткий вывод

Codex сейчас получает только базу:

| Поверхность | Текущее состояние |
| --- | --- |
| `AGENTS.md` | Route-first Codex instructions with deterministic router, skills, subagents, and AgentOS boundary |
| `.codex/` | только `config.toml` и `hooks.json`, всего 918 bytes |
| `.agents/skills/` | 41 Codex-native repository skills, official repo-scoped skills path |
| `.claude/skills/` | 30 skills, 277,875 bytes |
| `.claude/pipelines/` | 3 pipeline: feature, bugfix, security-patch |
| `.claude/agents/` | 10 agent definitions плюс `PROTOCOL.md` |

## Implementation Status

This audit is now implemented as a first Codex-native skill layer:

- 41 repository-scoped Codex skills live under the official `.agents/skills/` path.
- The pack covers all Claude skill categories either as a direct Codex port or as a Codex-native replacement: feature work, route-first selection, pipelines, audit, debug, security, design review, design production, Figma, Mermaid boards, migrations, API work, coverage, health checks, memory, setup, integrations, sprint/task queue, template sync, skill maintenance, dependency updates, and OpenAI model guidance.
- v4 adds production product goal, design-system workflow, product UX audit, and cross-project lesson promotion skills.
- `docs/AGENT_PIPELINES.md` is the agent-neutral pipeline source for feature, bugfix, security patch, design, template maintenance, and release work.
- `scripts/codex-route-task.js` is the deterministic Codex route source for skills, subagents, pipeline, risk, shared rules, and orchestrator owner.
- `scripts/validate-codex-skills.js`, `scripts/test-codex-routing.js`, `scripts/test-template.sh`, `scripts/validate-template.sh`, and `scripts/check-drift.sh` now validate the Codex skill and routing surface.
- `setup.sh`, `setup.bat`, and `scripts/sync-template.sh` ship and sync `.agents/skills/**` while preserving `.agents/skills/project-*`.
- Official OpenAI docs checked on 2026-05-19 confirm that repo-scoped Codex skills belong in `.agents/skills`, not `.codex/skills`.

Оставшийся разрыв: Codex route choice is now explicit and testable, but runtime fan-out still depends on Codex multi-agent support and the parent agent choosing to spawn workers. AgentOS, when present, remains the orchestrator.

## Что нельзя переносить механически

1. Claude skills не являются готовыми Codex skills.
   - 7 файлов не имеют обязательного `name` в YAML frontmatter: `domain-business-review`, `domain-communication-review`, `domain-design-review`, `domain-health-review`, `domain-science-review`, `domain-software-review`, `strategic-review`.
   - Ни один Claude skill не содержит `agents/openai.yaml`.
   - Длинные domain skills лежат целиком в `SKILL.md`, хотя для Codex лучше держать короткий `SKILL.md` и уводить массивный материал в `references/`.

2. Claude agent model не совпадает с Codex execution model.
   - `agent-router`, `pipeline`, `sprint` и pipeline-файлы говорят про `Opus`, `Sonnet`, subagents и Claude Code команды.
   - Для Codex это надо переписать как workflow в текущем агенте: explicit phases, gates, handoff notes, verification, без model-routing обещаний.

3. Bootstrap/sync пока не гарантируют доставку новых Codex skills в существующие проекты.
   - Official Codex docs use `.agents/skills` for repo-scoped skills, not `.codex/skills`.
   - `setup.*` and `sync-template.sh` need explicit coverage for `.agents/skills/**`, иначе новые skills не попадут в старые проекты через template sync.

4. Локальный validator из `skill-creator` сейчас не запускается.
   - Команда `python ...quick_validate.py` падает: Python не установлен в окружении.
   - Для шаблона с обязательным Node 20+ лучше добавить Node-based skill validator или явно завести Python/uv prerequisite.

## Codex Skill Format Target

Целевой формат для template-owned skills:

```text
.agents/
  skills/
    codex-feature-workflow/
      SKILL.md
      agents/openai.yaml
      references/
    codex-design-workflow/
      SKILL.md
      agents/openai.yaml
      references/
```

Правила для каждого Codex skill:

- `SKILL.md` содержит только `name` и `description` в YAML frontmatter.
- `description` включает trigger conditions, потому что Codex читает его до загрузки тела skill.
- Тело `SKILL.md` держится коротким: workflow, обязательные gates, какие references читать.
- Длинные доказательные списки, domain catalogs и pipeline detail лежат в `references/`.
- `agents/openai.yaml` генерируется или поддерживается вместе с `SKILL.md`.
- Skill не должен хардкодить model, reasoning effort, approval или sandbox.

Official docs confirm repo-scoped Codex skills live under `.agents/skills`.

## Аудит всех Claude Skills

| Skill | Статус для Codex | Приоритет | Что делать |
| --- | --- | --- | --- |
| `add-feature` | Port | P1 | Перенести как `codex-feature-workflow`; заменить Claude references на общие `PROJECT_SPEC`, `tasks/current`, `_reference`, `.claude/library`; оставить vertical slice flow. |
| `agent-metrics` | Adapt | P3 | Завязать на Codex memory tools и fallback files; не блокирует стартовый Codex layer. |
| `agent-router` | Rewrite | P1 | Не портировать model routing. Сделать Codex routing guidance в `AGENTS.md` и/или `codex-task-router` skill: keywords -> rules/skills, без Opus/Sonnet. |
| `api-contract` | Port | P1 | Полезен для Codex. Разделить: короткий workflow в `SKILL.md`, detailed API/versioning/error patterns в `references/api-contracts.md`. |
| `audit` | Port | P0 | Базовый skill для текущего запроса и будущих review tasks. Должен читать `critical-thinking`, `research-first`, `self-verification`. |
| `coverage` | Port | P2 | Перенести после core skills. Длинные coverage strategy sections вынести в reference. |
| `daily-brief` | Defer | P3 | Зависит от session logs и daily memory. Нужен после стабилизации memory/logging у Codex. |
| `debug` | Port | P0 | Один из первых Codex skills: reproduce -> isolate -> fix -> verify -> lessons. Убрать Claude model language. |
| `decompose` | Port | P2 | Полезен для L/XL задач. Можно встроить в `codex-pipeline-workflow` или сделать отдельным. |
| `domain-business-review` | Adapt | P3 | Добавить `name`, сократить body, вынести evidence catalog в references. Не первый пакет. |
| `domain-communication-review` | Adapt | P3 | Добавить `name`, оставить как review skill для текста/маркетинга; не смешивать с writer rules. |
| `domain-design-review` | Port | P0 | Исправить frontmatter, разделить на review reference. Это review skill, но Codex также нужен отдельный production design skill. |
| `domain-health-review` | Defer | P3 | High-stakes domain. Переносить только после policy обновления: обязательная актуальная проверка источников и disclaimers. |
| `domain-science-review` | Adapt | P2 | Полезен как evidence/research review. Добавить `name`, вынести catalog. |
| `domain-software-review` | Port | P1 | Хороший Codex review skill. Добавить `name`, split references. |
| `health-check` | Port | P1 | Перенести с template validation commands: `check-drift`, `test-template`, `sync-agents`, `validate-template`. |
| `memory-router` | Port | P1 | Адаптировать под доступные Codex memory tools и fallback files. Должен быть коротким и конкретным. |
| `migrate` | Adapt | P2 | Оставить как high-risk DB workflow с обязательным планом, rollback и test gates. Нужны stack-specific references. |
| `modify-api` | Port | P1 | Маленький и полезный. Связать с `api-contract` и testing rules. |
| `pipeline` | Rewrite | P0 | Ключевой разрыв. Переписать в `codex-pipeline-workflow`: phases/gates без Claude subagent promises. |
| `security-audit` | Port | P0 | Перенести рано. Должен trigger на security/vulnerability/CVE/injection/XSS и требовать high-risk verification. |
| `self-update` | Adapt | P1 | Переписать через Codex `skill-creator`, `tasks/lessons.md`, research cache и template/project boundary. |
| `setup-integrations` | Adapt | P2 | Сохранить Engram/MCP setup, убрать Claude-only assumptions, добавить Codex/Zed notes. |
| `setup-project` | Rewrite | P0 | Нужна Codex-specific версия, которая обновляет `AGENTS.md`, Codex skills/install path, hooks и shared docs. |
| `setup-telegram` | Defer | P3 | Сейчас Claude Code oriented. Переносить только после проверки реального Codex remote-control flow. |
| `sprint` | Rewrite | P2 | Автономный loop возможен, но нужно переписать без Claude subagents и с Codex interruption/approval constraints. |
| `strategic-review` | Port | P1 | Добавить `name`, вынести 30 principles/sources в reference, оставить short OODA review workflow. |
| `task-queue` | Adapt | P2 | Сделать agent-agnostic queue protocol. Сейчас текст говорит `Claude Code agents`. |
| `test-rules` | Port | P2 | Перенести вместе с validator. Нужен для regression tests по rules/skills. |
| `update-deps` | Port | P1 | Маленький operational skill; добавить обязательное browsing/registry verification для актуальных advisory/package data. |

## Missing Codex Skills

Эти skills нужны Codex, но их нет в Claude как готовых units:

| Proposed skill | Why |
| --- | --- |
| `codex-design-workflow` | Production design/Figma/UI/CSS skill: 8-phase pipeline, token-first, component-first, screenshot validation, responsive checks. Сейчас есть только library rule и design review skill. |
| `codex-figma-workflow` | Thin Figma MCP execution skill: load figma-use guidance, search design system, use instances/tokens, validate screenshot. |
| `codex-pipeline-workflow` | Перенос `.claude/pipelines` в Codex execution model: current-agent phases, gates, handoff blocks, no model routing. |
| `codex-skill-maintenance` | Template-local wrapper around system `skill-creator`: where project skills live, how to validate, how to update sync/bootstrap/tests. |
| `codex-template-sync` | Operational skill for updating downstream projects without touching user-owned files. Useful because template sync is high-risk infrastructure. |

## Design Skill Plan

Design нужно поднять первым пакетом, потому что пользователь отдельно выделил дизайн.

### `codex-design-workflow` contents

Source material:

- `.claude/library/domain/domain-design-pipeline.md`
- `.claude/skills/domain-design-review/SKILL.md`
- `.claude/docs/domain-full/domain-design.md`
- Frontend guidance from `AGENTS.md` system prompt, only where it is generally reusable

Body should include:

1. Trigger: design/UI/Figma/CSS/frontend visual work, mockups, screens, components, game UI.
2. Required phases: context, analyze, reference, BOM, discover, compose, validate, iterate.
3. Hard gates: no raw visual values, no raw shapes when component exists, tokens -> components -> screens, layout mode/flex/grid everywhere, 8 states considered.
4. Validation: screenshot, responsive viewport checks, text overflow check, contrast/accessibility review.
5. When using Figma: call design-system discovery before creation; import instances before drawing; bind variables/styles; screenshot after structural changes.

References should include:

- `references/design-review.md`: evidence-based anti-patterns and practices from `domain-design-review`.
- `references/figma-mcp.md`: Figma MCP operational checklist.
- `references/frontend-ui.md`: code-side UI implementation checks for CSS/React/etc.

### `codex-figma-workflow` contents

Keep this separate from general design so non-Figma UI tasks do not load Figma-specific details.

Minimum gates:

- Load figma-use guidance before writes.
- Use `search_design_system` before creating components.
- Prefer existing components/styles/variables.
- Use `use_figma` for write operations; use `generate_figma_design` only for first-time web capture.
- Validate with `get_screenshot`.
- Register reusable tokens/components in `_reference/tool-registry.md`.

## Pipeline Port Plan

Recommended architecture:

```text
docs/
  AGENT_PIPELINES.md          # shared, agent-neutral pipeline definitions
.claude/
  pipelines/*.md              # Claude wrappers or compatibility pointers
.agents/
  skills/codex-pipeline-workflow/
    SKILL.md
    references/pipelines.md   # either generated from docs/AGENT_PIPELINES.md or points to it
```

Why not duplicate `.claude/pipelines` directly:

- Current pipeline definitions name Claude roles and models.
- Duplicating creates drift.
- A shared neutral pipeline lets Claude and Codex keep thin wrappers while using the same gates.

Minimum Codex pipelines:

| Pipeline | Codex phases |
| --- | --- |
| Feature | research -> brainstorm if risk warrants -> plan -> implement -> test -> review -> closeout |
| Bugfix | research -> reproduce -> diagnose -> fix -> regression test -> closeout |
| Security patch | research -> assess -> plan with user checkpoint -> minimal fix -> security verification -> tests -> closeout |
| Design | context -> analyze -> reference -> BOM -> discover -> compose -> screenshot validation -> iterate |

## Implementation Plan

### Phase 0 - Validate loading model

Goal: remove uncertainty before writing 20+ skills.

Tasks:

- Verify generated projects ship `.agents/skills`.
- Verify template sync delivers `.agents/skills/**` to existing projects.
- Keep project-specific skills under `.agents/skills/project-*`.

Exit criteria:

- One repeatable command or documented check proves generated projects contain template-owned Codex skills.

### Phase 1 - Infrastructure

Tasks:

- Add `.agents/skills/` to template-owned sync patterns in `sync-template.sh`.
- Add setup/bootstrap smoke asserting generated projects contain Codex skills.
- Add a Node-based `scripts/validate-codex-skills.*` because Python is not part of supported environment.
- Extend `scripts/sync-agents.sh` or `scripts/test-template.sh` to check:
  - every `.agents/skills/*/SKILL.md` has `name` and `description`;
  - no project-level model/effort/approval/sandbox defaults are introduced;
  - `AGENTS.md` stays under 32KB;
  - high-priority skills exist.

Exit criteria:

- Validation fails if a Codex skill is malformed or omitted from sync.

### Phase 2 - Core Codex pack

Create first 8 skills:

1. `codex-pipeline-workflow`
2. `codex-design-workflow`
3. `codex-figma-workflow`
4. `codex-audit`
5. `codex-debug`
6. `codex-security-audit`
7. `codex-setup-project`
8. `codex-feature-workflow`

Update `AGENTS.md` with:

- Skill discovery/trigger rules.
- Pipeline summary.
- Design-specific trigger note.
- Short instruction to prefer skills over re-reading long docs.

Exit criteria:

- A new Codex session can route implementation, audit, bugfix, security and design tasks without relying only on base `AGENTS.md`.

### Phase 3 - Developer quality pack

Create or port:

- `codex-api-contract`
- `codex-modify-api`
- `codex-coverage`
- `codex-health-check`
- `codex-memory-router`
- `codex-update-deps`
- `codex-decompose`
- `codex-self-update`
- `codex-strategic-review`
- `codex-domain-software-review`

Exit criteria:

- Codex can handle common software maintenance tasks with the same quality gates Claude has.

### Phase 4 - Domain review pack

Port after core behavior is stable:

- business
- communication
- science
- health

Rules:

- Fix missing `name`.
- Move long evidence catalogs to `references/`.
- Health remains high-stakes: require current-source verification before user-facing advice.

Exit criteria:

- Domain review skills validate and do not bloat default context.

### Phase 5 - Optional integrations and autonomy

Adapt or defer:

- `agent-metrics`
- `daily-brief`
- `setup-integrations`
- `task-queue`
- `sprint`
- `setup-telegram`

Exit criteria:

- Optional skills do not create false promises about Codex subagents, Telegram control or autonomous execution.

## Verification Plan

Run after implementation phases:

```bash
bash scripts/test-template.sh
bash scripts/sync-agents.sh
bash scripts/check-drift.sh
node scripts/test-codex-routing.js
node scripts/validate-codex-skills.js
node scripts/validate-codex-agents.js
```

Manual checks:

- Fresh generated project contains `.agents/skills`.
- Existing downstream project receives new Codex skills via `sync-template.sh --dry-run`.
- `AGENTS.md` stays below the 32KB limit.
- Template/release tasks route to `codex-template-sync`, `codex-skill-maintenance`, `codex-test-rules`, and `codex-health-check`.
- Design task routes to `codex-design-workflow`.
- Figma task routes to `codex-figma-workflow`.
- Mermaid board task routes to `codex-mermaid-board-workflow`.
- AgentOS fixture reports `orchestrator.owner = agentos`.
- No Codex project config hardcodes model, effort, approval or sandbox.

## Open Questions

1. Should `.agents/skills` be template-owned only, or should projects also support `project-*` Codex skills?
   - Recommendation: support both. Template skills are synced; `project-*` Codex skills are preserved.

2. Should pipeline definitions move out of `.claude/pipelines`?
   - Recommendation: yes, create shared `docs/AGENT_PIPELINES.md` and keep agent-specific wrappers thin.

3. Should Claude skills be fixed while porting?
   - Recommendation: yes for low-risk metadata defects such as missing `name`; do not rewrite content until Codex core pack is stable.

4. Should the template require Python because system `skill-creator` scripts use Python?
   - Recommendation: no. Supported environment already requires Node 20+; write a Node validator for template checks and leave Python only for optional system skill tooling.

## Remaining Work

The implementation phases above are complete for the template baseline. Remaining work is operational:

1. Run the first remote GitHub Actions validation after pushing the branch.
2. Use one real downstream repository to confirm `.agents/skills/**` delivery via `sync-template.sh --dry-run`.
3. Promote any project-specific Codex patterns into `.agents/skills/project-*` rather than editing template-owned skills directly.
