---
agent: agent
description: Change a Rust module or public interface without contract drift
---

Make the requested API change.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Inspect the interface and affected callers; keep visibility and surface minimal.
- Preserve contracts unless explicitly changed. Explain breaking changes and
  migration, including ownership, errors, cleanup and cancellation obligations.
- Update callers, Rust docs and meaningful contract tests together. Introduce no
  SDK or schema unless the requested behavior needs it.
