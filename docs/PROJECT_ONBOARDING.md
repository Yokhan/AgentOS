# Project Onboarding

AgentOS project onboarding is intentionally split into safe metadata repair and risky
template deployment. The orchestrator should not ask the user to type command tags by
hand; it should explain the action in chat and emit the matching PA command itself.

## Commands

- `[PROJECT_ONBOARD_PLAN:Other:balanced:5]` is read-only. It audits discovered git
  projects, counts metadata gaps, separates clean template canaries from dirty or
  blocked projects, and returns exact next commands.
- `[PROJECT_ONBOARD_AUDIT]` is read-only. It returns the raw project readiness table.
- `[PROJECT_CONNECT_MISSING:Other:balanced:dry]` previews bulk metadata repair.
- `[PROJECT_CONNECT_MISSING:Other:balanced]` applies metadata repair only: segment and
  permission profile.
- `[PROJECT_CONNECT:Project:Other:balanced:deploy,dry]` previews one project template
  deployment.
- `[PROJECT_CONNECT:Project:Other:balanced:deploy]` applies one project template
  deployment after review.

## Safe Flow

1. Run `[PROJECT_ONBOARD_PLAN:Other:balanced:5]` first when scope is unclear.
2. Apply metadata repair in bulk only after checking the dry-run output.
3. Deploy templates only as canary waves.
4. Never deploy a template into a dirty project until git status is clean or the dirty
   files are explicitly accepted.
5. After every canary, run health checks before selecting the next wave.

## UI Entry Point

The Workbench focus screen has a `prepare onboarding wave` action. It switches the
project filter to unmanaged projects and drafts `[PROJECT_ONBOARD_PLAN:Other:balanced:5]`
with the instruction to return commands and blockers.
