#!/bin/bash
# AgentOS: Universal project verification script
# Called by Gate Pipeline after each delegation completes.
# Exit code 0 = pass, non-zero = fail. Stdout/stderr captured as verify output.

PROJECT_DIR="${1:-.}"
cd "$PROJECT_DIR" || exit 1

echo "=== Verifying: $(basename "$PROJECT_DIR") ==="

# Auto-detect stack and verify
if [ -f "Cargo.toml" ]; then
    echo "[rust] cargo check..."
    cargo check --message-format=short 2>&1
    RUST_EXIT=$?
    if [ -f "clippy.toml" ] || grep -q "clippy" Cargo.toml 2>/dev/null; then
        echo "[rust] clippy..."
        cargo clippy --message-format=short 2>&1
    fi
    exit $RUST_EXIT
fi

if [ -f "package.json" ]; then
    if grep -q '"test"' package.json; then
        echo "[node] npm test..."
        npm test -- --passWithNoTests 2>&1
        exit $?
    elif grep -q '"build"' package.json; then
        echo "[node] npm run build..."
        npm run build 2>&1
        exit $?
    elif grep -q '"typecheck"' package.json; then
        echo "[node] typecheck..."
        npm run typecheck 2>&1
        exit $?
    fi
    echo "[node] no test/build script found"
    exit 0
fi

if [ -f "pyproject.toml" ] || [ -f "requirements.txt" ]; then
    if [ -f "pyproject.toml" ] && grep -q "mypy" pyproject.toml; then
        echo "[python] mypy..."
        python -m mypy . 2>&1
        exit $?
    fi
    echo "[python] syntax check..."
    python -m py_compile $(find . -name "*.py" -not -path "./.venv/*" | head -20) 2>&1
    exit $?
fi

if [ -f "project.godot" ]; then
    echo "[godot] no headless verify available"
    exit 0
fi

echo "[unknown] no verification script for this project type"
exit 0
