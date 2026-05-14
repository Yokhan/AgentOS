# Provider routing in AgentOS

Дата: 2026-05-14

## Зачем это нужно

AgentOS может работать без Claude. Если `claude_enabled=false`, все роли, которые раньше могли попасть в Claude, должны разрешаться в Codex до запуска команды. Пользователь не должен угадывать, кто реально выполняет задачу.

## Роли

1. `orchestrator` ведет основной чат, планы, стратегию и одиночные проектные команды.
2. `technical_reviewer` используется для Duo/review route.
3. `delegation` запускает проектных агентов и дочерние задачи.

## Правило fallback

1. Если Claude включен и доступен, роли могут использовать `claude` или `codex` по настройкам.
2. Если Claude выключен, `orchestrator`, `technical_reviewer`, `delegation` и explicit `claude` route должны стать `codex`.
3. Codex route выбирает модель и effort из Codex settings. Для делегаций Codex использует `delegation_codex_model` / `delegation_codex_effort`, если они заданы, иначе общие `codex_model` / `codex_effort`.
4. Settings показывает не только configured provider, но и effective provider. Effective route является пользовательским source of truth.

## Что должно быть видно в UI

1. Effective Provider Routes: роль, configured provider, effective provider, model/effort, health.
2. Active GPT account: label аккаунта, auth mode, transport, источник списка моделей.
3. Provider Diagnostics: доступность Claude/Codex и причина fallback.

## Gates

1. `scripts/check-settings-provider-ui.mjs` проверяет, что Settings не теряет effective routes, account snapshot и Claude disable contract.
2. Rust tests в `provider_runner.rs` проверяют, что disabled Claude route не уходит в Claude даже при старой конфигурации.
