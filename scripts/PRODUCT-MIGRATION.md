# PowerShell product migration audit

Reviewed 2026-10-06 against the maintained `scripts/` tree, embedded adapters in
`src/`, public CLI dispatch and the existing delivery cards. Classification is by
capability required for ordinary product use, including privileged installation
and recovery. Completed Rust orchestration does not imply a native Windows backend.
Keep working adapters until their replacements pass equivalent tests and affected
live qualification; no feasibility investigation or recipe minimisation is needed.

## Product/development boundary audit

GPU-PV feasibility is established. Users supply an existing VM; golden images,
disposable clones and laboratory rebuilds are development infrastructure.

| Sources | Classification | Boundary and remaining work |
|---|---|---|
| `src/main.rs`, `cli.rs`, `config.rs` | Product functionality | Configuration and dispatch. Mandatory golden-parent/child pins, reset deadlines and recreate-only recovery remain laboratory coupling; CORE-021 separates them. CORE-010 recovery must preserve user disks. |
| `inventory.rs`, `driver_environment.rs`, `environment_staging.rs`, `staging.rs`, `guest.rs`, `vm_settings.rs`, `validation.rs`, `probe.rs`, `account_rights.rs`, `runner.rs` | Reusable library functionality | Product contracts and orchestration. Reset protocol records remain compatibility data; default `policy_allows` denies reset. |
| `windows_inventory.rs`, `windows_native_inventory.rs`, `windows_driver_environment.rs`, `windows_environment_staging.rs`, `windows_guest.rs`, `windows_validation.rs`, `windows_probe.rs`, `windows_paths.rs`, `windows_account_rights.rs`, `windows_runner.rs`, `validation_transport.ps1` | Product functionality | Windows adapters and bounded workers. Preserve working code during native migration. Recreate-only staging uncertainty needs existing-VM recovery integration, not automatic disk destruction. |
| `windows_hyperv.rs`, `windows_hyperv_read.rs`, `windows_hyperv_wmi.rs`, `windows_hyperv_disk.rs` | Reusable library functionality | Native management/provider and read-only disk guards. Reset effects, reset-only provider templates and retained drive state compile only with `dev-harness`. Mandatory parent guards remain coupling for CORE-021/027. |
| `src/bin/hyper-gpu-runner.rs`, `hyper-gpu-client.rs`, `hyper-gpu-rights.rs`, `hyper-gpu-inventory-worker.rs`, `hyper-gpu-validation-worker.rs`, `hyper-gpu-stage.rs`, `hyper-gpu-guest-copy.rs` | Product functionality | Privileged boundary and product helpers. Reset dispatch/body is development-only. CORE-027 must enroll an existing VM without requiring a golden parent. |
| `src/bin/d3d11-probe.rs`, `d3d12-probe.rs`, `cuda-identity.rs` | Product functionality | Checked workload verification where appropriate; compilation/enumeration alone is not GPU proof. |
| `src/bin/dxgi-inventory.rs`, `hyper-gpu-driver-environment.rs` | Development tooling | Standalone inspection/comparison front ends; underlying discovery is reusable product code. |
| `src/bin/hyper_gpu_runner/settings_adapter.rs`, `settings_fakes.ps1`, `supervision.rs`, inline parser/process fixtures and module unit tests | Test-only | Historical adapter/oracle and supervision fixtures are `cfg(test)`, outside product execution. |
| `tools/test-harness/reset_*.rs` | Development tooling | Preserved fixed-runner reset, child deletion/recreation and reattachment; explicit `dev-harness` only, excluded from shipped functionality. |
| Maintained scripts listed below | Development tooling or product debt as specified | Target preparation, artifact building, comparisons and manual diagnostics remain outside product execution. Runner account/ACL/task installation is product debt; golden-image enrollment is laboratory support. |
| Ignored `local/scripts/` and historical HCS/reference procedures | Task-local development tooling or obsolete experiment | Not shipped. Preserve useful helpers/evidence; HCS work stays paused. No maintained automation is deleted by this audit. |

Qualification may use development reset to prepare targets and then exercise the
product. Target preparation does not gate native product migration or substitute
for public CLI behavior. Default release builds exclude the harness. Remaining
configuration/enrollment separation proceeds incrementally under CORE-021/027,
preserving the current laboratory's enrolled safety checks.

## Every maintained script

| Script | Classification | Retention or Rust replacement |
|---|---|---|
| [common/project-config.ps1](common/project-config.ps1) | Development only | Scalar TOML reader for tooling; product already uses `src/config.rs`. Runtime configuration selection remains CORE-021. |
| [setup/update-project-pins.ps1](setup/update-project-pins.ps1) | Still providing product functionality | Artifact pin maintenance stays development tooling; installed policy generation/configuration changes move to CORE-027 with CORE-021. |
| [setup/install-runner-v1.ps1](setup/install-runner-v1.ps1) | Still providing product functionality | Privileged enrollment, accounts, ACLs, task registration and installation preimages move to CORE-027; existing Rust LSA helper is reused. |
| [setup/restore-runner-v1.ps1](setup/restore-runner-v1.ps1) | Still providing product functionality | Exact-SID teardown and preimage restoration move to CORE-027. Retain as development recovery until demonstrated. |
| [setup/prepare-cuda-probe.ps1](setup/prepare-cuda-probe.ps1) | Development only | Pinned SDK/sample download and hash verification for building probe artifacts; ordinary validation consumes packaged, verified inputs. |
| [testing/check.ps1](testing/check.ps1) | Development only | Cargo quality gates and generated configuration drift checks. |
| [testing/check-project-config.ps1](testing/check-project-config.ps1) | Development only | Checked-in configuration/policy drift check; no ordinary application dependency. |
| [testing/check-docs.ps1](testing/check-docs.ps1) | Development only | Markdown links, prompt metadata and whitespace checks. |
| [testing/build-probe-shaders.ps1](testing/build-probe-shaders.ps1) | Development only | Compile pinned shader inputs during artifact preparation; no shader compiler step for ordinary users. |
| [testing/build-cuda-probe.ps1](testing/build-cuda-probe.ps1) | Development only | Build the pinned CUDA sample/FATBIN with provenance; runtime orchestration is Rust-owned. |
| [testing/run-host-probe-controls.ps1](testing/run-host-probe-controls.ps1) | Diagnostic only | Optional host control/oracle comparison and process self-tests; public guest verification uses the Rust worker. Do not call this script from product validation. |
| [testing/run-clean-child-qualification.ps1](testing/run-clean-child-qualification.ps1) | Development only | GPU-006 acceptance harness and evidence capture. Its staging/settings/start order is also needed by the product and belongs in CORE-006; the harness cannot substitute for public apply. |
| [diagnostics/inspect-disposable-gpu-runtime.ps1](diagnostics/inspect-disposable-gpu-runtime.ps1) | Diagnostic only | Manual read-only guest failure investigation; product health/reporting belongs to CORE-003/CORE-012 and CORE-026 transport migration. |
| [diagnostics/inspect-gpu-pv-timeline.ps1](diagnostics/inspect-gpu-pv-timeline.ps1) | Diagnostic only | Manual read-only PnP/event timeline comparison; ordinary diagnostic reports belong to CORE-012. |

There are 14 maintained scripts: eight development only, three diagnostic only,
three still providing product functionality, and no maintained temporary experiment.
Ignored `local/scripts/` procedures are task-local experiments/support, outside this
maintained inventory. No script is removed by this audit.

## Embedded PowerShell is also product debt

The rows below describe migration ownership and earlier qualification, not a new
requirement to port development reset. Disposable reset is excluded from CORE-025's
product gate and retained only in the opt-in development harness. Current native
Hyper-V implementation must be qualified before changing installed artifact pins.

| Capability and current source | Already Rust-owned | Remaining replacement |
|---|---|---|
| Host/GPU/VM inventory: `src/windows_inventory.rs`, `src/windows_native_inventory.rs` | Native registry/system/WMI facts, exact configured identities, typed availability and contained fixed Rust worker | CORE-024 replacement demonstrated against all 14 existing host/GPU/VM facts; production inventory no longer invokes PowerShell. Full payload discovery remains the existing native implementation. |
| GPU attach/detach, disk chain, inspect/start/shutdown: `src/bin/hyper-gpu-runner.rs`, `src/windows_hyperv.rs`, `src/windows_hyperv_disk.rs` | Enrolled identity, authorization, serialization, audit, contained fixed native WMI workers, Virtual Disk chain guards and independent provider readback. Reset effects are non-default development helpers. Historical cmdlet bodies compile only as test references. | CORE-025: demonstrate live native attachment/removal, settings and lifecycle qualification and affected essential workloads; inspection alone does not qualify mutations. |
| VM profile/resource read/apply: `src/windows_hyperv.rs` | Native provider field reads/writes, typed validation, stale-preimage refusal, durable preimages, reconciliation and independent readback; in-memory CIM fixtures exercise production decoding and the native publication sequence | CORE-025: live apply/readback/reapply and affected workload qualification. `src/bin/hyper_gpu_runner/settings_adapter.rs` is now only a historical cmdlet comparison fixture; CORE-023 remains its original tested result. |
| Guest copy/provisioning: `src/windows_guest.rs`, `src/windows_environment_staging.rs` | Complete manifest generation, host signatures/hashes, destination validation and receipt contract | CORE-026: Rust guest writer owns destination calculation, ACL/reparse/servicing checks, exclusive locks, copies, hashes, versions and atomic receipt publication. Current scripts perform substantial guest-side logic, beyond session glue. |
| Guest verification launch: `src/validation_transport.ps1`, `src/windows_validation.rs` | Rust worker owns sustained Code 0/device health, nvidia-smi and D3D11/D3D12/CUDA correctness, deadlines and report parsing | CORE-026: move remaining remote ACL/path/hash/file and launch supervision logic to fixed Rust worker/bootstrap. Investigate the smallest supported session/transfer bridge; PowerShell Direct is not a blanket exemption. |
| Plan/apply/status/remove/recover/lifecycle and re-stage | Contracts, staging/settings helpers and worker exist; most public commands are stubs | CORE-006/010/011/015/021: integrate typed native operations into one configured CLI. CORE-012 supplies Rust diagnostics. No shell qualification harness in the ordinary flow. |
| Runner install/restore and policy generation | Rust account-right helper, client/runner trust checks and typed configuration | CORE-027: native account/security/Task Scheduler installation and recovery; CORE-017 packages the result. |

`src/bin/hyper_gpu_runner/settings_fakes.ps1` is test-only cmdlet emulation,
classified **Development only**. Inline PowerShell process/parser fixtures are
also test-only. Native `src/windows_driver_environment.rs`, probe code, guest
validation worker and existing Rust contract modules should be reused, not rewritten.

## Order and acceptance

CORE-024 read-only inventory is native; the next slice is CORE-025, replacing privileged
Hyper-V adapters incrementally. CORE-026 replaces guest writer logic and narrows or
eliminates the remoting bridge. CORE-027 covers product setup/recovery. Public CLI
integration can proceed alongside these bounded migrations. Existing GPU-006 work
qualifies integration of the current adapters; it does not close native migration.

Each replacement needs focused success/failure/identity/timeout coverage and parity
of discovery/mapping/verification behavior. No historical file count defines success.
Rediscover the current signed host driver's complete associated payload; manifest
size and hashes are run data. Use variable-size artificial fixtures and retain
the measured baseline only as explicitly historical regression/evidence data.
Compare relevant working output and qualify affected baseline behavior before
removing the production script dependency. Privileged/security milestones require
independent architecture review. Do not improve product PowerShell except for a
necessary correctness/safety fix or a bounded behavior comparison needed for porting.

An exception must identify the missing capability, investigated Rust/COM/WMI/Win32
routes, exact external executable/arguments, typed input/result validation, bounded
execution, errors and recovery. Record the narrow result in DECISIONS. Existing
DEC-013/016/020 explain historical adapters; they do not establish v1 exceptions.

V1 acceptance executes installation/configuration, plan/apply/status, guest lifecycle,
verification, removal/recovery and explicit restaging using the packaged Rust CLI.
No manual `.ps1` steps or embedded product PowerShell beyond a specifically justified
interface exception; development/diagnostic scripts remain optional.
