# Branching Execution Map

Дата: 2026-04-28
Статус: первый срез внедрен

## Цель

Сделать AgentOS понятным как диспетчерскую систему. Пользователь должен видеть не только чат, а ход работы: кто сейчас работает, в каком проекте, каким provider/model, какие tools идут, где нужен approve, где подагент вернул результат оркестратору.

## Принцип

Чат остается командным интерфейсом. Карта исполнения становится главным объяснением происходящего.

Главная ветка сверху: оркестратор.
Ниже: проектные ветки.
События идут слева направо по времени.
Делегации уходят из ветки оркестратора в проектную ветку.
Feedback возвращается merge-edge обратно в оркестратор.

## Этапы

1. Read-only projection.
   Построить execution map из существующих stream/delegation/session событий. Не менять runners. Не создавать новый source of truth.

2. UI map.
   Показать lanes, events, spawn/merge edges, waiting-for-user overlay, русский статус и compact details.

3. Live enrichment.
   Добавить более богатые события из Claude/Codex/tool/delegation runners: tool stdout/stderr, thinking summary, provider/model, active work.

4. Control integration.
   Связать событие карты с чатом, approve-flow, route card и project navigation.

5. Release hardening.
   Smoke tests: startup, map render, pending approval visible, timeline projection stable.

## Риски

- Слишком много данных на экране. Митигировать фильтрами: active, waiting, failed, current run.
- Неполные события в старых stream-файлах. Митигировать fallback lanes/events и synthetic feedback nodes.
- Дублирование старого timeline UI. Митигировать заменой плоской timeline-карточки на branching map, а не добавлением еще одной панели.
- Потеря контекста после reload. Митигировать read-only projection из persisted jsonl/delegations.
- Сырой chain-of-thought. Не показывать. Разрешены только безопасные thinking summary/status.

## Definition of Done для первого большого среза

- Backend command `get_execution_map` возвращает lanes/events/edges/waiting/counts.
- Frontend грузит `executionMap` вместе с timeline/orchestration map.
- Chat context drawer показывает горизонтальную карту с ветками.
- Pending approvals видны как отдельный overlay.
- Есть тест backend projection.
- `npm run check:ui` и `cargo test` проходят.
