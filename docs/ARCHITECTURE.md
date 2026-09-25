# GPU-PV Architecture and Reference Map

Read the section relevant to the task. Fixed scope and design rationale live in
[DECISIONS.md](DECISIONS.md); work and acceptance criteria live in
[BACKLOG.md](BACKLOG.md). This document describes responsibilities and
evidence requirements, not implementation progress.

## Current state

The hardware-independent Rust CLI/library foundation provides help and version
output. Build and check commands are in [README.md](../README.md). The proposed
GPU-PV components below are not implemented.
The target is a Windows 11 x64 host and guest with an NVIDIA RTX 5060 8 GB.
That configuration remains **untested**, not a support claim.

The inherited research snapshot is dated **24 September 2026**, from AppSandbox
[`6f3adb6aafd4fc819d7715bdfacf52ac87df26a6`][upstream-commit] (0.1.9 version
bump). The pinned source map records inspection of that revision. Reorganizing
these notes does not constitute a new upstream audit, local inventory, build,
VM run, or graphics/compute/video test. Exact Windows editions/builds, NVIDIA
driver, available management facilities, and guest preparation remain unknown.

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

The foundation uses one Rust package with a CLI and reusable library. After a
reproducible hardware baseline exists, add ordinary modules for these responsibilities:

| Responsibility | Boundary |
|---|---|
| Inventory and diagnostics | Read host/guest identity, driver versions, partition availability, device status, and results. |
| GPU configuration | Validate intent, produce a concrete change plan, call native management, and verify the actual assignment. |
| Guest driver/runtime preparation | Produce a selected-driver manifest; transfer, validate, and restore necessary files/settings. |
| Capability probes and reporting | Invoke focused workloads and produce readable and machine-readable evidence. |
| Configuration and operation state | Versioned intent, immutable reviewed plan, effective-state comparison, per-VM/physical-GPU locks and durable recovery journal. |
| Maintenance and lifecycle | Call native VM start/shutdown/restart; detect driver/build drift; restage only through a new reviewed transaction. |

Configuration targets an existing VM ID. Windows owns VM lifecycle, storage,
and integration services. Keep GPU logic callable without a GUI; a future
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
and usage errors; `src/lib.rs` exposes the hardware-independent library boundary.
`tests/cli.rs` exercises the built executable. No backend has been selected.

Add `config` for validation in CORE-004, `windows`/`hyperv` for native adapters in
CORE-005, `gpu` for inventory in CORE-001 and `gpupv` for assignment in CORE-010.
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
mutation and refuse detected external conflicts. Each native action
records its preimage and outcome before the next action. Windows operations are
not one atomic transaction: recovery uses verified compensating actions, and may
stop with an exact manual recovery instruction when state has changed externally.

Driver staging records path, size/hash/version, origin, destination and original
absence/content plus registry type/value, ACL/owner, ICD JSON and junction state
where applicable. Reject path traversal, unexpected reparse points, changed source
hashes and overwriting unowned changes. Backups/journals have restricted access;
interruption, full disk and failed restoration cannot delete the only recovery
copy. Removal touches only project-owned changes and never removes the VM/disk.

Use the caller's Windows authorization for fixed native operations; do not install
a privileged service or silently elevate. Missing rights produce actionable errors.
Guest sessions use ephemeral credentials supplied securely at execution time, no
passwords on process command lines or in reports. Native subprocess adapters must
handle timeouts, structured output, stderr, encoding, cancellation and nonzero
exit status. These boundaries are tested separately from GPU capabilities.

The first release configures an existing VM, verifies prerequisites and provides
native lifecycle operations. It does not create networking, resize guest disks or
install an OS automatically. Unsupported save/checkpoint/migration/sleep paths are
reported as unvalidated and are never selected as automatic recovery. Hardware
qualification, not a reported quota, determines the recommended resource preset.

## Technical gaps and research gates

This is a question/risk index, not a second blocker or task-status store. A concrete
failure goes in the [blocker register](BACKLOG.md#blocker-register). References
were selectively checked on 2026-09-24; no target inventory or workloads were run.

| Gap | Evidence / unanswered question | Owning work and decision consequence |
|---|---|---|
| G1: supported interface versus supported system | Hyper-V cmdlets/WMI and HCS GPU fields are documented; client/GeForce deployment is outside Microsoft's supported GPU-P/DDA configurations. Installed edition, module shape and partition interface remain unknown. | HV-001/003; stop for missing prerequisites. Product promise remains version-pinned experimental compatibility, not vendor certification. |
| G2: reference reproducibility and safety | Pinned source has test-signing, certificate/setup and Secure Boot test-mode branches; solution builds can invoke packaging/signing. Which minimal signed artifact/provisioning route preserves project boundaries? | REF-002 before GPU-003/HV-002. Review only chosen dependencies; blocked safe baseline requires the decision route in DEC-007. |
| G3: VMMS versus HCS vendor behavior | HCS documents AllowVendorExtension and the GPU-PV 0xffff sentinel; secure-VM vendor escape restrictions exist. No established VMMS equivalence for our CUDA workload. | HV-003, GPU-005/006; compare identical runtimes and session before bounded HCS test. Never relax isolation to obtain parity. |
| G4: runtime servicing and provenance | Microsoft documents guest user-mode/host kernel driver pairing and disabled automatic full-VM driver-store copying in released OS. Exact NVIDIA files/ICDs and legal terms depend on the chosen package. | GPU-002/009; prove minimum native staging/restoration, then CORE-009/015 and REF-003. Hashes and actual execution, not filenames, establish the tested combination. |
| G5: target API feasibility | RTX specifications describe physical capability, not guest CUDA, video, interop or monitoring. Probe SDK/driver compatibility, Blackwell-capable CUDA toolchain and software fallback can confound results. | GPU-008/004/005; pin probe dependencies and host control; essential APIs gate GPU-006, optional APIs receive individual results. |
| G6: guest access and presentation | PowerShell Direct needs a local running configured guest, host Hyper-V rights and guest credentials. Offscreen rendering and interactive sessions may select different adapters. | HV-001, GPU-003/009; validate native transfer and named session. Offline servicing is a separately approved fallback, not a required second implementation. |
| G7: resource sharing | Cmdlets expose VRAM bytes and driver-defined compute/encode/decode units. Advertised limits/partition counts do not prove enforcement, safe VRAM budget or a supported number of guests. | HV-003, GPU-010; conservative one-guest envelope. GPU-015 independently measures two-guest contention; no percentage/fairness SLA. |
| G8: lifecycle and compatibility drift | No verified target guarantee for saved states, live checkpoints, host sleep or driver updates. File/registry/ACL changes may survive a failed setup. | GPU-011, CORE-014/015, GPU-012/013; cold recovery and stale-plan refusal before optional lifecycle claims. |
| G9: privileges and concurrency | Host management and guest system writes need appropriate rights; a second operator/process can invalidate a preimage. | HV-003, CORE-005/007/008/016; privilege matrix, VM/physical-GPU locks and state recheck. Multi-guest scheduling is not required for v1.0. |
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

The proposed default is an existing, dedicated, persistent Generation 2 Windows
11 VM managed by Hyper-V/VMMS. A clean guest may initially be prepared manually
with normal Windows tools; a general VM installer is not an early component.

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
| Hardware-free Windows CI | Configuration/identity validation, planner diffs, structured adapter failures, journal/recovery state machine, locking, path/secret handling and report contracts using fixtures/fakes. Never claims GPU execution. | CORE-019 establishes the baseline; CORE-001/013 extend it; each implementation card adds relevant cases. |
| Native management integration | Installed interfaces, rights, explicit VM/GPU selection, effective settings, guest transfer and legal lifecycle states on an authorized dedicated VM. | HV-003, GPU-009/011, CORE-005/008/002/010/011. |
| Physical target workloads | D3D11/D3D12 checked frames and CUDA checked kernels; per-API optional results; identical host control and guest inputs, explicit hardware renderer and session. | GPU-008 defines probes; GPU-004/005/006 and CORE-003 execute them. |
| Failure and maintenance | Interrupted/denied/full-disk/stale-plan operations, preimage conflicts, driver/build drift, device-not-ready diagnostics, cold restoration and a controlled driver transition. | CORE-014/015 and GPU-013. |
| Endurance and release | Repeated starts, authorized host reboots, sustained/pressure load and fresh-guest reproduction from candidate artifacts/instructions. | GPU-012/014 and DOC-005. |

Keep benchmark inputs/tolerances and timeout/abort limits in the probe/resource
evidence before execution. A timeout or missing runtime is not a skipped pass.
Every essential release probe needs correctness and NVIDIA hardware identity;
performance numbers alone are insufficient. Driver/build changes invalidate the
affected compatibility record until revalidated. Hardware runs require explicit
authorization for their protected setup/lifecycle changes; CI must not mutate a
developer's machine merely because tests were invoked.

Initial lifecycle support is graceful shutdown/start/restart and verified cold
recovery. Host sleep/hibernate, live save/restore, checkpoints and migration remain
unvalidated/outside the release guarantee. Report these states and instruct a
measured cold recovery, rather than silently discarding saved state.

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
[nv-5060]: https://www.nvidia.com/en-us/geforce/graphics-cards/50-series/rtx-5060-family/
[nv-license]: https://www.nvidia.com/en-us/drivers/nvidia-license/
[u-signing]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/sign/SIGNING.md
[u-nvapi-project]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/AppSandbox-NVIDIA-DLSS-shim.vcxproj
