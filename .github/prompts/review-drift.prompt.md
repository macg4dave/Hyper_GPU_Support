---
agent: agent
description: Review a diff for behavior, contract, documentation, and scope drift
---

Review the current diff. Lead with concrete findings, ordered by severity.

For each finding, provide:

- file and line or section;
- the behavior, contract, boundary, or rule that drifted;
- likely runtime, security, compatibility, licensing, or maintenance impact;
- the smallest correction.

Also check for unsupported GPU claims, missing environment evidence, accidental
loss of attribution/notices, proprietary binary redistribution, unreviewed host
mutations, unnecessary cross-platform scope, missing regression coverage, stale
support matrices, and unrelated changes. Passing tests are evidence, not a substitute
for review. If there are no findings, say so and name residual validation gaps.
