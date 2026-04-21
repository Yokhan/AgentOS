# AgentOS Dual-Agent Live Room Replan

## Purpose

This document is a critical re-check of the dual-agent orchestration direction.
It narrows the scope, removes unsafe assumptions, and defines a rollout that is
compatible with the current AgentOS architecture.

The previous direction was broadly correct, but too many concepts were moving at
once:

- visible dual chat
- debate
- arbitration
- strategy/tactic/plan/todo promotion
- file tracking
- leases
- Kiro-style artifacts
- parallel work

For a large implementation, that is too much ambiguity. This replan defines what
must exist first, what must be delayed, and what is explicitly forbidden until
the system is ready.

## Current Baseline

Canonical AgentOS pipeline:

```text
Strategy -> Tactic -> Plan -> Todo -> Delegation -> Gate -> Signal -> PA feedback
```

The live room must sit on top of this pipeline. It must not flatten it into
`strategy -> plan -> delegation` or replace it with a room-only artifact model.

Already implemented:

- provider abstraction for `Claude` and `Codex`
- persistent `MultiAgentSession`
- session ledger in `tasks/.session-events.jsonl`
- per-agent history
- `ask both` analysis round
- `FILES:` extraction into `current_working_set`
- auto-open `duo` for orchestrator when `Codex` is ready

Current limitations:

- there is no real live room stream yet
- agents do not see each other incrementally "online"
- there is no presence model
- there are no mentions, challenge turns, or rebuttal turns
- there is no write intent model
- there are no leases
- parallelism is analysis-only, not safe parallel execution

Conclusion: the system is currently a dual-response tool, not a live
multi-agent room.

## Critical Corrections

### 1. Do not model "hidden thoughts"

The system must not try to expose chain-of-thought or internal hidden reasoning.
What we need is a public collaboration layer:

- short public status updates
- short public arguments
- short public objections
- explicit file claims
- explicit write requests

This becomes the visible room protocol.

Design correction:

- replace "show both thoughts" with `public agent messages`
- replace "online thinking" with `presence + typing + incremental public notes`

### 2. Do not start with autonomous free-form debate

If two strong models are allowed to talk freely, they will often burn tokens and
time without operational progress.

Debate must be bounded:

- max turn budget per round
- explicit topic
- explicit challenger
- structured output
- hard stop requiring user or policy decision

Design correction:

- no infinite back-and-forth
- no autonomous "argue until consensus"

### 3. Do not start with parallel writes

Visible multi-agent work is useful before parallel file mutation exists.

Parallel write before intents and leases will cause:

- conflicting edits
- broken working state
- incoherent gate results
- user confusion about ownership

Design correction:

- phase 1 and phase 2 are read/reason/argue only
- writing remains single-writer until lease layer exists

### 4. The room must be the source of truth

Two isolated columns are not enough. The system needs one canonical event log
for:

- user prompts
- agent presence
- live chunks
- challenge/rebuttal actions
- file intents
- future write intents and lease events

Design correction:

- columns become views over one room stream
- ledger is no longer secondary; it is the runtime truth

### 5. Roles must be operational, not cosmetic

`Claude = product` and `Codex = technical` is only useful if prompts, turn
types, and permissions actually differ.

Design correction:

- each role gets a distinct room contract
- product agent frames, prioritizes, proposes
- technical agent critiques, verifies, constrains, de-risks
- only one side can become active writer later

### 6. Room output must promote into the right pipeline level

Not every room conclusion should become a strategy.

There are two valid downstream paths:

- `goal path`: room -> strategy -> tactic -> plan -> todo
- `ad-hoc path`: room -> plan -> todo

Design correction:

- strategy is for user goals in the real world
- plan is for concrete work in one project
- todo is the atomic execution unit
- agent todo becomes delegation; user todo stays manual

## Target UX

The target is not "two chats". The target is a `live room`.

One room per project or `_orchestrator`.

Visible participants:

- `You`
- `Claude PM`
- `Codex Tech`
- `System`

Main views:

1. `Room`
   Interleaved live event stream.

2. `Claude Log`
   Full Claude-only transcript and outputs.

3. `Codex Log`
   Full Codex-only transcript and outputs.

4. `Work`
   Working set, intents, future leases, future gate state.

Top controls:

- `Ask Both`
- `@Claude`
- `@Codex`
- `Challenge`
- `Rebuttal`
- `Arbitrate`
- later: `Grant Write`

Promotion controls after room discussion:

- `Promote to Strategy`
- `Promote to Ad-hoc Plan`
- later: `Promote to Project Plan`
- later: `Promote to Todo Batch`

Room event examples:

- `System: Round started`
- `Claude: I will frame the scope and propose a plan`
- `Codex: I disagree on migration risk`
- `Claude: Likely files: src/foo.rs, src/bar.rs`
- `System: Working set updated`
- `You: Ask Codex to challenge the rollout`
- `System: Challenge turn started`

## Room Protocol

This is the most important new design element.

Agents do not produce only one final blob. They emit room events.

### Event Types v1

```text
user_message
system_message
agent_presence
agent_typing
agent_note
agent_message
round_started
round_completed
challenge_requested
challenge_response
rebuttal_requested
rebuttal_response
file_intent_declared
working_set_updated
arbiter_decision
```

### Event Rules

- `agent_note` must be short, public, and operational
- `agent_message` is the completed visible response
- `agent_typing` is presence only, not content
- every room round has a `round_id`
- challenge and rebuttal events always reference prior event ids

### What is forbidden in v1

- hidden reasoning capture
- auto-generated endless note spam
- agents directly talking to each other without turn budget

## State Model v2

Current session state is too small for live collaboration. It should evolve.

### Session Additions

```rust
struct MultiAgentSession {
    id: String,
    title: String,
    project: String,
    status: SessionStatus,
    mode: SessionMode,
    participants: Vec<SessionParticipant>,
    current_working_set: Vec<String>,
    active_round_id: Option<String>,
    active_speaker: Option<String>,
    presence: HashMap<String, PresenceState>,
    pending_challenge: Option<PendingChallenge>,
    pending_rebuttal: Option<PendingRebuttal>,
    created_at: String,
    updated_at: String,
}
```

### Presence State

```rust
enum PresenceState {
    Idle,
    Thinking,
    Replying,
    Waiting,
    Blocked,
}
```

### Why this matters

Without explicit presence state, the UI cannot reliably show:

- who is currently active
- who is waiting for the other side
- whether the room is still progressing
- whether a challenge round is open

## Prompt Contract

The prompt contract must change. Each agent should know:

- who they are
- who the other visible participant is
- what role the other participant has
- what the room goal is
- what the latest public room messages are
- what the current working set is
- whether this is a normal round, challenge round, or rebuttal round

### Product Role Contract

Claude PM should optimize for:

- problem framing
- scope control
- task decomposition
- prioritization
- arbitration-ready proposals

Claude should not dominate technical correctness claims without challenge.

### Technical Role Contract

Codex Tech should optimize for:

- architecture
- implementation risk
- correctness
- testing and verification
- identifying weak assumptions

Codex should not take over product scoping unless explicitly asked.

## Debate Contract

Debate is useful only if it is structured.

### Challenge Format

Every challenge response should contain:

- `claim`
- `evidence`
- `risk`
- `proposal`

### Debate Limits

- max 2 turns per side for a single disagreement
- after that, user or policy arbitration is required

### Why

This prevents token burn, circular argument, and fake consensus.

## Working Set and Intent Model

The current `FILES:` footer is a good seed, but not enough.

We need a staged model:

1. `FILES`
   Likely touched files from analysis.

2. `WRITE_INTENT`
   Explicit request to mutate files.

3. `LEASE`
   Permission to actually write.

4. `GATE`
   Verification and acceptance.

### Rule

No file mutation should be allowed before `WRITE_INTENT` and later `LEASE`.

### Why this is critical

Otherwise the room looks coordinated, but runtime behavior is not.

## Kiro-Inspired Pipeline, Narrowed

Kiro inspiration remains useful, but only in a focused way.

Use:

- artifact-first execution
- explicit spec/design/tasks progression
- hooks and lifecycle thinking

Do not over-copy:

- do not turn every prompt into full spec/design/tasks overhead
- do not block simple conversations behind heavyweight artifacts

### Practical pipeline

For large tasks:

```text
Prompt
-> Room Round
-> Spec note
-> Design note
-> Task slice
-> Execution
-> Verification
-> Arbitration if needed
```

For small tasks:

```text
Prompt
-> Room Round
-> Execution
-> Verification
```

This is important. A heavyweight process for every task will make the product
slow and annoying.

## Rollout Plan

### Phase 0: Stabilize the Concept

Goal:

- freeze the contract for live room
- avoid starting leases too early

Tasks:

- finalize room event taxonomy
- finalize role prompt contract
- finalize challenge/rebuttal rules
- define what counts as public note vs final message

Exit criteria:

- documented event list
- documented turn budget
- documented forbidden behaviors

### Phase 1: Live Room

Goal:

- make the two agents visible in one interleaved room

Tasks:

- add room event stream backend on top of `SessionEvent`
- add presence updates
- add incremental `agent_note` and final `agent_message`
- add room view in UI
- keep existing per-agent logs as secondary tabs

Must not do yet:

- no write access changes
- no leases
- no autonomous long debates

Exit criteria:

- user can watch both agents in one live room
- roles are visible
- current speaker is visible
- per-agent logs still work

### Phase 2: Directed Interaction

Goal:

- let the user steer the agents against each other cleanly

Tasks:

- add `@Claude` and `@Codex`
- add `Challenge`
- add `Rebuttal`
- add arbitration decision event
- pass prior room events into prompts

Must not do yet:

- no parallel writes
- no lease enforcement

Exit criteria:

- user can direct one side to challenge the other
- room shows bounded turn-taking
- decisions are persisted in the ledger

### Phase 3: Intent Discipline

Goal:

- move from "talking about files" to explicit operational claims

Tasks:

- formalize `FILES` as structured data
- add `WRITE_INTENT` event type
- add intent conflict detection
- show disputed paths in UI

Must not do yet:

- no actual multi-writer mode

Exit criteria:

- working set is stable and visible
- write targets are explicit
- path disputes are visible before execution

### Phase 4: Single-Writer Enforcement

Goal:

- make file mutation safe

Tasks:

- implement single-writer policy
- add lease manager
- deny writes outside lease scope
- wire lease events into gate

Exit criteria:

- every write has a lease
- out-of-scope writes fail gate
- room shows who owns the write

### Phase 5: Safe Parallel Work

Goal:

- allow both agents to work in parallel when scopes do not overlap

Tasks:

- allow disjoint-scope leases
- add conflict detection by path prefix / working set overlap
- later add shadow worktree mode for overlapping proposals

Exit criteria:

- parallel work happens only on disjoint scopes
- conflicts become explicit instead of silent

## Anti-Goals

The following should be explicitly rejected during implementation review:

- "let both agents just talk freely for a while"
- "let them both edit and see what happens"
- "show hidden thoughts"
- "make every prompt go through full spec/design/tasks"
- "solve merge conflicts with LLM magic before leases exist"

## File-by-File Impact

### Backend

- `src-tauri/src/state.rs`
  Add room/presence/round metadata.

- `src-tauri/src/commands/multi_agent.rs`
  Main home for room orchestration, challenge/rebuttal, working set state.

- `src-tauri/src/commands/provider_runner.rs`
  Add support for visible room-oriented execution contracts if needed.

- `src-tauri/src/commands/gate.rs`
  Later: integrate intent and lease compliance.

### Frontend

- `src-ui/pages.js`
  Replace current duo layout with room-first UX.

- `src-ui/api.js`
  Add room actions and room polling/stream behavior.

- `src-ui/store.js`
  Add room state, presence, active speaker, directed turn state.

## Immediate Next Slice

The next implementation slice should be:

1. Room-first UI
2. Presence state
3. Interleaved room events
4. Directed interactions: `@Claude`, `@Codex`, `Challenge`, `Rebuttal`

Not leases yet.

That is the smallest slice that changes the product from "dual answers" to
"visible multi-agent collaboration".

## Final Recommendation

The correct order is:

```text
Live Room
-> Directed Interaction
-> Intent Discipline
-> Single Writer
-> Leases
-> Safe Parallel Work
```

Not:

```text
Parallel Write
-> Debate
-> Figure It Out Later
```

This sequence is slower at the beginning, but much safer and much more likely
to produce a system that remains understandable under load.
