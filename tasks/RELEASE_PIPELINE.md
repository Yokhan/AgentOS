# Agent OS Release Pipeline

## What is now wired

- Tauri updater plugin is enabled in `src-tauri/tauri.conf.json`.
- Windows updater signatures are generated during release builds.
- `bundle.createUpdaterArtifacts` is enabled.
- `scripts/build-release.ps1` builds the signed release and generates `latest.json`.
- `.github/workflows/release.yml` publishes installers, `.sig` files and `latest.json` on tag pushes.
- Installed builds persist the repo root in `%APPDATA%\Agent OS\bootstrap.json`, so the installed exe can still find the working repo/config.

## Local release build

Prerequisites:

- `~/.tauri/agent-os.key`
- matching public key already present in `src-tauri/tauri.conf.json`

Run:

```powershell
npm run build:updater
```

Expected outputs:

- `src-tauri/target/release/bundle/nsis/Agent OS_<version>_x64-setup.exe`
- `src-tauri/target/release/bundle/nsis/Agent OS_<version>_x64-setup.exe.sig`
- `src-tauri/target/release/bundle/msi/Agent OS_<version>_x64_en-US.msi`
- `src-tauri/target/release/bundle/msi/Agent OS_<version>_x64_en-US.msi.sig`
- `src-tauri/target/release/bundle/latest.json`

## GitHub release flow

1. Bump version in:
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
   - `package.json`
2. Push code.
3. Create and push tag:

```powershell
git tag v0.2.1
git push origin v0.2.1
```

4. GitHub Actions workflow `Release` builds and uploads:
   - NSIS installer
   - NSIS signature
   - MSI installer
   - MSI signature
   - `latest.json`

## Required GitHub secrets

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

If the key has no password, keep the password secret empty.

## Current updater endpoint

The app checks:

```text
https://github.com/Yokhan/AgentOS/releases/latest/download/latest.json
```

That URL must exist in the latest GitHub Release, or installed builds will have nothing to download.

## Important note

Running from the repo checkout should not auto-update itself.
Installed copies outside the repo root are the ones intended to auto-check for updates.
