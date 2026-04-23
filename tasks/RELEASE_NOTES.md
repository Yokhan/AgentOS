# Agent OS 0.2.7

- Duo sessions now have an explicit active orchestrator instead of hard-wiring orchestration to the default Claude PM participant.
- You can promote Codex Tech to orchestrator from Advanced runtime controls; switching orchestrator automatically grants write access when needed.
- Execution prompts now separate write access from orchestration ownership, so non-orchestrator writers no longer look like they can emit PA commands.
