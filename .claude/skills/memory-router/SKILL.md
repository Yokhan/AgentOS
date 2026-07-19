---
name: memory-router
description: "Unified memory layer: routes saves/searches to correct backend (Engram, files, brain vault). Includes dedup, TTL, and related links."
---

# Memory Router

Unified convention for all memory operations across agents. Routes to the correct backend based on content type.

## Save Routing

| Content Type | Backend | How |
|-------------|---------|-----|
| Session context (bugs, findings, WIP) | Engram | `mem_save(topic_key="...", content="...")` |
| User preferences, project config | Files | Write to `brain/02-projects/` or `tasks/` |
| Patterns, solutions, knowledge | Brain vault | Write to `brain/03-knowledge/` |
| Architecture decisions | Brain vault | Write to `brain/04-decisions/` using decision-record template |
| Lessons from corrections | Both | Write to `tasks/lessons.md` AND `mem_save(topic_key="lesson:...")` |

## Search Order

When looking for information:
1. **Engram** `mem_search(query)` — recent session context, cross-project
2. **Tasks** — Read `tasks/current.md` + `tasks/lessons.md` — current project state
3. **Brain** — Grep `brain/` — deep knowledge, patterns, decisions

Search Engram **globally** by default (omit project param). Filter by project only when results are too noisy.

## Dedup Protocol (before every save)

1. `mem_search` for existing observation with same `topic_key` prefix
2. If found and covers same topic: `mem_update(id=existing_id, content=merged_content)`
3. If found but different aspect: save with more specific topic_key
4. If not found: `mem_save` new

## TTL by Category

Include `TTL: YYYY-MM-DD` in observation content when saving.

| Category | TTL | Rationale |
|----------|-----|-----------|
| config | permanent | Infrastructure changes rarely |
| architecture | 6 months | Architecture evolves slowly |
| decision | 6 months | Decisions valid within project phase |
| pattern | 6 months | Patterns are stable |
| discovery | 3 months | Findings become stale |
| bugfix | 1 month | Bug fixed, context less relevant |
| learning | 3 months | Should promote to rule or expire |
| session | 1 week | Ephemeral session context |
| metrics | 3 months | Performance data for trending |

During `/weekly`: search for observations past TTL, delete with `mem_delete`.

## Related Links

When saving observations, tag related existing ones:
```
content += "\n**Related**: [topic_key_1], [topic_key_2]"
```

When loading an observation with Related tags, also load linked observations (1 level deep) for context.

## Topic Key Naming Convention

Format: `{namespace}:{category}:{specific}`

Examples:
- `agent:researcher:findings` — researcher's discoveries
- `agent:implementer:decision` — implementer's design choices
- `shared:architecture:overview` — project architecture (all agents)
- `lesson:supabase-rls` — lesson about RLS
- `metrics:implementer` — performance metrics for implementer
- `rule:activation:domain-health` — rule effectiveness tracking
- `benchmark:v2.4.0` — benchmark results for template version

## Fallback: Engram Unavailable

If Engram MCP is not connected or not responding, degrade gracefully:

| Normal (Engram) | Fallback (file-only) |
|-----------------|---------------------|
| `mem_save(topic_key, content)` | Append to `tasks/.memory-fallback.md` with same format |
| `mem_search(query)` | Grep `tasks/lessons.md` + `brain/03-knowledge/` |
| `mem_session_start` | Skip (session-start.sh handles log) |
| `mem_update(id, content)` | Find and edit entry in `.memory-fallback.md` |
| `mem_delete(id)` | Remove entry from `.memory-fallback.md` |

Detection: if first `mem_save` call errors with MCP connection failure → switch to fallback for rest of session. Do NOT retry MCP calls after first failure.

When Engram becomes available again: import `.memory-fallback.md` entries into Engram, then clear the file.

## Contextual Memory Preload

Before editing any file, agents should:
1. Extract module name from path (e.g., `src/features/auth/` → "auth")
2. `mem_search("{module_name}")` — find related bugs, decisions, patterns (or grep fallback files)
3. Read results before writing code

This prevents repeating past mistakes in the same area.
