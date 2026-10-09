---
agent: agent
description: Update build or CI automation without weakening quality gates
---

Make the requested build, packaging, or CI change.

Follow [AGENTS.md](../../AGENTS.md) and
[required checks/CI](../../docs/ENGINEERING.md#required-checks-and-ci).

- Inspect the actual Cargo, toolchain, lockfile and workflows. Preserve strict
  Windows checks, failure reporting and separately selected privileged tests.
- Keep automation proportional to the CLI; avoid unrelated platforms or release
  infrastructure. Pin downloaded inputs and retain license/provenance details.
- Validate affected commands and feature sets. Update build documentation only
  when commands or coverage change; report exact results and unrun checks.

## Testing Policy — Codex

Stop running tests, builds, compilation checks, or launching the UI after every small change.

Follow these rules:

1. **During development:** Make changes without automatically running tests, `cargo check`, `cargo test`, `cargo build`, or launching the GUI.
2. **At milestone completion:** Run relevant tests and build checks once, after all work for that milestone is complete.
3. **On explicit request:** Run tests whenever I specifically instruct you to.
4. **Small changes:** Do not test individual edits, UI adjustments, layout changes, refactoring, or documentation updates.
5. **Failures:** If a milestone test fails, fix the relevant issue and rerun only the necessary checks. Avoid repeatedly running the entire test suite.
6. **Exceptions:** If you believe immediate testing is essential, explain why and request permission first.

Prioritise implementing the planned work over repeatedly validating intermediate states.

**Important:** Do not interpret completing an individual task or subtask as completing a milestone. A milestone is complete only when all its planned tasks are finished.

At the end of each task, briefly report what changed and whether it remains untested. Do not automatically start validation.

This policy overrides existing instructions to test continuously unless I explicitly tell you otherwise.
