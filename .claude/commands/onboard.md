You are onboarding a new developer (or yourself after a long absence) to this project.

## Process

### Step 1: Project Overview
Read and summarize `CLAUDE.md`:
- What does this project do?
- What's the current status?
- What stack is being used?

### Step 2: Architecture
Read `docs/ARCHITECTURE.md` if it exists. Otherwise, infer from `src/` structure:
- What are the main modules?
- How do they depend on each other?
- Where is the entry point?

### Step 3: Recent Activity
```bash
git log --oneline -10              # last 10 commits
git branch --show-current          # current branch
git status --short                 # current state
```
Summarize: what's been worked on recently?

### Step 4: Key Decisions
Read `brain/04-decisions/` — list the 5 most recent decision records.
For each: what was decided and why?

### Step 5: Known Patterns & Pitfalls
Read `tasks/lessons.md` — summarize top 5 recurring patterns.
These are the mistakes to avoid.

### Step 6: How to Contribute
From `CLAUDE.md` Build & Test section:
- How to run the project
- How to run tests
- Branch naming conventions (`feat/`, `fix/`, `refactor/`, `docs/`)
- Which commands to use: `/implement`, `/sprint`, `/commit`, `/review`

### Step 7: First Task Recommendation
Read `tasks/current.md` — identify the most suitable "first task" for getting familiar with the codebase.
Prefer: small scope, well-defined, touches core patterns.

## Output
Save onboarding summary to `brain/00-inbox/onboarding-[date].md` and display to user.
