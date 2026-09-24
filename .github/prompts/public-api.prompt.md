---
agent: agent
description: Evolve a public Rust interface with a documented caller contract
---

Change the named public interface.

Follow [AGENTS.md](../../AGENTS.md), the shared
[API change workflow](api-change.prompt.md), and the engineering standards for
[documentation](../../docs/ENGINEERING.md#documentation).

- Confirm which callers need public access before expanding visibility.
- Document inputs, outputs, errors, ownership, and any cancellation, timeout, or
  safety obligations in the caller-facing contract.
- Add runnable examples or doctests where they clarify intended use; update
  existing public examples and compatibility tests affected by the change.
