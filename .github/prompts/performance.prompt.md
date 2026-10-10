---
agent: agent
description: Measure and improve performance with reproducible evidence
---

Investigate the requested performance goal.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Define the workload, metric and success criterion; measure before changing code.
- Preserve correctness, isolation and contracts. Prefer simple improvements before
  concurrency or dependencies; optional optimisation must not delay v1.
- Report before/after results with environment, variance and measurement limits.
  Add a repeatable benchmark when useful with checks deferred under AGENTS testing policy.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
