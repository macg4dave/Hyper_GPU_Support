# GPU-PV Architecture and Reference Map

Read the section relevant to the task. Fixed scope and design rationale live in
[DECISIONS.md](DECISIONS.md); work and acceptance criteria live in
[BACKLOG.md](BACKLOG.md). This document describes responsibilities and
evidence requirements, not implementation progress.

## Current state

The Rust CLI/library foundation provides help, version and a read-only inventory
command. Inventory uses a typed Rust report/selection boundary plus the bounded,
query-only Windows process adapter recorded in [DEC-013](DECISIONS.md#dec-013).
Build and check commands are in [README.md](../README.md). Mutating GPU-PV
components below are not implemented.
The target is a Windows 11 x64 host and guest with an NVIDIA RTX 5060 8 GB.
The user has run AppSandbox successfully on this hardware, establishing a
known-working HCS GPU-PV reference. HV-001 and CORE-001 captured host, adapter,
driver and query-rights inventory, but there is no registered VM and no
D3D11/D3D12/CUDA evidence. Native Hyper-V/VMMS assignment parity is still
untested; those narrower questions do not reopen general GPU-PV feasibility.

The inherited research snapshot is dated **24 September 2026**, from AppSandbox
[`6f3adb6aafd4fc819d7715bdfacf52ac87df26a6`][upstream-commit] (0.1.9 version
bump). The pinned source map records inspection of that revision. Reorganizing
these notes does not constitute a new upstream audit, local inventory, build,
VM run, or graphics/compute/video test. The exact host and management inventory
is in [HV-001 evidence](evidence/HV-001.md); guest preparation remains absent.

Windows provides the graphics kernel/VMBus path between guest user-mode drivers
and the host GPU. Full-VM driver provisioning still requires attention; this
project does not need a virtual GPU implementation or replacement NVIDIA kernel
driver. [Microsoft GPU-PV architecture][ms-gpupv]

Microsoft explicitly excludes client Windows and desktop-class hardware from
its supported GPU-P/DDA configurations. This Windows 11/GeForce combination is
an experimental validation target;
successful workloads establish a measured configuration, not a change to vendor
support policy. [Microsoft support boundaries][ms-support]

## Proposed components

The foundation uses one Rust package with a CLI and reusable library. After HV-001
supplies exact target identities, build a thin vertical slice from ordinary modules
for these responsibilities and validate each privileged step on the disposable VM:

| Responsibility | Boundary |
|---|---|
| Inventory and diagnostics | Read host/guest identity, driver versions, partition availability, device status, and results. |
| GPU configuration | Validate intent, produce a concrete change plan, call native management, and verify the actual assignment. |
| Guest driver/runtime preparation | Produce a selected-driver manifest; transfer and validate necessary files/settings in a disposable child. |
| Capability probes and reporting | Invoke focused workloads and produce readable and machine-readable evidence. |
| Configuration and operation state | Versioned intent, immutable reviewed plan, effective-state comparison, per-VM/physical-GPU locks and bounded audit records. |
| Maintenance and lifecycle | Call native VM start/shutdown/restart; detect driver/build drift; recreate/restage only through a new reviewed operation. |

Configuration targets one designated disposable VM ID and an immutable parent
image identity. Windows owns VM lifecycle, storage, differencing disks and
integration services. Keep GPU logic callable without a GUI; a future
configuration editor may call the same core. Fixed scope and backend selection
criteria are recorded in [DECISIONS.md](DECISIONS.md).

Implement these boundaries in Rust, preferring Rust libraries and native Windows
bindings such as `windows`. Evaluate direct Win32/WMI access before a cmdlet adapter;
use an existing Windows utility when required without recreating Hyper-V. Any
non-Rust glue or shim requires the documented exception process in
[engineering standards](ENGINEERING.md#rust-and-native-windows). A C ABI does not
require C/C++ implementation. Each workaround still needs a reproduced failure,
affected driver/build range, validation and removal condition. Process adapters
use fixed operations, typed parameters and structured data, never arbitrary scripts.

## Foundation source layout

`src/main.rs` owns process I/O and exit codes; `src/cli.rs` owns argument parsing
and usage errors; `src/inventory.rs` owns typed facts, validation, reporting and
the replaceable source contract; `src/windows_inventory.rs` owns the bounded
query-process transport and Rust target/VM selection. `src/lib.rs` exposes the
library boundary and `tests/cli.rs` exercises the built executable. No mutation
backend has been selected.

Add `config` for validation in CORE-004, privileged `windows`/`hyperv` adapters in
CORE-005 and `gpupv` for assignment in CORE-002. Revisit CORE-001's bounded query
transport after HV-003 fixes the native interface/rights matrix.
Diagnostics/logging belong in `diagnostics` when inventory introduces operations
to report. Introduce shared `error`/`types` modules when multiple callers need
them; utilities stay with their owning responsibility until reuse is demonstrated.
These are navigation intentions, not empty files or fixed backend interfaces.

## Configuration and recovery contract

These are proposed implementation boundaries, not existing modules or commands.
CORE-004 chooses the small serialization format and CLI syntax. Configuration
identifies VM GUID, explicit GPU identity, measured resource settings and driver
manifest; it contains no guest password, arbitrary shell script or default-GPU
fallback. Unknown schema versions/fields and ambiguous identity fail validation.
Keep intended, observed and last-validated state separate; PnP paths can change
after driver servicing and must be reconciled against identity, never guessed.

The operation flow is read -> validate -> plan -> authorize -> revalidate -> apply
-> verify. Read-only planning writes no protected state. Apply checks the plan's
environment fingerprint and VM state again under exclusive VM and physical-GPU
locks acquired in a fixed order. Recheck existing/in-flight assignments across
VMs under the GPU lock so simultaneous commands cannot bypass the one-guest limit.
Windows users/tools do not honor our locks; verify native state again at each
mutation and refuse detected external conflicts. Record the requested operation,
validated identities, each native result and final verification. Host mutations
retain the small preimage needed to detach the adapter or undo a project-owned
setting; uncertain guest state is recovered by recreating the disposable child,
not by building a general transaction/rollback engine.

The clean Windows 11 parent VHDX is shut down, versioned, access-controlled and
never attached for experimental writes. This target uses one persistent Generation
2 VM shell with a fixed VM GUID, firmware, vTPM, MAC address and guest identity;
each run replaces only its differencing VHDX. This is a single-machine state reset,
not deployment of the image to another virtual computer. A second VM or concurrent
descendant is outside this exception and requires a generalized parent.
Driver staging in the child records path, hash/version, origin, destination and
registry/ICD settings for repeatability and diagnosis, but guest recovery discards
the child. Reject path traversal, unexpected reparse points, changed source hashes
and writes outside the configured child/guest scope. The runner must never accept
the parent path or another VM as a mutation target.

Ordinary commands use the caller's unelevated Windows token. A separately approved
privileged test runner may execute only fixed, typed operations needed by the GPU-PV
experiment. Its installed executable and policy are administrator-owned outside the
repository; the agent can submit a bounded request and read a result but cannot
replace the executable, edit its policy or supply a command line/script. Policy pins
one logical disposable slot, GPU identity, parent/child roots, allowed operations
and timeouts. For this fixed-identity target, `reset` validates the administrator-
enrolled VM GUID and removes/recreates only its child disk; it never removes or
recreates the VM shell. Caller input supplies neither a VM GUID nor a path. Every
request/result is logged and invalid, ambiguous or stale identity fails closed.
Installing, updating or broadening this boundary needs new approval;
tool sandbox approval is not Windows elevation. Do not run the editor or arbitrary
repository binaries with a general elevated token merely for convenience.

The runner may be hosted by an on-demand Scheduled Task or an equivalent small
native launcher after HV-003 proves the minimum Windows rights. Prefer a dedicated
principal in Hyper-V Administrators when its measured operations succeed; use a
broader administrator token only for an individually justified operation that the
limited principal cannot perform. It is not a background GPU service or product
control plane. [New-VHD differencing disks][ms-new-vhd]
[Task Scheduler security contexts][ms-task-security]
[Hyper-V Administrators][ms-hyperv-admins]

Guest sessions use ephemeral credentials supplied securely at execution time, no
passwords on process command lines or in reports. Native subprocess adapters must
handle timeouts, structured output, stderr, encoding, cancellation and nonzero
exit status. These boundaries are tested separately from GPU capabilities.

The development path prepares a clean parent manually, creates one disposable VM
from a differencing disk, verifies prerequisites and provides native lifecycle
operations. It does not install Windows, create general networking, resize guest
disks or manage arbitrary VMs. Unsupported save/checkpoint/migration/sleep paths are
reported as unvalidated and are never selected as recovery. Hardware qualification,
not a reported quota, determines the recommended resource preset.

## Installation media and local image baseline

Use an official, unmodified Windows 11 x64 ISO. AppSandbox's pinned source exposes
two different operations that its `iso-patch.exe` name can obscure:

- Its legacy one-argument mode copies the source ISO to a new UDF image and replaces
  only `efi/microsoft/boot/efisys.bin` and `cdboot.efi` with Microsoft's existing
  `_noprompt` variants. This suppresses the optical-media “Press any key” prompt for
  automation; it does not add GPU drivers or change `install.wim`. The input ISO is
  mounted read-only and remains unchanged. [Pinned legacy mode][u-iso-patch]
- The current Windows create path calls `--to-vhdx`: it reads `install.wim` or
  `install.esd` from the ISO, creates/partitions a new GPT VHDX, applies the selected
  image with DISM, installs UEFI boot files with `bcdboot`, and stages files into the
  new Windows volume. This modifies the VHDX, not the installation ISO.
  [Pinned create path][u-core-create] [Pinned converter][u-iso-to-vhdx]

AppSandbox stages an answer file, setup scripts, its agent/input/clipboard/audio
helpers, custom VDD/VAD display/audio drivers, optional OpenSSH, and—on the shared
Windows/macOS provisioning path—conditional shared-memory and NetKVM drivers. It
also stages selected host GPU driver files under `HostDriverStore`; NVIDIA profiles,
runtime shims and mapping layers have a separate guest provisioning path. Its
answer/setup flow can create a local administrator and one-time autologon, bypass
network OOBE, change recovery/boot-status policy, disable automatic device
encryption for templates, run Sysprep, and optionally enable test signing/install
test certificates. ARM64-only setup branches bypass TPM/RAM/Secure Boot checks.
[Pinned answer/setup generator][u-win-provision] [Pinned staging manifest][u-disk]

Those changes support unattended product installation, AppSandbox guest services,
its custom display/audio/transport/network paths, templating and broad runtime
compatibility. They are not prerequisites for Windows GPU-PV. This project's x64
Generation 2 baseline keeps Secure Boot and vTPM enabled, performs a normal Windows
installation, and adds only the measured GPU assignment and minimum matching guest
runtime after installation on a disposable child. Driver/runtime work can use
PowerShell Direct or explicitly approved offline servicing; a custom resources ISO
is unnecessary. Modified media is reconsidered only after a reproducible essential
workload failure proves that neither normal post-install configuration nor scoped
child-disk servicing can supply a required pre-boot change.

The local artifact layout is documented in [`data/README.md`](../data/README.md).
The repository-relative `data/` tree is the small-machine default; future versioned
configuration supplies one canonical absolute data root for external storage and
derives all leaves from it. The privileged runner pins the resolved parent, child
and result roots and rejects reparse-point escapes.

The golden workflow is deliberately native and shallow:

1. Create a normal Generation 2 VM from the original ISO with Windows Secure Boot,
   vTPM, at least two virtual processors, 4 GB RAM and a 64 GB-or-larger VHDX; use
   the eventual test VM's hardware profile. [Windows 11 VM requirements][ms-win11-vm]
2. Install the selected edition legitimately, apply normal updates/integration
   support, and add no GPU-PV assignment, copied host driver payload or AppSandbox
   component. Do not embed credentials or product keys in the image.
3. Choose the identity model before sealing. A parent deployed to newly registered
   VMs must use `sysprep /generalize /oobe /shutdown /mode:vm`. This project's
   one-at-a-time target instead retains one fixed VM shell and deliberately skips
   Sysprep so its completed local account and guest identity survive resets.
   [Sysprep VM mode][ms-sysprep]
4. Detach the shut-down build disk without booting it again. Place the parent
   at a stable, versioned path under `images/golden/`, record its edition/build and
   SHA-256, back it up, and protect it with ACLs so the experimental identity and
   runner cannot write it.
5. For each test, use `New-VHD -Differencing -ParentPath <parent>` to create one
   writable child under `images/disposable/` and attach it to the enrolled fixed VM
   shell. Verify `Get-VHD` reports the intended `ParentPath` before start. Never
   rename, move, resize, mount writable, service or boot the parent while any child
   exists. [New-VHD][ms-new-vhd]
6. Perform GPU-PV/runtime experiments only in the child. On damage or uncertain
   state, shut down and recreate only the enrolled child through the authorized
   runner. Do not merge a test child into the golden parent and do not use
   checkpoints as the recovery contract.

The fixed-shell exception preserves the guest SID/MachineGuid, VM GUID, vTPM and
network identity and therefore must never be used to create a second independently
registered clone. A differencing disk still depends on the exact parent path and
identity. Activation remains separate from identity and licensing; the owner chose
not to gate this development image on activation or the remaining offered updates.
One active child is project policy, not a licensing conclusion.
[Microsoft Windows 11 virtualization licensing][ms-win11-license]

## Display and presentation boundary

GPU-PV assignment exposes a render/compute device; desktop presentation is a
separate concern. Use VMConnect, Enhanced Session Mode or RDP for operator access
when available, while essential probes select the intended NVIDIA adapter and check
offscreen output/hardware identity. A responsive remote desktop, the Microsoft
Remote Display Adapter or the host's existing Phaze virtual display driver is not
evidence that the guest workload used the RTX 5060.

AppSandbox's IddCx virtual monitor and transport serve its product display path.
They are not copied into this project. A custom indirect display device becomes a
candidate only if a named essential workload fails specifically because no suitable
display target exists and the same workload succeeds with that component. Display
convenience, frame transport and low-latency presentation remain outside v1.

## Technical gaps and research gates

This is a question/risk index, not a second blocker or task-status store. A concrete
failure goes in the [blocker register](BACKLOG.md#blocker-register). References
were selectively checked on 2026-09-24; no target inventory or workloads were run.

| Gap | Evidence / unanswered question | Owning work and decision consequence |
|---|---|---|
| G1: native interface versus working HCS reference | AppSandbox works on the user's target through HCS; client/GeForce deployment remains outside Microsoft's supported GPU-P/DDA configurations. Installed VMMS/WMI partition identity and mutation behavior remain unknown. | HV-001/003 and CORE-001/005; test the native equivalent, then isolate only a demonstrated VMMS/HCS gap. Product claims remain version-pinned, not vendor certification. |
| G2: reference reproducibility and safety | Pinned source has test-signing, certificate/setup and Secure Boot test-mode branches; solution builds can invoke packaging/signing. Which minimal signed artifact/provisioning route preserves project boundaries? | REF-002 before GPU-003/HV-002. Review only chosen dependencies; blocked safe baseline requires the decision route in DEC-007. |
| G3: VMMS versus HCS vendor behavior | HCS documents AllowVendorExtension and the GPU-PV 0xffff sentinel; secure-VM vendor escape restrictions exist. No established VMMS equivalence for our CUDA workload. | HV-003, GPU-005/006; compare identical runtimes and session before bounded HCS test. Never relax isolation to obtain parity. |
| G4: runtime servicing and provenance | Microsoft documents guest user-mode/host kernel driver pairing and disabled automatic full-VM driver-store copying in released OS. Exact NVIDIA files/ICDs and legal terms depend on the chosen package. | GPU-002/009; prove minimum staging in a disposable child, then CORE-009/015 and REF-003. Hashes and actual execution, not filenames, establish the tested combination; discard the child for recovery. |
| G5: target API feasibility | RTX specifications describe physical capability, not guest CUDA, video, interop or monitoring. Probe SDK/driver compatibility, Blackwell-capable CUDA toolchain and software fallback can confound results. | GPU-008/004/005; pin probe dependencies and host control; essential APIs gate GPU-006, optional APIs receive individual results. |
| G6: guest access and presentation | PowerShell Direct needs a local running configured guest, host Hyper-V rights and guest credentials. VMConnect/RDP display and offscreen workloads may select different adapters. | HV-001, GPU-003/009; validate native transfer and explicit adapter identity. Custom display infrastructure is evidence-triggered, not part of the baseline. |
| G7: resource sharing | Cmdlets expose VRAM bytes and driver-defined compute/encode/decode units. Advertised limits/partition counts do not prove enforcement, safe VRAM budget or a supported number of guests. | HV-003, GPU-010; conservative one-guest envelope. GPU-015 independently measures two-guest contention; no percentage/fairness SLA. |
| G8: lifecycle and compatibility drift | No verified target guarantee for saved states, live checkpoints, host sleep or driver updates. File/registry/ACL changes may survive in a failed child. | GPU-011, CORE-014/015, GPU-012/013; discard/recreate the child and refuse stale plans before optional lifecycle claims. |
| G9: privileges and concurrency | Host management and guest system writes need appropriate rights; a second operator/process can invalidate observed state. | HV-003, CORE-005/007/008/016; privilege matrix, runner-owned disposable-slot enrollment, VM/physical-GPU locks and state recheck. Multi-guest scheduling is not required for v1.0. |
| G10: distribution | AppSandbox MIT ownership does not cover all bundled components; Windows NVIDIA driver redistribution is not assumed. Project license/distribution/signing choice is the owner's. | REF-003 and DOC-004; see DEC-008 options. Exclude drivers, OS images, credentials and keys from artifacts. |

Primary-source basis: [GPU-PV/WDDM][ms-gpupv], [support boundary][ms-support],
[HCS schema][ms-hcs-schema], [assignment][ms-add-gpu], [resource fields][ms-set-gpu],
[partition inventory][ms-wmi-gpu], [PowerShell Direct][ms-psdirect],
[NVIDIA driver terms][nv-license] and the pinned files below. These sources define
interfaces and constraints, not passing results on this machine.

## Reference implementation hazards

The selective re-review for DOC-002 found additional reasons to avoid importing
the full reference provisioning path:

- [Build/signing instructions][u-signing] describe test-signed development drivers
  and Release packaging/signing hooks. [HCS][u-hcs] test mode and [disk setup][u-disk]
  include security-affecting branches. REF-002 must inspect the exact artifact path;
  a full solution build or setup script is not an approved baseline procedure.
- [Guest agent][u-guest-agent] provisioning/copy notifications can mask graphics
  provisioning errors. Code-43 device cycling is conditional, and DRS setup broadens
  permissions. Our workflow needs explicit substep results and scoped recovery.
- [Copy logic][u-p9] can skip equal-sized files without a content check.
  [Runtime provisioning][u-provision] changes runtime files, ICD paths, ownership
  and junctions; missing optional components can still return success. Verify hashes
  and full preimages; do not copy its broad permissions or success semantics.
- [NVAPI project][u-nvapi-project] includes compute identity hooks; the component
  name does not limit it to DLSS. [Adapter hooks][u-hooks] and [compute hooks][u-compute-hooks]
  patch dispatch/import behavior. Treat these as fragile compatibility mechanisms,
  with exact source/build dependency and version-range evidence, not supported
  Windows GPU management interfaces.

No code has been adopted. Windows cmdlets, PowerShell Direct and existing servicing
replace custom VM plumbing, Plan9 transfer and disk-writing machinery where the
experiments validate that replacement. Custom display transport is outside v1.0.

## Upstream reference map

The relevant responsibilities form this path:

```text
Host partitionable GPU discovery -> Windows GPU assignment
Host NVIDIA driver files -> guest HostDriverStore/runtime provisioning
Guest application -> Windows/vendor runtimes -> Windows GPU-PV -> host GPU

Guest desktop -> display/remoting transport -> viewer (separate concern)
```

AppSandbox creates/starts Windows VMs through HCS, rather than managing ordinary
persistent Hyper-V Manager/VMMS VMs. Both use Windows virtualization; selecting
a different management route does not mean rebuilding GPU-PV.

| Pinned source / responsibility | Observed dependency and intended treatment |
|---|---|
| [`gpu_enum.c`][u-enum], `gpu_enumerate` | SetupAPI/Configuration Manager enumerate partition-adapter interfaces, device identity, driver service and INF location, and map `DriverStore` to `HostDriverStore`; also prepare NVIDIA runtime/profile shares. Reimplement discovery and a manifest for the explicitly selected RTX 5060, using native management queries plus SetupAPI where needed. |
| [`hcs_vm.c`][u-hcs], `hcs_apply_gpu` | After creation/start, `HcsModifyComputeSystem` updates `VirtualMachine/ComputeTopology/Gpu`: `List` maps an adapter-interface path to `65535`, or uses `Default`, with `AllowVendorExtension=true`. Use as the assignment reference; first compare native Hyper-V cmdlets. |
| [`disk_util.c`][u-disk], `generate_vhdx_manifest` | Pre-stages driver files in the guest disk; NVIDIA shims, DRS profiles and mapping layers follow a separate provisioning path. Retain path/file-selection knowledge; use Windows transfer or disk servicing rather than the custom filesystem engine. |
| [`hcs_vm.c` Plan9 shares][u-plan9], [`vm_agent.c`][u-host-agent], [`agent.c`][u-guest-agent], [`p9copy.c`][u-p9] | Host driver shares and Hyper-V socket metadata feed guest copying/provisioning; the agent also attempts a GPU device restart for code 43. Replace the custom protocol with PowerShell Direct/file transfer where feasible; device recovery must be explicit. |
| [`gl_vk_provision.c`][u-provision] | Validates/deploys NVIDIA runtimes/shims, rewrites Vulkan ICD manifests, replaces applicable OpenGL/OpenCL/CUDA entry points, prepares NVAPI/NGX paths, and stages Optical Flow/OptiX DLLs when found. Test matching unmodified runtimes first; adapt only demonstrated needs with an exact change/restore manifest. |
| [`adapter_identity.c`][u-identity], [`adapter_hooks.c`][u-hooks] | D3DKMT/DXGI queries and hooks reconcile host/guest adapter identity. Conditional compatibility work, separate from management; reimplement a small attributed component in Rust only if needed, preserving the required ABI. Non-Rust source needs a documented technical exception. |
| [`opengl_shim.c`][u-gl], [`vulkan_shim.c`][u-vk] | Forward to real runtimes and adapt discovery/adapter identity. Candidates for measured OpenGL/Vulkan failures, not a mandatory general layer. |
| [`cuda_shim.c`][u-cuda], [`opencl_shim.c`][u-cl], [`cuda_opencl_adapter_hooks.c`][u-compute-hooks] | Forward vendor calls and correct adapter LUID mismatches affecting compute discovery/interop. Evaluate with execution and sharing tests, not enumeration alone. |
| [`nvapi_shim.c`][u-nvapi] | Wraps NVAPI identity queries for DLSS/NGX **and hosts hooks used by CUDA/OpenCL identity workarounds**. An adopted compute fix may require this wrapper even while DLSS work is deferred; DLL presence proves no vendor capability. |
| [`d3dlayers.c`][u-layers] | Acquires Microsoft OpenGL/OpenCL/Vulkan-on-D3D mapping layers. Defer unless a workload needs them; report translated and native NVIDIA execution separately. |
| [`vdd.cpp`][u-vdd], [`vm_display_idd.c`][u-display] | IddCx/UMDF virtual monitor and desktop transport/viewer. The VDD is display-only and does not pin the render adapter. Defer custom display drivers/transport; assess VMConnect/RDP separately from rendering and compute. |
| [`asb_core.c`][u-core] and product callers | Coordinates lifecycle, shares and provisioning; Python headless clients invoke this core. Use as call-flow reference only; UI, networking, SSH, clipboard, audio, snapshots, installers and a Python/HTTP control plane are outside the GPU core. |

`65535` is `0xffff`, the HCS GPU-PV request for an available partition. It is not
an 8-GB allocation, percentage, or unrestricted access. `AllowVendorExtension`
is documented, but its capability effects and any VMMS equivalent require
comparison on the target build. [HCS GPU schema][ms-hcs-schema]

`hcs_start_vm` can report successful VM start after GPU assignment fails, and
an unavailable selection can fall back to the default GPU. Our configuration
operation should report assignment failure and actual adapter identity.
Attachment, driver readiness, runtime loading, and workload success must be
distinct results.

No upstream source component is proven necessary on this target. Discovery,
assignment, matching driver staging, and validation are required responsibilities;
specific shim and presentation dependencies remain experiments. Adaptations must
retain provenance and applicable notices as described in
[DECISIONS.md](DECISIONS.md).

## Native Windows boundaries

The proposed default is one disposable Generation 2 Windows 11 VM managed by
Hyper-V/VMMS and backed by a differencing VHDX whose clean parent is never used
for experiments. Prepare and seal the parent manually with normal Windows tools;
a general VM installer is not an early component.

| Need | Native facility to evaluate |
|---|---|
| Host GPU discovery | [`Get-VMHostPartitionableGpu`][ms-get-gpu]; SetupAPI for device/driver detail. |
| Assignment/inspection | [`Add-VMGpuPartitionAdapter -InstancePath`][ms-add-gpu], `Get-VMGpuPartitionAdapter`, `Remove-VMGpuPartitionAdapter`. |
| Resource requests | [`Set-VMGpuPartitionAdapter`][ms-set-gpu] VRAM, encode, decode, and compute fields; inspect advertised and effective values. |
| Direct native access from Rust | WMI/CIM `root\virtualization\v2`, including [`Msvm_PartitionableGpu`][ms-wmi-gpu] and [`Msvm_GpuPartitionSettingData`][ms-wmi-settings]. |
| Guest execution/transfer | [PowerShell Direct][ms-psdirect], `Invoke-Command -VMId`, persistent sessions and `Copy-Item -ToSession/-FromSession`; requires guest credentials/integration support. |
| Guest storage/preparation | Existing Windows/Hyper-V tools; approved offline VHDX servicing only when necessary. |
| Diagnostics/presentation | PnP status, DXGI/D3DKMT, event logs and guest API probes; VMConnect/RDP for access. |

These are facilities to discover, not commands already validated on this host or
a requirement to implement application logic in PowerShell.
Confirm cmdlets and parameters on the installed client build. Preserve reported
resource units/limits; arbitrary values do not establish percentages or access
to all physical VRAM. Begin with one guest and measure host headroom on 8 GB.

The normal Hyper-V role needs an eligible Windows edition; Windows 11 Home
does not provide it. Discover the edition before backend selection.
[Hyper-V installation requirements][ms-hyperv-install]

If VMMS cannot reproduce a reference capability, isolate assignment,
vendor-extension behavior, runtime provisioning, and display-session differences
before a minimal HCS comparison. HCS requires more caller-owned provisioning and
lifecycle work. [HCS management model][ms-hcs-overview]

## Validation contract

All capabilities below remain **untested on the target**. Record results per API
and workload using `pass`, `fail`, `blocked`, `untested`, or `unsupported with
evidence`; missing tests are not unsupported features.

| Capability | Minimum useful evidence |
|---|---|
| Partition/device readiness | Intended RTX 5060 assigned, guest virtual render device healthy, matching runtime/driver provenance. |
| Direct3D | D3D11/D3D12 rendering, correct output and hardware identity; D3D9/10 when comparing upstream's graphics set. |
| OpenGL/Vulkan | Version/extensions, selected ICD and real rendering; distinguish NVIDIA, D3D translation, and software paths. |
| CUDA | Device identity, allocation, transfer, kernel execution and checked output; representative compute workload. |
| OpenCL/DirectCompute/DirectML | Separate execution and correctness result for each claimed API. |
| Video encode | NVENC or named hardware backend; H.264, HEVC and AV1 separately, with codec/profile, throughput and checked output. |
| Video decode | Named NVDEC/CUVID or D3D video path; actual streams and checked output per codec. |
| Cross-API interop | Graphics/compute shared resources and zero-copy video decode, independently of standalone success. |
| Other vendor capabilities | Optical Flow/OptiX where exposed; separate named probes for ray tracing, Tensor workloads, NVAPI/NGX/DLSS and monitoring. |
| Stability/performance | Repeat starts, sustained load, host responsiveness, VRAM pressure, device-loss/error reporting and recovery. |

Each result must retain date, project/upstream revision, host edition/build/x64,
GPU PCI identity and model, exact host driver, guest edition/build/x64, staged
file versions/hashes, backend/effective configuration, API/runtime/probe versions,
exact steps/command, workload inputs, output/result and relevant logs. Graphics
results also identify session type and renderer/adapter; WARP or other software
rendering is not evidence of NVIDIA acceleration.

Physical [RTX 5060 specifications][nv-5060], including AV1 encode/decode, guide
probe selection, not guest promises. Runtime loading is separate from execution;
a CUDA kernel does not prove NVENC, OptiX, interop, or full physical-device access.
Physical display outputs, full device management, fixed quotas and all vendor
extensions must not be assumed available through GPU-PV.

## Test lanes

The [engineering testing policy](ENGINEERING.md#testing) owns code coverage,
determinism and isolation; this table maps project evidence to tasks.

| Lane | Verifies | Evidence owner |
|---|---|---|
| Hardware-free Windows CI | Configuration/identity validation, planner diffs, structured adapter failures, audit/locking and disposable-recreation decisions, path/secret handling and report contracts using fixtures/fakes. Never claims GPU execution. | CORE-019 establishes the baseline; CORE-001/013 extend it; each implementation card adds relevant cases. |
| Native management integration | Installed interfaces, rights, explicit VM/GPU selection, effective settings, guest transfer and legal lifecycle states on an authorized dedicated VM. | HV-003, GPU-009/011, CORE-005/008/002/010/011. |
| Physical target workloads | D3D11/D3D12 checked frames and CUDA checked kernels; per-API optional results; identical host control and guest inputs, explicit hardware renderer and session. | GPU-008 defines probes; GPU-004/005/006 and CORE-003 execute them. |
| Failure and maintenance | Interrupted/denied/full-disk/stale-plan operations, identity conflicts, driver/build drift, device-not-ready diagnostics, disposable recreation and a controlled driver transition. | CORE-014/015 and GPU-013. |
| Endurance and release | Repeated starts, authorized host reboots, sustained/pressure load and fresh-guest reproduction from candidate artifacts/instructions. | GPU-012/014 and DOC-005. |

Keep benchmark inputs/tolerances and timeout/abort limits in the probe/resource
evidence before execution. A timeout or missing runtime is not a skipped pass.
Every essential release probe needs correctness and NVIDIA hardware identity;
performance numbers alone are insufficient. Driver/build changes invalidate the
affected compatibility record until revalidated. Hardware runs require explicit
authorization for their protected setup/lifecycle changes; CI must not mutate a
developer's machine merely because tests were invoked.

Initial lifecycle support is graceful shutdown/start/restart, adapter detach and
verified disposable-child recreation. Host sleep/hibernate, live save/restore,
checkpoints and migration remain unvalidated/outside the release guarantee. Report
these states and instruct recreation from the unchanged parent.

## Source references

[upstream-commit]: https://github.com/jamesstringer90/appsandbox/commit/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6
[u-enum]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/gpu_enum.c#L285
[u-hcs]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/hcs_vm.c#L1393
[u-disk]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/disk_util.c#L2105
[u-plan9]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/hcs_vm.c#L1060
[u-host-agent]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/vm_agent.c#L273
[u-guest-agent]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/agent.c#L567
[u-p9]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/p9copy.c
[u-provision]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/gl_vk_provision.c
[u-identity]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/adapter_identity.c
[u-hooks]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/adapter_hooks.c
[u-gl]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/opengl_shim.c
[u-vk]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/vulkan_shim.c
[u-cuda]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/cuda_shim.c
[u-cl]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/opencl_shim.c
[u-compute-hooks]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/cuda_opencl_adapter_hooks.c
[u-nvapi]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/nvapi_shim.c
[u-layers]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/d3dlayers.c
[u-vdd]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/vdd/vdd.cpp
[u-display]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/vm_display_idd.c
[u-core]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/asb_core.c#L3320
[u-core-create]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/asb_core.c#L1281
[u-iso-patch]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/iso-patch/iso-patch.c#L2
[u-iso-to-vhdx]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/iso-patch/iso-patch.c#L708
[u-win-provision]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/provision/win_provision.c#L132
[ms-gpupv]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/gpu-paravirtualization
[ms-support]: https://learn.microsoft.com/en-us/troubleshoot/windows-server/virtualization/troubleshoot-hyper-v-gpu-assignment-partitioning-passthrough-issues
[ms-hcs-schema]: https://learn.microsoft.com/en-us/virtualization/api/hcs/schemareference#gpuconfiguration
[ms-hcs-overview]: https://learn.microsoft.com/en-us/virtualization/api/hcs/overview
[ms-get-gpu]: https://learn.microsoft.com/en-us/powershell/module/hyper-v/get-vmhostpartitionablegpu?view=windowsserver2025-ps
[ms-add-gpu]: https://learn.microsoft.com/en-us/powershell/module/hyper-v/add-vmgpupartitionadapter?view=windowsserver2025-ps
[ms-set-gpu]: https://learn.microsoft.com/en-us/powershell/module/hyper-v/set-vmgpupartitionadapter?view=windowsserver2025-ps
[ms-wmi-gpu]: https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-partitionablegpu
[ms-wmi-settings]: https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-gpupartitionsettingdata
[ms-psdirect]: https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/powershell-direct
[ms-hyperv-install]: https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/get-started/install-hyper-v
[ms-new-vhd]: https://learn.microsoft.com/en-us/powershell/module/hyper-v/new-vhd
[ms-win11-vm]: https://learn.microsoft.com/en-us/windows/whats-new/windows-11-requirements#virtual-machine-support
[ms-sysprep]: https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/sysprep-command-line-options?view=windows-11
[ms-win11-license]: https://www.microsoft.com/licensing/guidance/Windows-11-Licensing-for-Virtual-Desktops
[ms-task-security]: https://learn.microsoft.com/en-us/windows/win32/taskschd/security-contexts-for-running-tasks
[ms-hyperv-admins]: https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/plan/security-best-practices/appendix-b--privileged-accounts-and-groups-in-active-directory#hyper-v-administrators
[nv-5060]: https://www.nvidia.com/en-us/geforce/graphics-cards/50-series/rtx-5060-family/
[nv-license]: https://www.nvidia.com/en-us/drivers/nvidia-license/
[u-signing]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/sign/SIGNING.md
[u-nvapi-project]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/AppSandbox-NVIDIA-DLSS-shim.vcxproj
