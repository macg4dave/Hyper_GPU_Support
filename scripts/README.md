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
PowerShell may support development, diagnostics and development-machine setup.
Any functionality required by ordinary product operation, installation or recovery
belongs in Rust unless a concrete interface limitation is documented narrowly.
The [classification audit](PRODUCT-MIGRATION.md) separates the Rust product
from the standalone `tools/lab` application. Runner-v1 setup/restore/pin scripts
maintain the laboratory runner only; product installation and enrollment are native
Rust. `testing/check.ps1` checks the root product without laboratory configuration.
Provisioning/qualification scripts consume the dynamically discovered manifest
and compare receipts for that run. Do not gate them on a historical file count
or supply a static NVIDIA runtime list; driver updates require rediscovery.

Disposable reset is compiled only with `cargo build --manifest-path tools/lab/Cargo.toml --target-dir local/lab-target --locked --release --features
dev-harness`. Rebuild/re-pin the runner using the existing reviewed setup flow
before clean-child qualification; a default runner denies `reset-slot`. Never
ship a `dev-harness` artifact. Product guest lifecycle and GPU management remain
available in default builds.

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
- **Non-elevated** — `testing/check.ps1` runs the root product Rust quality and
  documentation gates without laboratory configuration.
- **Elevated** — `testing/qualify-product-enrollment.ps1` verifies the designated
  disposable VM/GPU/disk identities, qualifies native product enrollment, failed
  installation admission and reinstall recovery, and checks the fixed task and
  artifact integrity. It invokes no guest mutation. Write results under `local/`.
- **Non-elevated** — `testing/check-project-config.ps1` validates the shared
  configuration and generated runner policy without changing protected state.
- **Non-elevated** — `testing/check-docs.ps1` validates repository-local Markdown link targets, prompt
  frontmatter and diff whitespace.
- **Elevated** — `testing/run-clean-child-qualification.ps1` recreates only the enrolled
  disposable child through the fixed Rust runner, invokes Rust full staging/reapply,
  settings/readback, public validation and graceful shutdown, and retains a combined
  report under configured test output. Guest credentials are entered only at local
  Rust prompts; `-KeepOpen` retains the interactive terminal after completion/failure.
  `-PreparedRun <report.json>` resumes a matching successful reset/start that has no
  staging receipt, with fresh enrolled-target inspection and Rust staging guards;
  uncertain/partial staging still requires child recreation. The preparation report
  and executing harness are archived with hashes, and native staging/validation
  stderr is retained without capturing console password input or feedback.
- **Non-elevated** — `testing/build-probe-shaders.ps1` deterministically rebuilds the pinned D3D11
  DXBC and D3D12 DXIL offscreen shaders from the reviewed HLSL source.
- **Non-elevated** — `setup/prepare-cuda-probe.ps1` downloads and hash-verifies only the versions,
  archives and CUDA Samples revision pinned under `tooling` in project configuration;
  it does not install software or change host environment state.
- **Non-elevated** — `testing/build-cuda-probe.ps1` builds the unchanged pinned `vectorAddDrv`
  sample for `sm_120`, verifies the FATBIN and retains its source/license/artifacts.
- **Elevated** — `diagnostics/inspect-disposable-gpu-runtime.ps1` verifies the running pinned
  disposable VM, reads its GPU-PV PnP status, package anchors, NVIDIA runtime query and
  recent guest System events through an interactive PowerShell Direct credential,
  then writes an ignored local diagnostic. It makes no guest or VM change.
- **Elevated** — `diagnostics/inspect-gpu-pv-timeline.ps1` verifies the same disposable
  target, then collects read-only VMBus/PnP properties and timestamped guest
  events for the initial or controlled-repeat GPU-PV interval into ignored
  local evidence. It uses an interactive PowerShell Direct credential.
- **Non-elevated** — `testing/run-host-probe-controls.ps1` performs the configured bounded host warm-up
  and measured correctness repetitions for D3D11, D3D12 and CUDA, enforcing
  output caps, timeouts, exact output oracles and cross-API LUID identity. Run
  it with `-SelfTest` to exercise timeout, overflow, cleanup and failure capture.
