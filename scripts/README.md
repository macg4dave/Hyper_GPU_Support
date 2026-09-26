# Development scripts

This tree contains maintained, reviewable scripts for repository development and
Windows test-environment operations. Use the smallest applicable category:

| Directory | Purpose |
|---|---|
| `setup/` | Repeatable development or test-environment setup |
| `diagnostics/` | Read-only environment and failure investigation |
| `hyperv/` | Explicitly scoped Hyper-V guest/test operations |
| `testing/` | Repository checks and test orchestration |

Run short commands directly. Put substantial PowerShell, multi-step procedures,
complex pipelines, conditional logic and meaningful failure handling in a named
script here, then run that file. Follow the authoritative
[engineering script standard](../docs/ENGINEERING.md#shell-commands-and-development-scripts)
and [permission boundary](../AGENTS.md#permission-boundary). A script's presence
does not authorize its effects.

Keep temporary, task-local scripts in ignored `local/scripts/`, not here. This
tree is for useful tooling that should be reviewed, debugged, modified and reused.
PowerShell may support development, diagnostics, setup and native Windows/Hyper-V
facilities; application behavior remains implemented in Rust.

Current maintained entry points:

- `testing/check.ps1` runs the normal Rust quality gates.
- `testing/check-docs.ps1` validates tracked local Markdown link targets, prompt
  frontmatter and diff whitespace.
