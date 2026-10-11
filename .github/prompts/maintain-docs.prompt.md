---
name: maintain-docs
description: Update authoritative documentation from current source and agreed decisions.
---

Read [AGENTS.md](../../AGENTS.md) and [documentation ownership](../../docs/ENGINEERING.md#documentation-ownership). Use [baseline](../../docs/BASELINE.md), [roadmap](../../docs/ROADMAP.md), [backlog](../../docs/BACKLOG.md), and [decisions](../../docs/DECISIONS.md) for their respective subjects.

1. Inspect current source and actual validation records before changing capability/status claims.
2. Update the authoritative page first, then linked summaries. Preserve fresh stable IDs and unrelated deletions.
3. Separate implemented, incomplete, unknown, and validated behavior. Source paths, old acceptance, simulation, and planned commands are not new recorded passes.
4. Keep machine-specific values/raw evidence in explicit local configuration/evidence. Do not duplicate policy in prompts or create competing roadmaps.
5. Check local links/anchors, IDs/dependencies, status consistency, instructions, and diff whitespace. Report exactly these checks and remaining limitations.

Documentation work alone does not authorize source/configuration edits, builds, tests, GUI launches, or live qualification. Follow [testing](../../docs/TESTING.md) for validation timing.
