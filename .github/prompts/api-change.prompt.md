---
agent: agent
description: Change a Rust module or public interface without contract drift
---

Make the requested API change.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[module design](../../docs/ENGINEERING.md#code-and-module-design),
[errors and lifetimes](../../docs/ENGINEERING.md#errors-and-operation-lifetimes),
and [required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

1. Map the existing interface, callers, implementation, documentation, and tests.
2. Keep visibility and surface area minimal; preserve contracts unless the task
   explicitly changes them. For a breaking change, document the rationale,
   affected callers, migration, and any applicable version impact.
3. Keep Windows effects behind focused interfaces and the core GUI-independent.
4. Test valid, malformed, boundary, unsupported, and failure behavior as relevant,
   including ownership and cleanup obligations across the interface.
5. Update affected callers, Rust documentation, examples, and contract tests in
   the same change. Do not introduce SDKs, schemas, or generated artifacts unless
   the task requires them.
