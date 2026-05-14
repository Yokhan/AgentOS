# Settings/provider visibility slice

Дата: 2026-05-14

## Что закрыто

1. Settings теперь показывает одну таблицу `Effective Provider Routes` для `orchestrator`, `reviewer` и `delegation`.
2. В таблице видно configured provider, effective provider, model/effort и health.
3. Codex settings показывают active GPT account label, auth mode, transport и источник моделей.
4. Backend provider status отдает `role_settings.technical_reviewer`, чтобы UI не гадал.
5. Поведение `claude_enabled=false` описано в `docs/PROVIDER_ROUTING.md`.
6. Добавлен gate `scripts/check-settings-provider-ui.mjs`, он включен в `npm run check:ui`.

## Что осталось в большом плане

1. Onboarding helper для массового подключения/восстановления проектов.
2. Release readiness checklist/dashboard.
3. Recovery smoke для reload/cancel/network drop.
