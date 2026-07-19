#!/usr/bin/env bash
# bootstrap-mcp.sh — Auto-detect, install, and configure MCP servers
# Usage: bash scripts/bootstrap-mcp.sh [--dry-run] [--install] [--check] [--zed] [--with-n8n] [--tool-profile=full]
#
# Modes:
#   (default)    Detect installed servers, merge into .mcp.json
#   --install    Install missing required MCP servers and the selected pinned tool profile
#   --check      Health check configured servers and the selected pinned tool profile
#   --zed        Also generate Zed context_servers config
#   --with-n8n   Explicitly install/start optional n8n workflow automation
#   --tool-profile=core|auto|full  Select the pinned code-intelligence arsenal
#   --dry-run    Show what would be done without writing files
#
# Environment detection:
#   - Claude Code CLI (terminal): writes .mcp.json (mcpServers format)
#   - Zed AI Chat panel: writes to Zed settings.json (context_servers format)
#   - Both can coexist — script handles both when --zed is passed
#
# Merge rules:
#   - Existing servers are PRESERVED (never removed)
#   - Newly detected servers are ADDED if not already present
#   - Deprecated servers (memcp, claude-memory) get "disabled": true

set -euo pipefail

normalize_drive_path() {
  local path="$1"
  case "$path" in
    /[A-Z]/*)
      printf '/%s%s\n' "$(printf '%s' "${path:1:1}" | tr 'A-Z' 'a-z')" "${path:2}"
      ;;
    *)
      printf '%s\n' "$path"
      ;;
  esac
}

SCRIPT_DIR="$(normalize_drive_path "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)")"
[ -f "$SCRIPT_DIR/lib/platform.sh" ] && source "$SCRIPT_DIR/lib/platform.sh"

DRY_RUN=false
DO_INSTALL=false
DO_CHECK=false
DO_ZED=false
DO_N8N=false
TOOL_PROFILE=full

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --install) DO_INSTALL=true ;;
    --check) DO_CHECK=true ;;
    --zed) DO_ZED=true ;;
    --with-n8n) DO_N8N=true ;;
    --tool-profile=*) TOOL_PROFILE="${arg#*=}" ;;
    --help|-h)
      echo "Usage: $0 [--dry-run] [--install] [--check] [--zed] [--with-n8n] [--tool-profile=core|auto|full]"
      echo ""
      echo "  --install   Install missing required MCP servers and the selected tool profile"
      echo "  --check     Verify configured servers and the selected tool profile"
      echo "  --zed       Also configure Zed AI chat panel"
      echo "  --with-n8n  Explicitly install/start optional n8n"
      echo "  --tool-profile  Select core, stack-aware auto, or all ten tools (default: full)"
      echo "  --dry-run   Show what would change without writing"
      exit 0
      ;;
  esac
done

case "$TOOL_PROFILE" in
  core|auto|full) ;;
  *) echo "ERROR: Unknown tool profile: $TOOL_PROFILE"; exit 1 ;;
esac

# --- OS and environment detection ---

detect_os() {
  _detect_os
}

detect_arch() {
  _detect_arch
}

detect_zed_settings_path() {
  local os
  os=$(detect_os)
  case "$os" in
    macos)   echo "$HOME/Library/Application Support/Zed/settings.json" ;;
    linux)   echo "$HOME/.config/zed/settings.json" ;;
    windows) echo "$APPDATA/Zed/settings.json" ;;
  esac
}

is_zed_environment() {
  [ -n "${ZED_TERM:-}" ] || pgrep -x "zed" &>/dev/null 2>&1 || pgrep -x "Zed" &>/dev/null 2>&1
}

create_temp_json_file() {
  _temp_file "mcp-merged"
}

OS=$(detect_os)
ARCH=$(detect_arch)
ENGRAM_VERSION=$(node -e "const c=require('./_reference/code-intelligence-tools.json');console.log(c.tools.find(t=>t.id==='engram').version)" 2>/dev/null || echo "1.19.0")

# --- Check for Node.js (needed for JSON merge) ---
if ! command -v node &>/dev/null; then
  echo "ERROR: Node.js required for JSON merge. Install: https://nodejs.org/"
  exit 1
fi
# Legacy compat: scripts that use $PYTHON will get node
PYTHON="node"

# --- Check for node/npm (needed for context-router MCP) ---
HAS_NODE=false
if command -v node &>/dev/null && command -v npm &>/dev/null; then
  HAS_NODE=true
  echo "Node.js: $(node --version), npm: $(npm --version)"
else
  echo "WARNING: Node.js/npm not found. Context-router MCP will not work."
  echo "  Install: https://nodejs.org/ or: winget install OpenJS.NodeJS.LTS"
fi

# --- Check for docker (needed for n8n) ---
HAS_DOCKER=false
if command -v docker &>/dev/null; then
  HAS_DOCKER=true
  echo "Docker: $(docker --version 2>/dev/null | head -c 40)"
else
  echo "INFO: Docker not found. n8n auto-install unavailable (optional)."
fi

# --- Install helpers ---

install_engram() {
  echo ""
  echo "--- Installing Engram ---"

  # Method 1: Go install (if Go available)
  if command -v go &>/dev/null; then
    echo "Go found. Installing pinned Engram v$ENGRAM_VERSION"
    if GOBIN="$HOME/.local/bin" go install "github.com/Gentleman-Programming/engram/cmd/engram@v$ENGRAM_VERSION" 2>/dev/null; then
      echo "Engram installed via Go."
      return 0
    fi
    echo "go install failed, trying binary download..."
  fi

  # Method 2: Download binary from GitHub releases
  local bin_dir="$HOME/.local/bin"
  mkdir -p "$bin_dir"

  local bin_name="engram" download_os="$OS" archive_ext="tar.gz"
  [ "$OS" = "windows" ] && bin_name="engram.exe"
  [ "$download_os" = "macos" ] && download_os="darwin"
  [ "$OS" = "windows" ] && archive_ext="zip"

  local asset_name="engram_${ENGRAM_VERSION}_${download_os}_${ARCH}.${archive_ext}"
  local release_base="https://github.com/Gentleman-Programming/engram/releases/download/v${ENGRAM_VERSION}"
  local temp_dir archive_path checksums_path
  temp_dir=$(_temp_dir "engram-install")
  archive_path="$temp_dir/$asset_name"
  checksums_path="$temp_dir/checksums.txt"

  echo "Downloading pinned asset: $release_base/$asset_name"
  if command -v curl &>/dev/null; then
    curl -fsSL "$release_base/$asset_name" -o "$archive_path" || return 1
    curl -fsSL "$release_base/checksums.txt" -o "$checksums_path" || return 1
  elif command -v wget &>/dev/null; then
    wget -q "$release_base/$asset_name" -O "$archive_path" || return 1
    wget -q "$release_base/checksums.txt" -O "$checksums_path" || return 1
  else
    echo "ERROR: curl or wget is required to download Engram."
    return 1
  fi

  local expected_hash actual_hash extracted_binary
  expected_hash=$(awk -v asset="$asset_name" '$2 == asset || $2 == "*" asset {print $1}' "$checksums_path")
  actual_hash=$(_get_hash "$archive_path")
  if [ -z "$expected_hash" ] || [ "$expected_hash" != "$actual_hash" ]; then
    echo "ERROR: Engram checksum verification failed."
    return 1
  fi

  if [ "$OS" = "windows" ]; then
    powershell.exe -NoProfile -Command "Expand-Archive -LiteralPath '$archive_path' -DestinationPath '$temp_dir/extracted' -Force" || return 1
  else
    mkdir -p "$temp_dir/extracted"
    tar -xzf "$archive_path" -C "$temp_dir/extracted" || return 1
  fi

  extracted_binary=$(find "$temp_dir/extracted" -type f -name "$bin_name" -print -quit)
  if [ -z "$extracted_binary" ]; then
    echo "ERROR: Engram archive did not contain $bin_name."
    return 1
  fi
  cp "$extracted_binary" "$bin_dir/$bin_name"
  chmod +x "$bin_dir/$bin_name" 2>/dev/null || true
  echo "Engram v$ENGRAM_VERSION installed to $bin_dir/$bin_name"
  echo "NOTE: Make sure $bin_dir is in PATH."
  return 0
}

# --- Detection helpers ---

detect_engram() {
  command -v engram.exe &>/dev/null || command -v engram &>/dev/null || {
    local win_path="$HOME/.local/bin/engram.exe"
    local unix_path="$HOME/.local/bin/engram"
    [ -f "$win_path" ] || [ -f "$unix_path" ]
  }
}

detect_engram_path() {
  if command -v engram.exe &>/dev/null; then
    command -v engram.exe
  elif command -v engram &>/dev/null; then
    command -v engram
  else
    local win_path="$HOME/.local/bin/engram.exe"
    local unix_path="$HOME/.local/bin/engram"
    if [ -f "$win_path" ]; then
      echo "$win_path"
    elif [ -f "$unix_path" ]; then
      echo "$unix_path"
    else
      echo "engram"
    fi
  fi
}

detect_engram_version() {
  local engram_path version_output
  engram_path=$(detect_engram_path)
  version_output=$("$engram_path" --version 2>/dev/null || true)
  printf '%s' "$version_output" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1
}

detect_obsidian() {
  [ -d "brain" ]
}

detect_godot() {
  [ -f "project.godot" ] || [ -f "godot-mcp/build/index.js" ]
}

detect_godot_mcp_path() {
  if [ -f "godot-mcp/build/index.js" ]; then
    echo "godot-mcp/build/index.js"
  else
    echo "godot-mcp/build/index.js"
  fi
}

detect_web_project() {
  if [ -f "package.json" ]; then
    grep -qE '"(react|next|@docusaurus|vue|angular|svelte)"' package.json 2>/dev/null
  else
    return 1
  fi
}

detect_chrome_devtools() {
  detect_web_project && command -v chrome-devtools-mcp &>/dev/null
}

detect_figma() {
  command -v figma-mcp &>/dev/null 2>&1
}

# --- Health check ---

check_engram_health() {
  local engram_path
  engram_path=$(detect_engram_path)
  if [ "$(detect_engram_version)" = "$ENGRAM_VERSION" ] && \
    ("$engram_path" mcp --help &>/dev/null 2>&1 || "$engram_path" --version &>/dev/null 2>&1); then
    return 0
  fi
  return 1
}

run_health_check() {
  echo ""
  echo "=== MCP Health Check ==="

  local all_ok=true

  echo -n "  engram: "
  if detect_engram; then
    if check_engram_health; then
      echo "OK (responds)"
    else
      echo "INSTALLED but NOT RESPONDING"
      all_ok=false
    fi
  else
    echo "NOT INSTALLED (required!)"
    all_ok=false
  fi

  echo -n "  .mcp.json: "
  if [ -f ".mcp.json" ]; then
    if node -e "const m=JSON.parse(require('fs').readFileSync('.mcp.json','utf8')).mcpServers||{};if(!m.engram||!m['codebase-memory-mcp']||!m['context-router'])process.exit(1)" &>/dev/null; then
      echo "OK (required servers configured)"
    else
      echo "CORRUPT (invalid JSON!)"
      all_ok=false
    fi
  else
    echo "MISSING (run bootstrap-mcp.sh first)"
    all_ok=false
  fi

  echo -n "  context-router build: "
  if [ -f "mcp-servers/context-router/dist/index.js" ]; then
    echo "OK"
  else
    echo "MISSING (run bootstrap-mcp.sh without --dry-run)"
    all_ok=false
  fi

  echo -n "  .codex/config.toml: "
  if [ -f "scripts/configure-codex-mcp.js" ] && node scripts/configure-codex-mcp.js --check &>/dev/null; then
    echo "OK (managed MCP block is current)"
  else
    echo "MISSING OR DRIFTED (run bootstrap-mcp.sh first)"
    all_ok=false
  fi

  # Check Zed config if in Zed environment
  if is_zed_environment; then
    local zed_path
    zed_path=$(detect_zed_settings_path)
    echo -n "  zed settings: "
    if [ -f "$zed_path" ]; then
      if node -e "const d=JSON.parse(require('fs').readFileSync('$zed_path','utf8'));if(!d.context_servers)process.exit(1)" &>/dev/null 2>&1; then
        echo "OK (context_servers found)"
      else
        echo "NO context_servers (run with --zed to configure)"
      fi
    else
      echo "NOT FOUND at $zed_path"
    fi
  fi

  echo ""
  if $all_ok; then
    echo "All checks passed."
    return 0
  else
    echo "Some checks FAILED. Fix issues above, then re-run with --check."
    return 1
  fi
}

if [ "$DO_CHECK" = true ]; then
  CHECK_STATUS=0
  run_health_check || CHECK_STATUS=1
  if [ -f "scripts/code-intelligence-tools.js" ]; then
    node scripts/code-intelligence-tools.js check --profile "$TOOL_PROFILE" || CHECK_STATUS=1
  fi
  exit "$CHECK_STATUS"
fi

# --- Main: Detection phase ---

echo "=== MCP Server Bootstrap ==="
echo "OS: $OS | Arch: $ARCH"
echo "Detecting available MCP servers..."
echo ""

DETECTED_SERVERS=""
ENABLED=()
DISABLED=()
ENGRAM_INSTALLED=false

# 1. engram (REQUIRED)
echo -n "  engram: "
if detect_engram; then
  ENGRAM_PATH=$(detect_engram_path)
  CURRENT_ENGRAM_VERSION=$(detect_engram_version)
  if [ "$DO_INSTALL" = true ] && [ "$CURRENT_ENGRAM_VERSION" != "$ENGRAM_VERSION" ]; then
    if [ "$DRY_RUN" = true ]; then
      echo "DRIFT (v${CURRENT_ENGRAM_VERSION:-unknown}; would upgrade to v$ENGRAM_VERSION)"
    else
      echo "DRIFT (v${CURRENT_ENGRAM_VERSION:-unknown}; upgrading to v$ENGRAM_VERSION)"
      install_engram
      ENGRAM_PATH=$(detect_engram_path)
      CURRENT_ENGRAM_VERSION=$(detect_engram_version)
    fi
  else
    echo "ENABLED ($ENGRAM_PATH, v${CURRENT_ENGRAM_VERSION:-unknown})"
  fi
  ENABLED+=("engram")
  ENGRAM_INSTALLED=true
  DETECTED_SERVERS+="engram|{\"command\":\"$ENGRAM_PATH\",\"args\":[\"mcp\"]}
"
else
  if [ "$DO_INSTALL" = true ]; then
    if [ "$DRY_RUN" = true ]; then
      echo "NOT FOUND — would install (--dry-run)"
    else
      install_engram
      # Re-detect after install
      if detect_engram; then
        ENGRAM_PATH=$(detect_engram_path)
        echo "  engram: ENABLED after install ($ENGRAM_PATH)"
        ENABLED+=("engram")
        ENGRAM_INSTALLED=true
        DETECTED_SERVERS+="engram|{\"command\":\"$ENGRAM_PATH\",\"args\":[\"mcp\"]}
"
      else
        echo "  engram: INSTALL FAILED — creating stub (will error at runtime!)"
        ENABLED+=("engram (stub)")
        DETECTED_SERVERS+="engram|{\"command\":\"engram\",\"args\":[\"mcp\"]}
"
      fi
    fi
  else
    echo "NOT FOUND (required!)"
    echo ""
    echo "  To auto-install:  bash scripts/bootstrap-mcp.sh --install --tool-profile=$TOOL_PROFILE"
    echo "  Manual install:   https://github.com/Gentleman-Programming/engram/releases"
    echo ""
    ENABLED+=("engram (stub)")
    DETECTED_SERVERS+="engram|{\"command\":\"engram\",\"args\":[\"mcp\"]}
"
  fi
fi

# 2. codebase-memory-mcp (parser-backed code graph)
echo -n "  codebase-memory-mcp: "
if command -v codebase-memory-mcp &>/dev/null; then
  echo "ENABLED ($(command -v codebase-memory-mcp))"
else
  echo "CONFIGURED (installed by the selected tool profile)"
fi
ENABLED+=("codebase-memory-mcp")
DETECTED_SERVERS+="codebase-memory-mcp|{\"command\":\"codebase-memory-mcp\",\"args\":[],\"env\":{\"CBM_ALLOWED_ROOT\":\".\"}}
"

# 3. obsidian-mcp (only if brain/ exists)
echo -n "  obsidian-mcp: "
if detect_obsidian; then
  echo "ENABLED (brain/ directory found)"
  ENABLED+=("obsidian-mcp")
  DETECTED_SERVERS+="obsidian|{\"command\":\"obsidian-mcp-server\",\"args\":[\"--vault\",\"./brain\"],\"env\":{\"OBSIDIAN_API_KEY\":\"placeholder\"}}
"
else
  echo "DISABLED (no brain/ directory)"
  DISABLED+=("obsidian-mcp")
fi

# 4. godot (only if project.godot exists)
echo -n "  godot: "
if detect_godot; then
  GODOT_PATH=$(detect_godot_mcp_path)
  echo "ENABLED ($GODOT_PATH)"
  ENABLED+=("godot")
  DETECTED_SERVERS+="godot|{\"command\":\"node\",\"args\":[\"$GODOT_PATH\"]}
"
else
  echo "DISABLED (no project.godot — optional)"
  DISABLED+=("godot")
fi

# 5. figma-desktop (optional)
echo -n "  figma-desktop: "
if detect_figma; then
  echo "ENABLED"
  ENABLED+=("figma-desktop")
  DETECTED_SERVERS+="figma-desktop|{\"url\":\"http://127.0.0.1:3845/mcp\"}
"
else
  echo "DISABLED (not installed — optional)"
  DISABLED+=("figma-desktop")
fi

# 6. chrome-devtools (only for web projects)
echo -n "  chrome-devtools: "
if detect_chrome_devtools; then
  echo "ENABLED (web project detected)"
  ENABLED+=("chrome-devtools")
  DETECTED_SERVERS+="chrome-devtools|{\"command\":\"npx\",\"args\":[\"chrome-devtools-mcp@latest\"]}
"
else
  if detect_web_project; then
    echo "DISABLED (web project but chrome-devtools-mcp not installed)"
  else
    echo "DISABLED (not a web project)"
  fi
  DISABLED+=("chrome-devtools")
fi

# 7. context-router (template MCP — dynamic rule loading)
echo -n "  context-router: "
if [ -f "mcp-servers/context-router/package.json" ]; then
  if [ "$DRY_RUN" = true ]; then
    if [ -f "mcp-servers/context-router/dist/index.js" ]; then
      echo "ENABLED (built dynamic rule routing)"
    else
      echo "CONFIGURED (would install dependencies and build)"
    fi
    ENABLED+=("context-router")
    DETECTED_SERVERS+="context-router|{\"command\":\"node\",\"args\":[\"mcp-servers/context-router/dist/index.js\"]}
"
  else
    if [ ! -d "mcp-servers/context-router/node_modules" ]; then
      echo -n "installing deps... "
      (cd mcp-servers/context-router && npm ci --silent 2>/dev/null) || true
    fi
    if [ -d "mcp-servers/context-router/node_modules" ] && \
      (cd mcp-servers/context-router && npm run build --silent >/dev/null 2>&1); then
      echo "ENABLED (built dynamic rule routing)"
      ENABLED+=("context-router")
      DETECTED_SERVERS+="context-router|{\"command\":\"node\",\"args\":[\"mcp-servers/context-router/dist/index.js\"]}
"
    else
      echo "FAILED (dependency install or build failed)"
      DISABLED+=("context-router")
    fi
  fi
else
  echo "DISABLED (mcp-servers/context-router/ not found)"
  DISABLED+=("context-router")
fi

# 8. n8n (workflow automation — optional)
echo -n "  n8n: "
N8N_URL="${N8N_URL:-http://localhost:5678}"
# Check if n8n is already running
if curl -s --connect-timeout 2 "$N8N_URL/healthz" >/dev/null 2>&1 || \
   curl -s --connect-timeout 2 "$N8N_URL/api/v1/workflows" >/dev/null 2>&1; then
  echo "ENABLED (running at $N8N_URL)"
  ENABLED+=("n8n")
elif [ "$DO_N8N" = true ]; then
  # Method 1: npm (lightweight, no Docker overhead)
  if command -v npm &>/dev/null; then
    if command -v n8n &>/dev/null; then
      echo "INSTALLED (npm global). Start with: n8n start"
      ENABLED+=("n8n")
    else
      echo -n "installing via npm... "
      if npm install -g n8n --silent 2>/dev/null; then
        echo "INSTALLED (npm). Start with: n8n start"
        ENABLED+=("n8n")
      else
        echo "npm install failed"
      fi
    fi
  fi
  # Method 2: Docker (if npm failed or unavailable)
  if ! echo "${ENABLED[*]}" | grep -q "n8n" 2>/dev/null; then
    if command -v docker &>/dev/null && docker ps &>/dev/null 2>&1; then
      echo -n "installing via docker... "
      if docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q "^n8n$"; then
        docker start n8n >/dev/null 2>&1 || true
        echo "STARTED (existing container)"
      else
        docker run -d --name n8n -p 5678:5678 \
          -v n8n_data:/home/node/.n8n \
          -e N8N_SECURE_COOKIE=false \
          --restart unless-stopped \
          n8nio/n8n:latest >/dev/null 2>&1 && echo "INSTALLED (docker)" || echo "FAILED"
      fi
      sleep 3
      if curl -s --connect-timeout 5 "$N8N_URL/healthz" >/dev/null 2>&1; then
        ENABLED+=("n8n")
      else
        DISABLED+=("n8n")
      fi
    else
      echo "SKIPPED (no npm or docker available)"
      echo "    Install: npm install -g n8n  OR  docker run n8nio/n8n"
      DISABLED+=("n8n")
    fi
  fi
else
  echo "NOT RUNNING (optional — start with: n8n start OR --with-n8n)"
  DISABLED+=("n8n")
fi

# DEPRECATED
echo -n "  memcp: "
echo "DEPRECATED (will be disabled if present)"
DISABLED+=("memcp")

echo -n "  claude-memory: "
echo "DEPRECATED (will be disabled if present)"
DISABLED+=("claude-memory")

echo -n "  codegraphcontext: "
echo "DEPRECATED (package unavailable; will be disabled if present)"
DISABLED+=("codegraphcontext")

# --- Merge phase: .mcp.json (Claude Code CLI) ---

echo ""
echo "--- Merge (.mcp.json for Claude Code) ---"

EXISTING_JSON="{}"
if [ -f ".mcp.json" ]; then
  EXISTING_JSON=$(cat .mcp.json)
  echo "Found existing .mcp.json — merging (existing servers preserved)"
else
  echo "No existing .mcp.json — creating new"
fi

MERGED_JSON_PATH=$(create_temp_json_file)
MERGED_JSON=$(node -e "
const fs=require('fs');
const existing=JSON.parse(process.argv[1]);
const servers=existing.mcpServers||{};
const lines=process.argv[2].trim().split('\n');
const managed=new Set(['context-router','engram','codebase-memory-mcp']);
const added=[],updated=[],preserved=[];
for(const line of lines){
  if(!line.trim())continue;
  const i=line.indexOf('|');if(i<0)continue;
  const key=line.slice(0,i),val=JSON.parse(line.slice(i+1));
  if(servers[key]&&!managed.has(key)){preserved.push(key);continue;}
  if(servers[key])updated.push(key);else added.push(key);
  servers[key]=val;
}
const deprecated=['memcp','claude-memory','codegraphcontext'],disabled=[];
for(const d of deprecated){if(servers[d]){servers[d].disabled=true;disabled.push(d);}}
if(added.length)process.stderr.write('Added: '+added.join(',')+'\n');
if(updated.length)process.stderr.write('Updated managed: '+updated.join(',')+'\n');
if(preserved.length)process.stderr.write('Preserved: '+preserved.join(',')+'\n');
if(disabled.length)process.stderr.write('Disabled (deprecated): '+disabled.join(',')+'\n');
existing.mcpServers=servers;
fs.writeFileSync(process.argv[3],JSON.stringify(existing,null,2));
" "$EXISTING_JSON" "$DETECTED_SERVERS" "$MERGED_JSON_PATH" 2>&1)

if [ -n "$MERGED_JSON" ]; then
  echo "$MERGED_JSON"
fi

if [ ! -f "$MERGED_JSON_PATH" ]; then
  echo "ERROR: Failed to create merged .mcp.json payload."
  exit 1
fi

MCP_JSON=$(cat "$MERGED_JSON_PATH")
rm -f "$MERGED_JSON_PATH"

echo ""
echo "--- Summary ---"
echo "Enabled:  ${ENABLED[*]}"
echo "Disabled: ${DISABLED[*]}"
echo ""

if [ "$DRY_RUN" = true ]; then
  echo "Would write .mcp.json:"
  echo "$MCP_JSON"
  echo ""
  echo "(Dry run — no files modified)"
else
  if [ -f ".mcp.json" ]; then
    cp ".mcp.json" ".mcp.json.bak"
    echo "Backed up .mcp.json to .mcp.json.bak"
  fi
  echo "$MCP_JSON" > .mcp.json
  echo "Generated .mcp.json"
fi

echo ""
echo "--- Merge (.codex/config.toml for Codex) ---"
if [ "$DRY_RUN" = true ]; then
  node scripts/configure-codex-mcp.js --dry-run
else
  node scripts/configure-codex-mcp.js
fi

# --- Zed configuration ---

if [ "$DO_ZED" = true ] || is_zed_environment; then
  echo ""
  echo "--- Zed AI Chat Configuration ---"
  echo ""
  echo "Zed uses context_servers in its own settings.json (NOT .mcp.json)."
  echo "This is needed ONLY if you use Zed's built-in AI chat panel."
  echo "If you use Claude Code in Zed's terminal — .mcp.json above is enough."
  echo ""

  ZED_SETTINGS_PATH=$(detect_zed_settings_path)

  # Generate context_servers snippet from detected servers
  ZED_SNIPPET=$(node -e "
const lines=process.argv[1].trim().split('\n');
const cs={};
for(const line of lines){
  if(!line.trim())continue;
  const i=line.indexOf('|');if(i<0)continue;
  const key=line.slice(0,i),val=JSON.parse(line.slice(i+1));
  if(val.command){const e={command:{path:val.command,args:val.args||[]}};if(val.env)e.command.env=val.env;cs[key]=e;}
  else if(val.url){cs[key]={url:val.url};}
}
console.log(JSON.stringify({context_servers:cs},null,2));
" "$DETECTED_SERVERS")

  if [ "$DRY_RUN" = true ]; then
    echo "Would add to Zed settings ($ZED_SETTINGS_PATH):"
    echo "$ZED_SNIPPET"
  else
    if [ -f "$ZED_SETTINGS_PATH" ]; then
      # Merge into existing Zed settings
      cp "$ZED_SETTINGS_PATH" "${ZED_SETTINGS_PATH}.bak"
      node -e "
const fs=require('fs');
const settings=JSON.parse(fs.readFileSync('$ZED_SETTINGS_PATH','utf8'));
const snippet=JSON.parse(process.argv[1]);
const cs=settings.context_servers||{};
const newCs=snippet.context_servers||{};
const added=[];
for(const[k,v]of Object.entries(newCs)){if(!cs[k]){cs[k]=v;added.push(k);}}
settings.context_servers=cs;
fs.writeFileSync('$ZED_SETTINGS_PATH',JSON.stringify(settings,null,2));
console.log(added.length?'Added to Zed: '+added.join(','):'Zed settings already up to date.');
" "$ZED_SNIPPET"
      echo "Updated $ZED_SETTINGS_PATH (backup: .bak)"
    else
      echo "Zed settings not found at: $ZED_SETTINGS_PATH"
      echo ""
      echo "Add this to your Zed settings.json manually:"
      echo "$ZED_SNIPPET"
    fi
  fi
fi

# --- Final notes ---

if [ "$ENGRAM_INSTALLED" = false ] && [ "$DO_INSTALL" = false ]; then
  echo ""
  echo "WARNING: Engram not installed. Memory features will not work."
  echo "  Auto-install: bash scripts/bootstrap-mcp.sh --install --tool-profile=$TOOL_PROFILE"
  echo "  Verify after:  bash scripts/bootstrap-mcp.sh --check"
fi

if [ "$DO_INSTALL" = true ] && [ -f "scripts/code-intelligence-tools.js" ]; then
  echo ""
  echo "--- Installing pinned code-intelligence profile: $TOOL_PROFILE ---"
  INSTALL_ARGS=(install --profile "$TOOL_PROFILE")
  [ "$DRY_RUN" = true ] && INSTALL_ARGS+=(--dry-run)
  node scripts/code-intelligence-tools.js "${INSTALL_ARGS[@]}"
fi

echo ""
echo "Done. Next steps:"
echo "  1. Open/trust this project in Codex so project .codex/config.toml is loaded"
echo "  2. Restart Codex after changing MCP configuration"
echo "  3. Verify: bash scripts/bootstrap-mcp.sh --check --tool-profile=$TOOL_PROFILE"
if is_zed_environment && [ "$DO_ZED" != true ]; then
  echo "  4. For Zed AI chat: re-run with --zed flag"
fi
