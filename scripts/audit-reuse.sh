#!/bin/bash
# Audit Reuse — detect duplicate code, extraction candidates, stale registry entries
# shellcheck source=lib/platform.sh
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
# Usage:
#   bash scripts/audit-reuse.sh          # full scan + update registry
#   bash scripts/audit-reuse.sh --quick  # lightweight (session-start)
#   bash scripts/audit-reuse.sh --report # report only, no writes

MODE="full"
[ "$1" = "--quick" ] && MODE="quick"
[ "$1" = "--report" ] && MODE="report"

REGISTRY="_reference/tool-registry.md"
SCAN_DIRS=""

# Detect source directories
for d in src lib app packages; do
  [ -d "$d" ] && SCAN_DIRS="$SCAN_DIRS $d"
done

if [ -z "$SCAN_DIRS" ]; then
  echo "No source directories found (src/, lib/, app/, packages/). Nothing to audit."
  exit 0
fi

# Detect stack
detect_import_pattern() {
  if [ -f package.json ] || [ -f tsconfig.json ]; then
    echo "js"
  elif [ -f requirements.txt ] || [ -f pyproject.toml ] || [ -f setup.py ]; then
    echo "python"
  elif [ -f go.mod ]; then
    echo "go"
  elif [ -f Cargo.toml ]; then
    echo "rust"
  else
    echo "generic"
  fi
}

STACK=$(detect_import_pattern)

echo "=== Reuse Audit ($MODE mode) ==="
echo "Stack: $STACK"
echo "Scanning: $SCAN_DIRS"
echo ""

# --- Quick mode: just check registry health ---
if [ "$MODE" = "quick" ]; then
  if [ ! -f "$REGISTRY" ]; then
    echo "WARNING: No tool registry. Run: bash scripts/scan-project.sh"
    exit 0
  fi

  # Count project entries (lines with | that aren't headers or placeholders)
  PROJECT_ENTRIES=$(grep -E "^\| [^_|]" "$REGISTRY" 2>/dev/null | grep -cv "^| Tool\|^| Function\|^| Component" || echo 0)
  if [ "$PROJECT_ENTRIES" -lt 3 ]; then
    echo "WARNING: Tool registry has only $PROJECT_ENTRIES project entries."
    echo "Run: bash scripts/audit-reuse.sh"
  else
    echo "OK: Tool registry has $PROJECT_ENTRIES entries."
  fi

  # Check for stale entries (files referenced but deleted)
  STALE=0
  while IFS='|' read -r _ _ path _; do
    path=$(echo "$path" | xargs 2>/dev/null)
    [ -z "$path" ] && continue
    [ "$path" = "Path" ] && continue
    [[ "$path" == _* ]] && continue
    if [ ! -e "$path" ]; then
      STALE=$((STALE + 1))
    fi
  done < <(grep "^|" "$REGISTRY" 2>/dev/null)

  if [ "$STALE" -gt 0 ]; then
    echo "WARNING: $STALE stale entries in tool registry (files deleted)."
  fi
  exit 0
fi

# --- Full / Report mode ---

PROMOTIONS=0
DUPLICATES=0
UNREGISTERED=0
STALE=0

# 1. Find heavily-imported functions (3+ importers = promote to shared/)
echo "[1/4] Scanning for promotion candidates (imported from 3+ files)..."

case "$STACK" in
  js)
    # Find all import sources and count unique importers
    grep -rhE "^import .+ from ['\"]\.\.?/" $SCAN_DIRS 2>/dev/null | \
      sed -E "s/.*from ['\"]([^'\"]+)['\"].*/\1/" | \
      sort | uniq -c | sort -rn | \
      while read -r count source; do
        if [ "$count" -ge 3 ]; then
          echo "  PROMOTE: $source imported in $count files"
          PROMOTIONS=$((PROMOTIONS + 1))
        fi
      done
    ;;
  python)
    grep -rhE "^from \." $SCAN_DIRS --include="*.py" 2>/dev/null | \
      sed -E "s/^from ([^ ]+) import.*/\1/" | \
      sort | uniq -c | sort -rn | \
      while read -r count source; do
        if [ "$count" -ge 3 ]; then
          echo "  PROMOTE: $source imported in $count files"
          PROMOTIONS=$((PROMOTIONS + 1))
        fi
      done
    ;;
  go)
    grep -rhE "\"[^\"]+\"" $SCAN_DIRS --include="*.go" 2>/dev/null | \
      grep -v "^//" | \
      sed -E 's/.*"([^"]+)".*/\1/' | \
      grep -v "^fmt\|^os\|^io\|^net\|^log\|^strings\|^strconv\|^encoding\|^context\|^sync\|^time\|^testing" | \
      sort | uniq -c | sort -rn | \
      while read -r count source; do
        if [ "$count" -ge 3 ]; then
          echo "  PROMOTE: $source imported in $count files"
        fi
      done
    ;;
  rust)
    grep -rhE "^use (crate|super)::" $SCAN_DIRS --include="*.rs" 2>/dev/null | \
      sed -E "s/^use ([^;]+);.*/\1/" | \
      sort | uniq -c | sort -rn | \
      while read -r count source; do
        if [ "$count" -ge 3 ]; then
          echo "  PROMOTE: $source imported in $count files"
        fi
      done
    ;;
  *)
    echo "  (generic stack — skipping import analysis)"
    ;;
esac

# 2. Find duplicate function definitions
echo ""
echo "[2/4] Scanning for duplicate definitions..."

case "$STACK" in
  js)
    PATTERN="^(export )?(async )?(function |const |let |class )\w+"
    EXT="--include=*.ts --include=*.tsx --include=*.js --include=*.jsx"
    ;;
  python)
    PATTERN="^(def |class )\w+"
    EXT="--include=*.py"
    ;;
  go)
    PATTERN="^func \w+"
    EXT="--include=*.go"
    ;;
  rust)
    PATTERN="^(pub )?(fn |struct |enum |trait )\w+"
    EXT="--include=*.rs"
    ;;
  *)
    PATTERN="^(function |def |func |class |pub fn )\w+"
    EXT=""
    ;;
esac

# Extract function names, find duplicates across files
grep -rhE "$PATTERN" $SCAN_DIRS $EXT 2>/dev/null | \
  sed -E 's/^(export |pub |async )*//; s/^(function |const |let |def |func |class |fn |struct |enum |trait )//; s/[^a-zA-Z0-9_].*//' | \
  sort | uniq -c | sort -rn | \
  while read -r count name; do
    [ -z "$name" ] && continue
    [ ${#name} -lt 4 ] && continue
    if [ "$count" -ge 2 ]; then
      echo "  DUPLICATE: $name defined in $count files"
      DUPLICATES=$((DUPLICATES + 1))
    fi
  done

# 3. Find shared utilities not registered in tool-registry
echo ""
echo "[3/4] Checking for unregistered shared utilities..."

for shared_dir in shared utils helpers common lib; do
  for scan_root in $SCAN_DIRS; do
    dir="$scan_root/$shared_dir"
    [ -d "$dir" ] || continue
    find "$dir" -type f \( -name "*.ts" -o -name "*.js" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) 2>/dev/null | while read -r f; do
      if [ -f "$REGISTRY" ] && ! grep -q "$f" "$REGISTRY" 2>/dev/null; then
        echo "  UNREGISTERED: $f"
        UNREGISTERED=$((UNREGISTERED + 1))
      fi
    done
  done
done

# 4. Check for stale registry entries
echo ""
echo "[4/4] Checking for stale registry entries..."

if [ -f "$REGISTRY" ]; then
  while IFS='|' read -r _ _ path _; do
    path=$(echo "$path" | xargs 2>/dev/null)
    [ -z "$path" ] && continue
    [ "$path" = "Path" ] && continue
    [ "$path" = "Signature" ] && continue
    [[ "$path" == _* ]] && continue
    [[ "$path" == scripts/* ]] && continue  # template scripts checked separately
    if [ ! -e "$path" ]; then
      echo "  STALE: $path (file deleted, remove from registry)"
      STALE=$((STALE + 1))
    fi
  done < <(grep "^|" "$REGISTRY" 2>/dev/null)
fi

# --- Report ---
echo ""
echo "=== Audit Summary ==="
echo "Promotion candidates (3+ imports): check output above"
echo "Duplicate definitions: check output above"
echo "Unregistered shared utils: check output above"
echo "Stale registry entries: $STALE"

if [ "$MODE" = "report" ]; then
  echo ""
  echo "(report mode — no changes written)"
  exit 0
fi

# --- Update registry (full mode only) ---
echo ""
echo "Updating registry timestamp..."

if [ -f "$REGISTRY" ]; then
  # Update last scan date
  if grep -q "^_Last scan:" "$REGISTRY" 2>/dev/null; then
    if command -v _sed_i &>/dev/null; then
      _sed_i "s/^_Last scan:.*/_Last scan: $(date +%Y-%m-%d)_/" "$REGISTRY"
    else
      sed -i "s/^_Last scan:.*/_Last scan: $(date +%Y-%m-%d)_/" "$REGISTRY" 2>/dev/null || true
    fi
  fi
fi

echo "Done. Review output and update _reference/tool-registry.md as needed."
