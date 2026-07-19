#!/bin/bash
# cache.sh — Session-level cache for expensive computations
# Source this: source scripts/lib/cache.sh

CACHE_DIR=".session-cache"
mkdir -p "$CACHE_DIR" 2>/dev/null

# Check if cache entry is valid (newer than source file)
_cache_valid() {
  local key="$1" source_path="$2"
  local cache_file="$CACHE_DIR/$key"
  [ -f "$cache_file" ] || return 1
  [ -f "$source_path" ] || return 1
  [ "$cache_file" -nt "$source_path" ]
}

# Read from cache
_cache_get() {
  cat "$CACHE_DIR/$1" 2>/dev/null
}

# Write to cache
_cache_set() {
  local key="$1"
  shift
  echo "$@" > "$CACHE_DIR/$key"
}

# Invalidate all caches
_cache_clear() {
  rm -rf "$CACHE_DIR"
  mkdir -p "$CACHE_DIR" 2>/dev/null
}

# Check if any file in directory is newer than cache
_cache_stale_for_dir() {
  local key="$1" dir="$2"
  local cache_file="$CACHE_DIR/$key"
  [ -f "$cache_file" ] || return 0
  local newer
  newer=$(find "$dir" -newer "$cache_file" -type f 2>/dev/null | head -1)
  [ -n "$newer" ]
}
