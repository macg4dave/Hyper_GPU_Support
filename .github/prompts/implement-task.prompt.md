---
name: implement-task
description: Implement one requested task using the existing product boundaries.
---

Follow [AGENTS.md](../../AGENTS.md), [backlog](../../docs/BACKLOG.md), [engineering](../../docs/ENGINEERING.md), and [architecture](../../docs/ARCHITECTURE.md). Read the other handbook pages relevant to the task. This prompt does not authorize a backlog sweep or live operations.

1. Identify the requested card and verify dependencies against current source/status. Preserve user changes/deletions.
2. Reuse the shared Rust model/workflow, protected setup/worker/runner, and existing Slint design. Correct demonstrated gaps rather than inventing parallel implementations.
3. Implement the smallest coherent change satisfying the card, with meaningful regression coverage where needed. Keep implementation separate from hardware qualification.
4. Follow [testing](../../docs/TESTING.md): normally defer builds/tests until the agreed milestone completes or the user requests them. Label unrun changes accurately.
5. Update implementation and validation separately in the backlog and refresh affected authoritative documents. Do not close a milestone without gate evidence.
6. Report changes, actual checks, uncertainty, and the next task; stop after this requested task.

Prepare targets, effects, evidence, and stop conditions before asking for scoped live authorization. Never automatically retry uncertain GPU work.
