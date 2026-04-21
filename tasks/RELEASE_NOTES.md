# Agent OS 0.2.0

- Tauri desktop app with installers for Windows (`NSIS` and `MSI`).
- Auto-update pipeline wired for installed builds via GitHub Releases.
- Installed binaries can recover the working repo root using bootstrap state.
- Mixed-provider orchestration foundation for Claude and Codex.
- Updated release tooling:
  - signed updater artifacts
  - `latest.json` generation
  - GitHub Actions release workflow
