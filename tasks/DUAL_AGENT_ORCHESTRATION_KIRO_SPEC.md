# AgentOS Dual-Agent Orchestration with Kiro-Inspired Pipeline

## Статус

Draft v1

## Зачем этот документ

Нужна система, в которой:

- пользователь видит одновременно `Claude` и `Codex`;
- агенты могут спорить, критиковать друг друга и работать параллельно;
- запись в файлы контролируется системой, а не "на честном слове";
- работа идет через артефакты и гейты, а не только через чат;
- состояние файлов, текущая работа и ownership видны в UI и в runtime state.

Этот документ проектирует такую систему поверх текущей архитектуры AgentOS.

## Что реально есть в Kiro, а что нет

По официальным материалам Kiro подтверждаются следующие паттерны:

- `Specs` как структура из `requirements.md` или `bugfix.md`, `design.md`, `tasks.md`.
- Явный трехфазный workflow: требования/анализ -> дизайн -> задачи.
- Исполнение задач со статусами и обновлением прогресса.
- `Steering` как постоянный контекст проекта.
- `Hooks` на события IDE/agent lifecycle/tool usage/spec task execution.
- `Delegate` / background task lifecycle для автономных задач.

По публичной документации Kiro не подтверждается как штатная функция:

- видимая multi-agent дискуссия двух LLM в одной сессии;
- файловые блокировки или lease manager;
- arbitration UI, где пользователь выбирает победителя спора;
- параллельная запись нескольких агентов в один workspace с управлением конфликтами.

Вывод: берем у Kiro артефактный pipeline, steering, hooks, task-state thinking и lifecycle background work. Файловые lease, arbitration и dual-visible-agent orchestration проектируем как собственное расширение AgentOS.

## Контекст текущего AgentOS

Сейчас у AgentOS уже есть хорошая база:

- pipeline `Strategy -> Tactic -> Plan -> Todo -> Delegation -> Gate -> Signal -> PA feedback`;
- coarse per-directory lock;
- stream chat;
- delegation execution;
- gate pipeline;
- graph scan;
- file-backed runtime state в `tasks/`.

Это означает, что проект нужно строить не как "новый отдельный продукт", а как надстройку над существующими:

- `src-tauri/src/state.rs`
- `src-tauri/src/commands/chat_stream.rs`
- `src-tauri/src/commands/claude_runner.rs` -> будущий provider runner
- `src-tauri/src/commands/delegation.rs`
- `src-tauri/src/commands/gate.rs`
- `src-tauri/src/commands/graph*.rs`
- `src-ui/*`
- `tasks/*` как persistent store

## Цели

1. Дать пользователю два видимых агентных потока: `Claude` и `Codex`.
2. Разрешить режимы `parallel`, `debate`, `review`, `arbiter`.
3. Ввести контролируемую модель владения файлами и write access.
4. Сделать работу артефактной: spec/design/tasks/execution/review/decision.
5. Увязать это с текущими `Plan`, `Todo`, `Delegation`, `Gate`, `Signal`, `Graph`.
6. Сохранить возможность поэтапного rollout без резкого усложнения UX.

## Не-цели v1

- Не делать сразу произвольную параллельную запись двух агентов в один и тот же файл.
- Не заменять git merge интеллектуальной магией LLM.
- Не строить полноценную distributed lock system между машинами.
- Не переводить весь продукт на новый runtime store за один релиз.

## Главный архитектурный тезис

Нужна не просто "вторая вкладка чата", а `Multi-Agent Session`, где чат, артефакты, ownership файлов, гейты и арбитраж связаны одним ledger.

Иначе получится два красивых окна, которые:

- не знают, кто над чем работает;
- могут конфликтовать по изменениям;
- спорят без evidence;
- не дают пользователю управляемого качества.

## Предлагаемая модель

### 1. Session

Вводится новая сущность:

```rust
struct MultiAgentSession {
    id: String,
    title: String,
    project: String,
    status: SessionStatus,
    mode: SessionMode,
    participants: Vec<Participant>,
    active_artifact_id: Option<String>,
    current_working_set: Vec<String>,
    created_at: i64,
    updated_at: i64,
}
```

Где:

- `SessionMode = Solo | Review | Debate | Parallel | Arbitration`
- `Participant = User | ClaudePm | CodexTech | OptionalAgent(String)`

### 2. Agent run

Каждый запуск агента хранится как отдельная сущность:

```rust
struct AgentRun {
    id: String,
    session_id: String,
    provider: ProviderKind,
    role: AgentRole,
    objective: String,
    input_artifact_ids: Vec<String>,
    output_artifact_ids: Vec<String>,
    lease_ids: Vec<String>,
    status: RunStatus,
    started_at: i64,
    finished_at: Option<i64>,
}
```

Где:

- `ProviderKind = Claude | Codex`
- `AgentRole = Product | Technical | Reviewer | Challenger | Executor | ArbiterAdvisor`

### 3. Shared ledger

У каждой сессии есть единый журнал событий:

- user prompt;
- agent started;
- agent message chunk;
- artifact created;
- artifact superseded;
- file intent declared;
- lease acquired;
- lease denied;
- gate started;
- gate passed/warned/failed;
- debate round opened/closed;
- arbitration requested;
- decision applied.

Это источник истины для UI и постфактум аудита.

## Kiro-inspired artifact pipeline для AgentOS

### Канонический pipeline

```text
Prompt
-> Spec
-> Design
-> Task Graph
-> Execution
-> Verification
-> Debate / Review
-> Arbitration
-> Merge Decision / Release to Todo state
```

### Артефакты

```rust
enum ArtifactKind {
    Spec,
    Design,
    TaskGraph,
    FileIntent,
    ExecutionLog,
    Review,
    DebateClaim,
    GateReport,
    Decision,
}
```

Каждый artifact:

- принадлежит session;
- имеет автора;
- может ссылаться на файлы;
- может ссылаться на другие artifacts;
- имеет версию;
- может быть marked as superseded, approved, rejected.

### Почему это важно

Без артефактов спор агентов быстро становится неоперабельным:

- один говорит "надо так";
- другой говорит "нет, так";
- пользователь вручную сопоставляет большие текстовые ответы;
- связь с реальными файлами и задачами теряется.

С артефактами спор становится структурным:

- тезис;
- доказательство;
- ссылка на diff/test/gate/graph;
- предложенное действие.

## Модель состояния файлов

Система должна знать не только "файл изменен", но и "что с ним сейчас происходит".

### FileWorkState

```rust
enum FileWorkState {
    Untracked,
    CandidateContext,
    InWorkingSet,
    ClaimedForAnalysis,
    ProposedWrite,
    ExclusiveWrite,
    ModifiedPendingGate,
    UnderReview,
    Accepted,
    Rejected,
    Disputed,
    StaleLease,
}
```

### Смысл состояний

- `Untracked`: файл вне текущей сессии.
- `CandidateContext`: граф или агент предложил файл как релевантный.
- `InWorkingSet`: файл вошел в текущий scope работы.
- `ClaimedForAnalysis`: агент изучает файл, но не пишет.
- `ProposedWrite`: агент заявил намерение менять файл.
- `ExclusiveWrite`: агент получил write lease.
- `ModifiedPendingGate`: файл изменен, но еще не прошел gate.
- `UnderReview`: второй агент или пользователь проверяет изменения.
- `Accepted`: изменения приняты и lease снят.
- `Rejected`: изменения отклонены.
- `Disputed`: по файлу есть конфликт решений или конкурентные предложения.
- `StaleLease`: lease просрочен или владелец пропал.

### Почему это нужно

Если система знает только "dirty / clean", она не умеет ответить:

- кто сейчас работает с файлом;
- кто собирается его менять;
- можно ли запускать второго агента;
- по какому файлу спорят;
- что ждет gate;
- что можно безопасно auto-approve.

## Lease manager

### Базовый принцип

Ни один агент не получает право писать в файл без lease.

### Типы lease

```rust
enum LeaseMode {
    Read,
    ProposeWrite,
    ExclusiveWrite,
}
```

### Lease record

```rust
struct FileLease {
    id: String,
    session_id: String,
    owner_run_id: String,
    owner_role: AgentRole,
    path_pattern: String,
    mode: LeaseMode,
    reason: String,
    ttl_secs: u64,
    heartbeat_at: i64,
    created_at: i64,
}
```

### Правила

1. `Read` lease могут пересекаться.
2. `ProposeWrite` можно выдавать нескольким агентам, но это не дает права писать.
3. `ExclusiveWrite` запрещает другой `ExclusiveWrite` и любые write-like действия на том же scope.
4. Любой фактический file write без `ExclusiveWrite` = hard gate fail.
5. Lease выдается по file path или path prefix.
6. Lease имеет heartbeat и TTL.
7. Stale lease снимается системой или вручную через UI.

### Разрешение конфликтов

Если два агента хотят писать в пересекающиеся scope:

- по умолчанию один получает `ExclusiveWrite`, второй остается `ProposeWrite` или `Read`;
- если пользователь хочет сравнить решения, второй переводится в `shadow worktree` на следующем этапе развития;
- merge без review/arbitration запрещен.

### Важная рекомендация

В v1 не давать двум агентам одновременно писать в один checkout в пересекающиеся файлы. Это резко снижает риск хаоса.

## Working set

Каждая сессия должна иметь явный `working set`:

- user-selected files;
- graph-neighbor files;
- уже измененные файлы;
- файлы из текущего spec/task;
- файлы под lease.

Working set нужен, чтобы:

- ограничивать context window;
- показывать текущий scope работы;
- строить точные intents;
- не размазывать обсуждение по всей репе.

## Debate model

### Режимы

- `Review`: один агент предлагает, второй только проверяет.
- `Debate`: второй агент обязан оспорить или подтвердить решение по структуре.
- `Arbitration`: пользователь видит два предложения и выносит решение.
- `Parallel`: оба агента работают параллельно, но только по disjoint scope.

### Debate round

```rust
struct DebateRound {
    id: String,
    session_id: String,
    topic: String,
    claim_artifact_id: String,
    counter_artifact_id: Option<String>,
    evidence_artifact_ids: Vec<String>,
    status: DebateStatus,
}
```

### Формат аргумента

Каждый спор должен быть структурирован:

- `claim`
- `evidence`
- `risk`
- `proposed action`

Без этого спор деградирует в два длинных ответа без оперативной ценности.

### Ограничение на споры

Нужен лимит:

- максимум 2 раунда без эскалации;
- после этого требуется user decision или forced review resolution.

Иначе агенты будут тратить токены на риторическое хождение по кругу.

## Gate system

Текущий Gate pipeline уже существует. Его надо расширить, не выбрасывая.

### Новый состав gate

1. `Mechanical gate`
- build/test/lint/script exit code
- diff stats
- cost/time budget

2. `Lease compliance gate`
- были ли write без lease
- вышел ли агент за заявленный file scope
- были ли stale/conflicting leases

3. `Graph impact gate`
- попали ли изменения в заявленный impact set
- появились ли неожиданные затронутые узлы

4. `LLM review gate`
- `Codex` ревьюит `Claude` или наоборот
- вердикт: `pass | warn | fail`

5. `Arbitration gate`
- если по файлу или design есть конфликт, gate не закрывается без решения пользователя или policy-based winner selection

### Gate outputs

```rust
struct GateReport {
    id: String,
    session_id: String,
    run_id: String,
    mechanical: GateVerdict,
    lease: GateVerdict,
    graph: GateVerdict,
    llm_review: GateVerdict,
    arbitration: GateVerdict,
    summary: String,
}
```

### Связь с Todo/Delegation

Результат multi-agent выполнения должен проецироваться обратно в текущую модель:

- `Todo` не становится done только потому, что агент так сказал;
- `Delegation` не становится effective complete, если lease/gate/arbitration не закрыты;
- `Signal` должен уметь сообщать не только `build failed`, но и `write outside scope`, `lease conflict`, `debate unresolved`.

## Graph integration

Граф нужен не только для визуализации, а как operational subsystem.

### Что graph должен уметь для этой системы

1. Построить `candidate working set` по entry file.
2. Выделить `impact set` для ожидаемых изменений.
3. Подсветить зоны потенциального конфликта между агентами.
4. Проверить, не ушел ли агент за пределы ожидаемой области.
5. Помочь бить работу на disjoint scopes.

### Ограничение

Пока graph неточен, нельзя делать из него единственный источник истины для leases. Он должен быть advisory + validating layer, а не абсолютный scheduler.

## UI модель

### Layout v1

- левая колонка: `Claude`
- правая колонка: `Codex`
- нижняя или центральная панель: `Session Ledger`
- боковая панель: `Working Set / File Leases / Artifacts / Gates`

### Что пользователь должен видеть сразу

- кто сейчас говорит;
- кто сейчас пишет;
- какие файлы заняты;
- какие файлы только предложены к изменению;
- какой artifact сейчас активен;
- есть ли конфликт;
- что именно требует решения пользователя.

### Обязательные действия пользователя

- `Ask Claude`
- `Ask Codex`
- `Start Debate`
- `Ask for Review`
- `Grant Write Lease`
- `Revoke Lease`
- `Resolve Conflict`
- `Approve Decision`
- `Reject Decision`

## Runtime storage proposal

В духе текущего AgentOS разумно сохранить file-backed store:

- `tasks/.sessions.json`
- `tasks/.session-events.jsonl`
- `tasks/.file-leases.json`
- `tasks/.artifacts.jsonl`
- `tasks/.debates.jsonl`

Это согласуется с уже существующей моделью runtime state и не требует на старте поднимать БД.

## Изменения по backend слоям

### 1. Provider abstraction

Текущее `claude_runner.rs` должно стать общим provider layer:

- `provider_runner.rs`
- единый интерфейс `run_provider` / `run_provider_stream`
- `ProviderKind::Claude`
- `ProviderKind::Codex`

Без этого весь остальной дизайн останется на уровне UI-иллюзии.

### 2. State layer

В `state.rs` добавить:

- `multi_agent_sessions`
- `agent_runs`
- `file_leases`
- `artifacts`
- `debate_rounds`

И persistence helpers для них.

### 3. Chat stream

`chat_stream.rs` нужно расширить до multi-stream:

- отдельный output stream на каждого агента;
- aggregation stream для ledger;
- event typing вместо "просто текстовых чанков".

### 4. Delegation

`delegation.rs` должно уметь:

- запускать executor/reviewer как разные provider roles;
- требовать file intents до исполнения;
- учитывать lease conflicts;
- создавать gate dependency на review/arbitration.

### 5. Gate

`gate.rs` нужно расширить:

- lease compliance checks;
- graph impact checks;
- llm review stage;
- unresolved conflict stage.

### 6. Signals / Inbox / Plans

Нужно добавить новые типы событий:

- lease conflict;
- stale lease;
- write outside scope;
- unresolved debate;
- arbitration pending;
- shadow worktree merge required.

## Hooks model в духе Kiro

Kiro полезен здесь прежде всего идеей hooks на lifecycle события.

Для AgentOS можно добавить internal hooks на:

- session start;
- provider spawn;
- file intent submit;
- lease acquire/release;
- before tool execution;
- after tool execution;
- before gate;
- after gate;
- debate open/close;
- arbitration requested;
- decision finalized.

Эти hooks должны использоваться не для "магии", а для:

- логирования;
- валидации policy;
- автозапуска verify;
- автоочистки stale leases;
- сигналов в UI.

## Предлагаемый policy set

### Default policy

- Оба агента видимы.
- Debate opt-in.
- Писать может только один агент за раз.
- Второй агент в режиме `review/challenge`.

### Advanced policy

- Parallel writes только по непересекающимся scope.
- Graph помогает предложить разбиение scope.
- При конфликте система отклоняет второй lease.

### Experimental policy

- Shadow worktree на конфликтный scope.
- Два независимых implementation proposal.
- Потом user arbitration + merge workflow.

## Самокритика этого дизайна

### 1. Система резко сложнее текущей

Это уже не просто chat + delegation, а coordination platform. Если делать все сразу, риск перегрузить и runtime, и UI.

### 2. Два агента не гарантируют истину

Если оба опираются на неполный контекст или граф неточен, можно получить "уверенный спор двух ошибающихся систем". Поэтому debate не заменяет mechanical gates.

### 3. Lease manager легко превратить в бюрократию

Если надо вручную подтверждать каждый write intent, пользователь устанет. Нужны policy и sane defaults.

### 4. Deadlock и starvation реальны

Один агент может держать lease слишком долго, второй будет постоянно ждать. Нужны TTL, heartbeat и manual override.

### 5. Graph quality станет критической зависимостью

Как только leases и impact начинают опираться на graph, ошибки графа начинают ломать orchestration.

### 6. Стоимость и latency вырастут кратно

Dual visible agent, debate, review gate и shadow workflows легко удваивают или утраивают стоимость.

### 7. Пользователь может утонуть в информации

Если в UI одновременно показывать оба стрима, ledger, leases, artifacts и gates без четкой иерархии важности, система станет тяжелой в использовании.

### 8. Full parallel writing в одном checkout - плохая идея для v1

Даже с leases это хрупко. Правильнее сначала сделать single writer + visible reviewer.

## Упрощенная рекомендация

Самый рациональный v1:

- два видимых агента;
- shared ledger;
- artifacts;
- file intents;
- single writer;
- второй агент как reviewer/challenger;
- arbitration по запросу;
- mechanical + lease + llm gates.

Это уже даст:

- видимость обоих;
- спор;
- контроль качества;
- контроль записи;
- минимально приемлемую сложность.

## Phased rollout

### Phase 0. Provider abstraction

Сделать общий runner для `Claude` и `Codex`.

Готовность:

- одинаковый API запуска и стриминга;
- конфиг provider role;
- telemetry per provider.

### Phase 1. Visible dual chat

Показать два независимых стрима в одной сессии.

Безопасность:

- никакой параллельной записи;
- оба агента read-only или только один write-enabled.

### Phase 2. Session ledger + artifacts

Добавить event log, artifacts и явный session state.

Польза:

- можно спорить по evidence;
- можно восстанавливать историю;
- можно строить UI без ad-hoc парсинга текста.

### Phase 3. File intents + leases

Ввести `ProposeWrite` и `ExclusiveWrite`.

Правило:

- write без lease = fail.

### Phase 4. Review/Debate/Arbitration

Добавить bounded debate rounds и явное user decision.

### Phase 5. Parallel disjoint work

Разрешить параллельную запись только по непересекающимся scope.

### Phase 6. Shadow worktrees

Добавить отдельные worktree для конфликтных альтернатив.

### Phase 7. Full integration with Plans/Todos/Signals

Связать session outcomes с существующими `Plan`, `Todo`, `Delegation`, `Gate`, `Signal`.

## Detailed execution roadmap

### Phase 0A. Provider layer and capability surface

Цель:

- убрать хардкод `Claude-only` из точки расширения;
- завести явную модель provider roles;
- видеть, что реально доступно в окружении.

Изменения:

- `src-tauri/src/commands/provider_runner.rs`
  - `ProviderKind`
  - role resolution
  - provider status snapshot
  - safe Codex adapter через command template, а не через хардкод неизвестных CLI флагов
- `src-tauri/src/commands/claude_runner.rs`
  - остается как Claude-specific backend
- `src-tauri/src/commands/config.rs`
  - provider config keys используются как runtime contract
- `src-ui/pages.js`
  - provider status и provider role configuration в settings

Acceptance criteria:

- backend знает `orchestrator_provider` и `technical_reviewer_provider`;
- UI показывает, найден ли `claude` и найден/готов ли `codex`;
- если `codex` не настроен, система падает с явным config error, а не с молчаливым no-op.

### Phase 0B. Session state and persistent ledger

Цель:

- дать dual-agent оркестрации собственный runtime state;
- не смешивать session lifecycle с обычными chat/delegation логами.

Изменения:

- `src-tauri/src/state.rs`
  - `MultiAgentSession`
  - `SessionEvent`
  - persistence в `tasks/.sessions.json`
  - event log в `tasks/.session-events.jsonl`
- `src-tauri/src/commands/multi_agent.rs`
  - create/list/get session
  - run participant
  - per-participant history

Acceptance criteria:

- можно создать dual-agent session;
- сессия переживает перезапуск;
- все обращения к агентам пишут события в общий ledger.

### Phase 1A. Dual visible session UI

Цель:

- пользователь видит обоих участников;
- каждый поток отображается отдельно;
- ledger виден рядом, а не спрятан в логи.

Изменения:

- `src-ui/store.js`
  - active session
  - per-participant histories
  - session events
- `src-ui/api.js`
  - create/load/run session commands
- `src-ui/chat.js` или новый `src-ui/pages.js` block
  - dual-pane layout
  - participant selector / buttons
  - session ledger panel

Acceptance criteria:

- можно открыть одну сессию и видеть два независимых ответа;
- участники не смешиваются в один history stream;
- ledger отражает order of operations.

### Phase 1B. Single-writer policy

Цель:

- пока нет leases, уже не допустить "оба пишут одновременно".

Изменения:

- session policy
  - один participant write-enabled
  - второй reviewer/challenger
- UI
  - кто writer, кто reviewer
- backend
  - policy guard при запуске write-capable run

Acceptance criteria:

- система явно показывает current writer;
- второй агент не получает write-like run без перевода роли.

### Phase 2A. File intents

Цель:

- отделить "я изучаю" от "я хочу менять".

Изменения:

- `FileIntent` artifact
- new session events:
  - intent_declared
  - intent_accepted
  - intent_rejected
- UI working set panel

Acceptance criteria:

- перед исполнением агент может заявить scope изменений;
- пользователь видит proposed files до фактической записи.

### Phase 2B. Leases

Цель:

- сделать запись контролируемой, а не подразумеваемой.

Изменения:

- `tasks/.file-leases.json`
- state + commands:
  - acquire lease
  - release lease
  - heartbeat
  - stale detection
- gate integration:
  - write without lease = fail

Acceptance criteria:

- запись без lease детектируется;
- stale lease можно безопасно снять;
- UI показывает ownership по файлам.

### Phase 3A. Debate and arbitration

Цель:

- спор должен быть структурным и ограниченным по раундам.

Изменения:

- `DebateRound`
- claim / counterclaim artifacts
- arbitration decision artifact
- UI actions:
  - challenge
  - respond
  - arbitrate

Acceptance criteria:

- максимум 2 раунда без forced decision;
- спор всегда привязан к evidence;
- результат спора попадает в ledger.

### Phase 3B. Gates for multi-agent mode

Цель:

- dual-agent workflow не должен обходить existing safety rails.

Изменения:

- `gate.rs`
  - lease compliance gate
  - llm review gate
  - unresolved debate gate
  - graph impact gate
- `signals.rs`
  - новые signal kinds

Acceptance criteria:

- task нельзя закрыть при unresolved conflict;
- сигнал виден в UI и в orchestration feedback loop.

### Phase 4. Parallel disjoint work

Цель:

- разрешить реальный parallel only where safe.

Изменения:

- graph-assisted scope split
- non-overlapping lease validation
- optional `shadow worktree`

Acceptance criteria:

- два агента могут работать одновременно только по disjoint scope;
- пересечение scope ведет к deny или arbitration path.

## Immediate implementation order

Порядок для реальной разработки без расползания:

1. Provider layer
2. Session state + ledger
3. Minimal session commands
4. Settings surface for provider readiness
5. Dual visible session UI
6. Single-writer policy
7. File intents
8. Leases
9. Debate
10. Gate extensions
11. Parallel disjoint work

## What should not be done early

- Не тащить graph в роль единственного scheduler до улучшения его точности.
- Не запускать full parallel writes в общем checkout до leases и conflict policies.
- Не делать automatic winner selection между агентами без явного evidence model.
- Не смешивать session events и обычные chat logs в один формат.

## Критерии готовности v1

Считать v1 успешным, если:

1. Пользователь видит два независимых агентных потока.
2. Второй агент умеет критиковать или подтверждать первого.
3. Любая запись в файл проходит через lease.
4. UI явно показывает ownership и текущий work scope.
5. Gate умеет валить задачу за нарушение file scope.
6. Сессия воспроизводима по ledger и artifacts.

## Критерии, при которых rollout надо останавливать

- leases создают слишком много ручной работы;
- graph слишком неточен для conflict detection;
- dual chat удваивает шум, но не повышает качество решений;
- arbitration занимает слишком много внимания пользователя;
- provider abstraction нестабильна и ломает текущий single-agent workflow.

## Рекомендуемые первые implementation slices

### Slice A

- provider abstraction;
- session model;
- dual chat UI;
- ledger без leases.

### Slice B

- file intents;
- single writer lease;
- lease events в UI;
- gate fail за write without lease.

### Slice C

- technical review mode;
- debate round;
- arbitration action.

### Slice D

- graph-assisted working set;
- parallel disjoint write scopes.

## Итоговая рекомендация

Для AgentOS правильный путь не в "просто открыть два чата", а в построении `Multi-Agent Session` с:

- двумя видимыми агентами;
- артефактным pipeline в духе Kiro;
- явным состоянием файлов;
- lease manager;
- gates;
- bounded debate;
- арбитражем пользователя.

Но стартовать нужно с ограниченного варианта:

- два видимых агента;
- один writer;
- второй reviewer/challenger;
- write через lease;
- debate только по требованию;
- graph как помощник, а не как абсолютный диспетчер.

Это дает лучший баланс между силой системы и риском превратить продукт в неуправляемый orchestration monster.

## Источники

- Kiro Specs: https://kiro.dev/docs/specs/
- Kiro Steering: https://kiro.dev/docs/steering/
- Kiro Hooks: https://kiro.dev/docs/hooks/
- Kiro CLI Hooks: https://kiro.dev/docs/cli/hooks/
- Kiro Specs Best Practices: https://kiro.dev/docs/specs/best-practices/
- Kiro background task lifecycle: https://kiro.dev/docs/cli/experimental/delegate
- Kiro task states for autonomous tasks: https://kiro.dev/docs/autonomous-agent/using-the-agent/creating-tasks/
