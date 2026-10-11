---
name: task-plan
description: Plan one requested backlog task against current source.
---

Follow [AGENTS.md](../../AGENTS.md), [roadmap](../../docs/ROADMAP.md), [backlog](../../docs/BACKLOG.md), and [engineering](../../docs/ENGINEERING.md). This prompt adds no execution authorization.

1. Identify the requested task, dependencies, acceptance criteria, and relevant handbook pages. Inspect current source and working-tree changes first.
2. Distinguish already implemented paths from concrete gaps and missing validation. Use [baseline](../../docs/BASELINE.md) as an index, not proof of current behavior.
3. Resolve discoverable questions through read-only inspection; ask only for material choices or information that cannot be discovered.
4. Produce a decision-complete plan for one task: behavior, reused interfaces, failure cases, documentation/status changes, and milestone validation timing.
5. Identify live/privileged effects and prepare a concrete target/effect/evidence sequence before requesting authorization. Planning must not perform them.

Do not edit application files, run builds/tests, launch the UI, or advance into another task merely to prepare the plan. Respect the current user's scope and explicit permissions.
