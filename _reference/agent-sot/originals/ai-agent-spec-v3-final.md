# Спецификация: AI-агентная разработка с Claude Code

> **v3.1 · Март 2026 · Универсальный документ**
>
> Закинь в чат Claude Code с промптом:
> «Прочитай этот документ. Настрой рабочее пространство моего проекта по нему. Мой проект — [описание, стек]. Адаптируй всё под мой стек.»

> **Источники:** Ian Bull — "Sinks, Not Pipes" (Feb 2026), "Working Memory Cliff" (Dec 2025), "Planning Is the Bottleneck" (Jan 2026), "Code Reviews Are the Wrong Abstraction" (Jan 2026), "Beads" (Jan 2026) · Vasilopoulos "Codified Context Infrastructure" (arXiv:2602.20478, Feb 2026, 108K-line project, 283 sessions) · Zhang et al. "Agentic Context Engineering" (ICLR 2026) · Steve Yegge — Beads: git-native agent memory · Boris Cherny — создатель Claude Code (tips Jan–Mar 2026) · Boris Tane — annotated plan workflow · Martin Fowler / Birgitta Böckeler "Context Engineering for Coding Agents" · Vuong Ngo — scaffolding approach for monorepos · Nx "Monorepos & AI" · Anthropic official docs (skills, rules, hooks, memory) · HumanLayer research on CLAUDE.md · Trail of Bits claude-code-config · Community: shanraisshan/claude-code-best-practice, calv.info/agents-feb-2026

---

## 1. ФУНДАМЕНТ: ПОЧЕМУ СТАНДАРТНЫЙ КОД НЕ РАБОТАЕТ С АГЕНТАМИ

### 1.1. Проблема

AI-агент каждую сессию — это «новый разработчик без памяти». Не помнит прошлых решений, не знает неявных зависимостей, не видит архитектурных намерений. Кодовая база сама должна быть инструкцией.

Из Codified Context paper: одиночные manifest-файлы (CLAUDE.md, .cursorrules) работают до ~1000 строк кода. Дальше — нужна **многоуровневая контекстная инфраструктура**.

### 1.2. Шесть столпов agent-ready архитектуры

| # | Принцип | Суть | Источник |
|---|---------|------|----------|
| 1 | **Sinks, not Pipes** | Компонент принимает вход → делает работу → останавливается. Нет каскадов побочных эффектов. | Ian Bull |
| 2 | **Deep Modules** | Простой интерфейс, сложная реализация. index.ts = граница. | Ousterhout, Pocock |
| 3 | **Progressive Disclosure** | Знание в слоях: hot → specialist → cold. Загружается по необходимости. | Codified Context |
| 4 | **Data ≠ Behavior** | Конфигурации/таблицы отделены от логики обработки. | DoD |
| 5 | **Объективная верификация** | Каждое изменение: typecheck + lint + test + boundary-check. Агент получает feedback, не угадывает. | Общепринято |
| 6 | **Scaffolding > Instructions** | Шаблоны-генераторы надёжнее текстовых инструкций для обеспечения консистентности. | Vuong Ngo, Nx |

### 1.3. Три уровня контекстной инфраструктуры

Из Codified Context paper (108K-строчный проект, 283 сессии, solo-разработчик):

| Уровень | Что | Загрузка | Реализация в Claude Code |
|---------|-----|----------|--------------------------|
| **Tier 1: Hot Memory** | Конвенции, триггеры, протоколы | Каждая сессия | CLAUDE.md + .claude/rules/ |
| **Tier 2: Specialists** | Доменные эксперты с встроенным знанием | По задаче | .claude/agents/ + .claude/skills/ |
| **Tier 3: Cold Memory** | Спецификации, схемы, ADR | По запросу | docs/, README в поддиректориях |

**Критический вывод:** более 50% контента успешных агент-спецификаций — доменное знание проекта (факты, формулы, паттерны, известные ошибки), а не поведенческие инструкции. Не бойся длинных skill-файлов для специализированных агентов.

**Предупреждение о brevity bias** (Zhang et al., ICLR 2026): при итеративной оптимизации контексты стремятся схлопнуться в короткие generic промпты. CLAUDE.md должен быть коротким, но специализированные agents/skills — нет. Это разные уровни.

### 1.4. Working Memory Cliff — количественные пределы агента

Эксперимент Ian Bull (Dec 2025, GPT-5.1, подтверждено на других моделях):

| Элементов | Точность | Вывод |
|-----------|----------|-------|
| 10-20 | 100% | Безопасная зона |
| 30 | 90% | Первые ошибки |
| 40-70 | 20-60% | Зона подбрасывания монеты |
| 100+ | 0-10% | Почти полный отказ |

**«Cliff» наступает на 30-40 элементах.** Это прямо влияет на дизайн:

- **Файл < 250 строк** — не просто «хорошая практика», а предел рабочей памяти. Агент физически не удержит в голове файл на 500 строк.
- **Задача = 1 изменение.** Если задача требует трекинга >30 взаимосвязанных изменений, разбей на подзадачи.
- **Модуль экспортирует < 20 публичных функций.** Больше — агент начнёт путаться в интерфейсе.
- **Список DON'T < 10 пунктов.** Больше — деградация следования всем правилам.

Ключевой вывод Bull: «Модель должна описывать вычисление, а не выполнять его». Агент — это планировщик и оркестратор, а не процессор данных.

### 1.5. Метасдвиг: planning = bottleneck

Ian Bull, «Planning Is Becoming the Bottleneck» (Jan 2026): когда AI делает код дешёвым, **решения становятся узким горлышком**. Этот документ помогает агенту кодить правильно, но главная ответственность человека — решить ЧТО кодить и ЗАЧЕМ.

Человек в этой модели — это не программист, а:
- **Архитектор** — определяет структуру и границы модулей
- **Менеджер ставок** — решает, какие гипотезы проверять
- **Ревьюер изменений** — оценивает intent и impact, а не style и syntax (раздел 7.4)

---

## 2. АРХИТЕКТУРА ПРОЕКТА

### 2.1. Почему монорепо

Для агентов монорепо объективно лучше полирепо:
- Весь контекст (схема, API, реализация) доступен в одном месте
- Агент видит зависимости между модулями без переключения репозиториев
- Cross-cutting рефакторинг в одном PR
- Единый CLAUDE.md, единые правила, единый CI

Источники: Spectro Cloud ("surprising benefits when integrating AI tooling"), Puzzmo ("monorepo is perfect for working with an LLM"), Nx ("agents work faster with immediate feedback").

### 2.2. Структура директорий

```
project/
├── CLAUDE.md                         # Tier 1: < 200 строк
├── .claude/
│   ├── settings.json                 # Hooks, permissions
│   ├── settings.local.json           # Личное (gitignored)
│   ├── rules/                        # Tier 1: Модульные правила
│   │   ├── code-style.md
│   │   ├── testing.md
│   │   ├── git-workflow.md
│   │   └── architecture.md
│   ├── skills/                       # Tier 2: По запросу
│   │   ├── add-feature/SKILL.md
│   │   ├── modify-api/SKILL.md
│   │   └── debug/SKILL.md
│   ├── agents/                       # Tier 2: Субагенты
│   │   ├── reviewer.md
│   │   └── simplifier.md
│   └── commands/                     # Рутинные workflow
│       ├── implement.md
│       └── commit-push-pr.md
│
├── docs/                             # Tier 3: Cold memory
│   ├── ARCHITECTURE.md
│   ├── DATA_DESIGN.md
│   ├── API_CONTRACTS.md
│   └── DECISIONS.md                  # ADR
│
├── templates/                        # Scaffolding-шаблоны (раздел 5)
│   ├── feature/
│   ├── api-endpoint/
│   └── component/
│
├── _reference/                       # Эталонные реализации (раздел 4)
│   └── README.md                     # Какой модуль считать эталоном
│
├── src/ или packages/
│   ├── shared/                       # Типы, утилиты, валидаторы
│   ├── core/                         # Доменная логика (без IO)
│   └── features/                     # Vertical slices
│       ├── auth/
│       │   ├── index.ts              # Единственный публичный вход
│       │   ├── auth.service.ts
│       │   ├── auth.test.ts          # Тесты РЯДОМ с кодом
│       │   └── auth.types.ts
│       └── [feature-name]/
│
├── scripts/
│   └── check-drift.sh               # Drift detection (раздел 6)
└── tests/
    ├── integration/
    ├── e2e/
    └── architecture/                 # Тесты архитектурных границ
```

### 2.3. Vertical Slices

Каждая фича — автономный модуль со всеми слоями. Агент получает полный контекст одной фичи без загрузки всего проекта.

### 2.4. Правила границ

- **index.ts = граница модуля.** Импорт только через index.ts.
- **Однонаправленные зависимости:** shared ← core ← features ← adapters/UI.
- **Файл < 250 строк.** Больше — сигнал к разбиению.

### 2.5. Data-Oriented Design

```
data.ts   — конфигурации, таблицы (легко менять)
processor.ts — чистая функция (input, config) => output (стабильно)
types.ts  — контракты
```

Агент может крутить данные балансировки без риска сломать процессор.

---

## 3. CLAUDE CODE: НАСТРОЙКА КОНТЕКСТА

### 3.1. Типология контента (Martin Fowler)

Прежде чем писать CLAUDE.md, разберись, что ты пишешь:

| Тип | Определение | Пример | Где размещать |
|-----|-------------|--------|---------------|
| **Instructions** | Конкретные указания «сделай так» | «Создавай E2E тест так: ...» | Skills, Commands |
| **Guidance** | Общие принципы «всегда делай» | «Тесты должны быть независимы» | Rules, CLAUDE.md |
| **Context Interfaces** | Описание, как агент может получить больше контекста | «Для схемы БД читай docs/DATA_DESIGN.md» | CLAUDE.md |

### 3.2. Типология триггеров загрузки (Martin Fowler)

| Кто решает загружать | Механизм | Детерминированность | Когда использовать |
|---------------------|----------|---------------------|-------------------|
| **Agent software** | Hooks | 100% | Форматирование, lint, защита main |
| **Human** | Commands, ручной вызов /skill | Контролируемо | Реализация фичи, commit, PR |
| **LLM** | Skills (авто-matching) | Недетерминированно | Доменные знания, паттерны |

**Правило:** для критических вещей — hooks (детерминированно). Для удобства — commands (контролируемо). Для «было бы хорошо» — skills (по усмотрению LLM).

### 3.3. CLAUDE.md (Tier 1, Hot Memory)

**Ограничения из исследований:**
- Frontier LLM: ~150-200 инструкций с разумной консистентностью (HumanLayer)
- Система Claude Code уже ~50 инструкций
- Остаётся ~100-150 до деградации
- LLM смещены к инструкциям в начале и конце
- **Целевой размер: < 200 строк** (лучшие команды: 60-100)

**Ключевой принцип: для каждой строки спроси «Если убрать это, агент будет делать ошибки?» Нет — удали.**

**Не класть в CLAUDE.md:**
- То, что агент видит из кода (стандартные конвенции языка)
- Детальные инструкции (→ skills/commands)
- Стилистические правила (→ .claude/rules/ или линтер+hooks)
- Часто меняющуюся информацию
- Сниппеты кода (устареют быстро)

**Что класть:**
- Что это за проект (1 строка)
- Ключевые технологии стека
- Карта проекта (какая папка для чего)
- Команды сборки/тестирования
- Ссылки на documents в docs/ (progressive disclosure)
- Критические DON'T (только те, что агент реально нарушает)
- Инструкции по компактификации

**Шаблон:**
```markdown
# [Название проекта]
[Одна строка — что это]

## Стек
[Ключевые технологии]

## Карта
- `src/features/` — vertical slices, каждый с index.ts
- `src/shared/` — типы, утилиты, валидаторы
- `src/core/` — доменная логика (без IO)
- `docs/` — архитектура, схемы, ADR
- `templates/` — scaffolding-шаблоны для новых модулей

## Команды
- `[pm] test` / `[pm] lint` / `[pm] typecheck` / `[pm] dev`

## Правила
- Импорт ТОЛЬКО через index.ts. Нет прямых импортов из внутренних файлов.
- core/ = чистые функции, нет IO
- Данные (data.ts) отделены от логики (processor.ts)
- Тесты рядом с кодом: module.test.ts
- ВСЕГДА: typecheck + lint + тесты модуля после изменений

## Контекст по запросу
Перед работой определи, что релевантно задаче:
- docs/ARCHITECTURE.md — модули и зависимости
- docs/DATA_DESIGN.md — схема данных
- docs/API_CONTRACTS.md — API контракты
- docs/DECISIONS.md — архитектурные решения
- _reference/README.md — эталонные реализации

## Scaffolding
При создании нового модуля используй шаблон из templates/.
Запусти: `[pm] generate [тип] [имя]` — или читай templates/[тип]/

## DON'T
- Файлы > 250 строк — разбивай
- Нет `any` — используй `unknown` + type guards
- Нет мутаций — возвращай новые объекты

## Compaction
Сохраняй: текущую задачу, изменённые файлы, результаты тестов, найденные проблемы.
```

### 3.4. .claude/rules/ (Tier 1, модульные)

Загружаются автоматически с приоритетом CLAUDE.md. Правила с `paths:` — условно по файлам.

```markdown
# .claude/rules/architecture.md
Architectural boundaries enforced by dependency-cruiser:
- shared/ не зависит ни от кого
- core/ зависит только от shared/. Нет IO (fetch, DB, fs).
- features/ зависит от shared/ и core/
- Каждый features/X/ экспортирует только через index.ts
При нарушении — typecheck или dependency-cruiser поймает.
```

```markdown
# .claude/rules/testing.md
---
paths:
  - "**/*.test.ts"
  - "**/*.spec.ts"
---
- Тесты независимы друг от друга (нет shared state)
- Используй фабрики для тестовых данных, не хардкоженные объекты
- Для чистых функций: property-based тесты с фиксированным seed
- Для async: тестируй и success, и error paths
```

### 3.5. Skills (Tier 2, по запросу)

SKILL.md body < 500 строк. Тяжёлые ресурсы — в отдельных файлах рядом. **Для специализированных агентов допустимо >50% доменного знания** (не только инструкции).

### 3.6. Agents (Tier 2, субагенты)

Каждый агент — отдельный контекст, своя роль, свой набор инструментов. Используй для задач, которые не нужно видеть в основном контексте.

### 3.7. Hooks (детерминированные, 100% гарантия)

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{
          "type": "command",
          "command": "npx prettier --write $FILE_PATH 2>/dev/null || true",
          "timeout": 10
        }]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{
          "type": "command",
          "command": "[ \"$(git branch --show-current)\" != \"main\" ] || { echo '{\"block\":true,\"message\":\"Cannot edit main\"}' >&2; exit 2; }",
          "timeout": 5
        }]
      }
    ]
  }
}
```

---

## 4. REFERENCE IMPLEMENTATIONS (Эталонные реализации)

### 4.1. Зачем

Существующий код — самый мощный контекст для агента. Вместо описания паттернов текстом — покажи работающий пример. Агент скопирует структуру, именование, стиль.

### 4.2. Как оформить

Создай `_reference/README.md`:
```markdown
# Эталонные реализации

Эти модули — канонические примеры паттернов проекта. При создании нового модуля
используй их как образец для структуры, именования и стиля.

| Паттерн | Эталон | Описание |
|---------|--------|----------|
| Feature slice | src/features/auth/ | Полный вертикальный срез |
| Core processor | src/core/scoring/ | data.ts + processor.ts (DoD) |
| Shared validator | src/shared/validators/user.ts | Zod-схема + inferred type |
| Integration test | tests/integration/auth.test.ts | Паттерн с setup/teardown |
```

### 4.3. В skills

В каждом SKILL.md добавь ссылку на эталон:
```markdown
## Эталон
Перед реализацией прочитай `src/features/auth/` — это каноническая реализация паттерна.
Следуй той же структуре файлов, именованию и стилю.
```

---

## 5. SCAFFOLDING (Генераторы шаблонов)

### 5.1. Проблема, которую решает scaffolding

CLAUDE.md + rules описывают паттерны текстом. Агент всё равно генерирует «технически корректный, но стилистически чужеродный» код. Scaffolding даёт агенту готовую структуру для заполнения.

### 5.2. Реализация (минимальная, без внешних MCP)

```
templates/
├── feature/
│   ├── README.md            # Когда и как использовать
│   ├── {{name}}.service.ts  # Или просто example-файлы
│   ├── {{name}}.test.ts
│   ├── {{name}}.types.ts
│   └── index.ts
├── api-endpoint/
│   └── ...
└── component/
    └── ...
```

Можно реализовать через:
- **Простой скрипт:** `pnpm generate feature my-feature` копирует шаблон с подстановкой
- **plop/hygen:** файловые генераторы с промптами
- **Nx generators:** для Nx-монорепо
- **MCP-сервер (scaffold-mcp):** агент вызывает генератор как tool

### 5.3. В CLAUDE.md

```markdown
## Scaffolding
При создании нового модуля — СНАЧАЛА посмотри templates/[тип]/ и используй как каркас.
Не пиши бойлерплейт с нуля.
```

---

## 6. DRIFT DETECTION (Обнаружение расхождений)

### 6.1. Проблема

Из Codified Context paper: «устаревшие спеки привели к генерации кода, который конфликтовал с недавними рефакторами. Агент обратился к deprecated-путям, потому что спецификация не была обновлена.»

### 6.2. Решение

```bash
#!/bin/bash
# scripts/check-drift.sh — запускать перед крупными сессиями

echo "=== Drift Detection ==="

# 1. Проверить, что docs/ обновлены
for doc in docs/*.md; do
  age=$(( ($(date +%s) - $(stat -c %Y "$doc" 2>/dev/null || stat -f %m "$doc")) / 86400 ))
  if [ "$age" -gt 30 ]; then
    echo "⚠️  $doc не обновлялся $age дней"
  fi
done

# 2. Проверить, что index.ts каждого модуля существует
find src/features -mindepth 1 -maxdepth 1 -type d | while read dir; do
  [ -f "$dir/index.ts" ] || echo "❌ Нет index.ts: $dir"
done

# 3. Проверить архитектурные границы
npx dependency-cruiser src --output-type err 2>/dev/null || echo "⚠️  dependency-cruiser не установлен"

# 4. Проверить что CLAUDE.md < 200 строк
lines=$(wc -l < CLAUDE.md)
[ "$lines" -le 200 ] || echo "⚠️  CLAUDE.md: $lines строк (лимит 200)"

echo "=== Done ==="
```

---

## 7. WORKFLOW

### 7.1. Plan → Annotate → Implement (Boris Tane)

Самый надёжный workflow для нетривиальных задач. План — **физический файл**, не чат.

```
1. Человек: Ставит задачу
2. Агент: Пишет plan.md (Plan Mode)
3. Человек: Открывает plan.md в редакторе, добавляет inline-аннотации
   - "not optional" рядом с параметром
   - "эта retry-логика избыточна, consumer уже ретраит"
   - "здесь нужна Zod-валидация, не ручная проверка"
4. Агент: Обновляет план по аннотациям
5. Человек: Утверждает → "реализуй по плану, не отступая"
6. Агент: Имплементация фаза за фазой
7. Hooks: typecheck + lint + format (автоматически)
8. Человек: Ревью diff
```

**Почему это работает:** к моменту реализации все архитектурные решения уже приняты. Имплементация становится механической, не творческой. Это предотвращает сценарий, когда агент делает «разумное, но неверное допущение» на 3-й минуте и строит на нём 15 минут.

### 7.2. Управление контекстом

| Действие | Когда | Почему |
|----------|-------|--------|
| `/compact` | ~50% контекста | Не ждать авто. Добавь инструкцию что сохранять. |
| `/clear` | Смена задачи | Старый контекст активно мешает новой задаче |
| `/rewind` (Esc×2) | Агент пошёл не туда | Откат лучше починки в том же контексте |
| Handoff | Конец сессии | `bd sync` + `bd close` (Beads), или docs/current-task.md |
| Субагенты | Исследование, ревью | Изолированный контекст, не загрязняет основной |
| Worktrees | Параллельная работа | `git worktree add` — каждый агент в своём checkout |

### 7.3. Выбор режима

| Сложность | Режим | Когда |
|-----------|-------|-------|
| 1 файл, простое | Vanilla | Багфикс, мелкая правка |
| 3-10 файлов | Plan Mode → Implement | Новая фича, рефакторинг |
| Архитектурное решение | Ultrathink | Слово "ultrathink" в промпте |
| Исследование / ревью | Субагент | Не тратить основной контекст |
| Повторяющаяся задача | /loop | Мониторинг PR, сводки |

### 7.4. Change Review, не Code Review

Ian Bull, «Code Reviews Are the Wrong Abstraction» (Jan 2026): с AI-генерированным кодом ревью стиля и синтаксиса — пустая трата. Человек ревьюирует **изменение**, а не **код**.

**Code review** (автоматизировать): Читаемо? Идиоматично? Чисто написано? → Линтер + hooks.

**Change review** (человек): Что изменилось в поведении? Кого затрагивает? Какие failure modes? Как откатить? Как узнать что работает?

Когда можно НЕ ревьюировать: intent узкий, результат наблюдаем, blast radius понятен, легко откатить. Если все четыре — да, второй ревьюер не нужен.

Когда обязательно: security boundaries, billing, concurrency, public API, database migrations, performance-critical paths.

---

### 7.5. Beads — структурированная память между сессиями

#### Проблема с plan.md

Steve Yegge (создатель Beads): «Markdown plans bit-rot быстро. К дню два у тебя PLAN_v2.md, TASKS.md и PROJECT_PHASES.md, и все они друг другу противоречат. Агенты получают деменцию от конкурирующих документов.»

plan.md из раздела 7.1 работает для **одной сессии**. Для работы длиной в дни/недели нужна **структурированная, запрашиваемая память** — Beads.

#### Что такое Beads

Git-native issue tracker, спроектированный для AI-агентов (Steve Yegge, Jan 2026):
- **SQLite** (.beads/beads.db) — быстрые локальные запросы (gitignored)
- **JSONL** (.beads/issues.jsonl) — git-tracked, синхронизация через push/pull
- **`bd ready --json`** — возвращает неблокированные, приоритизированные задачи. Никакого парсинга markdown.

#### Workflow с Beads

**Начало сессии:**
```
bd ready                    # Что доступно?
bd show <id>                # Контекст задачи
bd update <id> --status in_progress
```

**Конец сессии ("land the plane"):**
```
bd close <id>               # Завершить задачу
bd sync                     # Экспорт + commit
git push
```

**Ключевой паттерн: issues как handoff-точки.** Когда ты убиваешь сессию и начинаешь новую, агент запускает `bd ready` и знает где подхватить.

#### Четыре типа зависимостей

| Тип | Назначение | Влияет на ready? |
|-----|-----------|-----------------|
| **blocks** | X не может начаться, пока Y не завершён | Да |
| **parent-child** | Задача принадлежит эпику | Да |
| **related** | Связаны, но не блокируют | Нет |
| **discovered-from** | Найдено при работе над другим | Нет (аудит) |

`discovered-from` — самый мощный тип. Когда агент чинит баг и замечает утечку памяти в другом сервисе — он может зафиксировать это как discovered-from, не теряя фокус.

#### Когда план, когда Beads

| Горизонт | Инструмент | Почему |
|----------|-----------|--------|
| Одна сессия | plan.md + аннотации (Boris Tane) | Быстро, не требует setup |
| Дни/неделя | Beads | Структурировано, запрашиваемо, переживает сессии |
| Квартал/бэклог | Jira/Linear/GitHub Issues | Beads не для долгого бэклога |

#### Setup

```bash
curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash
cd your-project
bd init
bd setup claude       # Установить hooks для Claude Code
```

В CLAUDE.md добавить:
```
## Inter-session memory
Используй `bd` для трекинга задач. `bd ready --json` — что делать. `bd close <id>` — завершить. Перед концом сессии: `bd sync`.
```

---

## 8. ВЕРИФИКАЦИЯ

### 8.1. Автоматические gates (обязательные)

```bash
typecheck      # Типы согласованы
lint           # Стиль и правила
test:unit      # Юнит-тесты модуля
test:arch      # Архитектурные границы (dependency-cruiser)
```

### 8.2. Архитектурные тесты

```javascript
// dependency-cruiser config
{
  forbidden: [
    { name: 'no-circular', from: {}, to: { circular: true } },
    { name: 'core-no-io',
      from: { path: '^src/core' },
      to: { path: '(pg|prisma|drizzle|fastify|express|fetch|axios)' } },
    { name: 'no-deep-imports',
      from: { pathNot: '^src/features/([^/]+)/' },
      to: { path: '^src/features/([^/]+)/(?!index\\.)' } },
    { name: 'shared-independent',
      from: { path: '^src/shared' },
      to: { path: '^src/(features|core)' } }
  ]
}
```

### 8.3. Тестирование самих инструкций

Из Anthropic skill authoring docs: создай skill с одним агентом (Claude A), протестируй с другим (Claude B на свежей сессии).

Процесс:
1. Напиши/обнови SKILL.md
2. Открой свежую сессию Claude Code
3. Дай задачу, которая должна триггернуть skill
4. Проверь: нашёл ли агент нужную информацию? Применил ли правила? Обработал ли edge case?
5. Если нет — уточни description в YAML frontmatter или реструктурируй контент

---

## 9. АНТИПАТТЕРНЫ И ЗАЩИТА

| Проблема | Симптом | Решение |
|----------|---------|---------|
| **Context overflow** | Забывает начало разговора | `/compact` на 50%. Субагенты. Маленькие задачи. |
| **Brevity bias** | Skills/agents становятся слишком generic | Встраивай доменное знание (>50% контента). Не только инструкции. |
| **God-object drift** | Файл растёт > 250 строк | Hook-предупреждение. Правило в rules/. |
| **Implicit coupling** | Изменение A ломает B | index.ts границы + dependency-cruiser |
| **Style inconsistency** | Каждый раз другой паттерн | Scaffolding-шаблоны + reference implementations |
| **Stale context** | Агент на deprecated-путях | Drift detection script. Даты в документах. |
| **Hallucinated APIs** | Агент выдумывает метод | Zod-схемы + typecheck как gate |
| **Architectural erosion** | Границы размываются | Architecture tests в CI |
| **Token waste** | Перегруженные rules/ | Каждое правило — trade-off. Path-scoping для условных правил. |

---

## 10. ИТЕРАТИВНОЕ УЛУЧШЕНИЕ

### Каждая ошибка агента → новое правило

```
Ошибка стилистическая  → .claude/rules/code-style.md
Ошибка архитектурная   → .claude/rules/architecture.md + dependency-cruiser rule
Ошибка доменная        → .claude/skills/ или docs/
Повторяющаяся ошибка   → hook (100% гарантия)
Ошибка консистентности → templates/ (scaffolding шаблон)
```

### Еженедельный цикл

1. `/insights` — что шло не так на этой неделе
2. Обнови CLAUDE.md, rules, skills по результатам
3. Запусти `scripts/check-drift.sh`
4. Удали устаревшие правила (они тоже вредят)

---

## 11. ЧЕКЛИСТ: НАСТРОЙКА НОВОГО ПРОЕКТА

### Фаза 1: Структура (10 мин)
- [ ] Создать директории по шаблону 2.2
- [ ] Настроить package manager workspaces (для монорепо)
- [ ] TypeScript strict mode
- [ ] Линтер + форматтер

### Фаза 2: Контекст для агента (15 мин)
- [ ] CLAUDE.md по шаблону 3.3
- [ ] .claude/rules/ — code-style, testing, architecture
- [ ] docs/ARCHITECTURE.md

### Фаза 3: Автоматика (10 мин)
- [ ] Hooks в .claude/settings.json (format, protect main)
- [ ] Permissions для безопасных команд
- [ ] Тестовый фреймворк + первый тест

### Фаза 4: Паттерны (15 мин)
- [ ] Reference implementation — один полный feature slice
- [ ] templates/ — хотя бы один scaffold-шаблон
- [ ] _reference/README.md

### Фаза 5: Верификация (10 мин)
- [ ] dependency-cruiser с правилами границ
- [ ] `typecheck + lint + test` проходят на пустом проекте
- [ ] scripts/check-drift.sh

### Фаза 6: Skills и commands (15 мин)
- [ ] skill: add-feature
- [ ] command: implement
- [ ] command: commit-push-pr

### Ongoing
- [ ] Каждая ошибка → правило/hook/template
- [ ] Еженедельно: /insights → обновить
- [ ] Тестировать инструкции на свежих сессиях
- [ ] Для многодневной работы: `bd init` + `bd setup claude` (Beads)

---

## 12. СВОДКА ПРИНЦИПОВ

1. **Архитектура = промпт.** Структура кода важнее любого CLAUDE.md.
2. **Sinks, not Pipes.** Компоненты завершают работу, не запускают каскады.
3. **Deep Modules.** Простой интерфейс, сложная реализация. index.ts = граница.
4. **Data ≠ Behavior.** Конфиги отделены от процессоров.
5. **Progressive Disclosure.** Hot → Specialist → Cold. Загружается по необходимости.
6. **Scaffolding > Instructions.** Шаблоны надёжнее текстовых описаний паттернов.
7. **Reference > Description.** Покажи работающий пример, а не описывай словами.
8. **Plan → Annotate → Implement.** Никогда не давай агенту кодить без утверждённого плана.
9. **Объективная верификация.** typecheck + lint + test + architecture-check после каждого изменения.
10. **Каждая ошибка = новое правило.** Система защиты растёт итеративно.
11. **Детерминированность по приоритету.** Hooks > Commands > Skills. Критичное — на hooks.
12. **Монорепо для агентов.** Весь контекст в одном месте = лучшие результаты.
13. **Working Memory Cliff.** Файл < 250 строк, задача < 30 связанных изменений, DON'T < 10 пунктов. Это не стиль — это когнитивные пределы модели.
14. **Structured memory > markdown plans.** Для многосессионной работы: Beads (queryable, git-native), не plan_v3.md.
15. **Change review > Code review.** Человек ревьюирует intent и impact. Стиль — на автоматику.
16. **Planning = bottleneck.** Агент кодит. Человек решает ЧТО кодить и ЗАЧЕМ. Это главная работа.
