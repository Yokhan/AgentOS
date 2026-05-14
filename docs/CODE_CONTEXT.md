# Code Context в AgentOS

## Короткий вывод

Кодовый контекст был не декоративным: AgentOS уже умел строить граф файлов и импортов, отдавать его через PA-команды и добавлять компактный граф в делегации.

Но этого было недостаточно для прод-уровня. Система давала "граф одного проекта", а не рабочий контракт для агента: что за задача, какие проекты связаны, какие файлы важны, что делать если контекста мало, и как запросить дополнительный срез у оркестратора.

## Что теперь является контрактом

Основной формат: `agentos.code_context.v1`.

Он возвращает bounded context bundle:

- список проектов;
- фокус задачи;
- hot spots по графу зависимостей;
- совпадения по фокусу задачи;
- циклы зависимостей;
- fallback-индекс файлов, если import-граф пустой;
- безопасные snippets запрошенных файлов;
- инструкции, как агенту запросить больше контекста.

## Как оркестратор вызывает контекст

PA-команда:

```text
[CODE_CONTEXT:ProjectA,ProjectB]shared auth login flow[/CODE_CONTEXT]
```

Для точечного анализа:

```text
[GRAPH_IMPACT:Project:file]
[GRAPH_DEPENDENTS:Project:file]
[GRAPH_VERIFY:Project]
```

## Как проектный агент получает контекст

При approve делегации AgentOS теперь добавляет к задаче не только category context, а task-scoped code context bundle.

Это значит: агент в проекте получает компактный архитектурный срез без ручного запроса пользователя.

Если этого мало, агент должен не гадать, а явно попросить оркестратор:

```text
Нужен дополнительный контекст:
[CODE_CONTEXT:ProjectA,ProjectB]что именно нужно понять[/CODE_CONTEXT]
```

## HTTP API для внешних агентов

Endpoint:

```http
POST /api/code-context
Authorization: Bearer <AgentOS API token>
Content-Type: application/json
```

Body:

```json
{
  "projects": ["ProjectA", "ProjectB"],
  "focus": "shared authentication and login UI",
  "files": ["src/auth.ts", "src/login.tsx"],
  "include_files": true,
  "max_chars": 18000
}
```

Ответ:

```json
{
  "status": "ok",
  "schema": "agentos.code_context.v1",
  "projects": ["ProjectA", "ProjectB"],
  "truncated": false,
  "warnings": [],
  "context": "..."
}
```

## Что сканер понимает сейчас

Языки и файлы:

- Rust: `rs`;
- TypeScript/JavaScript: `ts`, `tsx`, `js`, `jsx`;
- Python: `py`;
- Godot/GDScript: `gd`, `tscn`, `tres`, `gdshader`, `shader`;
- C# файлы видны в индексе как `cs`, но глубокое разрешение C# imports пока не считается production-grade.

Сканер исключает heavy/cache директории: `node_modules`, `target`, `dist`, `.git`, `.next`, `.venv`, `.godot`, `.cache`, `ia-memory` и похожие.

## Ограничения

- Это статический анализ. Он не понимает runtime DI, reflection, dynamic imports, generated code и часть alias-конфигов.
- Межпроектные связи пока задаются через явный запрос нескольких проектов, а не выводятся автоматически из package/workspace manifests.
- Для C#/Unity/Godot C# нужен отдельный parser, если появится активная задача на game-engine UI в C#.
- Контекст намеренно ограничен по размеру, чтобы не ломать делегации токенами.

## Что считать готовым для общей auth-системы

Для общей аутентификации между проектами нормальный flow такой:

1. Оркестратор строит cross-project bundle:

```text
[CODE_CONTEXT:BackendProject,WebApp,MobileApp]shared auth, login, sessions, tokens[/CODE_CONTEXT]
```

2. Оркестратор делает план границ:

- где источник истины пользователя;
- где выдаются токены;
- какие клиенты зависят от auth SDK/API;
- какие файлы являются hot spots;
- какие проекты можно менять параллельно.

3. После этого делегирует проектным агентам конкретные части. Каждый агент получает task-scoped bundle автоматически.

## Что считать готовым для 3D/game-engine UI

Для Godot-проектов контекст теперь видит `gd`, `tscn`, `tres`, shader-файлы и `res://` ссылки.

Production flow:

1. Запросить контекст по game/UI проекту:

```text
[CODE_CONTEXT:GameUiProject]3D UI, scene graph, input, auth overlay[/CODE_CONTEXT]
```

2. Если есть общий backend/auth проект, строить bundle сразу по двум проектам:

```text
[CODE_CONTEXT:GameUiProject,AuthBackend]3D login UI + token exchange[/CODE_CONTEXT]
```

3. Делегировать только после того, как оркестратор определил точки интеграции и риски.

## Следующий уровень

Чтобы довести систему до полноценного code-intelligence слоя, нужны следующие шаги:

- manifest-aware cross-project graph: package.json, Cargo.toml, pyproject.toml, Godot project.godot;
- symbol-level context через tree-sitter/LSP;
- cache со stale markers вместо полного scan на каждый запрос;
- UI-панель "Code Context" с видимым bundle, warnings и кнопками attach/request;
- per-agent context budget: compact / standard / deep.
