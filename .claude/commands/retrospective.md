---
name: retrospective
description: "Deep weekly analysis: agent performance, rule effectiveness, learning velocity, cross-system data aggregation. Deeper than /weekly."
---

# /retrospective

Deep analysis across all systems. Run weekly or after significant milestones.

## Data Sources

1. `tasks/lessons.md` — error patterns, correction categories
2. Engram `mem_search(topic_key="metrics:")` — agent performance
3. Engram `mem_search(topic_key="rule:activation:")` — rule effectiveness
4. Engram `mem_search(topic_key="lesson:")` — learning patterns
5. `brain/01-daily/` — session logs for the period
6. `git log` — commits, branches, activity
7. `tasks/.memory-fallback.md` — fallback entries if Engram was unavailable

If Engram is not connected: use sources 1, 5, 6, 7 only. Data will be less complete but retrospective still works.

## Output Sections

### 1. Work Summary
```
Period: [date range]
Tasks completed: [N] ([breakdown by project/category])
Commits: [N]
Agent sessions: [N]
```

### 2. Agent Performance (from agent-metrics)
```
┌─────────────┬───────┬──────────┬────────────┬──────────┐
│ Agent       │ Tasks │ Avg Time │ Corrections│ Accuracy │
├─────────────┼───────┼──────────┼────────────┼──────────┤
│ implementer │ ...   │ ...      │ ...        │ ...%     │
│ reviewer    │ ...   │ ...      │ ...        │ ...%     │
│ ...         │       │          │            │          │
└─────────────┴───────┴──────────┴────────────┴──────────┘
```

### 3. Rule Effectiveness (from rule activation logs)
```
Most triggered: [rule] ([N]x)
Most prevented errors: [rule] ([N] prevented)
Never triggered: [rules] — candidates for unload or removal
```

### 4. Learning Velocity
```
Week 1: [N] errors (baseline)
Week 2: [N] errors (velocity: X.XX)
Week 3: [N] errors (velocity: X.XX)
Trend: improving / stable / regressing

Recurring errors: [patterns that appeared 2+ times]
→ Candidate for rule promotion
```

### 5. Recommendations
Based on data, suggest:
- Rules to CREATE (from recurring error patterns)
- Rules to REMOVE (never triggered, 10+ sessions)
- Agents to SPECIALIZE (from project-specific correction patterns)
- Memory to CLEAN UP (expired TTL observations)
- Process improvements (based on where time was wasted)

## Actions After Retrospective

1. **Promote** recurring patterns to `project-*.md` rules
2. **Archive** promoted lessons from lessons.md
3. **Clean up** expired Engram observations
4. **Update** project agent overlays if specialization needed
5. **Save** retrospective to `brain/01-daily/retro-YYYY-MM-DD.md`
