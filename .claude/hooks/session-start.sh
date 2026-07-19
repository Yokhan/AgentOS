#!/bin/bash
# Session Start Hook — creates session log, shows reminders, checks template

# Node.js detection (required for JSON parsing)
if command -v node &>/dev/null; then NODE="node"
else NODE=""; fi

# CRITICAL: Node.js is required for template infrastructure
if [ -z "$NODE" ]; then
  echo ""
  echo "============================================="
  echo "  WARNING: Node.js not found"
  echo "============================================="
  echo ""
  echo "  This agent template uses Node.js for:"
  echo "  - Template sync (manifest JSON parsing)"
  echo "  - Drift detection (hash verification)"
  echo "  - MCP servers"
  echo ""
  echo "  Install Node.js: https://nodejs.org/"
  echo "============================================="
  echo ""
  # Don't exit 1 — that would block Claude Code entirely.
  # with degraded functionality.
fi

# TEST_MODE guard
if [ "$TEST_MODE" = "1" ]; then
  echo "test-mode: would create session log and show reminders"
  exit 0
fi

# Session log
if [ -d "brain/01-daily" ]; then
  d=$(date +%Y-%m-%d)
  f="brain/01-daily/${d}.md"
  if [ ! -f "$f" ]; then
    echo "# Session Log: ${d}" > "$f"
  fi
  echo "" >> "$f"
  echo "## Session started: $(date +%H:%M)" >> "$f"
  branch=$(git branch --show-current 2>/dev/null || true)
  if [ -n "$branch" ]; then
    echo "Branch: $branch" >> "$f"
  fi
  echo "" >> "$f"
fi

# Context restore (replaces manual reminders)
if [ -f "scripts/context-restore.sh" ]; then
  bash scripts/context-restore.sh 2>/dev/null || true
else
  # Fallback: manual reminders
  [ -f "tasks/lessons.md" ] && echo "REMINDER: Read tasks/lessons.md"
  [ -f "tasks/current.md" ] && echo "REMINDER: Read tasks/current.md" && head -3 tasks/current.md 2>/dev/null || true
fi

# Context budget check
if [ -f "scripts/measure-context.sh" ]; then
  bash scripts/measure-context.sh --budget 200 2>/dev/null | tail -3
fi

# Tool registry check
if [ -f "_reference/tool-registry.md" ]; then
  REGISTRY_ENTRIES=$(grep -cE "^\| [^_|]" _reference/tool-registry.md 2>/dev/null) || REGISTRY_ENTRIES=0
  if [ "$REGISTRY_ENTRIES" -lt 8 ] && [ -d src ]; then
    echo "WARNING: Tool registry has few entries ($REGISTRY_ENTRIES). Run: bash scripts/audit-reuse.sh"
  fi
elif [ -d src ]; then
  echo "WARNING: No _reference/tool-registry.md found. Run: bash scripts/scan-project.sh"
fi

# PROJECT_SPEC.md freshness check
if [ -f "PROJECT_SPEC.md" ]; then
  # Date math in pure bash (no python needed)
  spec_date=$(grep -E '^[0-9]{4}-[0-9]{2}-[0-9]{2}' PROJECT_SPEC.md 2>/dev/null | tail -1 | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}')
  if [ -n "$spec_date" ]; then
    now_epoch=$(date +%s 2>/dev/null || echo 0)
    spec_epoch=$(date -d "$spec_date" +%s 2>/dev/null || date -j -f "%Y-%m-%d" "$spec_date" +%s 2>/dev/null || echo 0)
    if [ "$now_epoch" -gt 0 ] && [ "$spec_epoch" -gt 0 ]; then
      days_old=$(( (now_epoch - spec_epoch) / 86400 ))
      if [ "$days_old" -gt 7 ]; then
        echo "WARNING: PROJECT_SPEC.md is ${days_old} days old. Regenerate it (see .claude/library/process/context-first.md)."
      fi
    fi
  fi
else
  echo "ACTION: PROJECT_SPEC.md not found. Generate it now (see .claude/library/process/context-first.md)."
fi

# Uncommitted changes warning
unstaged=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [ "$unstaged" -gt 0 ] 2>/dev/null; then
  echo "WARNING: $unstaged uncommitted changes from previous session"
fi

# Backup current.md
if [ -f "tasks/current.md" ]; then
  cp tasks/current.md tasks/.current.md.bak 2>/dev/null || true
fi

# Template version check
if [ -f ".template-manifest.json" ] && [ -f "scripts/check-drift.sh" ]; then
  proj_ver=$(node -e "console.log(JSON.parse(require('fs').readFileSync('.template-manifest.json','utf8')).template_version)" 2>/dev/null || true)
  script_ver=$(grep -o 'TEMPLATE_VERSION="[^"]*"' scripts/check-drift.sh 2>/dev/null | head -1 | cut -d'"' -f2 || true)
  if [ -n "$proj_ver" ] && [ -n "$script_ver" ] && [ "$proj_ver" != "$script_ver" ]; then
    echo "WARNING: Template outdated ($proj_ver vs $script_ver). Run /update-template."
  fi
fi

# Template fetch check (lightweight, no network)
if git remote get-url template >/dev/null 2>&1; then
  last_fetch=$(git log -1 --format=%ct refs/remotes/template/main 2>/dev/null || git log -1 --format=%ct refs/remotes/template/master 2>/dev/null || echo 0)
  now=$(date +%s 2>/dev/null || echo 0)
  if [ "$now" -gt 0 ] && [ "$last_fetch" -gt 0 ]; then
    days_since=$(( (now - last_fetch) / 86400 ))
    if [ "$days_since" -gt 7 ]; then
      echo "INFO: Template not checked in ${days_since}d."
    fi
  fi
fi

# Task queue check
if [ -f tasks/queue.md ] && [ -s tasks/queue.md ]; then
  QUEUED=$(grep -c "^## Queued:" tasks/queue.md 2>/dev/null) || QUEUED=0
  if [ "$QUEUED" -gt 0 ]; then
    echo "QUEUE: $QUEUED queued task(s) in tasks/queue.md — review before starting"
  fi
fi

# Compaction snapshot recovery hint
if [ -f tasks/.compaction-snapshot.md ]; then
  echo "RECOVERY: Compaction snapshot found. Read tasks/.compaction-snapshot.md if context was lost."
fi

# Lessons count (learning velocity indicator)
if [ -f tasks/lessons.md ]; then
  LESSON_COUNT=$(grep -c "^### " tasks/lessons.md 2>/dev/null) || LESSON_COUNT=0
  if [ "$LESSON_COUNT" -gt 40 ]; then
    echo "INFO: $LESSON_COUNT lessons in lessons.md — consider running /weekly to promote patterns to rules"
  fi
fi

# Agent frontmatter validation
if [ -d ".claude/agents" ]; then
  AGENT_ERRORS=0
  for agent_file in .claude/agents/*.md; do
    [ -f "$agent_file" ] || continue
    # Skip PROTOCOL.md — it's a shared protocol doc, not an agent definition
    [ "$(basename "$agent_file")" = "PROTOCOL.md" ] && continue
    # Check frontmatter exists (starts with ---)
    first_line=$(head -1 "$agent_file" 2>/dev/null)
    if [ "$first_line" != "---" ]; then
      echo "ERROR: $agent_file missing frontmatter (no --- on line 1)"
      AGENT_ERRORS=$((AGENT_ERRORS + 1))
      continue
    fi
    # Check required fields: name, model
    has_name=$(head -10 "$agent_file" | grep -c "^name:" 2>/dev/null) || has_name=0
    has_model=$(head -10 "$agent_file" | grep -c "^model:" 2>/dev/null) || has_model=0
    if [ "$has_name" -eq 0 ] || [ "$has_model" -eq 0 ]; then
      echo "ERROR: $agent_file frontmatter missing name or model field"
      AGENT_ERRORS=$((AGENT_ERRORS + 1))
    fi
  done
  if [ "$AGENT_ERRORS" -gt 0 ]; then
    echo "WARNING: $AGENT_ERRORS agent(s) have broken frontmatter — they won't launch correctly"
  fi
fi

# MCP health check: verify Engram is reachable
if [ -f ".mcp.json" ]; then
  HAS_ENGRAM=$(node -e "const d=JSON.parse(require('fs').readFileSync('.mcp.json','utf8'));console.log(d.mcpServers?.engram&&!d.mcpServers.engram.disabled?'yes':'no')" 2>/dev/null || echo "no")
  if [ "$HAS_ENGRAM" = "yes" ]; then
    ENGRAM_CMD=$(node -e "console.log(JSON.parse(require('fs').readFileSync('.mcp.json','utf8')).mcpServers.engram.command)" 2>/dev/null || echo "")
    if [ -n "$ENGRAM_CMD" ] && ! command -v "$ENGRAM_CMD" &>/dev/null 2>&1; then
      echo "WARNING: Engram configured but binary not found ($ENGRAM_CMD). Memory will use file fallback."
      echo "  Fix: bash scripts/bootstrap-mcp.sh --install"
    fi
  fi
else
  echo "INFO: No .mcp.json found. Run: bash scripts/bootstrap-mcp.sh --install"
fi

# Memory fallback file — import reminder
if [ -f tasks/.memory-fallback.md ] && [ -s tasks/.memory-fallback.md ]; then
  FALLBACK_COUNT=$(grep -c "^## " tasks/.memory-fallback.md 2>/dev/null) || FALLBACK_COUNT=0
  if [ "$FALLBACK_COUNT" -gt 0 ]; then
    echo "RECOVERY: $FALLBACK_COUNT entries in tasks/.memory-fallback.md — import to Engram when available"
  fi
fi

exit 0
