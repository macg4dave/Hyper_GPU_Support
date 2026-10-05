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
