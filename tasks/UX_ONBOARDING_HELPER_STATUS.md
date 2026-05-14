# UX Onboarding Helper Status

## Goal

Make project onboarding usable from natural language: the user asks the orchestrator to
connect or recover projects, and AgentOS returns a safe wave plan instead of forcing
manual scripts and command tags.

## Implemented

- Added read-only `[PROJECT_ONBOARD_PLAN:segment:permission:limit]`.
- Added dirty-worktree aware canary selection for template deployment.
- Added exact next-command output for metadata dry-run, metadata apply, and one-project
  template canary.
- Added Workbench `prepare onboarding wave` action.
- Added UI/backend guard `scripts/check-onboarding-plan-ui.mjs`.
- Added operator doc `docs/PROJECT_ONBOARDING.md`.

## Safety Contract

- Bulk operations are limited to metadata repair.
- Template deployment is one project at a time.
- Dirty or non-git-status-readable projects are listed as blocked, not auto-deployed.
