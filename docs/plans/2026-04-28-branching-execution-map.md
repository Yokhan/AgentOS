# Branching Execution Map

Дата: 2026-04-28
Статус: второй срез внедрения

## Цель

Сделать AgentOS понятным как диспетчерскую систему. Пользователь должен видеть не только чат, а ход работы: кто сейчас работает, в каком проекте, каким provider/model, какие команды или делегации идут, где нужен approve, где подагент вернул результат оркестратору.

## Принцип

Чат остается главным командным интерфейсом. Карта исполнения объясняет происходящее вокруг чата.

Главная ветка сверху: оркестратор.

Ниже: проектные ветки.

События идут слева направо по времени.

Делегации уходят из ветки оркестратора в проектную ветку.

Feedback возвращается merge-edge обратно в оркестратор.

## Этапы

1. Read-only projection.
   Построить execution map из существующих stream/delegation/session событий. Не менять runners и не создавать новый source of truth.

2. UI map.
   Показать lanes, events, spawn/merge edges, waiting-for-user overlay, русский статус и compact details.

3. Live enrichment.
   Добавить события из живого backend-state: running/verifying/pending делегации должны быть видны даже если stream-output пока молчит.

4. Async hardening.
   Исправить ложные stuck-состояния, heartbeat watchdog и повторный auto-trigger одних и тех же critical signals.

5. Control integration.
   Связать события карты с чатом, approve-flow, route card и project navigation.

6. Release hardening.
   Smoke tests: startup, map render, pending approval visible, live running delegation visible, timeline projection stable.

## Текущий срез

Сделано:

- Backend command `get_execution_map` возвращает lanes/events/edges/waiting/counts.
- Frontend грузит `executionMap` вместе с timeline/orchestration map.
- Chat context drawer показывает горизонтальную карту с ветками.
- Pending approvals видны отдельным overlay.
- Running/verifying делегации попадают в карту из live state даже без stream-output.
- `Delegation.started_at` отделяет время реального запуска от времени постановки в очередь.
- Stale sensor считает возраст running/verifying от `started_at`, а не от pending `ts`.
- Delegation stream heartbeat корректно выходит по sentinel `0` и больше не считает `now - 0`.
- Auto-trigger подтверждает обработанные critical signals append-only ack-записью и не крутит один и тот же сигнал по кругу.

## Риски

- Слишком много данных на экране. Митигировать фильтрами: active, waiting, failed, current run.
- Неполные события в старых stream-файлах. Митигировать fallback lanes/events и live delegation state.
- Дублирование старого timeline UI. Митигировать заменой плоской timeline-карточки на branching map, а не добавлением еще одной панели.
- Потеря контекста после reload. Митигировать read-only projection из persisted jsonl/delegations.
- Сырой chain-of-thought. Не показывать. Разрешены только безопасные thinking summary/status.

## Definition of Done для большого среза

- Backend projection покрыт тестами.
- UI check проходит.
- Rust tests проходят.
- Pending approval видно без поиска в чате.
- Running delegation видно без новых stream events.
- После reload карта восстанавливает состояние из persisted files.
