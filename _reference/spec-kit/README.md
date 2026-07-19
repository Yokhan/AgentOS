# Managed Spec Kit Snapshot

This directory contains a tracked snapshot of GitHub Spec Kit plus a manifest
that records the upstream ref and commit.

## Policy

- Track stable upstream release tags by default.
- Keep the snapshot small: docs, templates, scripts, integration catalogs, and
  license only.
- Do not copy the upstream `.git` directory or generated package artifacts.
- Use the snapshot for agent reading/offline reference.
- Use the official `specify` CLI for project initialization.

## Commands

Validate local snapshot:

```bash
node scripts/validate-spec-kit.js
```

Check whether the local stable tag is stale against upstream:

```bash
bash scripts/sync-spec-kit.sh --check
```

Update to the latest stable tag:

```bash
bash scripts/sync-spec-kit.sh --latest-tag
```

Deploy Spec Kit into the current project using the pinned ref:

```bash
bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .
```

For Windows-native PowerShell helpers, use `--script ps`.

## Snapshot Contents

- `upstream/README.md`
- `upstream/spec-driven.md`
- `upstream/docs/installation.md`
- `upstream/docs/quickstart.md`
- `upstream/docs/upgrade.md`
- `upstream/docs/reference/*.md`
- `upstream/templates/`
- `upstream/scripts/bash/`
- `upstream/scripts/powershell/`
- `upstream/integrations/catalog*.json`
