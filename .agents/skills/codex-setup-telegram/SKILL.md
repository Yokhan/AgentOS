---
name: codex-setup-telegram
description: "Adapt Telegram or remote-control integration setup for Codex projects without assuming Claude-only commands. Trigger on Telegram bot, remote control, or chatops setup."
---

# Codex Setup Telegram

Telegram integration is optional. Treat it as project infrastructure, not as a default template requirement.

## Workflow

1. Read `integrations/telegram/README.md` if present.
2. Identify which local agent entrypoint will receive commands.
3. Keep tokens and chat IDs in local environment files or secret stores.
4. Add a dry-run or echo-mode test before enabling writes.
5. Document project-specific commands outside template-owned files.

Do not promise autonomous unattended execution unless the project already has explicit safety gates and approval policy for it.
