# Development scripts

This tree contains maintained, reviewable scripts for repository development and
Windows test-environment operations. Use the smallest applicable category:

| Directory | Purpose |
|---|---|
| `common/` | Shared, side-effect-free project-configuration reader |
| `setup/` | Repeatable development or test-environment setup |
| `diagnostics/` | Read-only environment and failure investigation |
| `hyperv/` | Explicitly scoped Hyper-V guest/test operations |
| `testing/` | Repository checks and test orchestration |

Run short commands directly. Put substantial PowerShell, multi-step procedures,
complex pipelines, conditional logic and meaningful failure handling in a named
script here, then run that file. Follow the authoritative
[engineering script standard](../docs/ENGINEERING.md#shell-commands-and-development-scripts),
[Windows elevation policy](../docs/ENGINEERING.md#windows-elevation-and-uac)
and [development/test authorization](../AGENTS.md#development-and-test-authorization).
Normal project effects on the designated disposable VM are authorized; physical-
host lifecycle is the explicit permission boundary.

Keep temporary, task-local scripts in ignored `local/scripts/`, not here. This
tree is for useful tooling that should be reviewed, debugged, modified and reused.
PowerShell may support development, diagnostics, setup and native Windows/Hyper-V
facilities; application behavior remains implemented in Rust.

Current maintained entry points and their required privilege:

- **Non-elevated** — `common/project-config.ps1` reads the supported scalar subset of the authoritative
  `config/project.toml`; maintained scripts use it instead of copying mutable values.
- **Non-elevated** — `setup/update-project-pins.ps1` regenerates the installed runner-policy artifact
  and updates or verifies hash-pinned release binaries after a reviewed build.

- **Elevated** — `setup/install-runner-v1.ps1` installs the hash-pinned, least-privilege
  one-shot runner only after capturing recovery preimages, staging under
  administrator-only ACLs and quiescing the old reset task. Its fixed Rust LSA
  helper adds only the five reviewed batch/deny rights to the newly enrolled SID;
  `#Requires -RunAsAdministrator` fails before execution under the wrong token.
- **Elevated** — `setup/restore-runner-v1.ps1` disables/quiesces the runner tasks and restores
  the captured executable, policy, task security and ACL preimages, removes only
  those rights from the persisted exact SID, and retains audit/results;
  `#Requires -RunAsAdministrator` fails before execution under the wrong token.
- **Non-elevated** — `testing/check.ps1` runs the normal Rust quality gates and generated
  configuration/policy drift check.
- **Non-elevated** — `testing/check-project-config.ps1` validates the shared
  configuration and generated runner policy without changing protected state.
- **Non-elevated** — `testing/check-docs.ps1` validates repository-local Markdown link targets, prompt
  frontmatter and diff whitespace.
- **Non-elevated** — `testing/build-probe-shaders.ps1` deterministically rebuilds the pinned D3D11
  DXBC and D3D12 DXIL offscreen shaders from the reviewed HLSL source.
- **Non-elevated** — `setup/prepare-cuda-probe.ps1` downloads and hash-verifies only the versions,
  archives and CUDA Samples revision pinned under `tooling` in project configuration;
  it does not install software or change host environment state.
- **Non-elevated** — `testing/build-cuda-probe.ps1` builds the unchanged pinned `vectorAddDrv`
  sample for `sm_120`, verifies the FATBIN and retains its source/license/artifacts.
- **Non-elevated** — `testing/run-host-probe-controls.ps1` performs the configured bounded host warm-up
  and measured correctness repetitions for D3D11, D3D12 and CUDA, enforcing
  output caps, timeouts, exact output oracles and cross-API LUID identity. Run
  it with `-SelfTest` to exercise timeout, overflow, cleanup and failure capture.
