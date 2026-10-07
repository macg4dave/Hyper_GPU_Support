---
agent: agent
description: Evolve a public Rust interface with a documented caller contract
---

Change the named public interface.

Use the [API change workflow](api-change.prompt.md) under
[AGENTS.md](../../AGENTS.md). Confirm which callers require public access.
Document the caller contract and add useful runnable examples/doctests; update
affected compatibility tests. Do not create a public SDK for an internal CLI need.

The user-approved architecture rebase takes precedence over historical task scope.
Use runtime existing-VM identities and the revised core → native GUI → allocation/
vendor roadmap. Old fixed-slot code is research/contributor tooling in `tools/lab/`.
Do not preserve laboratory coupling, baseline driver pins or repeated diagnosis as
product architecture. Default operation verifies health plus checked graphics;
extended CUDA/stress remain optional. The core never imports the laboratory.
