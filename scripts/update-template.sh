#!/usr/bin/env bash
# update-template.sh — Update project from newer template version
# Delegates to sync-template.sh for the actual sync logic.
# Usage: ./scripts/update-template.sh /path/to/agent-project-template [--dry-run] [--force]

exec "$(dirname "$0")/sync-template.sh" "$@"
