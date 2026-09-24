---
agent: agent
description: Evolve a public API while preserving compatibility and documentation
---

Change the named public API or SDK surface.

- Map all local callers, implementations, documentation, examples, and tests.
- Keep visibility and surface area minimal.
- Prefer additive compatibility; for a breaking change, document rationale,
  migration, and version impact.
- Use structured errors and avoid leaking platform implementation details.
- Keep the core usable without a GUI; do not introduce cross-platform or
  AppSandbox API compatibility requirements.
- Add contract tests for valid, invalid, boundary, and unsupported operations.
- Update API documentation and all shipped clients/examples in the same change.
- Run focused validation and report exact results.
