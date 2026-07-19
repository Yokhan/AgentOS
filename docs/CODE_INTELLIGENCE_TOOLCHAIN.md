<!-- PROGRESSIVE_STATUS
id: code-intelligence-toolchain-design
status: done
updated: 2026-07-19
readiness: 100
plan: 100
inventory: 100
production: 100
cleanup: 100
tags: tooling,context,code-graph,template
next: benchmark real projects before claiming measured savings or broad rollout
-->

# Десять инструментов как один рабочий процесс

Проверено 19 июля 2026 года. Версии и лицензии принадлежат
`_reference/code-intelligence-tools.json`.

## Решение

`context-router` не входит в десятку: это маршрутизатор процесса. Он и
`scripts/codex-route-task.js` возвращают одинаковую строку
`CODE_INTELLIGENCE` с последовательностью инструментов для текущей задачи.

Постоянно работают только два code-intelligence MCP:

- Engram хранит решения и контекст между сессиями;
- codebase-memory хранит один parser-backed граф кода.

Остальные восемь — короткоживущие CLI или LSP. Они не держат второй граф, не
добавляют постоянные MCP-схемы в контекст и не расходуют память в простое.

Engram не надо удалять. Граф отвечает на вопрос «как устроен код сейчас».
Engram отвечает на вопрос «что решили, почему и где остановились». Это разные
данные и разные сроки жизни.

## Состав

| № | Инструмент | Когда вызывается | Что даёт агенту | Цена по ресурсам |
|---:|---|---|---|---|
| 1 | ripgrep | точная строка, конфиг, лог, финальная проверка | дешёвое доказательство по исходнику | без индекса, без idle RAM |
| 2 | Engram | продолжение, решение, handoff | не перечитывать историю сессий | около 25 МБ idle на проверенной машине; одна SQLite-база |
| 3 | codebase-memory | архитектура, call path, impact, debug | единый граф вместо серии `grep/read` | единственный постоянный индекс; бюджет задаётся отдельно при rollout |
| 4 | Probe | графа нет, индекс устарел, нужен первый быстрый срез | bounded AST-контекст без индекса | ноль idle RAM; RC, поэтому только on-demand |
| 5 | Serena | rename, references, сигнатура, точный refactor | LSP-точность на уровне символа | язык-сервер запускается на задачу и затем останавливается |
| 6 | ast-grep | structural search, codemod, массовая миграция | проверяемый AST-поиск и preview rewrite | без постоянного процесса |
| 7 | Repomix | переносимый handoff или ограниченный context pack | воспроизводимый pack и token tree | файл создаётся только по запросу; не источник истины |
| 8 | dependency-cruiser | циклы и границы в JS/TS | исполняемые архитектурные правила | только JS/TS, без idle RAM |
| 9 | Semgrep CE | security policy, deprecated API, postcondition миграции | структурные нарушения вместо догадок LLM | on-demand; не граф |
| 10 | Gitleaks | перед handoff, commit, release | секреты в рабочем дереве и Git-истории | on-demand; бесплатный локальный бинарник |

## Как они работают вместе

Маршрут не означает «запустить всё». Он выбирает минимальную цепочку:

| Задача | Цепочка |
|---|---|
| Архитектура, debug, blast radius | `codebase-memory -> ripgrep` |
| Точный текст, лог, конфиг | `ripgrep -> Probe` только при неоднозначности |
| Нет или устарел индекс | `Probe -> ripgrep -> codebase-memory` только если работа продолжается |
| Rename/refactor символа | `codebase-memory -> Serena -> ripgrep -> native tests` |
| Массовая миграция | `codebase-memory -> ast-grep -> Semgrep -> ripgrep -> native tests` |
| JS/TS границы и циклы | `codebase-memory -> dependency-cruiser -> ripgrep` |
| Security/release | `codebase-memory -> Semgrep -> Gitleaks -> ripgrep` |
| Продолжение/handoff | `Engram -> codebase-memory -> Repomix`; перед передачей — Gitleaks |

Проверить решение маршрутизатора можно без запуска инструментов:

```powershell
node scripts/code-intelligence-tools.js route --task "rename auth symbol"
node scripts/codex-route-task.js "rename auth symbol" --summary
```

## Установка и проверка

По умолчанию `full` устанавливает все десять, но активирует только Engram и
codebase-memory как code-intelligence MCP. `context-router` остаётся третьим
постоянным MCP, но относится к инфраструктуре агента.

```powershell
node scripts/code-intelligence-tools.js install --profile full --dry-run
node scripts/code-intelligence-tools.js install --profile full
node scripts/code-intelligence-tools.js check --profile full
```

Полный bootstrap также устанавливает Engram и собирает process router:

```bash
bash scripts/bootstrap-mcp.sh --install --tool-profile=full
bash scripts/bootstrap-mcp.sh --check --tool-profile=full
```

Для Codex активен marker-bounded блок в `.codex/config.toml`. Bootstrap сливает
его из `_reference/codex-mcp-config.toml`, не трогая остальные настройки
проекта. После bootstrap проект нужно открыть и подтвердить как trusted, затем
перезапустить Codex и проверить:

```bash
codex mcp list
```

В списке должны быть `context-router`, `engram` и `codebase-memory-mcp`.
`.mcp.json` оставлен только для совместимости с Claude Code и не является
источником MCP для Codex.

`auto` остаётся явным режимом для машины с жёстким лимитом диска. Он не является
дефолтом принятого стека.

Реальный `full` health-check на Windows должен вернуть десять `ok`. Probe —
изолированный RC: npm предупреждает о его устаревших транзитивных `tar` и
`glob`, поэтому он запускается только на задачу и не получает доверия графа.
На 19 июля 2026 года стабильного drop-in с тем же контрактом не найдено:
локальный cross-platform поиск по смысловому/булеву запросу, цельные AST-блоки,
без индекса, embedding-модели и демона. Слот пересматривается только против
этого контракта, а не по факту появления очередного semantic-search проекта.

## Что это меняет по токенам, памяти и времени

- Токены: граф и bounded AST-поиск должны уменьшить чтение файлов целиком, но
  точное число неизвестно до одинакового benchmark на реальных проектах.
- Контекст MCP: в постоянной схеме нет Serena, Probe, Repomix, ast-grep,
  dependency-cruiser, Semgrep и Gitleaks.
- Память: в простое платим за Engram, один граф и process router. LSP и scanners
  живут только во время своей задачи.
- Диск: один граф на проект; никакого второго persistent index. Лимит 3 ГБ на
  проект и глобальный лимит кэшей относятся к будущему rollout, не к этому
  изменению шаблона.
- Деньги и данные: стек бесплатный и local-first; обязательных SaaS и загрузки
  исходников нет.

Нельзя обещать «минус 50% токенов» по vendor benchmark. Локальный promotion gate
сравнивает одинаковые вопросы и gold answers: recall не ниже 90%, затем токены,
tool calls, latency, index time и peak memory. До такого прогона эффект помечен
как ожидаемый, а не достигнутый.

## Почему не взяты альтернативы

| Кандидат | Решение | Причина |
|---|---|---|
| Codesight | убрать | был выключен и дублировал graph/Repomix; наличие в конфиге не создавало рабочий путь |
| GitNexus | не включать по умолчанию | сильный граф, но PolyForm Noncommercial не годится для безопасного коммерческого шаблона |
| Aider repo map | только как архитектурный референс | второй coding agent ради одной карты — лишняя установка и владелец процесса |
| Narsil/Forgemax | benchmark-кандидат | второй граф дублирует индекс и усложняет выбор источника истины |
| Sourcegraph/Zoekt/SCIP | не для per-project bootstrap | полезны как отдельная multi-repo платформа, но тяжелее текущего масштаба и контракта |
| CodeQL/Joern | отдельная security lane | мощнее для специализированного анализа, но тяжелее ежедневного агентского цикла |
| [grepai](https://github.com/yoanbernabeu/grepai) / [ck](https://github.com/BeaconBay/ck) | не заменяют Probe | стабильные semantic-search CLI, но требуют embedding provider/model, собственного индекса и watcher; рядом с codebase-memory это второй постоянный индекс |
| [codesearch](https://github.com/flupkede/codesearch) | не заменяет Probe | локальный hybrid search и AST chunks, но сначала индексирует репозиторий и дублирует постоянный retrieval слой |
| [ast-outline](https://github.com/aeroxy/ast-outline) | наблюдать | стабильный zero-index outline, но не ищет реализацию по смысловому запросу; graph и ast-grep уже закрывают его основную пользу |

Первый факт, который заставит пересмотреть стек: codebase-memory не проходит
качество или бюджет на репрезентативных проектах. Тогда меняется один граф, а не
добавляется второй параллельный индекс.
