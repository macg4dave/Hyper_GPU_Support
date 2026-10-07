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

GPU-PV feasibility is established; implement and validate the product for an
existing selected VM. Reuse disposable-VM tooling for tests. Golden-image copying,
cloning, disk reset and laboratory setup stay outside the production path unless
an explicit user-facing roadmap task requires them. Test tooling may depend on
product code; product code must not depend on test tooling.

The user-approved architecture rebase takes precedence over historical task scope.
Use runtime existing-VM identities and the revised core → native GUI → allocation/
vendor roadmap. Old fixed-slot code is research/contributor tooling in `tools/lab/`.
Do not preserve laboratory coupling, baseline driver pins or repeated diagnosis as
product architecture. Default operation verifies health plus checked graphics;
extended CUDA/stress remain optional. The core never imports the laboratory.
