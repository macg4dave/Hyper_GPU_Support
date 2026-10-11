---
name: validate-milestone
description: Validate an explicitly requested milestone using its actual gate.
---

Follow [AGENTS.md](../../AGENTS.md), [testing](../../docs/TESTING.md), [roadmap](../../docs/ROADMAP.md), and [backlog](../../docs/BACKLOG.md). Name the milestone and its criteria; a task/subtask is not a milestone.

1. Confirm dependencies and inspect gate/test sources for effects. Verify tooling exists; do not invoke deleted scripts or silently restore old suites.
2. Run only applicable authorized checks. Normally use the non-hardware gate at milestone completion; separate mock previews and privileged/hardware trials.
3. Before live trials prepare a concrete target/effect/evidence/stop-condition run sheet and obtain scoped authorization. This prompt alone does not authorize them.
4. Record revision/working tree, commands, environment, timestamps, results, skipped stages, and limits. Stop at the first failed gate stage; report later stages as unrun.
5. Retain evidence/holds after uncertain live outcomes. Do not automatically rerun, restart the host, force power-off, or conduct unrequested live investigation.
6. Update task validation and milestone status from actual evidence, separating automated correctness, mock appearance, rendering, concurrent sharing, and packaging.

For documentation-only work, use applicable static checks. Such a task does not
close a product milestone or authorize builds, Rust tests, GUI launches, or
hardware trials on its own.
