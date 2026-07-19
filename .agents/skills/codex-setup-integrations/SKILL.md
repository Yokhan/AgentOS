---
name: codex-setup-integrations
description: "Set up optional integrations for memory, MCP, Codex docs, Telegram-like automation, Obsidian, or task trackers while preserving safe defaults and local-only secrets."
---

# Codex Setup Integrations

## Process

1. Identify the integration and required credentials.
2. Keep secrets out of git.
3. Prefer `scripts/bootstrap-mcp.sh` for MCP setup.
4. Update docs and `.mcp.json` only when the integration is project-owned.
5. Verify the integration with a minimal safe command.
6. Document manual restart or IDE steps.
