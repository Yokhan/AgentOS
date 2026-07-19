# Safe Defaults

The template ships conservative defaults so a fresh project is safe to copy, inspect, and update.

## Codex Config

- `.codex/config.toml` owns project toggles and a marker-bounded MCP block. Model, effort, approval, and sandbox stay in IDE or user-level config.
- `_reference/codex-mcp-config.toml` is the merge source of truth. `scripts/configure-codex-mcp.js` changes only the marker-bounded block, preserves project-owned Codex settings, and fails on conflicting unmanaged MCP tables.
- Codex loads project `.codex/config.toml` only after the project is trusted. Restart Codex after changing the MCP block, then verify with `codex mcp list`.
- Codex repo-scoped skills live in `.agents/skills/`; template-owned skills sync normally, while `project-*` skills are project-owned.
- Codex subagents live in `.codex/agents/`; template-owned agents sync normally, while `project-*` agents are project-owned.
- `scripts/codex-agent-policy.js` is the source of truth for template-owned role profiles and the `xhigh` reasoning ceiling. Parent model defaults remain user-owned.
- Codex route selection is explicit through `scripts/codex-route-task.js`; route state is local-only under `tasks/.active-codex-route.json`.
- Codex automatically starts `required` and genuinely useful `recommended` independent lanes, notifies the user, and honors explicit opt-out. Fan-out defaults to read-only workers; `implementer` is only for exact non-overlapping file scopes.
- Agent infrastructure changes must check `docs/AGENT_CONTEXT_SOT.md` and `_reference/agent-sot/sources.json`, then pass `node scripts/validate-agent-sot.js`.
- Spec Kit ships as an inert snapshot under `_reference/spec-kit/`; projects opt in by running `scripts/init-spec-kit.sh`.
- `.claude/settings.local.json` is local-only and must not ship in the template payload.
- Shared rules live under `.claude/library/`; project-specific additions use `project-*` files.

## MCP And Code-Intelligence Defaults

- `.codex/config.toml` configures the local process `context-router`, Engram for
  durable decisions, and the parser-backed `codebase-memory-mcp` graph for Codex.
  `.mcp.json` mirrors them only for Claude Code compatibility. The router is
  infrastructure and is not counted among the ten code-intelligence tools.
- `_reference/code-intelligence-tools.json` owns ten pinned capabilities. The
  default `full` profile installs all ten; `auto` remains a smaller opt-in. Only
  Engram and codebase-memory are persistent code-intelligence MCP surfaces.
- `bootstrap-mcp.sh` downloads the pinned third-party tool profile only through
  an explicit `--install --tool-profile=core|auto|full` command. A plain run
  builds the local router and merge-checks both MCP formats; `--dry-run` writes
  and installs nothing.

## Bootstrap Defaults

- `setup.sh` and `setup.bat` copy only tracked project-facing files.
- Starter overlays reset root `DESIGN.md`, `design-policy.ignore`, `tasks/current.md`, `tasks/goal.md`, `tasks/.research-cache.md`, `tasks/lessons.md`, and empty knowledge/audit folders.
- Maintainer logs, local fixtures, dependency artifacts, and machine-specific settings are excluded from generated projects.

## Update Contract

- `sync-template.sh` updates template-owned files from the manifest.
- `project-*` files, `.agents/skills/project-*`, `.codex/agents/project-*`, `CLAUDE.md`, `DESIGN.md`, `design-policy.ignore`, `PROJECT_SPEC.md`, `ecosystem.md`, `tasks/`, and `brain/` remain project-owned.
- When both local and template versions changed, sync writes `*.template-new` instead of overwriting silently.
