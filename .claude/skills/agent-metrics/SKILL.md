---
name: agent-metrics
description: "Log agent performance metrics to Engram after task completion. Enables tracking of accuracy, speed, and improvement over time."
---

# Agent Metrics

Log performance data after every completed task. Enables `/weekly` and `/retrospective` to analyze agent effectiveness.

## When to Log

After completing any task (XS through XL), the agent should save metrics.

## Metrics Format

Save to Engram (or `tasks/.memory-fallback.md` if Engram unavailable — see memory-router fallback):
```
mem_save(topic_key="metrics:{agent_name}", content="""
Task: {task description or issue ID}
Size: {XS/S/M/L/XL}
Duration: {estimated minutes}
Files changed: {N}
Lines changed: {approximate N}
Test runs: {N}
Test failures before fix: {N}
User corrections: {N}
Confidence: {HIGH/MEDIUM/LOW}
Circuit breaker triggered: {yes/no}
Gates passed: {N/N}
Escalated: {yes/no, to whom}
Date: {YYYY-MM-DD}
TTL: {date + 3 months}
""")
```

## What Each Metric Tells Us

| Metric | Healthy | Warning | Action |
|--------|---------|---------|--------|
| User corrections | 0 per task | >1 per task | Review verification gates |
| Test failures | 0-1 before fix | >3 same error | Check understanding of codebase |
| Confidence vs reality | HIGH = correct | HIGH but corrected | Calibration needed |
| Circuit breaker | never | >1/week | Investigate recurring blockers |
| Duration vs size | proportional | XS taking M-time | Check if classification is off |

## Aggregation (for /weekly and /retrospective)

Search: `mem_search(topic_key="metrics:")` — returns all agent metrics.
Fallback: grep `tasks/.memory-fallback.md` for `metrics:` entries.

Aggregate by:
- **Agent**: which agent is most/least accurate?
- **Size**: are M tasks taking too long? Are XS tasks over-ceremonied?
- **Corrections**: which agent gets corrected most?
- **Trends**: is accuracy improving week over week?

## Dashboard Format (for /retrospective output)

```
Agent Performance (last 7 days):
┌─────────────┬───────┬──────────┬────────────┬──────────┐
│ Agent       │ Tasks │ Avg Time │ Corrections│ Accuracy │
├─────────────┼───────┼──────────┼────────────┼──────────┤
│ implementer │ 12    │ 22 min   │ 3          │ 75%      │
│ reviewer    │ 8     │ 5 min    │ 0          │ 100%     │
│ researcher  │ 4     │ 15 min   │ 1          │ 75%      │
│ test-eng    │ 6     │ 10 min   │ 0          │ 100%     │
└─────────────┴───────┴──────────┴────────────┴──────────┘

Accuracy = (tasks - corrections) / tasks * 100
```
