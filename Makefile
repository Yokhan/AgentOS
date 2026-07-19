# Makefile — Common project operations
# Usage: make [target]

.PHONY: help setup test lint format check clean drift health

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Run project setup (configure for your stack)
	@echo "Run /setup-project in Claude Code to configure for your stack"

test: ## Run tests
	@npm test 2>/dev/null || pytest 2>/dev/null || go test ./... 2>/dev/null || cargo test 2>/dev/null || echo "No test runner configured. Run /setup-project first."

lint: ## Run linter
	@npx eslint . 2>/dev/null || ruff check . 2>/dev/null || golangci-lint run 2>/dev/null || cargo clippy 2>/dev/null || echo "No linter configured. Run /setup-project first."

format: ## Run formatter
	@npx prettier --write . 2>/dev/null || black . 2>/dev/null || gofmt -w . 2>/dev/null || cargo fmt 2>/dev/null || echo "No formatter configured. Run /setup-project first."

check: ## Run typecheck + lint + test
	@$(MAKE) lint && $(MAKE) test

drift: ## Check for architectural drift
	@bash scripts/check-drift.sh

health: ## Full project health check (drift + coverage + security)
	@echo "=== DRIFT ===" && bash scripts/check-drift.sh || true
	@echo "" && echo "=== SECURITY ===" && npm audit 2>/dev/null || pip audit 2>/dev/null || cargo audit 2>/dev/null || echo "No audit tool found"

clean: ## Clean build artifacts
	@rm -rf dist/ out/ .cache/ coverage/ htmlcov/ __pycache__/ target/ 2>/dev/null || true
	@echo "Cleaned build artifacts"

brain-search: ## Search Obsidian vault (usage: make brain-search QUERY="search term")
	@bash scripts/brain-search.sh "$(QUERY)"
