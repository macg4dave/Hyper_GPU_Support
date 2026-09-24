---
agent: agent
description: Measure and improve performance with reproducible evidence
---

Investigate the requested performance goal.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[module design](../../docs/ENGINEERING.md#code-and-module-design),
[testing](../../docs/ENGINEERING.md#testing), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

1. Define the environment, workload, metric, baseline, and success criterion.
2. Measure before changing code.
3. Preserve correctness, compatibility, isolation, and public behavior.
4. Prefer simple data-flow, allocation, or synchronization improvements before
   adding concurrency or dependencies.
5. Keep host/guest and GPU measurements separate from deterministic unit tests.
6. Record variance, warm-up behavior, driver versions, and measurement limits.
7. Add a repeatable benchmark or regression check when practical.
8. Run required quality and relevant correctness checks after optimization;
   report before/after results without overstating significance.
