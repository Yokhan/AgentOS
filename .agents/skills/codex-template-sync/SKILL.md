---
name: codex-template-sync
description: "Maintain or run template sync, bootstrap, downstream migration, manifest, allowlist, and release trust flows without touching user-owned files. Trigger on sync-template, setup payload, migration, or downstream update work."
---

# Codex Template Sync

Template sync is shared release infrastructure. Treat changes as MEDIUM/HIGH risk.

## Process

1. Read `docs/PRODUCT_BOUNDARY.md`, `docs/SAFE_DEFAULTS.md`, and `docs/SUPPORTED_ENVIRONMENTS.md`.
2. Use `docs/TEMPLATE_RELEASES.md#canonical-agent-update-protocol` as the update SOT.
3. Classify source versus generated/legacy downstream before choosing a command.
4. Read installed version from manifest and resolve one explicit stable tag; user/AgentOS target wins.
5. Verify remote, worktree, ownership, and overlays; never silently replace a conflicting remote.
6. Run pinned dry-run and apply with the same tag. Bare `--from-git` is canary-only.
7. If local sync is stale/broken, use the target release checkout's script with `--project-dir`.
8. Verify manifest target, diff, `*.template-new`, overlays, and checks before claiming success.
9. Preserve project-owned files and update Unix and Windows paths together.
10. Add smoke coverage, then run template, drift, routing, and focused sync gates.
