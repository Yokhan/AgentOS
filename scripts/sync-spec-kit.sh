#!/usr/bin/env bash
# Fetch a managed GitHub Spec Kit snapshot into _reference/spec-kit/upstream.
# Usage:
#   bash scripts/sync-spec-kit.sh --latest-tag
#   bash scripts/sync-spec-kit.sh --ref v0.8.13
#   bash scripts/sync-spec-kit.sh --check

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

REPO_URL="https://github.com/github/spec-kit.git"
TARGET_ROOT="_reference/spec-kit"
SNAPSHOT_ROOT="$TARGET_ROOT/upstream"
REF="v0.8.13"
MODE="sync"

usage() {
  cat <<'USAGE'
Usage: bash scripts/sync-spec-kit.sh [options]

Options:
  --ref <ref>       Sync this git ref, tag, branch, or commit.
  --latest-tag     Resolve and sync the latest vX.Y.Z tag.
  --main           Sync upstream main instead of the stable tag.
  --check          Compare local manifest ref with upstream latest tag/main.
  --dry-run        Print what would be synced.
  -h, --help       Show help.
USAGE
}

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "ERROR: Missing required command: $1" >&2
    exit 1
  }
}

latest_tag() {
  git ls-remote --tags "$REPO_URL" "refs/tags/v*" |
    "$NODE" -e '
const fs = require("fs");
const tags = fs.readFileSync(0, "utf8")
  .split(/\r?\n/)
  .map((line) => line.trim().split(/\s+/)[1] || "")
  .map((ref) => ref.replace(/^refs\/tags\//, "").replace(/\^\{\}$/, ""))
  .filter((tag) => /^v\d+\.\d+\.\d+$/.test(tag));
const unique = [...new Set(tags)];
unique.sort((a, b) => {
  const pa = a.slice(1).split(".").map(Number);
  const pb = b.slice(1).split(".").map(Number);
  for (let i = 0; i < 3; i += 1) {
    if (pa[i] !== pb[i]) return pb[i] - pa[i];
  }
  return 0;
});
if (!unique[0]) process.exit(1);
console.log(unique[0]);
'
}

resolve_commit() {
  local ref="$1"
  local lines
  lines=$(git ls-remote "$REPO_URL" "refs/tags/$ref" "refs/tags/$ref^{}" "refs/heads/$ref" "$ref" || true)
  if [ -z "$lines" ]; then
    echo "ERROR: Cannot resolve Spec Kit ref: $ref" >&2
    exit 1
  fi
  printf '%s\n' "$lines" |
    "$NODE" -e "
const fs = require('fs');
const ref = '$ref';
const lines = fs.readFileSync(0, 'utf8').trim().split(/\r?\n/).filter(Boolean);
const parsed = lines.map((line) => {
  const [sha, name] = line.trim().split(/\s+/);
  return { sha, name };
});
const preferred =
  parsed.find((item) => item.name === 'refs/tags/' + ref + '^{}') ||
  parsed.find((item) => item.name === 'refs/heads/' + ref) ||
  parsed.find((item) => item.name === 'refs/tags/' + ref) ||
  parsed[0];
if (!preferred?.sha) process.exit(1);
console.log(preferred.sha);
"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --ref)
      REF="${2:-}"
      [ -n "$REF" ] || { echo "ERROR: --ref requires a value" >&2; exit 1; }
      shift 2
      ;;
    --latest-tag)
      MODE="latest-tag"
      shift
      ;;
    --main)
      REF="main"
      shift
      ;;
    --check)
      MODE="check"
      shift
      ;;
    --dry-run)
      MODE="dry-run"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

need git
need node
NODE="${NODE:-node}"

if [ "$MODE" = "latest-tag" ]; then
  REF="$(latest_tag)"
fi

if [ "$MODE" = "check" ]; then
  if [ ! -f "$TARGET_ROOT/manifest.json" ]; then
    echo "ERROR: Spec Kit manifest missing: $TARGET_ROOT/manifest.json" >&2
    exit 1
  fi

  LOCAL_REF=$("$NODE" -e "const m=require('./$TARGET_ROOT/manifest.json'); console.log(m.ref || '')")
  LOCAL_COMMIT=$("$NODE" -e "const m=require('./$TARGET_ROOT/manifest.json'); console.log(m.commit || '')")
  LATEST_TAG="$(latest_tag)"
  LATEST_COMMIT="$(resolve_commit "$LATEST_TAG")"
  MAIN_COMMIT="$(resolve_commit main)"

  echo "Local ref:       $LOCAL_REF"
  echo "Local commit:    $LOCAL_COMMIT"
  echo "Latest tag:      $LATEST_TAG"
  echo "Latest tag SHA:  $LATEST_COMMIT"
  echo "Main SHA:        $MAIN_COMMIT"

  if [ "$LOCAL_REF" != "$LATEST_TAG" ] || [ "$LOCAL_COMMIT" != "$LATEST_COMMIT" ]; then
    echo "STALE: run bash scripts/sync-spec-kit.sh --latest-tag"
    exit 1
  fi

  echo "Spec Kit snapshot is current with latest stable tag."
  exit 0
fi

COMMIT="$(resolve_commit "$REF")"

if [ "$MODE" = "dry-run" ]; then
  echo "Would sync Spec Kit $REF ($COMMIT) into $SNAPSHOT_ROOT"
  exit 0
fi

TMP_DIR="$(_temp_dir spec-kit-sync)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Fetching Spec Kit $REF ($COMMIT)..."
git clone --depth 1 --branch "$REF" "$REPO_URL" "$TMP_DIR/spec-kit" >/dev/null 2>&1 || {
  git clone --depth 1 "$REPO_URL" "$TMP_DIR/spec-kit" >/dev/null 2>&1
  git -C "$TMP_DIR/spec-kit" checkout "$COMMIT" >/dev/null 2>&1
}

ACTUAL_COMMIT="$(git -C "$TMP_DIR/spec-kit" rev-parse HEAD)"
COMMIT_DATE="$(git -C "$TMP_DIR/spec-kit" log -1 --format=%cI)"

rm -rf "$SNAPSHOT_ROOT"
mkdir -p "$SNAPSHOT_ROOT/docs/reference" "$SNAPSHOT_ROOT/scripts" "$SNAPSHOT_ROOT/integrations"

cp "$TMP_DIR/spec-kit/README.md" "$SNAPSHOT_ROOT/README.md"
cp "$TMP_DIR/spec-kit/LICENSE" "$SNAPSHOT_ROOT/LICENSE"
cp "$TMP_DIR/spec-kit/spec-driven.md" "$SNAPSHOT_ROOT/spec-driven.md"
cp "$TMP_DIR/spec-kit/docs/installation.md" "$SNAPSHOT_ROOT/docs/installation.md"
cp "$TMP_DIR/spec-kit/docs/quickstart.md" "$SNAPSHOT_ROOT/docs/quickstart.md"
cp "$TMP_DIR/spec-kit/docs/upgrade.md" "$SNAPSHOT_ROOT/docs/upgrade.md"
cp "$TMP_DIR/spec-kit/docs/reference/core.md" "$SNAPSHOT_ROOT/docs/reference/core.md"
cp "$TMP_DIR/spec-kit/docs/reference/integrations.md" "$SNAPSHOT_ROOT/docs/reference/integrations.md"
cp "$TMP_DIR/spec-kit/docs/reference/extensions.md" "$SNAPSHOT_ROOT/docs/reference/extensions.md"
cp "$TMP_DIR/spec-kit/docs/reference/presets.md" "$SNAPSHOT_ROOT/docs/reference/presets.md"
cp -R "$TMP_DIR/spec-kit/templates" "$SNAPSHOT_ROOT/templates"
cp -R "$TMP_DIR/spec-kit/scripts/bash" "$SNAPSHOT_ROOT/scripts/bash"
cp -R "$TMP_DIR/spec-kit/scripts/powershell" "$SNAPSHOT_ROOT/scripts/powershell"
cp "$TMP_DIR/spec-kit/integrations/catalog.json" "$SNAPSHOT_ROOT/integrations/catalog.json"
cp "$TMP_DIR/spec-kit/integrations/catalog.community.json" "$SNAPSHOT_ROOT/integrations/catalog.community.json"

mkdir -p "$TARGET_ROOT"
"$NODE" -e "
const fs = require('fs');
const manifest = {
  repo: '$REPO_URL',
  ref: '$REF',
  commit: '$ACTUAL_COMMIT',
  commit_date: '$COMMIT_DATE',
  fetched_at: new Date().toISOString(),
  snapshot_root: '$SNAPSHOT_ROOT',
  update_command: 'bash scripts/sync-spec-kit.sh --latest-tag',
  check_command: 'bash scripts/sync-spec-kit.sh --check',
  deploy_command: 'bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .',
  snapshot_policy: 'Track stable Spec Kit release tags; compare with upstream using --check.'
};
fs.writeFileSync('$TARGET_ROOT/manifest.json', JSON.stringify(manifest, null, 2) + '\n');
"

node scripts/validate-spec-kit.js
echo "Synced Spec Kit $REF ($ACTUAL_COMMIT)."
