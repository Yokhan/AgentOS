# Contributing Guide

## Quick Start

1. Read `CLAUDE.md` to understand the project
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Make changes following the rules in `.claude/rules/` (router + project-specific)
4. Run `/commit` or ensure: typecheck + lint + tests all pass
5. Open a PR

## Code Style

See `.claude/library/technical/code-style.md`. Key rules:
- No `any` type — use `unknown` + type guards
- Immutability — return new objects, don't mutate
- Files < 375 lines — split by responsibility
- Functions < 30 lines — single responsibility

## Testing

- Tests colocated with source: `module.test.ts`, `test_module.py`
- Coverage targets: `core/` 90%, `features/` 80%, `shared/` 95%
- Run tests: `make test`

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat: add new feature`
- `fix: resolve bug`
- `refactor: restructure without behavior change`
- `docs: update documentation`
- `test: add or update tests`
- `chore: dependency updates, config changes`

## Pull Requests

- Keep PRs small (< 400 lines)
- Title follows commit convention
- Include: what changed, why, how to verify
- All CI checks must pass

## Reporting Issues

Use `brain/templates/bug-report.md` template when filing bugs.
