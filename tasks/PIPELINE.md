# AgentOS: Strategy → Tactic → Plan → Todo Pipeline

## Иерархия

```
STRATEGY — цель в мире пользователя
│  "Поднять доход с 100к до 200к/мес"
│  deadline: 2026-06-01
│  metrics: user-reported или proxy (% выполненных тактик)
│  status: active | paused | achieved | abandoned
│
├── TACTIC — направление для достижения цели
│   │  "Запустить премиум подписки в фитнес-экосистеме"
│   │  category: "Fitness & Health" (опционально, из categories.json)
│   │  status: active | done | blocked
│   │
│   ├── PLAN — конкретный план для проекта
│   │   │  project: "YokhanFitnessAppMY"
│   │   │  priority: HIGH | MED | LOW
│   │   │  depends_on: [другие планы]
│   │   │
│   │   ├── TODO — атомарная задача
│   │   │   "Подключить Stripe API"
│   │   │   assignee: agent | user
│   │   │   → если agent: создаётся Delegation → Gate → Signal
│   │   │   → если user: ждёт ручного выполнения
│   │   │
│   │   └── TODO "Создать premium endpoints"
│   │
│   └── PLAN project: "YokhanGymBot"
│       └── TODO "Добавить paywall middleware"
│
└── TACTIC "Запустить NeiroMoney"
    │  category: "Business"
    │
    └── PLAN project: "NeiroMoney"
        ├── TODO "MVP landing" (assignee: agent)
        └── TODO "Настроить payment" (assignee: user — ручной ввод ключей)
```

## Ключевые принципы

### 1. Strategy = цель, не список задач
Стратегия описывает ЧТО хочет пользователь в реальном мире. Не "задеплой X" (это план), а "увеличь доход" или "подготовь релиз к пятнице". Стратегий может быть несколько, они могут конфликтовать — PA балансирует.

### 2. Tactic = направление для группы проектов
Тактика отвечает на вопрос "как именно двигаться к цели?" Привязывается к категории проектов (Fitness & Health, Giants Vale, Infrastructure) но не обязана — "найти клиента" кросс-категорийная.

### 3. Plan = что сделать в конкретном проекте
План всегда привязан к одному проекту. Содержит Todo-шки. Имеет приоритет и зависимости от других планов (например: "сначала обнови shared-api-types, потом деплой бота").

### 4. Todo = атомарная задача
Два типа:
- **agent** → при исполнении создаётся Delegation → L1/L2/L3 → Gate → Signal
- **user** → юзер делает сам, отмечает вручную

### 5. Plan без стратегии (ad-hoc)
Простые задачи ("задеплой бота") создают Plan напрямую, без Strategy/Tactic. Виртуальная стратегия "ad-hoc" группирует такие планы. Не перегружает пользователя уровнями когда задача простая.

## PA команды

### Создание всей иерархии одной командой:
```
[STRATEGY:Поднять доход до 200к/мес]
deadline: 2026-06-01

TACTIC Fitness & Health: Запустить премиум подписки
  YokhanFitnessAppMY: подключить Stripe API (agent)
  YokhanFitnessAppMY: создать premium endpoints (agent)
  YokhanGymBot: добавить paywall (agent)

TACTIC Business: Запустить NeiroMoney
  NeiroMoney: MVP landing page (agent)
  NeiroMoney: настроить оплату (user)
[/STRATEGY]
```

### Создание ad-hoc плана (без стратегии):
```
[PLAN:Deploy fitness bot]
YokhanFitnessAppMY: pull and deploy
YokhanGymBot: pull and deploy
[/PLAN]
```
→ Создаёт Plan напрямую, привязывает к виртуальной стратегии "ad-hoc tasks".

### Добавление todo к существующему плану:
```
[TODO:plan_id]task description (agent|user)[/TODO]
```

## Auto-Verify: скриптовый трек без LLM

### Проблема
LLM может забыть отметить Todo как done. Delegation завершилась, но Todo висит pending. Или наоборот — LLM сказал "готово", но реально ничего не изменилось.

### Решение: verify conditions на Todo
Каждый Todo может иметь автоматическое условие проверки. Скрипт (не LLM) проверяет условие и проставляет статус.

```rust
struct Todo {
    // ...existing fields...
    verify: Option<VerifyCondition>,
}

enum VerifyCondition {
    FileExists(String),           // "src/payments/stripe.rs" exists
    GrepMatch(String, String),    // (file_glob, content_regex) — grep "stripe" src/**/*.ts
    CommandExits(String, i32),    // ("npm test -- --grep paywall", 0) — exit code match
    GraphHasImport(String),       // graph scan finds import of "stripe" in project
    GitChanged(String),           // "src/payments/" has changes in git diff
}
```

### PA задаёт условия при создании:
```
YokhanFitnessAppMY: подключить Stripe (agent) [verify: grep "stripe" src/**/*.ts]
YokhanGymBot: paywall (agent) [verify: cmd "npm test -- --grep paywall"]
NeiroMoney: landing page (agent) [verify: file src/pages/landing.tsx]
```

### Три статуса Todo completion:
```
DONE            — delegation завершилась, verify condition нет → доверяем LLM
VERIFIED ✓      — delegation завершилась + verify condition подтвердил скриптом
DONE_UNVERIFIED ⚠ — delegation "готова", но verify condition НЕ подтвердился
                    (LLM сказал done, скрипт не согласен → Signal warn)
```

### Автоматическая проверка (Sensor):

**При завершении delegation (в Gate Pipeline):**
```
Gate runs → verify script → PASS
         → ALSO check Todo.verify condition
         → condition met? → Todo = VERIFIED
         → condition NOT met? → Todo = DONE_UNVERIFIED + Signal(warn)
```

**Periodic sensor (каждые 30s):**
```
For each Todo with verify condition AND status != VERIFIED:
  check condition against current state (file system, graph, git)
  if NOW met:
    → Todo = VERIFIED ✓
    → Signal(info, "auto-verified: {todo.task}")
```

Это значит:
- Юзер руками поправил файл → sensor подхватит → Todo verified
- Агент сделал не то → verify condition не проходит → warn signal → PA видит
- Никто ничего не делал, но условие стало true (merge from branch) → auto-verify

### Связь с Graph:
`GraphHasImport("stripe")` — sensor вызывает `build_project_graph()` и проверяет есть ли нода/edge с "stripe" в imports. Graph уже умеет парсить imports для JS/TS/Rust/Python/Godot.

### Что НЕ автоматизируется:
- User-assigned todos (assignee: user) — только ручная отметка
- Todos без verify condition — доверяем LLM/delegation result
- Бизнес-метрики стратегии ("доход 200к") — user-reported

## Lifecycle

```
Strategy: active → achieved (все тактики done) | paused | abandoned
Tactic:   active → done (все планы done) | blocked
Plan:     draft → approved → executing → done | failed
Todo (agent):  pending → approved → queued → running → verifying
               → verified ✓ (скрипт подтвердил)
               → done_unverified ⚠ (LLM сказал done, скрипт не подтвердил)
               → failed ✗
Todo (user):   pending → done (ручная отметка)

Todo (agent) execution:
  queued → Delegation created
    → L1 balanced → L2 permissive → L3 PA decision
    → Gate Pipeline: verify script + diff + cost + Todo.verify condition
    → Signal: pass → auto-queue next | warn → PA sees | fail → PA auto-trigger
    → Sensor (30s): periodic verify condition check → auto-verify if met

Tactic auto-complete: all plans done → tactic done
Strategy auto-complete: all tactics done → strategy achieved
Strategy proxy metric: % tactics done / deadline distance
```
                                              (pass/warn/fail → PA feedback)
```

## PA контекст

PA видит в каждом сообщении:
```
[STRATEGIES]
Strategy: "Поднять доход до 200к" (active, 2/3 tactics done, deadline 2026-06-01)
  ⚠ CONFLICT: also active "Снизить нагрузку на команду" — может противоречить

  Tactic "Премиум подписки" (Fitness & Health) — 5/8 todos done
    Plan YokhanFitnessAppMY (3/4 done, HIGH) gate: ✓ ✓ ✓ ⚠
    Plan YokhanGymBot (2/4 done, MED) gate: ✓ ✗ (verify failed)

  Tactic "Запустить NeiroMoney" (Business) — 0/3 todos done
    Plan NeiroMoney (0/3, LOW) — not started

[SIGNALS]
  🔴 [gate] YokhanGymBot: Verify FAILED: test_api_connection
  🟡 [cost] Hourly spend $3.20 approaching $5.00 budget
[END STRATEGIES]
```

## Что меняется в коде

### Новые structs (strategy_models.rs):
```rust
struct Strategy {
    id, title, deadline: Option<String>,
    metrics: Option<String>,  // user-reported description
    tactics: Vec<Tactic>,
    status: StrategyStatus,  // Active, Paused, Achieved, Abandoned
}

struct Tactic {
    id, title,
    category: Option<String>,  // links to categories.json
    plans: Vec<Plan>,
    status: TacticStatus,  // Active, Done, Blocked
}

struct Plan {
    // existing: project, steps(→todos), priority, depends_on
    // rename steps → todos
}

struct Todo {
    // existing Step fields + assignee: Assignee
    id, task, status, response, depends_on, delegation_id,
    assignee: Assignee,  // Agent | User
}

enum Assignee { Agent, User }
```

### PA prompt builder (chat_parse.rs):
- Заменить [STRATEGIES] секцию на иерархическое дерево
- Добавить conflict detection (2+ active strategies)
- Добавить gate results inline с каждым todo

### PA command parser (pa_commands.rs):
- Парсить вложенный [STRATEGY:...TACTIC...PLAN...[/STRATEGY]]
- Парсить (agent) / (user) assignee в todo-шках
- [PLAN:...] остаётся для ad-hoc планов

### Frontend (Strategy View):
- 4-level tree: Strategy → Tactic → Plan → Todo
- Collapsible уровни
- Gate badges на todo-шках
- User todos: ручной чекбокс (не delegation)

## Миграция с текущего кода
1. Текущий Strategy.plans → станет Tactic{plans} с auto-generated тактикой
2. Текущий Step → переименуется в Todo, добавляется assignee: Agent (default)
3. Текущий [PLAN:...] → ad-hoc стратегия с одной тактикой
4. Backward compat: загрузка старых .strategies.json без tactics → обёртка в тактику "main"
