# Migration Matrix

Date: 2026-05-23
Target template version: `3.7.0`
Runner: `bash scripts/downstream-census.sh --brief --no-sync --search <workspace>` for read-only census; `bash scripts/downstream-census.sh --json <project>` for dry-run detail
Mode: dry-run only, no downstream files modified

## 3.7.0 Review Snapshot

- Source template local gate passed on 2026-05-23 after fixing `task-brief.sh` pipefail and downstream-safe `test-template.sh` bootstrap smoke routing.
- `PersonalAssistant` is the first confirmed downstream control-plane repo on template `3.7.0`.
- Latest read-only workspace census found 25 template-managed repos: 1 on `3.7.0`, 20 on `3.6.0`, and 4 with unknown template version.
- Do not bulk-sync the downstream fleet. Pick 1-2 canary repos for explicit dry-run review before rollout.
- The historical table below remains the last detailed dry-run proof set. Refresh it with 3.7 dry-run rows before cutting a broader rollout.

## Historical Dry-Run Proof Set

## Summary

| Project | Current | Classification | Updated | New | Conflicts | Preserved | Deprecated | Spec | AGENTS |
|---------|---------|----------------|---------|-----|-----------|-----------|------------|------|--------|
| `YokhanCallService` | `3.4.0` | `clean-dry-run` | 0 | 10 | 0 | 0 | 0 | yes | no |
| `amplitude-client` | `3.4.0` | `manual-merge` | 1 | 39 | 107 | 0 | 69 | no | no |
| `PixelTilemapGenerator` | `2.7.0` | `manual-merge` | 4 | 47 | 76 | 2 | 26 | no | no |
| `GIANTS VALE DUNGEONS` | `3.4.0` | `manual-merge` | 2 | 46 | 75 | 10 | 124 | yes | yes |
| `PersonalAssistant` | `unknown` | `manual-merge` | 1 | 43 | 75 | 4 | 47 | no | no |

## Notes

- `YokhanCallService` is the cleanest proof case: dry-run adds the new `3.5.0` surface without local conflicts.
- `amplitude-client` proved a real migration blocker in the old path: its local `scripts/sync-template.sh` failed under PowerShell -> Git Bash because `platform.sh` was not sourced from `/C/...`. The template now supports running the current sync script from the template repo with `--project-dir`, which bypasses broken legacy local copies.
- `PixelTilemapGenerator` is the oldest supported sample in this pass (`2.7.0`). It upgrades only with manual merge review, which is acceptable but must stay documented as a supported manual path.
- `GIANTS VALE DUNGEONS` shows the heaviest deprecation surface and preserved local project commands/rules. It is a good regression target for future conflict-resolution guidance.
- `PersonalAssistant` confirms the `unknown -> current` path can still be analyzed with the current template runner, but it is not a clean auto-update candidate.

## Supported Paths After This Pass

- `3.4.0 -> 3.5.0`: proven on multiple real repos, ranging from clean dry-run to documented manual-merge cases.
- `2.7.0 -> 3.5.0`: proven as manual-merge path on a real repo.
- `unknown -> 3.5.0`: proven as analyzable/manual-merge path via the template runner using `--project-dir`.

## Recommended Operator Flow

1. Run `bash scripts/downstream-census.sh --brief <project-dir ...>` from the template repo to classify the target projects.
2. For older or broken local sync scripts, run `bash scripts/sync-template.sh /path/to/template --project-dir /path/to/project --dry-run`.
3. Treat `clean-dry-run` repos as low-risk candidates.
4. Treat `manual-merge` repos as supported but review-required.
