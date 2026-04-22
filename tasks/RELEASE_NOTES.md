# Agent OS 0.2.5

- Duo `execute` rounds now actually run as execution rounds instead of being hard-forced into analysis-only review mode.
- The write-enabled orchestrator participant in multi-agent execution can now emit and execute PA commands such as `[DELEGATE]`, `[PLAN]`, and `[NOTIFY]` from duo responses.
- Non-writer participants remain read-only even during execution rounds, so the shared room no longer grants implicit write-capable execution to the technical reviewer.
