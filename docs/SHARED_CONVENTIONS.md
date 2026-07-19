# Shared Conventions (Agent-Agnostic)

> These conventions apply to ALL agents (Claude Code, Codex, future agents).
> Referenced by both `CLAUDE.md` and `AGENTS.md`.

Last reviewed: 2026-07-19 for template `4.9.0`; product/business outcome priority, client/executor accountability, truthful progressive JPEG planning and replacement, evidence-bound change strategy, `PROGRESSIVE_STATUS` project-slice reporting, semantic intent routing, one-wave GPT-5.6 fan-out with genuine child evidence, canonical pinned-tag updates, writing-mode and external-tool truth contracts, Codex MCP delivery, text/platform policy, CI hygiene, production design QA, register-aware design checks, screen anatomy, fixture sync, git dry-run preview, and agent-safe GitHub release entrypoints are enforced by validators.

## Product And Business Outcome Priority

Plans and improvements must name the product user and product/business outcome before the technical mechanism. The first priority is the user's experience and the app-specific business outcome: revenue, monetization, conversion, activation, retention, loyalty, support load, sales, or another KPI the application actually uses.

Technical perfection, refactors, tooling, framework changes, and architecture cleanup are second-order work unless they directly unlock, protect, or measurably improve that outcome.

## Client Executor Accountability

Treat the user as the client or product owner and the agent as the accountable executor. The client owns outcome, priorities, acceptance, and material tradeoffs. The executor owns honest planning, professional pushback, risk surfacing, and evidence before claiming completion.

Agreement is not the default. If a request conflicts with evidence, user outcome, safety, privacy, quality, platform constraints, or app-specific KPI, the agent must challenge it before acting.

Never claim tests passed, research was checked, review was completed, a release was published, or work is done unless there is fresh evidence or a cited existing artifact. If the evidence is missing, say what is verified, what is not verified, and what the next check is.

## Change Strategy Gate

During initial reading, a bounded repair-path check may reveal architecture drift, wrong ownership/SOT, duplicate state, or an obsolete path; invoke `.claude/library/process/change-strategy-gate.md` before the first patch when that evidence is causal. A second failed repair is the mandatory fallback breaker. Preserve verified user, data, public, security, and operational contracts, not old implementation. Choose the destination separately from the transition, compare alternatives against one evidence baseline, and require client approval only when the approved product, contract, data, security, release, cost, timeline, or reversibility envelope changes.

The gate is an overlay on the active bugfix, feature, migration, product, or strategy pipeline. Record it in the active orchestrator artifact; optional JSON records must pass `node scripts/validate-change-strategy.js`.

## Functions-in-Modules Pattern

All business logic lives in importable modules. Entry points only import and call.

### Rules

1. **Entry points** (`main.py`, `index.ts`, `cli.go`, `app.ts`, `run.sh`) contain ONLY:
   - Argument/config parsing
   - Imports from modules
   - Function calls
   - Process exit handling

2. **Modules** contain all business logic:
   - Pure functions preferred (same input → same output)
   - Side effects isolated at module boundaries
   - Each module is independently testable via import

3. **Threshold**: If an entry point exceeds **30 lines** of non-import code → extract logic to a module.

### Why

- **Testability**: Test business logic by importing — no need to spawn a process
- **Reusability**: Same logic callable from CLI, API, script, or test
- **Readability**: Entry point reads like a table of contents
- **Agent-agnostic**: Both Claude and Codex produce consistent structure

### Examples

**Bad** — logic in entry point:
```python
# main.py
import sys
import json

data = json.load(open(sys.argv[1]))
results = []
for item in data:
    if item["status"] == "active":
        score = item["value"] * 0.85 + item["bonus"]
        if score > 100:
            results.append({"id": item["id"], "score": score})
# ... 50 more lines of business logic
print(json.dumps(results))
```

**Good** — logic in module, entry point calls:
```python
# main.py
import sys
from scoring import process_items, format_output

def main():
    data = json.load(open(sys.argv[1]))
    results = process_items(data)
    print(format_output(results))

if __name__ == "__main__":
    main()
```

### Enforcement

Both agents check before presenting implementation:
- Does the entry point contain business logic beyond imports + calls?
- If yes → refactor before proceeding.

## File Encoding And Text Policy

**ALL text files MUST be UTF-8 without BOM, without mojibake, and without mixed line endings.** This is a release blocker.

### Rules

1. **Never write files in Windows-1251, latin1, CP1252, or any non-UTF-8 encoding**
2. **No UTF-8 BOM** (`EF BB BF`) because it breaks parsers and diffs
3. **No mojibake**: known double-decoded UTF-8 signatures and replacement characters are forbidden. Keep literal fixtures out of source; use Unicode escapes in tests.
4. **Russian text is expected** in this bilingual project. Cyrillic must stay readable as Cyrillic
5. **Line endings**: LF (`\n`) preferred. CRLF is tolerated for Windows batch files, but mixed endings inside one file are forbidden

### Enforcement

- Run `node scripts/validate-text-policy.js` before release or after encoding-sensitive edits.
- Post-write hook `.claude/hooks/check-encoding.sh` fails on invalid UTF-8, BOM, mixed line endings, mojibake, and unsafe shell platform assumptions.
- `scripts/validate-template.sh` and `scripts/test-template.sh` include the same text-policy gate.
- Tests that need mojibake fixtures must create them with Unicode escapes, not by pasting literal corrupted text into source files.

## Platform And OS Policy

**Shell scripts must detect the current OS through `scripts/lib/platform.sh`.** Do not assume Linux behavior on Windows.

### Rules

1. Source `scripts/lib/platform.sh` before OS, architecture, temp path, hash, or JSON helper logic.
2. Use `_detect_os`, `_detect_arch`, `_is_windows`, `_temp_file`, and `_temp_dir` instead of raw `uname`, `/tmp`, or direct `mktemp`.
3. Keep raw platform probes inside `scripts/lib/platform.sh` only.
4. On Windows, resolve Git Bash/PowerShell behavior explicitly. Do not assume `bash`, POSIX paths, or Linux temp directories exist in the parent process environment.

## GitHub Actions Runtime Policy

Template-owned GitHub workflows and CI templates must use Node 24-compatible official actions. Release and validation jobs must disable unnecessary `setup-node` package-manager cache because they run with repository tokens and do not install packages.
5. Update Unix and Windows paths together when changing template setup, sync, or validation behavior.

## Entry Point Naming Convention

| Language | Entry point | Convention |
|----------|------------|------------|
| Python | `main.py`, `cli.py`, `app.py` | `if __name__ == "__main__": main()` |
| TypeScript/JS | `index.ts`, `main.ts`, `cli.ts` | Named exports + top-level call |
| Go | `main.go`, `cmd/*.go` | `func main()` calls into `internal/` |
| Rust | `main.rs`, `bin/*.rs` | `fn main()` calls lib functions |
| Bash | `run.sh`, `cli.sh` | Source lib functions, call them |
