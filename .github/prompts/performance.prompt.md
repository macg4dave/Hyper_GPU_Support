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
  Add a repeatable benchmark when useful and run affected correctness checks.
