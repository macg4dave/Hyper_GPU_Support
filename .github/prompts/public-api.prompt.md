---
agent: agent
description: Evolve a public Rust interface with a documented caller contract
---

Change the named public interface.

Use the [API change workflow](api-change.prompt.md) under
[AGENTS.md](../../AGENTS.md). Confirm which callers require public access.
Document the caller contract and add useful runnable examples/doctests; update
affected compatibility tests. Do not create a public SDK for an internal CLI need.

The user-approved architecture rebase takes precedence over historical task scope.
Use runtime existing-VM identities and the revised core → native GUI → allocation/
vendor roadmap. Old fixed-slot code is research/contributor tooling in `tools/lab/`.
Do not preserve laboratory coupling, baseline driver pins or repeated diagnosis as
product architecture. Default operation verifies health plus checked graphics;
extended CUDA/stress remain optional. The core never imports the laboratory.

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
