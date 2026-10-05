---
agent: agent
description: Refactor code while preserving behavior and component boundaries
---

Perform the requested refactor.

Follow [AGENTS.md](../../AGENTS.md) and
[module design standards](../../docs/ENGINEERING.md#code-and-module-design).

- Establish current behavior from affected code/tests; keep the patch narrow.
- Preserve contracts and hardware behavior unless the task changes them. Avoid
  speculative traits, crates, cross-platform layers or unrelated modernization.
- Preserve meaningful test coverage and run affected checks. Update architecture
  only when component ownership or boundaries change.
