# GPU-009: AppSandbox NVIDIA path versus native guest failure

Inspected 2026-10-04 against current AppSandbox `main` and pinned 0.1.9 commit
[`6f3adb6`](https://github.com/jamesstringer90/appsandbox/commit/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6).
`git ls-remote upstream HEAD refs/heads/main` returned the same commit. This is
source inspection, not a new working AppSandbox guest run. AppSandbox's earlier
successful run on this host remains user-reported; no matching AppSandbox probe
output is available here.

## Actual upstream path

1. [`gpu_enumerate`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/gpu_enum.c#L285)
   enumerates the GPU partition adapter interface with SetupAPI, resolves its
   physical device and service, reads its display-driver `InfPath`, and uses
   `SetupGetInfDriverStoreLocationW` to locate the whole FileRepository package.
   It builds one unfiltered Plan9 share for that directory, mapping
   `DriverStore` to guest `HostDriverStore` (lines 366-434). A WMI fallback for
   driver files exists in the same file; it is not a hand-written file list.
2. [`start_vm_thread`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/asb_core.c#L1097)
   takes these shares and additionally offers the NVIDIA DRS profile directory,
   optional NVIDIA shim payloads and graphics layers (lines 1142-1153).
   [`hcs_start_vm`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/hcs_vm.c#L1454)
   starts the VM and then calls `hcs_apply_gpu`. That function sends an HCS
   `VirtualMachine/ComputeTopology/Gpu` update with `AssignmentMode=List`,
   the selected interface mapped to `65535`, and **`AllowVendorExtension=true`**
   (lines 1393-1458). A missing selected adapter may fall back to Default.
3. [`vm_agent.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/vm_agent.c#L273)
   sends the shares to its running guest agent. [`gpu_copy_thread`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/agent.c#L567)
   uses `p9_copy_share_ex` to recursively copy the unfiltered DriverStore share;
   its filters only select the separate shim share. The source contains no
   NVIDIA package manifest. The selected package's own INF/CAT/UMD files are
   copied together. NVIDIA OpenCL shim replacement has a special backup hook.
4. **The important initialization step:** [`check_gpu_error43` and
   `cycle_gpu_devices`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/agent.c#L172)
   identify Display-class `vrd.inf` devices and query Configuration Manager
   problem codes. After copied files, if Code 43 is present, `gpu_copy_thread`
   disables that exact devnode, waits one second and re-enables it (lines
   653-655). When all files were already present, it explicitly skips this
   cycle (lines 660-663). The nearby Hyper-V Video disable call is commented out.
5. The same thread then calls [`gl_provision`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/agent.c#L455)
   and [`nvidia_runtime_provision`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/agent/gl_vk_provision.c#L1075).
   `find_nvidia_driver_files` (line 400) uses D3DKMT to find a paravirtualized
   NVIDIA adapter and its OpenGL ICD path, substitutes `HostDriverStore` if
   needed, then finds sibling files. It does not infer NVIDIA packages by a
   static filename list. The provisioner can put Optical Flow and OptiX DLLs
   in System32, replace `nvcuda.dll` with an export-checked CUDA shim while
   backing up the original, replace `nvapi64.dll` with its wrapper, and wrap
   `nvopencl64.dll`. OpenGL/Vulkan work can replace System32 and SysWOW64
   `opengl32.dll`, rewrite NVIDIA ICD JSON with `.asbak` backups and register
   Khronos ICD keys. `gl_provision` also handles `dxil.dll` and OpenCLOn12/Dozen
   when its separate graphics-layer payload is present. Those pieces are not
   prerequisites established for this project's D3D11/D3D12/CUDA probes.
6. [`adapter_identity.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/adapter_identity.c),
   [`adapter_hooks.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/adapter_hooks.c),
   [`cuda_opencl_adapter_hooks.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/cuda_opencl_adapter_hooks.c),
   [`cuda_shim.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/cuda_shim.c)
   and [`nvapi_shim.c`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/nvidia/nvapi_shim.c)
   map guest, host and ICD adapter LUIDs, synthesize selected display identity
   for intercepted APIs, and bridge some CUDA/NVAPI queries. These are user-mode
   compatibility hooks. They do not install a kernel display driver or alter
   the `vrd.inf` PnP binding. No NVIDIA Container service install, NVIDIA INF
   install, PCI-ID registry spoof or CUDA environment-variable setup was found
   in this traced Windows GPU share/provisioning path. The host's original
   NVIDIA package service and INF relationship remains in its copied files;
   the guest PnP device uses Windows `vrd.inf`.

## Side-by-side comparison

### Hard-link inventory and rationale

The maintained product had one creation site: guest staging in
`src/windows_guest.rs` attempted to hard-link the already copied
`nvcuda_loader64.dll` as `System32\nvcuda.dll`, falling back to a copy. It was
introduced to mirror the host NVIDIA installer's CUDA loader alias and avoid
duplicate bytes. The existing fallback and AppSandbox's `CopyFileW` runtime
placement provide no evidence that shared NTFS file identity matters. This site
now always copies and verifies the required guest filename, length, version,
hash and protected ACL/path. `src/staging.rs` accepts only a copy receipt.

A separate ignored **one-off experiment**
(`local/scripts/experiment-gpu009-nvml-alias.ps1`) made a guest-local NVML
hard link solely to test whether `nvidia-smi`'s missing DLL explained Code 43.
That link was removed after the test. Repeating the experiment with a normal
copy produced the same result. It was never a product staging path. The host's
NVIDIA-installed hard links are observations, not links created by this
project. No hard-link creation remains under maintained `src/` or `scripts/`.

| Area | AppSandbox | This project | Classification and significance |
|---|---|---|---|
| Host selection | SetupAPI GPU-PV interface, physical display driver INF and DriverStore folder | Configured RTX 5060 interface correlated with installed package and signatures | Equivalent for the pinned target; our pin is stricter. |
| Assignment | HCS update **after start**, `List`/`65535`, `AllowVendorExtension=true` | VMMS GPU partition adapter assigned while off, then VM starts | Implemented differently; no proven VMMS equivalent to the HCS vendor flag. Strong candidate only after guest-side diagnosis or a controlled HCS comparison. |
| Driver package | Recursive copy of entire selected DriverStore folder to `HostDriverStore`; no fixed package file list | 217-file, hash-checked full package to the same mirror | Equivalent payload; our static manifest is generated from the installed package, not an AppSandbox list. |
| NVIDIA profiles | Copies host ProgramData DRS profiles if `nvlddmkm` service detected | No DRS copy | Missing; profile behavior is unlikely to explain a `vrd.inf` Code 43 before APIs run. |
| CUDA loader | Uses a verified file copy to place its shim at System32 `nvcuda.dll`, backing up vendor original | Copy-only staging from package `nvcuda_loader64.dll` to System32 `nvcuda.dll`, with size/hash/version checks; earlier live run used a hard link | Implemented differently; link identity has no demonstrated role. Shim differences matter only after CUDA is actually exercised. |
| NVAPI/OpenCL/OpenGL/Vulkan | Conditional System32, SysWOW64, ICD and LUID hooks | No hooks or registry changes | Missing by design until an API-specific failure is measured. No evidence yet that they fix Code 43. |
| Device recovery | If files copied and `vrd.inf` has Code 43, cycles that devnode after boot | A one-off manual cycle was tried; no automatic recovery | Missing persistent behavior. It briefly cleared Code 43 but the later readiness check again saw 43. |
| Device/runtime checks | Agent logs PnP status and reports copy result; user-mode hooks target API identity | Verified hashes/receipt; readiness checks PnP and would run `nvidia-smi` only after PnP is healthy | Implemented differently; current gate stops before NVIDIA/D3D/CUDA workloads. |

## Current failure evidence and limits

The earlier saved guest readout, `local/gpu009-readiness.json`
(2026-10-04 15:12 UTC), found a virtual `PCI\VEN_1414&DEV_008E` display device
named NVIDIA GeForce RTX 5060, bound to Windows `vrd.inf` version
`10.0.26100.1150`, with Config Manager **Code 43**. Hyper-V Video was healthy.
The diagnostic first verified the configured VM, child/parent, attached host GPU,
staging receipt, package directory and `nvcuda.dll` loader hash. The earlier
`local/gpu009-code43-cycle.json`
(14:59 UTC) recorded a same-device 43-to-0 transition after disable/enable;
the 15:12 check shows that success did not persist. A measured-resource change
before the later check used the host GPU's reported resource fields, including
nonzero compute, so the earlier zero-compute setting alone does not explain the
later Code 43. The staged package was reported applied and then restaged, but
these saved checks did not collect guest PnP event details, DLL load results,
`nvidia-smi`, D3D or CUDA execution. Do not describe those as passed or failed.

A fixed-runner `inspect` returned success and the enrolled child/parent identity
and protected parent hash, with the VM Off and zero GPU adapters (operation
`1791139965-598814600`). The fixed runner then attached the exact configured
RTX interface and started the same child. A fresh, identity-checked guest
readout at 19:11 UTC (`local/evidence/gpu-runtime-diagnostic.json`) again found
the `vrd.inf` virtual render device at **Code 43**. Its package INF, catalog,
CUDA loader and D3D UMD anchors all matched the pinned host hashes. Kernel-PnP
Configuration events 400 and 410 show `vrd.inf` configured and the
`VirtualRender` service started at boot; the later Code 43 remains unexplained
by those events. The guest's Microsoft Hyper-V Video device was healthy.

The packaged `nvidia-smi.exe` initially reported that it could not find
`nvml.dll`. Host `System32\nvml.dll` is a hard link to the same package's
`nvml_loader.dll`; our baseline copied the package but created only the CUDA
System32 alias. A one-file experiment created the equivalent **guest-local**
NVML hard link after checking the exact guest and package hash. `nvidia-smi`
then advanced to “Driver Not Loaded”; the GPU-PV devnode stayed Code 43 before
and after. The result is in ignored
`local/evidence/gpu009-nvml-alias-experiment.json`. This proves a missing NVML
loader alias for that tool, but also rules it out as a sufficient Code 43 fix.
The exact experimental alias was subsequently removed and the package source
rehash passed (`local/evidence/gpu009-nvml-alias-revert.json`, 19:18 UTC).
The `nvidia-smi` process exit code was not
captured reliably by the PowerShell diagnostic, so its bounded output and
unchanged PnP code are the evidence; no success is claimed.

## Clean copy-only run (2026-10-04)

The fixed runner removed the GPU adapter, recreated the exact disposable child
from the verified parent, and started it without a GPU adapter. The updated
Rust stager then copied the 217-file, 2,850,973,044-byte host-matched NVIDIA
manifest to guest `HostDriverStore` and copied `nvcuda_loader64.dll` to
`System32\nvcuda.dll`. Its success receipt requires guest package file count,
sizes and SHA-256 hashes, plus CUDA alias size, SHA-256, file version and
protected path/ACL checks. The alias method is `copy`; a hard-link receipt is
rejected. The runner then shut down the guest, attached the pinned RTX 5060
interface, and started it again (operations `1791142341-009827400`,
`1791142558-190781600`, `1791142775-341558500`,
`1791143460-835831000`, `1791143714-449240600`,
`1791143931-969365700`).

The fresh guest diagnostic at 20:02 UTC again found the `vrd.inf`/
`VirtualRender` device at Code 43. Kernel-PnP events 400/410 show it was
configured and started. `nvidia-smi` still could not find `nvml.dll` in
`System32`. A controlled **ordinary copy** of the manifest's
`nvml_loader.dll` to `System32\nvml.dll` passed length (1,499,368 bytes),
SHA-256 (`ac08344786840363012b31409a62529d6c53ef2658c68d190ba8c3f1fe7c24a2`)
and file-version (`8.17.16.1692`) checks. `nvidia-smi` advanced to
“Driver Not Loaded”; the PnP problem remained 43. This reproduces the earlier
hard-link experiment's result with an ordinary copy, so link identity is not
required for NVML DLL discovery. The one-off result is in ignored
`local/evidence/gpu009-nvml-copy-experiment.json`.

After these copies, cycling the exact `vrd.inf` devnode once using the
AppSandbox sequence (disable, one-second wait, enable) produced a Code 0
readout. A second diagnostic saved about 34 seconds later found Code 43,
but the original transition instant was not recorded. A controlled repeat
without that later diagnostic showed Code 0 through 20:20:06.132 UTC and
Code 43 at 20:20:07.160 UTC after only read-only status sampling. The
unknown VMBus child is the global DXGK GPU-PV channel and has Code 12; its
causal role is still unproven. See the
[`timeline and VMBus evidence`](GPU-009-timeline-vmbus.md). `nvidia-smi`
reported “Driver Not Loaded”
(`local/evidence/gpu009-copy-layout-cycle.json` and
`local/evidence/gpu-runtime-diagnostic.json`, ignored). Neither file-copy
semantics nor a one-time device cycle is a sufficient fix here. The material
unresolved layout/initialization difference is AppSandbox's HCS assignment
after guest start with `AllowVendorExtension=true` versus this project's VMMS
adapter assignment while off; that remains a hypothesis, not a proven cause.

No D3D11, D3D12 or CUDA workload has run in this guest. No physical-host
restart was performed. The recorded host/guest builds are
26300.9457/26200.9457, and NVIDIA is 616.92 (`32.0.16.1692`), as recorded
in CORE-009; that OS-build inequality remains a qualification variable,
not proof of cause.

## Ranked corrective experiments

1. **Most likely: device initialization sequence or VMMS/HCS GPU behavior.**
   AppSandbox applies GPU-PV after HCS start and explicitly requests the vendor
   extension, then cycles Code 43 after copying files. Our pre-start VMMS attach
   plus one transiently successful cycle leaves Code 43 on a later and a fresh
   boot with matching package anchors. The smallest discriminating next test is
   a same-input HCS/AppSandbox guest PnP check with a supported guest-access
   route, or a bounded read-only HCS configuration query for this VMMS guest.
   Do not infer that `AllowVendorExtension` alone is the cause: the API and
   assignment timing also differ. A backend mutation would need boundary review.
2. **Next: host/guest Windows pairing or deeper driver-load failure.** The
   package, CUDA and NVML loaders are present, but the 26300 host / 26200 guest
   pair or a hidden dependency may still prevent virtual render initialization.
   A same-build disposable guest or a specific load error would be the smallest
   proof. Do not install the NVIDIA INF into the guest or reboot the host on
   the present evidence.
3. **Lower confidence: user-mode identity hooks or DRS.** AppSandbox's CUDA,
   NVAPI and graphics shims address adapter/LUID and optional API paths. Test
   D3D11/D3D12/CUDA only after the PnP device remains Code 0. If an individual
   API fails with healthy PnP, isolate that one hook or profile dependency.

The read-only diagnostic is
`scripts/diagnostics/inspect-disposable-gpu-runtime.ps1`. It requires a running,
exact pinned disposable guest, an elevated host token and an interactively
supplied guest administrator credential. It writes ignored
`local/evidence/gpu-runtime-diagnostic.json`; it does not change guest state.
The approved runner was used for VM inspection, attachment and start; it has
fixed operations and no arbitrary guest diagnostic operation. The one-file
NVML experiment was a local test script under `local/scripts/`, not a product
provisioning change.
