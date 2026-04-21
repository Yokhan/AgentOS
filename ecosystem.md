# Ecosystem Map

> Cross-project dependency map. Maintained by agents, used by research-first protocol.
> Update when: new project added, dependency changes, API contract changes.

## This Project

- **Name**: _[project name]_
- **Role**: _[what it does in the ecosystem]_
- **Provides**: _[APIs, exports, services other projects consume]_

## Upstream Dependencies (this project CONSUMES)

| Project | What we use | How (API/import/DB) | Impact if down |
|---------|-------------|---------------------|----------------|
| _[name]_ | _[service/API]_ | _[method]_ | _[HIGH/MEDIUM/LOW]_ |

## Downstream Dependents (other projects CONSUME us)

| Project | What they use | How | Impact if we break |
|---------|---------------|-----|--------------------|
| _[name]_ | _[service/API]_ | _[method]_ | _[HIGH/MEDIUM/LOW]_ |

## Shared Resources

| Resource | Used by | Owner |
|----------|---------|-------|
| _[DB, queue, storage, API gateway]_ | _[project list]_ | _[who manages it]_ |

## Cross-Project Rules

- Before changing any API that appears in "Downstream Dependents" → check consumers first
- Before removing an export → grep all sibling project directories
- Shared DB tables → changes require coordination with table owner
- Breaking changes → version the API, don't modify in place

## Last Updated

_YYYY-MM-DD_
