#!/usr/bin/env bash
# PreToolUse — Deny access to sensitive file paths
# Blocks Read/Edit/Write on files containing secrets, keys, credentials.
# Inspired by dwarvesf/claude-guardrails deny rules.

[ "${TEST_MODE:-}" = "1" ] && echo "deny-paths: OK (test mode)" && exit 0

FILE="${FILE_PATH:-}"
[ -z "$FILE" ] && exit 0

# Normalize path for matching
NORM=$(echo "$FILE" | tr '\\' '/')

# === Sensitive path patterns ===
BLOCKED=""

case "$NORM" in
  # Environment files
  */.env|*/.env.local|*/.env.production|*/.env.staging|*/.env.*)
    BLOCKED="Environment file (.env)" ;;
  # SSH keys
  */.ssh/*|*/id_rsa*|*/id_ed25519*|*/id_ecdsa*|*/id_dsa*)
    BLOCKED="SSH key file" ;;
  # Credential files
  */credentials.json|*/service-account*.json|*/gcloud/*.json)
    BLOCKED="Cloud credentials file" ;;
  # Key files
  *.pem|*.key|*.p12|*.pfx|*.jks|*.keystore)
    BLOCKED="Private key / certificate file" ;;
  # Auth tokens / secrets
  */.npmrc|*/.pypirc|*/.docker/config.json|*/.kube/config)
    BLOCKED="Package manager / container credentials" ;;
  # Database files
  *.sqlite|*.db|*.sqlite3)
    # Only block if in home or system dirs, not project dirs
    case "$NORM" in
      */home/*|*/Users/*) BLOCKED="Database file in user directory" ;;
    esac ;;
  # Git credentials
  */.git-credentials|*/.gitconfig)
    BLOCKED="Git credentials file" ;;
  # Password managers
  *.kdbx|*.1pif|*/.password-store/*)
    BLOCKED="Password manager file" ;;
  # AWS/Azure/GCP configs
  */.aws/credentials|*/.azure/accessTokens.json|*/.config/gcloud/*)
    BLOCKED="Cloud provider credentials" ;;
esac

if [ -n "$BLOCKED" ]; then
  echo "{\"block\":true,\"message\":\"🛡️ ACCESS DENIED: $BLOCKED — $NORM\"}" >&2
  exit 2
fi

exit 0
