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
- Preserve meaningful test coverage with execution deferred under AGENTS testing policy. Update architecture
  only when component ownership or boundaries change.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
