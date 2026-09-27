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

- `setup/install-runner-v1.ps1` installs the hash-pinned, least-privilege
  one-shot runner only after capturing recovery preimages, staging under
  administrator-only ACLs and quiescing the old reset task.
- `setup/restore-runner-v1.ps1` disables/quiesces the runner tasks and restores
  the captured executable, policy, task security and ACL preimages while
  retaining audit/results.
- `testing/check.ps1` runs the normal Rust quality gates.
- `testing/check-docs.ps1` validates tracked local Markdown link targets, prompt
  frontmatter and diff whitespace.
- `testing/build-probe-shaders.ps1` deterministically rebuilds the pinned D3D11
  DXBC and D3D12 DXIL offscreen shaders from the reviewed HLSL source.
- `setup/prepare-cuda-probe.ps1` downloads and hash-verifies only the pinned,
  repository-local CUDA 13.4.1 compiler redistributables, CMake 4.4.3 and CUDA
  Samples commit; it does not install software or change host environment state.
- `testing/build-cuda-probe.ps1` builds the unchanged pinned `vectorAddDrv`
  sample for `sm_120`, verifies the FATBIN and retains its source/license/artifacts.
- `testing/run-host-probe-controls.ps1` performs the bounded host warm-up and
  three measured correctness repetitions for D3D11, D3D12 and CUDA, enforcing
  output caps, timeouts, exact output oracles and cross-API LUID identity. Run
  it with `-SelfTest` to exercise timeout, overflow, cleanup and failure capture.
