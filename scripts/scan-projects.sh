#!/bin/bash
# Scan directory for git repos and output JSON-friendly pipe-delimited data
# Usage: bash scripts/scan-projects.sh [directory]
# Default: ~/Documents

SCAN_DIR="${1:-$USERPROFILE/Documents}"
[ -d "$SCAN_DIR" ] || SCAN_DIR="${1:-$HOME/Documents}"
[ -d "$SCAN_DIR" ] || { echo "ERROR: $SCAN_DIR not found"; exit 1; }

cd "$SCAN_DIR" || exit 1

for dir in */; do
  dir="${dir%/}"
  [ -d "$dir/.git" ] || continue
  cd "$dir" || continue

  branch=$(git branch --show-current 2>/dev/null || echo unknown)
  last=$(git log -1 --format="%cr" 2>/dev/null || echo never)
  age=$(git log -1 --format="%ct" 2>/dev/null || echo 0)
  uncommitted=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')

  has_manifest=false
  tpl_ver=none
  if [ -f .template-manifest.json ]; then
    has_manifest=true
    if command -v node &>/dev/null; then
      tpl_ver=$(node -e "console.log(JSON.parse(require('fs').readFileSync('.template-manifest.json','utf8')).template_version||'?')" 2>/dev/null || echo "?")
    fi
  fi

  echo "$dir|$branch|$last|$age|$uncommitted|$has_manifest|$tpl_ver"
  cd ..
done
