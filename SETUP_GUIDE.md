# Как развернуть проект

> Версия: 4.9.0 | 2026-07-19
>
> При выпуске новой версии: перечитать этот файл, обновить устаревшие шаги,
> проверить все команды. Добавить в чеклист релиза.

---

## Что нужно

- **Codex Desktop или Codex CLI** — установлен и авторизован
- **Git** — установлен
- **Node.js 20+** — для metadata/MCP/helper-скриптов
- **Bash** — только для Linux/macOS и Unix release/maintenance окружения

---

## Быстрый старт (5 минут)

Ниже указан текущий stable-релиз `v4.9.0`. Перед rollout всё равно проверьте,
что GitHub `releases/latest` разрешается в этот tag.

```bash
git clone --branch v4.9.0 --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template
cd agent-project-template
bash setup.sh my-project
cd my-project
bash scripts/bootstrap-mcp.sh --install --tool-profile=full
codex
```

Windows:
```powershell
git clone --branch v4.9.0 --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template
cd agent-project-template
setup.bat
cd <generated-project>
codex
```

Откройте проект в Codex и подтвердите доверие к проекту. После bootstrap
перезапустите Codex и выполните `codex mcp list`. Должны быть видны
`context-router`, `engram` и `codebase-memory-mcp`.

---

## Optional: Spec Kit

The template ships a managed GitHub Spec Kit snapshot under `_reference/spec-kit/`.
It is not applied automatically during setup.

Validate the local snapshot:

```bash
node scripts/validate-spec-kit.js
```

Check whether the snapshot is stale against upstream stable tags:

```bash
bash scripts/sync-spec-kit.sh --check
```

Initialize Spec Kit in the current project using the pinned snapshot ref:

```bash
bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .
```

Use `--integration claude` for Claude Code or `--script ps` for PowerShell
helpers.

---

## Что делает каждый шаг

### 1. setup.sh / setup.bat

Скрипт создаёт **чистый дочерний проект** из шаблона:
1. Копирует только tracked project-facing payload (`.claude`, `.codex`, `scripts`, `docs`, `tasks`, `brain`, MCP tooling)
2. Накладывает clean starter-файлы для `tasks/current.md`, `tasks/.research-cache.md`, `tasks/lessons.md`
3. Не тащит maintainer-only артефакты (`n8n/`, `templates/`, временные фикстуры, локальные debug-файлы, локальные untracked-файлы)
4. Инициализирует git и `.template-manifest.json` для будущего `sync-template.sh`

### 2. bootstrap-mcp.sh --install

Скрипт делает четыре вещи:
1. **Находит** установленные MCP-серверы и определяет стек проекта.
2. **Устанавливает** зафиксированный профиль `core`, `auto` или `full` из `_reference/code-intelligence-tools.json`.
3. **Настраивает** process router отдельно, Engram для памяти решений и один
   parser-backed `codebase-memory-mcp` граф; остальные восемь инструментов
   остаются CLI/on-demand и не раздувают постоянный MCP-контекст.
4. **Безопасно сливает** управляемый MCP-блок в `.codex/config.toml`, сохраняя
   остальные настройки проекта. `.mcp.json` создаётся только для совместимости
   с Claude Code.

Флаги:
```
--install     Установить недостающие MCP-серверы и выбранный профиль инструментов
--check       Проверить MCP-серверы и выбранный профиль инструментов
--zed         Также настроить Zed AI chat panel
--dry-run     Показать, что изменится, не трогая файлы
```

### 3. /setup-project

11 фаз автоматической настройки:
1. Спрашивает стек, название, тип проекта
2. Создаёт структуру папок
3. Настраивает линтер, форматтер, тесты
4. Создаёт документацию, шаблоны, reference
5. Заполняет CLAUDE.md
6. Генерирует агентские оверлеи под стек

---

## MCP-серверы

MCP-серверы дают Codex дополнительные локальные возможности.

| Сервер | Зачем | Обязателен? |
|--------|-------|-------------|
| **Engram** | Память решений и handoff между сессиями | **Да** |
| codebase-memory-mcp | Parser-backed граф, call paths, routes и blast radius | Auto для кодовых проектов |
| Obsidian MCP | Прямой доступ к Obsidian-хранилищу (brain/) | Нет |
| Godot | Управление Godot-проектами | Нет |
| Figma | Работа с Figma | Нет |

### Engram — обязательный сервер

Engram хранит контекст между сессиями. Все функции памяти шаблона идут через него.

**Автоустановка** (рекомендуется):
```bash
bash scripts/bootstrap-mcp.sh --install --tool-profile=full
```

**Ручная установка:**
1. Скачайте бинарник: https://github.com/Gentleman-Programming/engram/releases
2. Положите в PATH (`~/.local/bin/`)
3. Запустите `bash scripts/bootstrap-mcp.sh`, чтобы обновить управляемый блок Codex.

**Что если Engram не установлен?**
Шаблон работает, но с ограничениями:
- Память пишется в файл `tasks/.memory-fallback.md` вместо БД
- Поиск по памяти работает через grep по файлам
- Когда Engram появится — записи импортируются из файла

---

## Codex, Claude Code и Zed AI Chat

Это разные клиенты. Конфиги MCP у них разные.

| Клиент | Конфиг MCP | Настройка |
|---|---|---|
| Codex Desktop/CLI/IDE | `.codex/config.toml` | `bootstrap-mcp.sh`; проект должен быть trusted |
| Claude Code | `.mcp.json` | compatibility payload из `bootstrap-mcp.sh` |
| Zed AI Chat | `settings.json` в папке Zed | `bootstrap-mcp.sh --zed` |

Codex не читает `.mcp.json`. Его источник — `.codex/config.toml`. Скрипт
`configure-codex-mcp.js` изменяет только управляемый блок и останавливается,
если находит конфликтующие неуправляемые таблицы MCP.

Можно использовать оба одновременно:
```bash
bash scripts/bootstrap-mcp.sh --install --tool-profile=full --zed
```

---

## Проверка здоровья

```bash
bash scripts/bootstrap-mcp.sh --check
```

Проверяет:
- Engram установлен и отвечает
- `.mcp.json` валидный
- Zed настроен (если в Zed)

Расширенная проверка шаблона:
```bash
bash scripts/check-drift.sh
```

10+ проверок: документы, размеры файлов, секреты, архитектура, шаблон, trust-hardening.

---

## Хуки — что происходит автоматически

| Когда | Что делает |
|-------|-----------|
| Начало сессии | Создаёт лог, показывает задачи и уроки, проверяет Engram |
| Конец сессии | Записывает статистику |
| Перед сжатием контекста | Сохраняет снимок состояния |
| После редактирования | Форматирует код, проверяет размер и синтаксис |
| Перед редактированием | Блокирует правку main-ветки и секретов |

Хуки настроены в `.claude/settings.json`. Проект-специфичные хуки — в `.claude/settings.local.json`.

---

## Основные команды

```
/setup-project     — настройка (один раз)
/implement         — план → код → тесты
/sprint            — автономная работа по списку
/review            — ревью изменений
/commit            — коммит
/status            — здоровье проекта
/hotfix            — быстрый фикс
/retrospective     — анализ за неделю
/update-template   — обновить шаблон
```

---

## Обновление шаблона

### Если агенту дали только ссылку на GitHub
Используйте единый протокол из `docs/TEMPLATE_RELEASES.md#canonical-agent-update-protocol`:

1. Определите тип рабочего каталога: исходный репозиторий шаблона, downstream-проект или старый проект без manifest. Шаблон нельзя синхронизировать в самого себя.
2. Прочитайте установленную версию из `.template-manifest.json`.
3. Явно указанная пользователем версия имеет приоритет. Иначе проверьте последний стабильный релиз: https://github.com/Yokhan/agent-project-template/releases/latest. Текущий stable tag: `v4.9.0`.
4. Проверьте `git remote get-url template` и не заменяйте конфликтующий remote без решения пользователя.
5. Запустите `--from-git --ref <tag> --dry-run`, затем примените тот же tag. Bare `--from-git` разрешён только для явно согласованного canary.
6. Если локальный sync-скрипт устарел или сломан, используйте скрипт из checkout целевого release tag с `--project-dir`.
7. До отчёта об успехе проверьте версию manifest, diff, сохранность `project-*`, конфликты `*.template-new` и downstream-тесты.

### Один проект
```bash
bash scripts/sync-template.sh /path/to/agent-project-template --dry-run
bash scripts/sync-template.sh /path/to/agent-project-template
```

### Из git-релиза шаблона
```bash
template_url="$(git remote get-url template 2>/dev/null || true)"
[ -n "$template_url" ] || git remote add template https://github.com/Yokhan/agent-project-template.git
[ -z "$template_url" ] || [ "$template_url" = "https://github.com/Yokhan/agent-project-template.git" ] || { echo "template remote conflict: $template_url"; exit 1; }
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run
bash scripts/sync-template.sh --from-git --ref v4.9.0
```

AgentOS может решать, какой проект и какой tag обновляет, но сам payload шаблона берётся из этого репозитория. Если AgentOS найден, Codex считает его orchestrator и не создаёт конкурирующий task graph.

`main` используйте только для разработки шаблона или явного canary-роллаута. Для нормального downstream-обновления нужен release tag.

### Все проекты
```bash
bash scripts/downstream-census.sh --no-sync --json ~/Documents
```

Сначала соберите список и dry-run для каждого проекта. Не запускайте массовый apply до просмотра всех preview.

### Если запускаете sync из template repo
```bash
bash /path/to/agent-project-template/scripts/sync-template.sh /path/to/agent-project-template --project-dir /path/to/my-project --dry-run
```

Обновляет: `.claude/`, `.agents/skills/`, `.codex/`, shipped `scripts/`, `README.md`, `SETUP_GUIDE.md` и именованные release-документы.
Сохраняет как project-owned: `CLAUDE.md`, `DESIGN.md`, `PROJECT_SPEC.md`, `ecosystem.md`, `brain/`, `tasks/`, `design-policy.ignore` и все `project-*` overlays.

`README.md` и `SETUP_GUIDE.md` в generated project остаются template-owned bootstrap docs. Project-specific onboarding и архитектурные детали храните в `CLAUDE.md`, `PROJECT_SPEC.md`, `ecosystem.md` и `docs/`.

---

## Структура после настройки

```
my-project/
├── .claude/           ← Агенты, правила, скиллы, хуки
├── .mcp.json          ← MCP-серверы (генерируется скриптом)
├── .codex/            ← Codex project hooks/config
├── brain/             ← Obsidian-хранилище
├── docs/              ← Архитектура, API, схема данных
├── mcp-servers/       ← Локальные MCP helper sources
├── scripts/           ← Автоматизация
├── tasks/             ← Текущая задача, очередь, уроки
├── AGENTS.md          ← Codex instructions
├── CLAUDE.md          ← Главный конфиг Claude
├── PROJECT_SPEC.md    ← Автозаполняемая project spec
├── ecosystem.md       ← Карта зависимостей проекта
└── SETUP_GUIDE.md     ← Этот файл
```

---

## Если что-то не работает

| Проблема | Решение |
|----------|---------|
| Codex не видит MCP | Доверьте проект, перезапустите Codex, затем `codex mcp list` и `bash scripts/bootstrap-mcp.sh --check` |
| Engram не найден | `bash scripts/bootstrap-mcp.sh --install --tool-profile=full` |
| Zed не видит серверы | `bash scripts/bootstrap-mcp.sh --install --tool-profile=full --zed` |
| Хуки не работают | `bash scripts/test-hooks.sh` |
| Форматирование не работает | Установите: `npm i -g prettier` / `pip install black` |
| «Template outdated» | `/update-template` |
| Файл > 375 строк | Разбейте. 375 — лимит рабочей памяти Claude |
| lessons.md > 50 записей | `/retrospective` — правила промоутятся |
| Codex MCP-блок устарел | `node scripts/configure-codex-mcp.js --check`, затем `bash scripts/bootstrap-mcp.sh` |
