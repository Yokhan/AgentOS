# Code Context в AgentOS

## Короткий вывод

Code Context не должен быть декоративной кнопкой. Это рабочий контракт между оркестратором, проектными агентами и backend-анализатором кода.

AgentOS строит ограниченный bundle: какие проекты затронуты, какой фокус задачи, какие файлы и зависимости важны, какие есть warnings, и как агенту запросить дополнительный срез.

## Основной формат

Schema: `agentos.code_context.v1`.

Команда для оркестратора:

```text
[CODE_CONTEXT:ProjectA,ProjectB]shared auth login flow[/CODE_CONTEXT]
```

Связанные команды:

```text
[GRAPH_IMPACT:Project:file]
[GRAPH_DEPENDENTS:Project:file]
[GRAPH_VERIFY:Project]
```

Если агенту мало контекста, он должен не гадать, а явно попросить:

```text
Нужен дополнительный контекст:
[CODE_CONTEXT:ProjectA,ProjectB]что именно нужно понять[/CODE_CONTEXT]
```

## Что входит в bundle

- список проектов;
- фокус задачи;
- hot spots по dependency graph;
- совпадения по тексту фокуса;
- циклы зависимостей;
- fallback-индекс файлов, если import graph пустой;
- manifest summary: `package.json`, `Cargo.toml`, `pyproject.toml`, `project.godot`;
- безопасные snippets запрошенных файлов;
- warnings и признак `truncated`;
- инструкции, как запросить больше контекста.

## Как это видно в UI

В чате есть кнопка `code context` для выбранного проекта.

После нажатия AgentOS строит backend bundle и показывает:

- context chip с типом `code`;
- проекты;
- schema;
- размер bundle;
- warnings;
- truncation;
- sample первых строк.

При отправке сообщения attached context уходит вместе с задачей в читаемом envelope:

```text
--- ATTACHED CONTEXT (kind=code; label=Project code context; schema=agentos.code_context.v1) ---
...
--- END ATTACHED CONTEXT ---

[USER_TASK]
...
```

Если backend bundle не построился, UI прикрепляет fallback-команду `[CODE_CONTEXT]`, чтобы оркестратор всё равно мог получить контекст через PA command path.

## Как проектный агент получает контекст

При approve делегации AgentOS автоматически добавляет task-scoped code context к задаче. Это защищает проектного агента от работы вслепую.

Если задача кросс-проектная, оркестратор должен сначала построить multi-project bundle и только потом делегировать части работы.

## Shared auth flow

Для общей аутентификации между проектами нормальный поток такой:

1. Оркестратор строит bundle:

```text
[CODE_CONTEXT:BackendProject,WebApp,MobileApp]shared auth, login, sessions, tokens[/CODE_CONTEXT]
```

2. Оркестратор определяет границы:

- где источник истины пользователя;
- где выдаются токены;
- какие клиенты зависят от auth API/SDK;
- какие файлы являются hot spots;
- какие проекты можно менять параллельно.

3. Только после этого задачи уходят проектным агентам.

## 3D/game-engine UI flow

Для game/UI задач нормальный поток такой:

```text
[CODE_CONTEXT:GameUiProject,AuthBackend]3D login UI, scene graph, input, auth overlay[/CODE_CONTEXT]
```

AgentOS уже видит Godot/GDScript-артефакты: `gd`, `tscn`, `tres`, shaders и `res://` ссылки. Для C#/Unity нужен отдельный parser, если появится активная production-задача.

## Ограничения

- Это статический анализ, не runtime tracer.
- Он не понимает все DI/reflection/dynamic imports/generated code.
- Cross-project связи пока задаются явным запросом нескольких проектов, а не выводятся полностью автоматически из workspace manifests.
- Контекст намеренно ограничен по размеру, чтобы не ломать делегации токенами.

## Следующий уровень

- manifest-aware cross-project graph links between projects;
- symbol-level context через tree-sitter/LSP;
- cache со stale markers вместо полного scan на каждый запрос;
- UI budgets: compact / standard / deep;
- per-agent context budgets для дешёвых и дорогих моделей.
