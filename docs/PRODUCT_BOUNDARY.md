# Product Boundary

What belongs to the template, what belongs to a generated project, and what must stay local-only.

## Shipped To Child Projects

These files and directories are part of the generated project payload:

- `.agents/`, `.claude/`, `.codex/`, `.github/`, `.vscode/extensions.json`
- `_reference/`, `brain/`, `docs/`, `integrations/`, `mcp-servers/`, `scripts/`, `tasks/`, `tests/`
- Root project files such as `AGENTS.md`, `CLAUDE.md`, `DESIGN.md`, `design-policy.ignore`, `README.md`, `SETUP_GUIDE.md`, `PROJECT_SPEC.md`, `ecosystem.md`, `.mcp.json`

Bootstrap copies only tracked files plus starter overlays from `templates/project-starter/`.

## Maintainer-Only State

These may exist in the template workspace but must never ship to child projects:

- `templates/`
- `n8n/`
- local fixtures and smoke projects
- debug and recovery logs such as `tasks/debug-recovery-log.md`
- maintainer research and audit history
- dependency artifacts such as `mcp-servers/*/node_modules` and `mcp-servers/*/dist`
- project-local settings such as `.claude/settings.local.json`

## Template-Owned Vs Project-Owned

Template-owned files are updated by `sync-template.sh` and should be treated as baseline infrastructure:

- `.claude/`
- `.agents/skills/`
- `.codex/agents/`
- `.codex/`
- shipped scripts and MCP helper sources
- release-facing bootstrap docs such as `README.md` and `SETUP_GUIDE.md`
- `_reference/tool-registry.md` and generated bootstrap metadata

Project-owned files are expected to evolve per repo and are preserved by template sync:

- `CLAUDE.md`
- `DESIGN.md`
- `design-policy.ignore`
- `PROJECT_SPEC.md`
- `ecosystem.md`
- `tasks/`
- `brain/`
- all `project-*` overlays under `.claude/`
- all `project-*` Codex skills under `.agents/skills/`
- all `project-*` Codex agents under `.codex/agents/`

## Local-Only Extension Surface

Use these for per-machine or per-repo customization without forking template-owned files:

- `.claude/settings.local.json`
- local MCP installation state
- runtime caches under `.memory/`, `.engram/`, `.session-cache/`
- route state such as `tasks/.active-rules` and `tasks/.active-codex-route.json`
- IDE-specific user settings outside the repo

## Release Rule

Before calling the template release-ready:

- generated projects must not contain maintainer-only or machine-local files
- template-owned docs must match the real bootstrap and update flow
- project-owned files must remain preserved by sync
- managed upstream snapshots such as `_reference/spec-kit/` must include a manifest and validator
