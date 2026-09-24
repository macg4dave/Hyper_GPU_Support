---
agent: agent
description: Make a focused Windows-native GPU-PV project change
---

Implement the requested change with the smallest coherent patch.

- Identify the owning component and keep the Windows GPU-PV architecture minimal.
- Inspect relevant implementation, tests, and documentation before editing.
- Prefer Rust and existing Windows/Hyper-V management interfaces, with minimal
  PowerShell glue where useful; follow established local build conventions.
- Use AppSandbox as a reference, recording provenance for adaptations without
  importing unrelated features or compatibility requirements.
- Handle errors explicitly and isolate privileged host/guest side effects.
- Add focused regression coverage, including failure and boundary cases where
  applicable.
- Update affected documentation, support status, and backlog notes.
- Run the relevant documented checks and report exact commands, results, and
  blockers.
- If validation requires host, driver, network, VM, signing, or guest mutation,
  stop at that boundary and request approval with a recovery path.
