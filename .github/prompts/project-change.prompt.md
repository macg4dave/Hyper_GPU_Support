---
agent: agent
description: Make a focused Windows-native GPU-PV project change
---

Implement the requested change with the smallest coherent patch.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Read the affected source and selected task; implement our validated recipe.
- Reproduce provisioning behavior through complete associated payload discovery
  for the selected signed host driver; do not assume a fixed count or static list.
- Keep mutable environment values in `config/project.toml`, core logic in Rust
  and privileged Windows effects behind bounded adapters.
- Add focused behavior/failure tests, run proportional checks and update the task
  result and affected contracts. Do not expand scope into feasibility research.
- For Windows product capabilities, use native Rust APIs/bindings. Consult the
  affected [migration audit](../../scripts/PRODUCT-MIGRATION.md) row; a Rust shell
  wrapper does not finish the port. Keep the working adapter until equivalent
  tests and affected baseline qualification demonstrate its replacement.

GPU-PV feasibility is established; implement and validate the product for an
existing selected VM. Reuse disposable-VM tooling for tests. Golden-image copying,
cloning, disk reset and laboratory setup stay outside the production path unless
an explicit user-facing roadmap task requires them. Test tooling may depend on
product code; product code must not depend on test tooling.
