---
agent: agent
description: Review a diff for behavior, contract, documentation, and scope drift
---

Review the current diff. Lead with concrete findings, ordered by severity.

Follow [AGENTS.md](../../AGENTS.md) and review against the relevant sections of
[ENGINEERING.md](../../docs/ENGINEERING.md), especially
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

For each finding, provide:

- file and line or section;
- the behavior, contract, boundary, or rule that drifted;
- likely runtime, security, compatibility, licensing, or maintenance impact;
- the smallest correction.

Check Rust policy and exception rationale, module boundaries, error preservation,
unsafe invariants, resource cleanup, meaningful test coverage, lint suppressions,
and unnecessary dependencies. Also check unsupported GPU claims, missing
environment evidence, attribution/notices, proprietary binary redistribution,
unreviewed protected mutations, stale capability documentation, and unrelated scope.
Report only checks actually run. Passing tests do not replace review; if there
are no findings, say so and name residual validation gaps.
