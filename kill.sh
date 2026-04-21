#!/bin/bash
# Kill all Agent OS processes
if command -v taskkill &>/dev/null; then
  chcp.com 65001 >nul 2>&1
  taskkill //f //im agent-os.exe 2>/dev/null
  taskkill //f //im cargo-tauri.exe 2>/dev/null
  echo "Killed Agent OS (Windows)"
else
  pkill -f agent-os 2>/dev/null
  pkill -f cargo-tauri 2>/dev/null
  echo "Killed Agent OS (Unix)"
fi
