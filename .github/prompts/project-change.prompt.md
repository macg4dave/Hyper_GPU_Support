---
agent: agent
description: Make a focused Windows-native GPU-PV project change
---

Implement the requested change with the smallest coherent patch.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[Rust and Windows](../../docs/ENGINEERING.md#rust-and-native-windows),
[shell commands and development scripts](../../docs/ENGINEERING.md#shell-commands-and-development-scripts),
[module design](../../docs/ENGINEERING.md#code-and-module-design),
[testing](../../docs/ENGINEERING.md#testing), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Identify the owning component and keep the Windows GPU-PV architecture minimal.
- Inspect the affected implementation and immediate dependencies, establish the
  relevant test baseline, and preserve behavior outside the task's acceptance.
- Implement functionality in Rust wherever technically possible. Investigate
  Rust-native alternatives and document a technical necessity before adding a
  language exception; convenience or upstream language choice is insufficient.
- Use AppSandbox as a reference, recording provenance for adaptations without
  importing unrelated features or compatibility requirements.
- Handle errors explicitly and isolate privileged host/guest side effects.
- Add behavioral coverage for meaningful new or changed logic, including failure,
  boundary, cleanup, and bug regression cases as applicable.
- Update affected documentation, support status, and backlog notes.
- Run the required checks and report exact commands, results, and blockers.
