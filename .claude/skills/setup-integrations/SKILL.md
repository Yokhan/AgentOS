---
name: setup-integrations
description: "Set up optional integrations: memory MCP, Telegram bot, Beads task tracker, Obsidian MCP. Trigger when user asks about integrations, memory, telegram, or persistent storage."
---

# Setup Integrations Skill

Guide user through setting up optional integrations. All work WITHOUT Ollama/Docker/GPU.

## Available Integrations

### 1. Memory MCP (persistent memory across sessions)
Read `integrations/memory-mcp/README.md` for full details.

**Quick setup (Engram — simplest):**
1. Download Go binary from GitHub releases
2. `claude mcp add engram -- engram mcp`
3. Done. Agent now has `mem_save` and `mem_search`.

**Auto-install (detects OS, downloads binary):**
```bash
bash scripts/bootstrap-mcp.sh --install
```

**Verify installation:**
```bash
bash scripts/bootstrap-mcp.sh --check
```

**For Zed AI chat panel (in addition to Claude Code):**
```bash
bash scripts/bootstrap-mcp.sh --install --zed
```

### 2. Telegram Bot (remote control from phone)
Read `integrations/telegram/README.md` for full details.

1. Create bot via @BotFather on Telegram
2. Get bot token
3. Install chosen package (CCGram for inline buttons, or simpler alternatives)
4. Configure .env with TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID
5. Set up hooks for notifications (PR ready, tests failed, sprint complete)

### 3. Beads (git-native task tracker)
Read `integrations/beads/README.md` for full details.

1. `curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash`
2. `bd init` in project root
3. `bd setup claude` for Claude Code hooks
4. Add Inter-session memory section to CLAUDE.md

### 4. Obsidian MCP (direct vault access)
Read `integrations/obsidian-mcp/README.md` for full details.

1. `npm install -g @stevenstav/obsidian-mcp`
2. `claude mcp add obsidian -- obsidian-mcp --vault ./brain`
3. Claude can now read/write vault directly via MCP

## After Setup
- Update `CLAUDE.md` with active integrations
- Update `.claude/settings.local.json` with MCP server configs

## Verify Installation
After each integration, verify it actually works:
- **Memory MCP**: Save a test memory, then retrieve it by keyword — confirm match returned
- **Telegram**: Send `/start` to the bot — confirm response received within 10s
- **Beads**: Run `bd list` — confirm no errors and project is initialized
- **Obsidian MCP**: Query a known note title — confirm content returned correctly

If verification fails: check logs, re-read the integration README, and troubleshoot before proceeding.
