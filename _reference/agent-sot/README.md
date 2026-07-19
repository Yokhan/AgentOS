# Agent SOT Reference

This directory stores the source registry and local interpretations for agent
instruction architecture. It exists to prevent blind edits to `AGENTS.md`,
`CLAUDE.md`, skills, subagents, hooks, routing, and template sync.

## Files

- `sources.json` - machine-readable source registry with trust level,
  last-checked date, refresh policy, and local conclusions.
- `top-works.md` - priority annotated bibliography for agent engineering and
  template maintenance decisions.
- `originals/ai-agent-spec-v3-final.md` - user-provided local spec imported as
  a byte-preserved source.

## Source Priority

1. Official vendor docs for current behavior:
   - OpenAI Codex docs for `AGENTS.md`, skills, hooks, subagents.
   - Claude Code docs for `CLAUDE.md`, skills, subagents, hooks.
   - Zed docs for ACP behavior and editor integration boundaries.
   - GitHub Spec Kit docs for spec-driven workflow shape.
2. User-provided local spec and project conventions.
3. Credible public implementation writeups and repos such as Trail of Bits,
   HumanLayer, and Nx.
4. Community discussion only as weak signal. It cannot override official docs.

## Local Conclusions

- Instruction files are routing and context entry points, not full manuals.
- Skills are the default home for reusable workflows and long domain knowledge.
- Hooks are the only deterministic layer for repeated enforcement.
- Subagents are useful for parallel exploration and review; write access needs
  explicit isolated ownership.
- Scaffolding and validators beat prose for mechanical consistency.
- Spec-driven artifacts are useful when task intent, constraints, and dependency
  order need to survive multiple sessions or agents.

## Maintenance

When changing agent infrastructure:

1. Read `docs/AGENT_CONTEXT_SOT.md`.
2. Check `sources.json` freshness.
3. Browse official URLs if the relevant source is stale or behavior-sensitive.
4. Update the local conclusion or source registry when the docs contradict the
   current template.
5. Run `node scripts/validate-agent-sot.js`.

Do not vendor full third-party documentation snapshots. Keep canonical URLs and
short summaries here, then browse the original when current behavior matters.
