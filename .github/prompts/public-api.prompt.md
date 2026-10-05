---
agent: agent
description: Evolve a public Rust interface with a documented caller contract
---

Change the named public interface.

Use the [API change workflow](api-change.prompt.md) under
[AGENTS.md](../../AGENTS.md). Confirm which callers require public access.
Document the caller contract and add useful runnable examples/doctests; update
affected compatibility tests. Do not create a public SDK for an internal CLI need.
