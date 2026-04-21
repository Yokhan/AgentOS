#!/bin/bash
# AgentOS: Check verify conditions for todos
# Called by sensor_verify_todos every 30s.
# Usage: check-todo-verify.sh <project_dir> <condition_type> <condition_arg1> [condition_arg2]
# Exit code 0 = condition met, non-zero = not met

PROJECT_DIR="$1"
COND_TYPE="$2"
ARG1="$3"
ARG2="$4"

cd "$PROJECT_DIR" || exit 1

case "$COND_TYPE" in
    file_exists)
        [ -f "$ARG1" ] && exit 0 || exit 1
        ;;
    grep_match)
        # ARG1 = glob, ARG2 = pattern
        git grep -q -E "$ARG2" -- "$ARG1" 2>/dev/null && exit 0 || exit 1
        ;;
    command_exits)
        # ARG1 = command, ARG2 = expected exit code (default 0)
        eval "$ARG1" > /dev/null 2>&1
        ACTUAL=$?
        [ "$ACTUAL" -eq "${ARG2:-0}" ] && exit 0 || exit 1
        ;;
    git_changed)
        # ARG1 = path pattern
        [ -n "$(git diff --name-only -- "$ARG1" 2>/dev/null)" ] && exit 0 || exit 1
        ;;
    *)
        echo "Unknown condition: $COND_TYPE"
        exit 1
        ;;
esac
