---
agent: agent
description: Update repository documentation to match verified behavior
---

Make the requested documentation change.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[documentation](../../docs/ENGINEERING.md#documentation),
[shell commands and development scripts](../../docs/ENGINEERING.md#shell-commands-and-development-scripts), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Distinguish observed or implemented behavior from planned work and hypotheses.
- Keep host OS, architecture, GPU vendor/model, driver, guest OS, and API
  terminology precise.
- Label capabilities as verified, partially verified, experimental, unsupported,
  or untested when appropriate.
- Preserve attribution and source commit/path references for adapted material;
  distinguish AppSandbox reference behavior from this project's implementation.
- Follow the document ownership map and update only material affected behavior,
  contracts, Rust docs or examples. Correct stale summaries/status inline; do not
  create a task or broader audit unless a technical or safety decision depends on it.
- Keep commands safe and reproducible; identify commands that mutate drivers,
  virtualization, networking, VM state, or guest disks.
- Check links touched by the change and validate changed code examples/doctests.
  Do not audit unrelated documents merely to reconfirm earlier checks.
- Avoid unrelated prose and formatting churn.
