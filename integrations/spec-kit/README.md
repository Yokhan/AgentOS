# Spec Kit Integration

Spec Kit is available in this template in two layers:

1. A local tracked upstream snapshot for agents:
   `_reference/spec-kit/upstream/`.
2. A deploy command that runs the official GitHub Spec Kit CLI at the pinned
   snapshot ref:
   `bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .`

Use this when a project needs Spec -> Plan -> Tasks -> Implement artifacts,
`.specify/memory/constitution.md`, and Spec Kit command files.

## Freshness

Check upstream freshness:

```bash
bash scripts/sync-spec-kit.sh --check
```

Update the tracked snapshot to the latest stable upstream tag:

```bash
bash scripts/sync-spec-kit.sh --latest-tag
```

Validate the local snapshot without network:

```bash
node scripts/validate-spec-kit.js
```

## Deployment

Default Codex setup:

```bash
bash scripts/init-spec-kit.sh --integration codex --script sh --project-dir .
```

Claude setup:

```bash
bash scripts/init-spec-kit.sh --integration claude --script sh --project-dir .
```

Windows PowerShell helper setup:

```bash
bash scripts/init-spec-kit.sh --integration codex --script ps --project-dir .
```

`scripts/init-spec-kit.sh` reads `_reference/spec-kit/manifest.json` so
downstream projects use the same upstream ref that was reviewed with the
template. It uses `uvx` by default to pin that ref. If you intentionally want
to use an already installed `specify` CLI, pass `--use-installed`.
