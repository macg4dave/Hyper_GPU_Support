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

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
