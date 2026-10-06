# GPU-PV architecture

This document specifies the current recipe and implementation boundaries.
[BACKLOG.md](BACKLOG.md) owns work/status; [DECISIONS.md](DECISIONS.md) owns
rationale. Read only the section needed for the current task.

## Current state

Normal Generation 2 Hyper-V / VMMS GPU-PV works on the Windows 11 x64 host and
guest with an NVIDIA RTX 5060 8 GB. The [measured baseline](evidence/GPU-PV-BASELINE.md)
passed sustained Code 0, `nvidia-smi`, checked D3D11/D3D12 hardware rendering and
CUDA allocation, transfer and kernel computation. Native Rust discovery matches
the discovered driver/runtime manifest for that measured driver. The product
contract is complete host-matched discovery, placement and verification, with
manifest size reported as data rather than used as a fixed acceptance condition.

The complete bounded Rust guest writer passed live full-manifest apply and
verified reapply on a clean child. The fixed Rust runner applies the validated
VM/resource settings with independent process readback. Composition with
readiness/workload checks is implemented through the fixed Rust validation worker;
combined clean-child qualification remains GPU-006. Product automation is incomplete.

The v1 product is a Rust CLI and TOML configuration for one configured Windows 11
VM and one RTX 5060. Windows owns virtualization, disk management and GPU-PV.
There is no GUI, scheduler, multi-GPU control plane or HCS VM platform. The
privileged runner is an on-demand operation boundary, not a background GPU service.
Complete provisioning stays intact through v1; minimum-file experiments and
CUDA/D3D interoperability are later work.

Historical: the initial provisioning recipe was developed during reference
research and subsequently validated independently on the target system. External
project research is not an implementation prerequisite.

## Product versus development tooling

The v1 operator supplies an existing Hyper-V VM and configuration, then inspects,
plans, applies GPU-PV/driver provisioning and verifies it. Golden images and
disposable rebuilds are our development strategy, not product APIs. Test tooling
may consume product contracts; default product builds do not compile or invoke
`tools/test-harness/` disposable-reset effects. Explicit `dev-harness` builds
retain the enrolled laboratory workflow and must not be shipped.

Current configuration, receipts and runner guards still assume the enrolled child
and protected parent. This is remaining coupling, not a v1 requirement: CORE-021
and CORE-027 must separate product VM selection/enrollment from laboratory pins;
CORE-010 must provide recovery without destroying a user disk. Preserve current
safety guards while implementing those replacements incrementally.

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
`provider-default` is not equivalent to this profile. The fixed `configure-slot`
operation requires an exact existing attachment, checks fresh provider resource
limits and existing Microsoft Windows Secure Boot/vTPM, and applies the typed
`[vm_profile]` and explicit `[resources]` values pinned in its installed policy.
It stores a durable settings preimage, rejects stale effective observations before
effects, and independently reads fresh provider objects in a new process before reporting applied
or already-applied. Failed or uncertain operations retain the reconciliation marker
and preimage. The setter process is not an effective-state oracle. Enabled automatic
checkpoints can be disabled by this operation; existing snapshots remain refused.
It never changes the host partition count or security devices.

### Driver/runtime manifest and guest placement

The Code 43 investigation exposed incomplete payload discovery: copying the main
DriverStore package and a few manually chosen runtimes omitted associated files
outside that package. The working full-payload approach enumerated the installed
GPU driver's associations, expanded its packages and placed external files at
their corresponding guest Windows locations. Package-only staging cannot safely
assume it has the complete runtime. Files and VM settings changed together in the
successful baseline; that run does not prove every individual file is necessary.

The provisioning contract is:

```text
selected GPU -> installed signed driver -> package trees + associated files
  -> classify sources -> calculate guest destinations -> discovered manifest
  -> copy -> verify every destination
```

GPU, driver, Windows or packaging revisions may change paths, lengths, hashes and
the number of destinations. Discover all supported associations rather than use
a static NVIDIA file list, expected baseline count or per-file hash table in code.

`src/driver_environment.rs` expands the kernel-service package and every associated
DriverStore package into complete trees, and includes every associated external
file. Sources must be regular files within the discovered Windows directory, with
safe ancestors and no reparse escape. Conflicting destinations, malformed paths
and changed pinned package inputs fail before guest writes.

The deterministic manifest records selected device/driver/INF/service, association
count, package roots and each source, logical destination, byte length and SHA-256.
Files are sorted case-insensitively by destination; the encoded contract has its
own digest. No hand-maintained NVIDIA DLL list determines the closure. Count and
aggregate bytes describe that discovered manifest; completeness follows from
successful enumeration, package expansion and verification, not numeric parity
with an older environment. Reject incomplete enumeration, missing sources and
conflicting destinations even when their count happens to match an older run.

| Discovered host path relative to Windows | Guest path relative to Windows |
|---|---|
| `System32\DriverStore\FileRepository\<package>\...` | `System32\HostDriverStore\FileRepository\<package>\...` |
| Every associated external Windows path | Same Windows-relative path |

External associations include System32/SysWOW64 loader/runtime files and other
installed driver paths. The initial RTX 5060 / NVIDIA 616.92 working baseline
produced 271 guest copy destinations; this is environment-specific historical
data, not an implementation requirement. Exact records remain in
[baseline evidence](evidence/GPU-PV-BASELINE.md).
Use ordinary byte-preserving copies. The guest binds the inbox virtual-render
driver (`vrd.inf` / VirtualRender); the working recipe did not install a conventional
NVIDIA INF or import NVIDIA registry state.

After a host-driver change, rediscover the selected signed driver, package roots
and complete associated payload; construct a new manifest and compare its identity
and entries with the guest's verified receipt. Do not reuse old paths, hashes,
versions or counts as the new payload. CORE-015 owns regeneration, comparison,
explicit restaging and essential workload requalification. Current package pins
still block unreviewed drift; their regeneration is unfinished, not an instruction
to make an updated driver match the historical payload. See
[configuration limitations](CONFIGURATION.md#implemented-schema-and-remaining-integration).

### Staging and guest startup

The baseline copied and verified the full manifest offline before any GPU-attached
boot. The Rust writer must establish the same complete bytes and destinations
before an attached start. The current writer uses PowerShell Direct and substantive
embedded guest file logic. CORE-026 moves that logic into a Rust guest writer and
investigates the smallest supported session/transfer bridge. Retaining that bridge
for v1 requires a specific interface limitation under DEC-027; current qualification
does not establish such an exception. A generalized storage/backend abstraction
is not required merely to imitate the experimental transport.

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
| Fixed guest readiness/workload orchestration | `src/validation.rs`, `src/windows_validation.rs`, `src/bin/hyper-gpu-validation-worker.rs` |

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

Public `validate` transfers only the fixed worker, graphics/CUDA workloads and
measured CRT prerequisites. Host and guest independently verify file lengths and
hashes; protected guest files and Microsoft-signed CRT inputs are checked before
execution. The native Rust worker observes a configured sustained Code 0 window,
then runs bounded nvidia-smi, D3D11, D3D12, CUDA identity and CUDA computation.
Each check records pass/fail/blocked/untested with bounded raw output and exact
identities. The host revalidates successful evidence before publishing it.
Graphics must select the configured hardware partition; standalone CUDA requires
the intended sole device but does not require CUDA/DXGI LUID equality.

The elevated development adapter uses the existing operation lock and exact
configured target checks. PowerShell Direct is fixed transfer/launch glue, not an
arbitrary command interface. A worker-owned hard deadline survives remoting loss;
native workload children enter a kill-on-close job before they execute. Credentials
are prompted locally and are absent from command lines and reports. This path is
implemented and behavior-tested; GPU-006 owns combined live qualification.

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

CORE-025's first native read slice runs the same pinned runner executable in fixed
`read-hyperv` mode, with no caller-selected arguments. Native WMI reads exact VM
identity/state/generation/version and host GPU capability triples; a suspended
launch into a kill-on-close job and an independent worker deadline contain COM.
The parent validates configuration binding and identities, then compares the VM
state with the retained disk/snapshot/guest-adapter inspection. Host GPU cmdlet
discovery is removed from runner `inspect`; remaining reads and mutations are
still migration debt. Read parity alone does not qualify GPU workloads.

Approved disposable-guest and non-rebooting runner work is normal testing. Physical
host restart, shutdown, logout or session termination requires explicit user
permission immediately beforehand. Detailed execution rules live in
[ENGINEERING.md](ENGINEERING.md#windows-elevation-and-uac) and
[AGENTS.md](../AGENTS.md#development-and-test-authorization).

## Development strategy: guest image and disposable VM

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

Use native Windows/Hyper-V interfaces for configuration, assignment and lifecycle;
native WMI and Windows identity facilities for discovery; PnP/DXGI/D3DKMT for
status and identity. Rust owns intent, validation, orchestration, supervision,
guest writer operations, integrity checks and reporting. Inventory uses native
registry/system APIs and the existing read-only WMI bindings in a fixed Rust
worker. The sibling worker accepts no query/path/target arguments and has its own
hard deadline; its parent launches suspended, assigns a kill-on-close job, then
resumes and bounds both streams, termination and reaping. The parent validates the
worker's compiled configuration digest, required fact keys and known target identities
before publishing its report; a stale sibling worker fails with rebuild guidance.
Detail queries select exact full identities before decoding provider properties,
so unrelated null driver/VM settings fields cannot poison target facts. Registered
VM count enumerates only identity keys separately. Selection uses the
configured full GPU interface/PnP identity and VM GUID/name, including among
unrelated VMs. Enabled effective Administrators or Hyper-V Administrators membership
is required before management queries because UAC-filtered WMI can silently return
an empty VM list. Missing, denied and unavailable facts remain distinct; module
discovery is unnecessary. VM generation/version come from realized
[Msvm_VirtualSystemSettingData](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-virtualsystemsettingdata)
and state from
[Msvm_ComputerSystem](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-computersystem).
Runner cmdlet adapters remain CORE-025 debt; guest backend logic is CORE-026 and
scripted runner setup/recovery is CORE-027. Preserve working implementations until
replacements are demonstrated. Any retained external session/utility call must
establish the narrow technical exception in DEC-027 and
[ENGINEERING.md](ENGINEERING.md#rust-and-native-windows), including investigated
native alternatives, exact invocation, validated results and bounded errors.

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
