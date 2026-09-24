---
agent: agent
description: Measure and improve performance with reproducible evidence
---

Investigate the requested performance goal.

1. Define the environment, workload, metric, baseline, and success criterion.
2. Measure before changing code.
3. Preserve correctness, compatibility, isolation, and public behavior.
4. Prefer simple data-flow, allocation, or synchronization improvements before
   adding concurrency or dependencies.
5. Keep host/guest and GPU measurements separate from deterministic unit tests.
6. Record variance, warm-up behavior, driver versions, and measurement limits.
7. Add a repeatable benchmark or regression check when practical.
8. Run focused correctness checks after optimization and report before/after
   results without overstating significance.

