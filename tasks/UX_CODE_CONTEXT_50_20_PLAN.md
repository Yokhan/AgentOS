# AgentOS 50+20: контекст, прозрачность и доведение UX до рабочего уровня

Дата: 2026-05-14

## Короткий вывод

Предыдущий 50+20 не был полностью закрыт как продукт. По коду закрыта большая часть каркаса: единый чат вместо отдельного Duo-экрана, левая навигация, live status, карта исполнения, уведомления отдельно от чата, provider routing, режимы доступа, базовый code context. Но пользовательский результат всё ещё не дотягивает до цели: ты не всегда понимаешь, что агент делает, что он получил в контекст, где он ждёт, и какой проект реально активен.

Этот план не добавляет ещё один “слой кабины”. Он полирует существующую систему вокруг одной цели: AgentOS должен повышать твою эффективность как управляющего несколькими агентами, а не требовать постоянного ручного пинка.

## Аудит предыдущего 50+20

Закрыто:

1. Duo перестал быть отдельным основным экраном и стал режимом чата.
2. Проектная навигация перенесена влево.
3. Введены route state, route diagnostics и сохранение текущего проекта.
4. Добавлены режимы доступа и provider/model routing.
5. Добавлены live status, stop/copy/details и защита от потери частичного output.
6. PA command batches сворачиваются и меньше засоряют чат.
7. Добавлены execution map, notifications page и фильтрация шумных heartbeat/state событий.
8. Делегации получили больше lifecycle-видимости.
9. Code context появился на backend-уровне и автоматически добавляется в delegation task.
10. Release gates ловят часть UI-регрессий: mojibake, missing imports, click wiring, overflow, chat render.

Не закрыто до уровня “можно спокойно жить в продукте”:

1. Code context был почти невидим в UI: непонятно, что реально прикреплено и сколько контекста уйдёт агенту.
2. Чат всё ещё смешивает разговор, execution state, PA traces, pending decisions и diagnostics.
3. Execution map местами показывает техническое состояние вместо смысловых событий.
4. Оркестратор не всегда сам объясняет, почему ждёт или почему stopped.
5. Project/plan/task hierarchy видна фрагментарно, а не как единая “операционная линия”.
6. Provider/account state есть, но его надо лучше связать с делегациями и проектными агентами.
7. Система recovery после reload/cancel стала лучше, но ещё требует smoke-сценариев.
8. Архитектурные документы частично устарели или имеют проблемы с читаемостью.

## 20 верхнеуровневых улучшений

1. Видимый Code Context. Перед отправкой пользователь должен видеть bundle, проекты, размер, warnings и truncation.
2. Контекст как контракт. Любой агент должен понимать, как запросить больше контекста через `[CODE_CONTEXT]`, `[GRAPH_IMPACT]`, `[GRAPH_DEPENDENTS]`.
3. Project-context inspector. В чате нужен компактный блок “что уйдёт агенту”, а не скрытая магия.
4. Связь project -> plan -> task -> agent. Каждый route должен показывать не только проект, но и плановый уровень.
5. Отдельный слой operational state. Чат должен оставаться разговором, а технические события должны уходить в карту/уведомления.
6. Semantic execution map. Карта показывает смысловые события: start, delegate, tool, output, feedback, verify, blocked, done.
7. Heartbeat hygiene. Heartbeat не должен создавать отдельные узлы карты или визуальный шум.
8. Waiting-state contract. Если агент ждёт, UI должен показывать “чего именно ждём” и “что можно сделать”.
9. Auto-next clarity. Если система auto-continues, она должна объяснять лимит, условие остановки и следующий критерий.
10. Provider/account observability. Видно, какой provider/model/account исполняет текущий run и дочерние делегации.
11. Delegation lifecycle. Pending, running, needs_user, blocked, failed, verified должны иметь одинаковые смыслы в backend и UI.
12. Route ownership. В любой момент понятно, кто сейчас lead: orchestrator, project agent, reviewer или user.
13. Context budgets. Compact/standard/deep контекст должны быть видимыми режимами, а не скрытым max_chars.
14. Cross-project work mode. Для shared auth, 3D UI и общих библиотек нужен явный multi-project bundle.
15. UI performance budget. Карта, чат и список проектов не должны зависать из-за 100+ событий или длинного output.
16. Recovery-first UX. Reload/cancel/network drop не должны стирать видимые результаты и active run.
17. Settings as control center. Provider routing, disabled providers, account labels и delegation defaults должны быть понятны в settings.
18. Docs as operator manual. Документы должны объяснять сценарии: как управлять 30 проектами, как запускать агента, как получать code context.
19. Regression gates by behavior. Проверять не только syntax, но и наличие ключевых UX-контрактов.
20. Release readiness dashboard. Перед релизом видно, какие gates пройдены, что не проверено, где риск.

## 50 конкретных задач

1. Добавить frontend API `loadCodeContextBundle()` для Tauri и HTTP.
2. Добавить store-сигналы `codeContextBusy`, `codeContextError`, `codeContextPreview`.
3. Сделать кнопку `code context` в chat route header.
4. По кнопке строить реальный backend bundle, а не только вставлять raw command.
5. Показывать context chip с типом, label, размером и warnings.
6. Добавить `CodeContextInspector` под chips.
7. Показывать проекты, schema, focus, truncation и sample bundle.
8. Если backend bundle не построился, прикреплять fallback `[CODE_CONTEXT]` command и явно показывать ошибку.
9. Перед отправкой заворачивать attached context в читаемый envelope.
10. После отправки очищать только потреблённые context chips.
11. Composer preview должен показывать “N code bundle / X chars”, а не просто “context”.
12. Добавить UI gate `check-code-context-ui.mjs`.
13. Подключить gate в `npm run check:ui`.
14. Обновить документацию `docs/CODE_CONTEXT.md` нормальным русским текстом.
15. В документации описать shared auth flow.
16. В документации описать 3D/game-engine UI flow.
17. В документации явно назвать текущие ограничения статического анализатора.
18. Добавить плановый документ 50+20 с аудитом предыдущего плана.
19. Проверить, что `CODE_CONTEXT` остаётся read-only PA command.
20. Проверить Rust tests по code_context.
21. Добавить в execution map группировку одинаковых provider heartbeat.
22. Скрывать state-sample events из основной карты.
23. Показывать heartbeat как состояние lane, а не как node.
24. Сделать предупреждение “карта неполная” actionable: что сделать дальше.
25. Разделить waiting cards на user decision, permission, retry/archive.
26. Для needs_user показывать полный текст запроса в details/side panel.
27. Кнопки approve/reject должны быть видны рядом с конкретным pending item.
28. После approve карта должна сразу обновлять state без ручного refresh.
29. Если delegation failed, UI должен показывать retry/status/archive как разные действия.
30. Чат должен показывать active run stage: startup, provider, model_output, tool, PA command, waiting_output, done.
31. Если provider молчит, показывать не только таймер, но и последний известный backend event.
32. Если stream завис на output, сохранить partial output и показать recovery action.
33. Не auto-scroll чат, если пользователь читает выше.
34. Минимальная история чата должна позволять дойти до трёх последних пользовательских запросов.
35. Уведомления не должны попадать в semantic execution map.
36. Notifications page должна иметь фильтры source/severity/project.
37. Settings должны показывать active GPT account label, если backend может его определить.
38. Settings должны позволять отключить Claude без hidden fallback.
39. Delegation defaults должны явно показывать provider для project agents.
40. Orchestrator должен иметь natural-language command path для массового onboarding проектов.
41. Добавить helper script для project connect/onboarding wave.
42. Скрипт подключения проекта должен проверять git, template, manifest, agent files, health.
43. Скрипт должен отдавать готовый report для оркестратора.
44. Code context должен уметь multi-project mode из UI.
45. Multi-project context должен показывать budget и warnings до отправки.
46. Добавить behavior smoke: attach code context -> send -> context envelope appears.
47. Добавить behavior smoke: reload keeps selected project and current route.
48. Добавить behavior smoke: execution map does not render heartbeat as nodes.
49. Добавить release checklist для code context и execution visibility.
50. Перед релизом прогнать `check:ui`, Rust tests и updater build.

## Этапы внедрения

1. Сначала закрыть видимый Code Context: API, chips, inspector, send envelope, UI gate.
2. Затем привести docs к операторскому виду: что система умеет, что не умеет, как работать со shared auth и 3D UI.
3. Затем добить execution map semantics: heartbeat как состояние lane, events только смысловые.
4. Затем улучшить waiting/approval UX: запросы пользователя, approve/reject/retry/status без догадок.
5. Затем добавить onboarding/project-connect помощники для оркестратора.
6. В конце сборка, тесты, updater build, релиз.

## Риски

1. Если слишком много контекста вкладывать в каждый запрос, агенты станут дороже и медленнее. Нужны budgets.
2. Если UI будет строить bundle синхронно без состояния busy/error, пользователь снова не поймёт, сработала ли кнопка.
3. Если карта исполнения будет смешивать heartbeat и semantic events, она останется шумной.
4. Если provider routing останется скрытым, отключение Claude/Codex будет выглядеть как “магия”.
5. Если не покрывать это gates, старые регрессии вернутся при следующей UX-волне.

## Статус этого прохода

Выполняется этап 1: видимый Code Context и проверка закрытия прошлого 50+20.

## Единый backlog: перенос недоделок из прошлого плана

Этот раздел является рабочим источником статуса. В него перенесены незакрытые пункты прошлого `UX_POLISH_50_20_PLAN.md` и новые пункты текущего плана.

### Закрыто в `0.3.33`

1. [x] Видимый Code Context: frontend API, backend bundle, chips, inspector, warnings, truncation, sample.
2. [x] Attached context отправляется агенту в явном envelope.
3. [x] Fallback `[CODE_CONTEXT]` прикрепляется, если backend bundle не построился.
4. [x] Документация `docs/CODE_CONTEXT.md` переписана как операторский контракт.
5. [x] UI gate `check-code-context-ui.mjs` добавлен в `check:ui`.
6. [x] Аудит прошлого `50+20` зафиксирован в этом плане.

### Закрыто в текущем проходе

1. [x] После отправки сообщения очищается не только `contextAttachments`, но и видимое состояние `codeContextPreview/codeContextError`.
2. [x] `ExecutionMap` refresh запускается после approve/reject waiting item, чтобы карта не требовала ручного refresh.
3. [x] Waiting cards получили `details` с id/project/status/action/full task.
4. [x] Предупреждение “карта неполная” стало actionable: кнопки `разобрать` и `refresh`.
5. [x] Добавлен gate `check-execution-map-ux.mjs`.
6. [x] Code Context получил budget selector: compact / standard / deep.
7. [x] Code Context получил кросс-проектный scope через comma-separated project input.
8. [x] Retry/status/archive draft actions теперь тоже обновляют execution map.
9. [x] `check-code-context-ui.mjs` проверяет budget/scope wiring.
10. [x] Добавлен `smoke-code-context-send.mjs`: проверяет, что attached Code Context реально уходит в message payload и очищается после send.
11. [x] `check-code-context-ui.mjs` теперь требует очистку preview/error и `[USER_TASK]` envelope.
12. [x] `check-execution-map-ux.mjs` теперь фиксирует контракт: heartbeat/provider state скрывается из event nodes.
13. [x] Waiting/blocker item в execution map получил отдельную readable detail panel с действиями approve/reject/retry/status.
14. [x] Notification center получил фильтры severity/source/project и route/delegation/run context chips.
15. [x] Добавлен gate `check-notification-center-ui.mjs`.
16. [x] Execution map lane rail теперь явно показывает owner role: orchestrator / project-agent / reviewer / user / agent.
17. [x] Route header получил project/plan/task/stage chips из activeScope и orchestrationMap; клик по plan/task открывает Plans.
18. [x] Добавлен behavior smoke `smoke-execution-map-noise.mjs`: heartbeat/provider state/root/non-semantic events не становятся узлами карты.

### Осталось: высокий уровень

1. [ ] Chat separation: окончательно разделить разговор, PA traces, pending decisions и diagnostics.
2. [ ] Semantic execution map: показывать только смысловые события и не пускать notifications/status spam в карту.
3. [ ] Waiting-state contract: единый backend/UI контракт для `needs_user`, `permission`, `blocked`, `failed`, `retry/archive`.
4. [ ] Project/plan/task/agent hierarchy: в каждом route показать, к какому плану/этапу/задаче относится работа.
5. [ ] Provider/account observability: связать provider/model/account с текущим run и дочерними делегациями.
6. [x] Context budgets: добавить compact/standard/deep режимы для code context.
7. [x] Cross-project context UI: выбирать несколько проектов для shared auth / 3D UI / общей библиотеки.
8. [ ] Recovery smoke: проверить reload/cancel/network drop без потери active run и partial output.
9. [ ] Settings polish: сделать provider routing, Claude disable, Codex defaults и delegation provider очевидными.
10. [ ] Onboarding helpers: natural-language path и helper для массового подключения/проверки проектов.
11. [ ] Release readiness dashboard: явный список пройденных gates и непроверенных рисков перед релизом.

### Осталось: конкретные задачи

1. [x] Добавить context budget selector в чат: compact / standard / deep.
2. [x] Перед attach показывать ожидаемый budget и лимиты.
3. [x] Добавить multi-project context input из chat route panel.
4. [x] Добавить `[CODE_CONTEXT:ProjectA,ProjectB]` UI-flow без ручного набора.
5. [x] Добавить behavior smoke: attach code context -> send -> envelope appears.
6. [x] Добавить behavior smoke: reload keeps selected project and route.
7. [x] Добавить behavior smoke: execution map does not render heartbeat as nodes.
8. [x] Добавить details panel для выбранного waiting item вместо только inline details.
9. [x] После retry/status/archive обновлять execution map так же, как approve/reject.
10. [ ] Убрать остаточный PA/status noise из chat transcript там, где он ещё попадает.
11. [x] Добавить фильтры notifications по source/severity/project.
12. [x] Связать notification item с project/route/delegation, если есть id.
13. [x] В execution map показывать lane owner: orchestrator / project-agent / reviewer / user.
14. [x] В route header показывать plan/task link, если activeScope его знает.
15. [ ] В project rail показать active plan/task badge.
16. [ ] В settings вывести effective provider для orchestrator/delegation/reviewer одной таблицей.
17. [ ] В settings вывести GPT account label/snapshot, если backend смог определить.
18. [ ] Проверить и задокументировать отключение Claude без скрытого fallback.
19. [ ] Добавить helper `project connect/onboarding wave` для оркестратора.
20. [ ] Helper должен проверять git, manifest, template, agent files, health.
21. [ ] Helper должен возвращать отчёт, который оркестратор может вставить в план.
22. [ ] Добавить UI action “prepare onboarding wave” без ручного набора команд.
23. [ ] Добавить operator doc: как вести 30 проектов через AgentOS.
24. [ ] Добавить operator doc: как запускать cross-project shared auth работу.
25. [ ] Добавить release checklist для code context, execution map и waiting UX.
26. [ ] Перед следующим релизом прогнать `check:ui`, Rust tests, updater build.
