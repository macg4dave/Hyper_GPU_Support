# GPU-PV architecture

This document specifies the current recipe and implementation boundaries.
[BACKLOG.md](BACKLOG.md) owns work/status; [DECISIONS.md](DECISIONS.md) owns
rationale. Read only the section needed for the current task.

## Current state

Normal Generation 2 Hyper-V / VMMS GPU-PV works on the Windows 11 x64 host and
guest with an NVIDIA RTX 5060 8 GB. The [measured baseline](evidence/GPU-PV-BASELINE.md)
passed sustained Code 0, `nvidia-smi`, checked D3D11/D3D12 hardware rendering and
CUDA allocation, transfer and kernel computation. Native Rust discovery matches
the complete 271-file driver/runtime manifest for that measured driver.

The complete bounded Rust guest writer passed live full-manifest apply and
verified reapply on a clean child. Application of the validated VM/resource
settings and composition with readiness/workload checks are still implementation
work. Experimental success is established; product automation is incomplete.

The v1 product is a Rust CLI and TOML configuration for one configured Windows 11
VM and one RTX 5060. Windows owns virtualization, disk management and GPU-PV.
There is no GUI, scheduler, multi-GPU control plane or HCS VM platform. The
privileged runner is an on-demand operation boundary, not a background GPU service.
Complete provisioning stays intact through v1; minimum-file experiments and
CUDA/D3D interoperability are later work.

Historical: the initial provisioning recipe was developed during reference
research and subsequently validated independently on the target system. External
project research is not an implementation prerequisite.

## Working recipe

### GPU discovery and VM identification

The configured VM GUID is authoritative. Verify its configured name, Generation 2
profile, attached disposable child, exact parent chain/hash and guest identity
before effects. A friendly name alone never selects a mutation target. Preserve
the existing firmware, vTPM and VM identity.

For the selected GPU, native COM/WMI discovery:

1. Confirms the exact configured interface in `Msvm_PartitionableGpu` under
   `root\virtualization\v2` and derives its physical PCI PnP DeviceID.
2. Resolves `Win32_PnPEntity.Service`, `Win32_PNPSignedDriver` model/version/INF
   and `Win32_SystemDriver.PathName` for that DeviceID under `root\cimv2`.
3. Resolves every `Win32_PNPSignedDriverCIMDataFile` association for that signed
   driver through WMI, including files outside its DriverStore package.
4. Validates the physical identity and pinned driver inputs; absent, denied,
   changed or ambiguous results fail without choosing another GPU.

`src/windows_driver_environment.rs` implements these queries without a PowerShell
subprocess. Its configured deadline bounds enumeration; synchronous COM connection
and individual provider resolutions are not cancelled by that deadline.
CORE-006 must supervise full discovery as a bounded child operation before public
plan/apply use; maintenance reuses that path. The current inspector alone does not
provide an end-to-end cancellable CLI operation.

### Hyper-V settings and GPU partition resources

Apply settings while the configured VM is off, then read them back through fresh
provider objects before its attached boot. The validated profile is:

| Setting | Validated value |
|---|---|
| VM type | Generation 2, normal Hyper-V / VMMS ownership |
| Memory / processors | Static 8 GiB, four virtual processors |
| Low / high MMIO | 3 GiB / 32 GiB |
| Guest-controlled cache types | Enabled |
| Expose virtualization extensions | Enabled |
| Checkpoints | Disabled |
| Secure Boot / TPM | Microsoft Windows Secure Boot and existing vTPM retained |
| Automatic stop | Guest shutdown |

The measured VM configuration version was 12.0; it is an observed environment
fact, not a mandate to recreate the VM shell. Mutable recipe settings belong in
typed configuration when application is implemented. Do not weaken Secure Boot,
signing or isolation.

Attach exactly the configured discovered interface with
`Add-VMGpuPartitionAdapter -InstancePath`. An exact existing assignment is a
verified no-op; another adapter, conflicting assignment or running VM is rejected.
Set and independently verify all four resource triples:

| Resource | Minimum = maximum = optimal in the validated profile |
|---|---:|
| VRAM | 500,000,000 |
| Decode | 500,000,000 |
| Compute | 500,000,000 |
| Encode | 9,223,372,036,854,775,808 |

These are opaque provider units, not physical VRAM quantities or performance
percentages. Preserve integer precision through serialization and comparison.
`provider-default` is not equivalent to this profile. The current runner attaches
the adapter but does not apply these settings or resource triples.

### Driver/runtime manifest and guest placement

`src/driver_environment.rs` expands the kernel-service package and every associated
DriverStore package into complete trees, and includes every associated external
file. Sources must be regular files within the discovered Windows directory, with
safe ancestors and no reparse escape. Conflicting destinations, malformed paths
and changed pinned package inputs fail before guest writes.

The deterministic manifest records selected device/driver/INF/service, association
count, package roots and each source, logical destination, byte length and SHA-256.
Files are sorted case-insensitively by destination; the encoded contract has its
own digest. No hand-maintained NVIDIA DLL list determines the closure. 271 is a
measured count rather than a future fixed limit.

| Discovered host path relative to Windows | Guest path relative to Windows |
|---|---|
| `System32\DriverStore\FileRepository\<package>\...` | `System32\HostDriverStore\FileRepository\<package>\...` |
| Every associated external Windows path | Same Windows-relative path |

The measured manifest contains 217 expanded package files and 54 external files,
including System32/SysWOW64 loader/runtime files and other installed driver paths.
Use ordinary byte-preserving copies. The guest binds the inbox virtual-render
driver (`vrd.inf` / VirtualRender); the working recipe did not install a conventional
NVIDIA INF or import NVIDIA registry state.

### Staging and guest startup

The baseline copied and verified the full manifest offline before any GPU-attached
boot. The Rust writer must establish the same complete bytes and destinations
before an attached start. Reusing the existing PowerShell Direct transport is
acceptable if it provides that contract; a second storage/backend abstraction is
not required merely to imitate the experimental transport.

Staging verifies source identity/signatures/hashes, target VM/disk/guest identity,
permitted logical destinations, transfer results and final guest length/hash for
every file. The privileged operation accepts only the validated selected-driver
manifest, never arbitrary host sources, Windows destinations or shell commands.
An applied receipt identifies the full manifest and effective recipe. A matching
reapply verifies every managed destination before reporting a no-op.

Timeouts, interruptions or partial writes leave uncertain guest state and require
disposable-child recreation. A receipt alone does not prove readiness. After
copies, settings and assignment are verified, start the VM through its native
lifecycle boundary and wait for bounded guest readiness. Probe binaries, shaders,
CUDA FATBIN and any required application runtime are separately hash-verified test
artifacts, outside the driver environment manifest.

### GPU readiness and workloads

Check the intended virtual-render devnode repeatedly over a bounded observation
window and report PnP status and relevant VMBus/device errors. The baseline recorded
116 consecutive Code 0 samples over approximately 120 seconds. Product deadlines
and observation settings belong in configuration. Then run `nvidia-smi` and the
essential checked workloads; device status or DLL loading cannot substitute for
computation.

| Check | Required behavior |
|---|---|
| `nvidia-smi` | Query succeeds and identifies the configured GPU/driver |
| D3D11 / D3D12 | Explicit NVIDIA hardware selection, offscreen rendering and verified full readback |
| CUDA | Target device/model/compute capability, allocation, host/device transfers, kernel execution and CPU-checked output |

Rust graphics probes use DXGI and D3DKMT physical IDs to validate configured PCI
identity when GPU-PV DXGI descriptors omit subsystem/revision. They reject software
and indirect-rendering paths; the checked 256×256 frame is a deterministic oracle.
The CUDA `sm_120` vector-add probe checks computation independently. Its identity
companion reports a host/guest LUID mismatch; guest D3D/CUDA LUID equality is not
an essential computation gate. Cross-API interoperability is not claimed.

## Components

Keep one Rust package with a reusable core and thin executable boundaries. Add
modules when implemented behavior needs them. No generic VM backend, guest agent
platform or transaction engine is required.

| Responsibility | Existing source |
|---|---|
| CLI/process output | `src/main.rs`, `src/cli.rs` |
| Typed configuration and plan/report contracts | `src/config.rs` |
| Inventory and full manifest discovery | `src/inventory.rs`, `src/windows_inventory.rs`, `src/windows_driver_environment.rs`, `src/driver_environment.rs` |
| Guest transfer, integrity and receipt | `src/guest.rs`, `src/staging.rs`, `src/windows_guest.rs` |
| Fixed privileged protocol/native operations | `src/runner.rs`, `src/windows_runner.rs`, `src/bin/hyper-gpu-runner.rs`, `src/bin/hyper-gpu-client.rs` |
| Hardware selection and checked workloads | `src/probe.rs`, `src/windows_probe.rs`, probe binaries |

`src/environment_staging.rs` binds all source mappings and receipt evidence;
`src/windows_environment_staging.rs` independently repeats native discovery before
authorizing the complete guest write. `hyper-gpu-stage` uses this full writer.
It shares the existing target/signature/servicing/session/ACL preflight, uses the
same exclusive staging marker and retains it after uncertain effects. Each copy
is verified before publication and all final destinations are rehashed before
publishing the separate full-environment receipt. Fresh staging requires no attached
GPU; matching reapply verifies the saved identities and every managed file.
Live staging qualification passed in [CORE-022](BACKLOG.md#core-022); validated settings are
[CORE-023](BACKLOG.md#core-023), and probe integration is
[CORE-003](BACKLOG.md#core-003). [GPU-006](BACKLOG.md#gpu-006) composes these into
clean-child Rust reproduction.

## Configuration and recovery contract

[`config/project.toml`](../config/project.toml) owns non-secret mutable intent and
test settings. Discover OS/driver inventory through Windows, validate once into
typed values, and retain identity/integrity pins where the installed privileged
policy requires them. [CONFIGURATION.md](CONFIGURATION.md) distinguishes current
settings from remaining integration.

Before effects, revalidate target identity, disk chain, driver inputs, native state
and conflicts under bounded operation locks. Keep proposed changes, observed state
and last successful validation distinct. Record identities, native results, final
readback and useful recovery guidance. External Windows tools do not honor our
locks, so detect changed native state before effects.

Ordinary development runs unelevated. Known privileged operations use the
administrator-installed exact-policy runner or the authorized elevated development
adapter where the runner does not yet expose that operation. Protected executable/
policy pin the slot, VM GUID, GPU, paths, operations and audit output. Mutual pipe
authentication verifies runner owner/client SID before dispatch. Fixed Windows
cmdlet transports are supervised and bounded; callers cannot supply scripts.
Inspection/reset supervision observes native process read-byte activity, with an
inactivity watchdog inside a shared finite runner budget. Startup/request time
and transition/publication reserves count against that budget; the client waits
for the outer limit. Activity never substitutes for the parent hash or native
readback. A failed or uncertain mutation retains reconciliation state.
Guest credentials exist only at runtime, never in TOML, command lines or reports.

Approved disposable-guest and non-rebooting runner work is normal testing. Physical
host restart, shutdown, logout or session termination requires explicit user
permission immediately beforehand. Detailed execution rules live in
[ENGINEERING.md](ENGINEERING.md#windows-elevation-and-uac) and
[AGENTS.md](../AGENTS.md#development-and-test-authorization).

## Guest image and disposable VM

The clean parent is shut down, versioned, access-controlled and never writable by
an experiment. One persistent Generation 2 VM shell retains VM GUID, firmware/vTPM
and guest identity; reset replaces only its differencing child. The fixed-shell
parent is not a portable clone image. Another VM identity requires a separately
prepared generalized parent. The runner accepts no caller-selected VM/disk and
never targets the parent. Uncertain guest state recovers by replacing the child;
project-owned host settings/assignment retain the small preimage needed for removal.

Use normal Windows 11 installation media and integration support. No custom guest
OS platform is required. Local disks, drivers and other large artifacts follow
[`data/README.md`](../data/README.md); resolved paths remain under the configured
data root and privileged enrollment rejects reparse escapes.

## Display and presentation boundary

VMConnect, Enhanced Session Mode or RDP provide operator access. A working desktop
does not establish NVIDIA acceleration; probes select/check the render device
offscreen. Custom display drivers, streaming/remote desktop transport, save/restore,
migration, checkpoints and host sleep/hibernate are outside v1.

## Native Windows boundaries

Use Windows Hyper-V management for configuration, assignment and lifecycle; native
WMI for discovery; supported PowerShell Direct for guest sessions; PnP/DXGI/D3DKMT
for status and identity. Rust owns intent, validation, orchestration, supervision,
integrity checks and reporting. Existing fixed cmdlet adapters remain where they
provide a measured, bounded Windows interface. Rewriting working transports is
not required for release. New non-Rust application logic requires the technical
exception process in [ENGINEERING.md](ENGINEERING.md#rust-and-native-windows).

## Validation contract

Record actual checks: revision, host/guest builds and x64 architecture, GPU/driver,
full manifest digest, effective VM/resource settings, API/runtime/probe inputs,
session, workload output and outcome. The [baseline](evidence/GPU-PV-BASELINE.md)
supplies established evidence, not a request to repeat feasibility. New driver/build
combinations need affected workload validation before being advertised as qualified.

Build differences are qualification warnings rather than automatic failures.
Changed driver identity, invalid signatures or active/unexplained servicing block
mutation. Use `pass`, `fail`, `blocked` or `untested`; missing results do not
establish unsupported capability. Extra APIs, codecs, vendor extensions and interop
need their own workloads only when claimed. No broad optional API matrix blocks v1.

## Test lanes

| Lane | Purpose |
|---|---|
| Hardware-free Windows checks | Meaningful configuration, identity, planning, path/hash, protocol, timeout/failure and CLI behavior tests |
| Disposable-VM integration | Complete provisioning, exact settings, readiness, checked workloads and supported lifecycle/recovery |
| Maintenance and release | Full-manifest restage, bounded clean-child/lifecycle repetitions and final candidate rehearsal |

Run checks proportional to the change and reuse unchanged results. CI does not
imply GPU execution. Fresh clean-child reproduction demonstrates product automation,
not renewed feasibility. Repetition quantities/abort limits belong in the owning
task/configuration; physical-host reboots are not a routine release requirement.
