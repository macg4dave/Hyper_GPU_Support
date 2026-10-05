---
agent: agent
description: Add meaningful behavioral, failure, and regression coverage
---

Add focused tests that verify the requested behavior and its relevant failure paths.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[testing](../../docs/ENGINEERING.md#testing).

- Test changed behavior and meaningful boundaries/failures; avoid tests that mirror
  internals or a mechanical matrix with no implementation consequence.
- Keep tests deterministic and isolated; use bounded readiness checks. Select
  privileged/hardware runs explicitly through the approved path.
- Reuse established baseline results. Hardware runs should verify changed Rust
  behavior or reproducibility. Report actual workload results and material gaps.
