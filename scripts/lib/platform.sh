#!/bin/bash
# platform.sh - Cross-platform helper functions
# Source this file: source "$(dirname "$0")/lib/platform.sh"
# All scripts MUST use these helpers instead of calling node/sed -i/date -I directly.

# --- OS, architecture, and temp path detection ---
# Keep raw uname and platform-specific temp path logic here only.
_detect_os() {
  local raw_os
  raw_os="$(uname -s 2>/dev/null || printf '%s' "${OS:-Windows}")"
  case "$raw_os" in
    Linux*) echo "linux" ;;
    Darwin*) echo "macos" ;;
    MINGW*|MSYS*|CYGWIN*|Windows*|win32*) echo "windows" ;;
    *)
      if [ -n "${OS:-}" ] && printf '%s' "$OS" | grep -qi "windows"; then
        echo "windows"
      else
        echo "unknown"
      fi
      ;;
  esac
}

_detect_arch() {
  local raw_arch
  raw_arch="$(uname -m 2>/dev/null || echo "x86_64")"
  case "$raw_arch" in
    x86_64|amd64) echo "amd64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) echo "$raw_arch" ;;
  esac
}

_is_windows() {
  [ "$(_detect_os)" = "windows" ]
}

_to_shell_path() {
  local input="$1"
  if command -v cygpath >/dev/null 2>&1 && printf '%s' "$input" | grep -qE '^[A-Za-z]:\\'; then
    cygpath -u "$input"
  else
    printf '%s\n' "$input"
  fi
}

_temp_base_dir() {
  local base_dir=""
  if [ -n "${TMPDIR:-}" ]; then
    base_dir="$TMPDIR"
  elif _is_windows; then
    base_dir="${TEMP:-${TMP:-.}}"
  else
    base_dir="/tmp"
  fi

  base_dir="$(_to_shell_path "$base_dir")"
  mkdir -p "$base_dir" 2>/dev/null || base_dir="."
  printf '%s\n' "$base_dir"
}

_temp_file() {
  local prefix="${1:-tmp}"
  local base_dir
  base_dir="$(_temp_base_dir)"

  if command -v mktemp >/dev/null 2>&1; then
    mktemp "$base_dir/${prefix}.XXXXXX"
  else
    printf '%s/%s.%s\n' "$base_dir" "$prefix" "$$"
  fi
}

_temp_dir() {
  local prefix="${1:-tmp}"
  local base_dir temp_dir
  base_dir="$(_temp_base_dir)"

  if command -v mktemp >/dev/null 2>&1; then
    mktemp -d "$base_dir/${prefix}.XXXXXX"
    return
  fi

  temp_dir="$base_dir/${prefix}.$$"
  mkdir -p "$temp_dir"
  printf '%s\n' "$temp_dir"
}

# --- Node.js detection ---
# Required for JSON parsing. Python is NOT used.
if [ -z "${NODE:-}" ]; then
  if command -v node &>/dev/null; then
    NODE="node"
  else
    NODE=""
  fi
  export NODE
fi

# Run node with auto-detected binary. Usage: _node -e "console.log(1)"
_node() {
  if [ -z "$NODE" ]; then
    echo "ERROR: Node.js not found. Install Node.js: https://nodejs.org/" >&2
    return 1
  fi
  "$NODE" "$@"
}

# JSON helpers via node (replacing python json module)
# Usage: _json_get file.json "key" -> prints value
_json_get() {
  local file="$1" key="$2"
  _node -e "const d=JSON.parse(require('fs').readFileSync('$file','utf8'));const v=$key;console.log(typeof v==='object'?JSON.stringify(v):v??'')" 2>/dev/null
}

# Usage: _json_set file.json '{"key":"value"}' -> merges into file
_json_set() {
  local file="$1" patch="$2"
  _node -e "
const fs=require('fs');
let d={};try{d=JSON.parse(fs.readFileSync('$file','utf8'))}catch{}
Object.assign(d,JSON.parse('$patch'));
fs.writeFileSync('$file',JSON.stringify(d,null,2));
" 2>/dev/null
}

# Usage: _json_valid file.json -> exit 0 if valid, 1 if not
_json_valid() {
  _node -e "JSON.parse(require('fs').readFileSync('$1','utf8'))" 2>/dev/null
}

# Portable sed -i (macOS requires -i '', GNU requires -i)
_sed_i() {
  if sed --version 2>/dev/null | grep -q GNU; then
    sed -i "$@"
  else
    # macOS/BSD sed
    sed -i '' "$@"
  fi
}

# Portable ISO date (macOS date lacks -I flag)
_date_iso() {
  date -u "+%Y-%m-%dT%H:%M:%S" 2>/dev/null || date "+%Y-%m-%dT%H:%M:%S"
}

# Portable file modification time in epoch seconds
_stat_mtime() {
  local file="$1"
  if stat --version 2>/dev/null | grep -q GNU; then
    stat -c %Y "$file" 2>/dev/null
  elif [ "$(_detect_os)" = "macos" ]; then
    stat -f %m "$file" 2>/dev/null
  else
    _node -e "console.log(Math.floor(require('fs').statSync('$file').mtimeMs/1000))" 2>/dev/null || echo 0
  fi
}

# Cross-platform SHA-256
_get_hash() {
  local file="$1"
  if command -v sha256sum &>/dev/null; then
    sha256sum "$file" | cut -d' ' -f1
  elif command -v shasum &>/dev/null; then
    shasum -a 256 "$file" | cut -d' ' -f1
  else
    certutil -hashfile "$file" SHA256 2>/dev/null | sed -n '2p' | tr -d ' ' | tr 'A-F' 'a-f'
  fi
}

# Check if a command exists
_require() {
  local cmd="$1"
  local msg="${2:-$cmd is required but not found}"
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: $msg"
    return 1
  fi
}

# Require node (with helpful error)
_require_node() {
  if [ -z "$NODE" ]; then
    echo "ERROR: Node.js is required but not found."
    echo "Install Node.js: https://nodejs.org/"
    return 1
  fi
}

# Legacy aliases (for backward compatibility during migration)
PYTHON="${NODE:-}"
_python() { _node "$@"; }
_require_python() { _require_node; }
