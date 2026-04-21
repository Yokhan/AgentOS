# Agent OS 0.2.3

- Managed projects now bootstrap and sync through the sibling `agent-project-template` repo instead of embedding stale template logic inside AgentOS.
- Windows bash resolution now finds Git Bash automatically for template sync, health checks, DNS, SSL, and other shell-backed flows.
- Project scanning falls back to `tasks/current.md` milestone text when `PROJECT_SPEC.md` does not expose a phase.
- Windows release automation now allows enough time for the signed Windows build and writes updater `latest.json` without a UTF-8 BOM.
