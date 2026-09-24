---
agent: agent
description: Change a public or headless API without contract drift
---

Make the requested API change.

1. Locate the current API documentation, implementation, client, and tests.
2. Prefer additive, backward-compatible behavior unless a breaking change is
   explicit.
3. Update the contract documentation alongside the implementation.
4. Validate missing, malformed, boundary, and unsupported inputs.
5. Keep platform-specific effects behind existing core/backend boundaries.
6. Update examples, SDK behavior, changelog/backlog notes, and tests affected by
   the contract.
7. Run the focused checks documented by the repository and report exact results.

Do not invent an OpenAPI schema or generated artifact unless the repository
actually adopts one.

