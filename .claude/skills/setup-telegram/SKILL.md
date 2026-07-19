---
name: setup-telegram
description: "Set up Telegram bot integration for remote control of Claude Code from your phone"
---

# Setup Telegram Skill

## When to Use
When the user wants to control Claude Code sessions from Telegram.

## Overview
Two main options:
1. **CCGram** — best for inline approve/deny buttons, multi-session management
2. **Claude-Code-Remote** — best for hook-based notifications + replies

## Setup Process

### Step 1: Create Telegram Bot
Guide the user:
1. Open Telegram, find @BotFather
2. Send `/newbot`
3. Choose a name and username
4. Copy the bot token

### Step 2: Get User ID
Guide the user:
1. Open Telegram, find @userinfobot
2. Send any message
3. Copy your user ID (numeric)

### Step 3: Choose and Install

#### Option A: CCGram (Recommended)
```bash
npx @jsayubi/ccgram init
```
This runs an interactive wizard that:
- Asks for bot token and user ID
- Installs hooks into ~/.claude/settings.json
- Starts background service

Features:
- Inline buttons for permission approve/deny
- Session management (list, switch, interrupt, resume)
- Smart notification suppression (silent if you're at terminal)

#### Option B: Claude-Code-Remote
```bash
git clone https://github.com/JessyTsui/Claude-Code-Remote.git
cd Claude-Code-Remote
npm install
npm run setup
```
Interactive wizard configures Telegram credentials and hooks.

Features:
- Multi-platform (Telegram + Email + LINE)
- Hook-based architecture
- Start task locally → get notified → reply from phone

### Step 4: Test
1. Start Claude Code with a simple task
2. Check that notification arrives in Telegram
3. Reply from Telegram — verify it reaches Claude Code

### Step 5: Security Notes
- All messages go through Telegram servers — never share passwords/API keys via bot
- ALLOWED_USERS whitelist restricts who can control the bot
- Use per-session spending caps if available

Read `integrations/telegram/README.md` for full details.
