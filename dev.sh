#!/bin/bash
# Dev helper — kill old instance + rebuild + launch
cd "$(dirname "$0")"

# Kill old instances
if command -v taskkill &>/dev/null; then
  chcp.com 65001 >nul 2>&1
  taskkill //f //im agent-os.exe 2>/dev/null
  sleep 1
else
  pkill -f agent-os 2>/dev/null
  sleep 1
fi

echo "=== Building Agent OS (dev) ==="
cargo tauri dev
