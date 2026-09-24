# GPU-PV Architecture and Reference Map

Read the section relevant to the task. Fixed scope and design rationale live in
[DECISIONS.md](DECISIONS.md); work and acceptance criteria live in
[BACKLOG.md](../BACKLOG.md). This document describes responsibilities and
evidence requirements, not implementation progress.

## Current state

This is a documentation-only project. The proposed components below are not
implemented; there is no Cargo project or established application build command.
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

Microsoft's Server GPU-P/DDA guidance does not establish support for this
Windows 11/GeForce configuration. It is an experimental validation target;
successful workloads establish a measured configuration, not a change to vendor
support policy. [Microsoft support boundaries][ms-support]

## Proposed components

After a reproducible baseline exists, use one Rust package with a CLI and
reusable library, organized as ordinary modules:

| Responsibility | Boundary |
|---|---|
| Inventory and diagnostics | Read host/guest identity, driver versions, partition availability, device status, and results. |
| GPU configuration | Validate intent, produce a concrete change plan, call native management, and verify the actual assignment. |
| Guest driver/runtime preparation | Produce a selected-driver manifest; transfer, validate, and restore necessary files/settings. |
| Capability probes and reporting | Invoke focused workloads and produce readable and machine-readable evidence. |

Configuration targets an existing VM ID. Windows owns VM lifecycle, storage,
and integration services. Keep GPU logic callable without a GUI; a future
configuration editor may call the same core. Fixed scope and backend selection
criteria are recorded in [DECISIONS.md](DECISIONS.md).

Start with small Windows PowerShell adapters where they avoid duplicating
Hyper-V logic. Use fixed operations and structured data, not interpolated
arbitrary shell fragments. Direct Win32/WMI Rust bindings need a specific
reliability or capability benefit. Necessary C/C++ shims belong behind a narrow,
documented ABI; each workaround needs a reproduced failure, affected driver/build
range, validation, and removal condition.

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
| [`adapter_identity.c`][u-identity], [`adapter_hooks.c`][u-hooks] | D3DKMT/DXGI queries and hooks reconcile host/guest adapter identity. Conditional compatibility work, separate from management; retain or port a small attributed C ABI component only if needed. |
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
| Direct access, if justified | WMI/CIM `root\virtualization\v2`, including [`Msvm_PartitionableGpu`][ms-wmi-gpu] and [`Msvm_GpuPartitionSettingData`][ms-wmi-settings]. |
| Guest execution/transfer | [PowerShell Direct][ms-psdirect], `Invoke-Command -VMId`, persistent sessions and `Copy-Item -ToSession/-FromSession`; requires guest credentials/integration support. |
| Guest storage/preparation | Existing Windows/Hyper-V tools; approved offline VHDX servicing only when necessary. |
| Diagnostics/presentation | PnP status, DXGI/D3DKMT, event logs and guest API probes; VMConnect/RDP for access. |

These are facilities to discover, not commands already validated on this host.
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
