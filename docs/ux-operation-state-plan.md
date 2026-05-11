# AgentOS UX Operation State Plan

Дата: 2026-05-11

Цель: сделать AgentOS понятным во время живой работы. Пользователь должен видеть, кто сейчас работает, что делает, где ждёт, какие дочерние агенты запущены, какие команды выполнены и что требует решения.

## Принципы

1. Не переписывать execution flow одним рывком.
2. Сначала добавить слой наблюдаемости рядом со старым `poll_stream`, `activeRun`, `delegation_stream`.
3. Не рисовать heartbeat как рабочие события.
4. Чат остаётся чатом, а диагностика и карта исполнения живут в отдельной рабочей зоне.
5. Если provider не отдаёт reasoning/tool deltas, UI честно показывает: процесс жив, output молчит, последний смысловой event был тогда-то.
6. Любая новая панель должна отвечать на три вопроса: что сейчас происходит, почему ждём, что делать пользователю.

## Текущие проблемы

1. Codex route почти не даёт semantic stream: backend видит subprocess и heartbeat, но не шаги работы модели.
2. `activeRun` собирается локально во frontend и теряется при reload, смене проекта или долгой операции.
3. Chat UI перегружен: route controls, live state, transcript status, execution map, PA command traces и сообщения конкурируют за одно место.
4. Делегации, PA commands, gate и provider heartbeat живут в разных потоках данных.
5. Execution map смешивает heartbeat/status samples и смысловые события.
6. Пользователь не видит, где именно операция зависла: provider, command parser, delegation approval, gate или UI polling.

## Целевая модель

Вводим `OperationState`: backend-owned snapshot текущей работы.

Минимальная структура:

```json
{
  "operation_id": "op-...",
  "parent_id": null,
  "root_id": "op-...",
  "actor": "orchestrator | project_agent | gate | agentos",
  "project": "_orchestrator",
  "provider": "codex",
  "model": "gpt-5.5",
  "mode": "act",
  "access": "write",
  "phase": "provider | command | delegation | gate | waiting | done | failed",
  "status": "running | waiting | needs_user | done | failed | cancelled",
  "current_action": "waiting for codex output",
  "current_tool": null,
  "last_semantic_event": "DELEGATE_STATUS finished",
  "last_semantic_ts": "2026-05-11T...",
  "heartbeat_ts": "2026-05-11T...",
  "blocked_by": null,
  "waiting_for": "provider_output | user_approval | gate | child_agent",
  "children": ["op-child-..."],
  "events": []
}
```

## Этапы

### Этап 0. Baseline

Зависимости: нет.

Действия:

1. Зафиксировать текущий dirty worktree.
2. Проверить, что app собирается до изменений.
3. Зафиксировать smoke-сценарии:
   - solo Codex chat;
   - orchestrator act/full;
   - PA command execution;
   - delegation approve;
   - gate result;
   - reload во время running operation;
   - cancel во время provider wait.

Готово, когда есть список команд проверки и понятный baseline.

### Этап 1. Backend OperationState

Зависимости: Этап 0.

Файлы:

- `src-tauri/src/commands/operation_state.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/main.rs` или место регистрации Tauri commands

Действия:

1. Добавить in-memory store операций.
2. Добавить append-only JSONL audit: `tasks/.operations.jsonl`.
3. Добавить команды:
   - `get_operation_snapshot()`;
   - `get_operation_events(limit)`;
   - `clear_terminal_operations()`.
4. Сделать операции ограниченными по размеру: последние 200 events, terminal cleanup.

Готово, когда frontend может получить snapshot даже без новых emit-точек.

### Этап 2. Emit Adapters

Зависимости: Этап 1.

Файлы:

- `src-tauri/src/commands/chat_stream.rs`
- `src-tauri/src/commands/pa_commands.rs`
- `src-tauri/src/commands/delegation.rs`
- `src-tauri/src/commands/delegation_stream.rs`
- `src-tauri/src/commands/gate.rs`
- `src-tauri/src/commands/process_manager.rs`

Действия:

1. Chat stream emits:
   - run started;
   - provider waiting;
   - provider heartbeat;
   - text output;
   - tool start/result;
   - run done/failed/cancelled.
2. PA loop emits:
   - command parsed;
   - command started;
   - command result;
   - malformed command warning;
   - auto-continue turn.
3. Delegation emits:
   - queued;
   - approved;
   - L1/L2/L3 started;
   - permission escalation;
   - gate started;
   - gate result;
   - delegation done/failed.
4. Heartbeat events update operation state but do not create timeline cards by default.

Готово, когда одна orchestrator задача показывает parent operation и дочерние delegation operations.

### Этап 3. Frontend Operation Store

Зависимости: Этап 1, частично Этап 2.

Файлы:

- `src-ui/store.js`
- `src-ui/api.js`
- `src-ui/run-state.js`

Действия:

1. Добавить `operationSnapshot` и `operationEvents`.
2. Добавить polling `loadOperationSnapshot()`.
3. Не удалять `activeRun`.
4. Сделать merge bridge: если operation snapshot пуст, UI использует `activeRun`.
5. При reload восстанавливать live state из backend snapshot.

Готово, когда reload не стирает понимание активной операции.

### Этап 4. Live Operation Bar

Зависимости: Этап 3.

Файлы:

- `src-ui/chat.js`
- `src-ui/styles.css` или текущий CSS-файл

Действия:

1. Добавить постоянную компактную панель:
   - actor/provider/model;
   - phase/status;
   - current action;
   - last semantic event age;
   - heartbeat age;
   - waiting for;
   - primary action: stop/details/approve if applicable.
2. Убрать дублирование с `LiveStatusStrip` только после smoke.
3. Состояния:
   - normal running;
   - provider silent but alive;
   - no heartbeat;
   - needs user;
   - failed;
   - done.

Готово, когда пользователь за 2 секунды понимает, что сейчас делает оркестратор.

### Этап 5. Execution Flow Cleanup

Зависимости: Этап 2, Этап 3.

Файлы:

- `src-tauri/src/commands/execution_map.rs`
- `src-ui/chat.js`
- `src-ui/views.js`

Действия:

1. Построить карту из semantic operation events.
2. Heartbeat агрегировать в lane status.
3. Горизонтальная карта:
   - строки = actor/project;
   - колонки = semantic event order;
   - связи = delegation/feedback/gate.
4. Если видна только ветка orchestrator, явно показывать причину: нет child events, child hidden by scope, delegation not started, waiting approval.
5. Вынести карту из chat body в центральную workspace view.

Готово, когда карта показывает не “provider работает provider работает”, а цепочку работы.

### Этап 6. UI Declutter

Зависимости: Этап 4, Этап 5.

Файлы:

- `src-ui/chat.js`
- `src-ui/views.js`
- CSS

Действия:

1. Правый чат: только переписка, live bar, composer.
2. Route details свернуть в одну строку над composer.
3. Raw outputs, diagnostics, command table и maps держать в drawer/details.
4. Левая навигация остаётся проектной, но не забирает центральный смысл.
5. Нижняя floating panel переключает центральные views: поток, фокус, проекты, планы, сигналы.

Готово, когда экран не выглядит как cockpit и не прячет главное действие.

### Этап 7. Tests And Release

Зависимости: Этапы 1-6.

Проверки:

1. Rust compile.
2. Frontend build.
3. Smoke на live chat.
4. Smoke на delegation approve.
5. Reload while running.
6. Cancel while running.
7. Strategy page не падает.
8. No mojibake in user-facing Russian strings.
9. Disk/cache guard не регрессит.

Release gate:

1. `git diff --check`
2. `npm run build`
3. `cargo check` или project build command
4. создать tag/release только после ручного запуска app.

## Rollback

1. `OperationState` не управляет execution, поэтому его можно отключить без потери старого behavior.
2. Frontend должен иметь fallback на `activeRun`.
3. UI components включать постепенно.
4. Старые stream buffers не удалять до отдельного cleanup-релиза.

## Первый рабочий инкремент

1. Создать `operation_state.rs`.
2. Зарегистрировать commands.
3. Добавить frontend store + polling.
4. Подключить chat_stream run start/heartbeat/done.
5. Показать Live Operation Bar в чате.

Ожидаемый пользовательский эффект после первого инкремента: во время Codex-запроса видно не пустоту, а живой статус: provider, model, pid/heartbeat, последняя смысловая активность и причина ожидания.

## Статус 2026-05-11

Сделано:

1. Добавлен backend `operation_state.rs`.
2. Зарегистрированы Tauri-команды `get_operation_snapshot`, `get_operation_events`, `clear_terminal_operations`.
3. `AppState` получил in-memory `operations`.
4. `chat_stream` пишет operation events для start, provider wait, heartbeat, model output, PA commands, auto-continue, done/cancel.
5. `delegation` пишет operation events для queued, started, L1, L2, L3 decision, gate started, done/failed.
6. Frontend получил `operationSnapshot`, `operationEvents`, polling и `OperationLiveBar`.
7. Старый `LiveStatusStrip` оставлен fallback: он скрывается, когда новый snapshot уже даёт данные.
8. `OperationLiveBar` получил раскрываемые детали последних смысловых событий без засорения основного чата.

Проверено:

1. `cargo check --manifest-path src-tauri/Cargo.toml` — ok.
2. `npm.cmd run check:ui` — ok.
3. `npm.cmd test` — 61 tests passed.

Следующий этап:

1. Перевести центральную execution map на semantic operation events.
2. Свернуть route diagnostics в drawer, чтобы chat оставался только перепиской.
3. Добавить детальный экран “почему ждём” по выбранной операции.
4. Добавить smoke-сценарий reload while running.

## Статус 2026-05-11, stream/output fix

Причина зависания на output:

1. Старый transport писал все события оркестратора в один файл `tasks/.stream-_orchestrator.jsonl`.
2. Старые background threads могли дописывать `text` или `done` от прошлого запуска уже после старта нового.
3. Frontend poll не знал `run_id`, поэтому мог принять чужой `done/output` за текущий запуск.
4. Установленный AgentOS 0.3.21 продолжал auto-run loop и спавнил новые `codex exec`, поэтому ручное убийство одного PID не останавливало источник.

Сделано:

1. `stream_chat` теперь создает `run_id` до stream buffer и пишет в файл вида `.stream-{chat}-{run_id}.jsonl`.
2. `stream_chat` возвращает `run_id` во frontend.
3. `poll_stream` принимает `run_id` и читает только per-run buffer.
4. `stop_chat` принимает `run_id` и пишет cancel/done marker в нужный per-run buffer.
5. Frontend ждет результат `stream_chat`, сохраняет `providerRunId`, poll-ит только свой `run_id` и игнорирует события с чужим `run_id`.
6. `text` и `done` events теперь тоже несут `run_id`.
7. Execution map теперь предпочитает live `OperationState` rows, если они есть, и падает обратно на старый timeline только как fallback.

Проверено:

1. `cargo check --manifest-path src-tauri/Cargo.toml` - ok.
2. `npm.cmd run check:ui` - ok.
3. `cargo test --manifest-path src-tauri/Cargo.toml` - 61 passed.
4. `npm.cmd test` - 61 passed.
5. `git diff --check` - ok, только CRLF warnings.

Остаточный риск:

1. Открытое установленное окно 0.3.21 может продолжать старый auto-run loop до перезапуска приложения.
2. После установки новой сборки нужно проверить реальный сценарий: отправка сообщения, stop, reload during run, повторный запуск без смешивания output.
