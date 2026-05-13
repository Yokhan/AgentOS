# Project Onboarding System

## Problem

Connecting projects through the orchestrator was too slow because the flow was not a product feature. It depended on long chat instructions, manual segment edits, manual permission edits, and separate template sync calls.

## Target Flow

1. Audit all git repos under `documents_dir`.
2. Show what is missing for each repo: segment, permission profile, template files.
3. Connect one repo with one command.
4. Optionally sync the agent template in the same operation.
5. Refresh AgentOS state without requiring app restart.
6. Let the orchestrator choose these commands from natural-language user requests.

## Interfaces

- Natural language to orchestrator: "проверь подключение проектов", "подключи zolt", "подключи недостающие проекты".
- Chat/PA command: `[PROJECT_ONBOARD_AUDIT]`
- Chat/PA command: `[PROJECT_CONNECT:Project:Segment:balanced]`
- Chat/PA command: `[PROJECT_CONNECT:Project:Segment:balanced:dry]`
- Chat/PA command: `[PROJECT_CONNECT:Project:Segment:balanced:deploy]`
- Chat/PA command: `[PROJECT_CONNECT_MISSING:Other:balanced:dry]`
- Chat/PA command: `[PROJECT_CONNECT_MISSING:Other:balanced]`
- CLI: `npm run connect:project -- -Audit`
- CLI: `npm run connect:project -- -Project MyRepo -Segment Other -Permission balanced`
- CLI: `npm run connect:project -- -Project MyRepo -Segment Other -Permission balanced -DeployTemplate`
- CLI: `npm run connect:project -- -All -Segment Other -Permission balanced -DryRun`
- CLI: `npm run connect:project -- -All -Segment Other -Permission balanced`

## Write Targets

- `n8n/dashboard/segments.json`
- `n8n/config.json` -> `project_permissions`
- Optional project template files via `agent-project-template/scripts/sync-template.sh`

## Safety

- Users should not need to type PA tags manually. The orchestrator prompt contains intent routing and recommended next PA commands.
- `Plan/read-only` may run only read-only diagnostics and dry-runs, such as onboarding audit or bulk dry-run.
- Metadata apply and template sync require act/write mode or an explicit non-dry PA command from the orchestrator.
- Default connect mode does not modify project contents.
- `-DryRun` and `:dry` show planned changes only.
- Template sync only runs when `-DeployTemplate` or `:deploy` is explicit.
- Bulk connect only repairs metadata: segment and permission. It does not touch project files.
- Permission profile is restricted to `restrictive`, `balanced`, `permissive`.

## Remaining Product Work

- Add a Settings UI panel for onboarding audit and one-click connect.
- Add batch connect after single-project flow is proven stable.
- Add per-project readiness badges to the left project navigation.
