#!/usr/bin/env bash
# setup.sh — Create a new agent-ready project from this template
# Usage: ./setup.sh [project-name]

set -euo pipefail

# Parse flags and project name
IS_ORCHESTRATOR=false
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --orchestrator) IS_ORCHESTRATOR=true; shift ;;
    *) POSITIONAL_ARGS+=("$1"); shift ;;
  esac
done
set -- "${POSITIONAL_ARGS[@]}"

# Check dependencies
if ! command -v git >/dev/null 2>&1; then
    echo "ERROR: git is not installed. Please install git first."
    exit 1
fi

# Get project name
if [ -n "${1:-}" ]; then
    PROJECT_NAME="$1"
else
    read -rp "Enter project name: " PROJECT_NAME
fi

if [ -z "$PROJECT_NAME" ]; then
    echo "ERROR: Project name cannot be empty."
    exit 1
fi

# Sanitize name
PROJECT_DIR="${PROJECT_NAME// /-}"
PROJECT_DIR="${PROJECT_DIR,,}"

if [ -d "$PROJECT_DIR" ]; then
    echo "ERROR: Directory '$PROJECT_DIR' already exists."
    exit 1
fi

echo "Creating project: $PROJECT_DIR"

# Capture template remote BEFORE changing to project directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_REMOTE=$(cd "$SCRIPT_DIR" && git remote get-url origin 2>/dev/null || echo "")

# Copy template (exclude Command Center files — those stay in template root only)
cp -r "$(dirname "$0")" "$PROJECT_DIR"
# Remove Command Center files that should NOT be in individual projects
rm -rf "$PROJECT_DIR/n8n" "$PROJECT_DIR/start.sh" "$PROJECT_DIR/start.bat" "$PROJECT_DIR/setup.sh" "$PROJECT_DIR/setup.bat" "$PROJECT_DIR/templates"

cd "$PROJECT_DIR"

# Create project-local settings (never touched by template sync)
if [ ! -f .claude/settings.local.json ]; then
  cp .claude/settings.local.json.example .claude/settings.local.json 2>/dev/null || true
fi

# Remove template git history
rm -rf .git

# Generate template manifest
generate_manifest() {
  local manifest=".template-manifest.json"
  local today
  today=$(date +%Y-%m-%d)

  # Determine hash command
  local hash_cmd
  if command -v sha256sum >/dev/null 2>&1; then
    hash_cmd="sha256sum"
  elif command -v shasum >/dev/null 2>&1; then
    hash_cmd="shasum -a 256"
  else
    echo "WARNING: No SHA-256 tool found. Skipping manifest generation."
    return
  fi

  get_hash() {
    $hash_cmd "$1" | awk '{print $1}'
  }

  get_category() {
    local fpath="$1"
    case "$fpath" in
      .claude/settings.json) echo "template" ;;
      .claude/rules/*.md) echo "template" ;;
      .claude/agents/*.md) echo "template" ;;
      .claude/skills/*/SKILL.md) echo "template" ;;
      .claude/commands/*.md) echo "template" ;;
      .claude/hooks/*.sh) echo "template" ;;
      scripts/*.sh) echo "template" ;;
      .editorconfig) echo "template" ;;
      Makefile) echo "template" ;;
      SECURITY.md) echo "template" ;;
      CONTRIBUTING.md) echo "template" ;;
      CLAUDE.md) echo "project" ;;
      tasks/*) echo "project" ;;
      brain/*) echo "project" ;;
      .gitignore) echo "hybrid" ;;
      .vscode/extensions.json) echo "hybrid" ;;
      *) echo "" ;;
    esac
  }

  # Collect all files that belong in the manifest
  local files=()
  local patterns=(
    ".claude/settings.json"
    ".claude/rules/"*.md
    ".claude/agents/"*.md
    ".claude/skills/"*/SKILL.md
    ".claude/commands/"*.md
    ".claude/hooks/"*.sh
    "scripts/"*.sh
    ".editorconfig"
    "Makefile"
    "SECURITY.md"
    "CONTRIBUTING.md"
    "CLAUDE.md"
    ".gitignore"
    ".vscode/extensions.json"
  )

  # Add pattern-matched files
  for pattern in "${patterns[@]}"; do
    for f in $pattern; do
      [ -f "$f" ] && files+=("$f")
    done
  done

  # Add tasks/* and brain/* recursively
  if [ -d "tasks" ]; then
    while IFS= read -r -d '' f; do
      files+=("$f")
    done < <(find tasks -type f -print0 2>/dev/null)
  fi
  if [ -d "brain" ]; then
    while IFS= read -r -d '' f; do
      files+=("$f")
    done < <(find brain -type f -print0 2>/dev/null)
  fi

  # Build JSON
  {
    printf '{\n'
    local tpl_ver
    tpl_ver=$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' "$SCRIPT_DIR/CLAUDE.md" 2>/dev/null || echo "3.0.0")
    printf '  "template_version": "%s",\n' "$tpl_ver"
    printf '  "template_remote": "%s",\n' "$TEMPLATE_REMOTE"
    printf '  "created": "%s",\n' "$today"
    printf '  "updated": "%s",\n' "$today"
    printf '  "files": {\n'

    local first=true
    for f in "${files[@]}"; do
      local cat
      cat=$(get_category "$f")
      [ -z "$cat" ] && continue
      local hash
      hash=$(get_hash "$f")
      if [ "$first" = true ]; then
        first=false
      else
        printf ',\n'
      fi
      printf '    "%s": { "category": "%s", "hash": "%s" }' "$f" "$cat" "$hash"
    done

    printf '\n  }\n'
    printf '}\n'
  } > "$manifest"

  echo "Generated $manifest"
}

generate_manifest

# Initialize new git repo
git init

# Make scripts executable
chmod +x scripts/*.sh 2>/dev/null || true
git update-index --chmod=+x scripts/check-drift.sh 2>/dev/null || true

# Create initial commit
git add -A
git commit -m "chore: initialize project from agent-project-template v2"

# Test hooks compatibility
if [ -f scripts/test-hooks.sh ]; then
  echo "Testing hooks compatibility..."
  bash scripts/test-hooks.sh || echo "WARNING: Some hooks may need adjustment."
fi

# Store template origin for future updates
if [ -n "$TEMPLATE_REMOTE" ]; then
    git remote add template "$TEMPLATE_REMOTE" 2>/dev/null || true
    echo "Template remote added: $TEMPLATE_REMOTE"
    echo "Run 'bash scripts/sync-template.sh --from-git' to check for updates."
fi

# Auto-bootstrap MCP servers (context-router, engram, n8n if docker available)
# If orchestrator — replace CLAUDE.md with orchestrator template
if [ "$IS_ORCHESTRATOR" = true ]; then
  echo "Setting up as ORCHESTRATOR project..."
  ORCH_TEMPLATE="$SCRIPT_DIR/templates/orchestrator/CLAUDE.md"
  if [ -f "$ORCH_TEMPLATE" ]; then
    cp "$ORCH_TEMPLATE" CLAUDE.md
    echo "Orchestrator CLAUDE.md installed"
  fi
  mkdir -p tasks/chats brain
  echo "Orchestrator directories created"
fi

echo ""
echo "--- Auto-bootstrapping MCP servers ---"
if [ -f scripts/bootstrap-mcp.sh ]; then
  bash scripts/bootstrap-mcp.sh --install 2>&1 || echo "WARNING: MCP bootstrap had issues. Run manually: bash scripts/bootstrap-mcp.sh --install"
fi

echo ""
echo "✅ Project '$PROJECT_DIR' created successfully!"
echo ""
if [ "$IS_ORCHESTRATOR" = true ]; then
  echo "Next steps:"
  echo "  1. cd $PROJECT_DIR"
  echo "  2. Open Command Center: cd .. && bash start.sh"
  echo "  3. Dashboard will use this project as orchestrator"
else
  echo "Next steps:"
  echo "  1. cd $PROJECT_DIR"
  echo "  2. Open in Claude Code or Zed"
  echo "  3. Run /setup-project to configure for your stack"
fi
echo ""
echo "Included: 18 library rules, 10 agents, 29 skills, 23 commands, 9 MCP tools"
