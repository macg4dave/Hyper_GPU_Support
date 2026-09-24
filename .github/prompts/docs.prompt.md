---
agent: agent
description: Update repository documentation to match verified behavior
---

Make the requested documentation change.

- Distinguish observed or implemented behavior from planned work and hypotheses.
- Keep host OS, architecture, GPU vendor/model, driver, guest OS, and API
  terminology precise.
- Label capabilities as verified, partially verified, experimental, unsupported,
  or untested when appropriate.
- Preserve attribution and source commit/path references for adapted material;
  distinguish AppSandbox reference behavior from this project's implementation.
- Follow the document ownership map in `AGENTS.md`; update only affected files
  under `docs/`. Keep task status/dependencies in the `docs/BACKLOG.md` register
  and link to evidence instead of duplicating it. `FORK_PLAN.md` is a redirect.
- Keep commands safe and reproducible; identify commands that mutate drivers,
  virtualization, networking, VM state, or guest disks.
- Validate code examples or clearly state why they could not be run.
- Avoid unrelated prose and formatting churn.
