---
agent: agent
description: Make a focused Windows-native GPU-PV project change
---

Implement the requested change with the smallest coherent patch.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Read the selected backlog card and affected source; reuse working Rust entry
  points and implement the missing approved GUI behavior directly.
- Preserve runtime schema, native Windows adapters, privilege and recovery.
  Keep operator intent separate from discovered inventory and contributor values.
- Add focused tests for meaningful new logic; defer execution under AGENTS.
  Record behavior, concrete blockers and validation pending on the existing card.
- Do not expand into feasibility, migration, qualification or documentation work
  unless a specific defect blocks the requested product change.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
