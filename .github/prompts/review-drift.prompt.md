---
agent: agent
description: Review a diff for behavior, contract, documentation, and scope drift
---

Review the current diff. Lead with concrete findings, ordered by severity.

Follow [AGENTS.md](../../AGENTS.md) and review against the relevant sections of
[ENGINEERING.md](../../docs/ENGINEERING.md).

For each finding, provide:

- file and line or section;
- the behavior, contract, boundary, or rule that drifted;
- likely runtime, security, compatibility, licensing, or maintenance impact;
- the smallest correction.

Focus on changed behavior, Rust/FFI correctness, target safety, recovery, useful
tests, contracts and required attribution. Trust established baseline results;
inspect historical evidence only for a concrete regression. Report actual checks
and material gaps. If there are no findings, say so.

Check product PowerShell dependencies, including embedded scripts. Use the affected
[migration audit](../../scripts/PRODUCT-MIGRATION.md) row to distinguish optional
tooling from normal CLI/setup/recovery behavior. Flag new shell wrappers, broad
interface exceptions and native-completion claims unsupported by the actual backend.
Require demonstrated replacement before deleting a working adapter; preserve the
runner trust boundary and uncertainty/recovery behavior through the port.
Flag fixed payload counts, static NVIDIA file/hash lists and reuse of old driver
entries as current inventory. Manifest/receipt comparisons must bind the freshly
discovered run; exact historical fixture assertions remain legitimate when labelled.

GPU-PV feasibility is established; implement and validate the product for an
existing selected VM. Reuse disposable-VM tooling for tests. Golden-image copying,
cloning, disk reset and laboratory setup stay outside the production path unless
an explicit user-facing roadmap task requires them. Test tooling may depend on
product code; product code must not depend on test tooling.

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
