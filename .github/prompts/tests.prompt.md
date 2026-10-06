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
- For native adapter ports, cover identity/access/provider errors, stale state,
  verified no-op, deadlines and uncertain mutation as applicable. Compare affected
  observable results with the working baseline; passing the existing PowerShell
  adapter tests alone does not qualify its Rust replacement.
- Use small variable-size provisioning fixtures for enumeration, mapping,
  deduplication/collisions, missing files, hashes and manifest validation. Label
  historical driver regression fixtures; their count is not a product acceptance gate.
