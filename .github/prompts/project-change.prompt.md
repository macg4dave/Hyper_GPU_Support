---
agent: agent
description: Make a focused Windows-native GPU-PV project change
---

Implement the requested change with the smallest coherent patch.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Read the affected source and selected task; implement our validated recipe.
- Keep mutable environment values in `config/project.toml`, core logic in Rust
  and privileged Windows effects behind bounded adapters.
- Add focused behavior/failure tests, run proportional checks and update the task
  result and affected contracts. Do not expand scope into feasibility research.
