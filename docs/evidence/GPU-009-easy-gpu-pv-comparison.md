# GPU-009: Easy-GPU-PV versus the normal Hyper-V path

## Conclusion and scope

Comparison completed 2026-10-05. HCS-owned-guest work is paused. The product
remains Rust around an ordinary Generation 2 Windows 11 Hyper-V VM.
Easy-GPU-PV is the primary reference for VM settings, partition assignment and
driver destinations; AppSandbox is secondary for GPU-PV internals and measured
API compatibility issues. See [DEC-024](../DECISIONS.md#dec-024).

The comparison initially found that the native experiment had **not reproduced
Easy-GPU-PV's configuration**.
Initial low/high MMIO were 128 MiB/512 MiB rather than 3 GiB/32 GiB, guest-controlled
cache types and nested virtualization are false rather than true, and the
stager omits 53 of its 54 associated non-DriverStore destinations. This is a
concrete reason to return to the ordinary VM path. It does **not** establish
which difference causes Code 43, or prove that all 53 destinations are necessary.
The full-copy clean-child test below now sustained **Code 0** and passed
**nvidia-smi, D3D11, D3D12 and CUDA vector addition**. It establishes a working
normal Hyper-V baseline without isolating the contribution of each change.
The working guest is preserved; full Rust writer integration and CUDA/D3D LUID
correlation remain incomplete.

The initial comparison performed source inspection, read-only host/VM/driver inventory,
arithmetic evaluation and host manifest validation. It did not apply the upstream
settings, copy drivers into the guest, restart a VM, run a guest workload or
execute HCS operations. Earlier guest Code 43 evidence remains the last recorded
guest result at that point. The follow-on full-inventory/live experiment is
recorded below; do not confuse the initial snapshot with its resulting state.

## Pinned source and dependencies

Inspected James Stringer's Easy-GPU-PV at commit
[`2353d36325e18c759ca3888e6591e18e5f371011`][easy-commit], returned by
`git ls-remote ... HEAD` and verified in an ignored local shallow checkout.
The repository is archived; its replacement announcement does not change this
project's scope. The relevant source is:

| Source | Responsibility inspected |
|---|---|
| [`CopyFilesToVM.ps1`, lines 2484-2486][easy-stage-call] | Calls driver copy while the Windows volume is mounted, before creating/starting the VM. |
| [`CopyFilesToVM.ps1`, lines 4303-4390][easy-vm] | GPU selection, all resource settings, Generation 2 VM settings, TPM and lifecycle. Top-level parameters use 50% resource allocation; the helper's unused default is 100%. |
| [`Add-VMGpuPartitionAdapterFiles.psm1`][easy-files] | Actual service/package discovery and associated-file destination rules. |
| [`Update-VMGpuPartitionDriver.ps1`][easy-update] | Existing-VM shutdown, offline disk copy and conditional restart. |
| [`PreChecks.ps1`][easy-prechecks] | Friendly-name discovery through partitionable-GPU hardware IDs; not a workload test. |

Only inbox Windows PowerShell, WMI/CIM, PnP, Hyper-V and filesystem facilities
are needed for this extracted path. ISO conversion, installation automation,
remote desktop, virtual display, audio, user creation and autologon are excluded.
No upstream script was executed or imported. No source or proprietary payload
was vendored. The inspected tree has no root LICENSE file; AppSandbox's MIT
license must not be assumed to license Easy-GPU-PV. Preserve attribution and
resolve applicable permissions before copying upstream implementation text.

### Copy-module line trace

| Pinned lines | Effective behavior |
|---|---|
| 1-7 | Parameters: host, mounted guest volume, selected GPU name. |
| 8-10 | Normalize the guest drive-letter colon. |
| 12-18 | AUTO resolves first partitionable interface and an OK matching PnP device; reads its service. |
| 19-22 | Named selection resolves the first OK matching PnP device and service. |
| 23-27 | Comment/logging, then signed-driver rows selected by DeviceName; no Microsoft-provider exclusion. |
| 29 | Create HostDriverStore root. |
| 32-38 | Read service binary path, derive package directory, copy its entire tree if absent. |
| 39-43 | Initialize arrays; process every selected signed-driver row. |
| 44-46 | Escape DeviceID, construct the WMI antecedent and filter all CIMDataFile associations. |
| 47-48 | Record driver name and ID. |
| 49-51 | Create the NVIDIA Corporation drivers directory for NVIDIA. |
| 52-56 | Decode the dependent file path and read file/version metadata. |
| 57-63 | DriverStore file: derive its package directory and copy the entire tree if absent. |
| 64-72 | Other file: replace host drive with guest drive, create its directory and overwrite with a normal copy. |
| 74-77 | Close loops/function; no INF installation, registry import, service creation or startup action. |

### Why the copy-only update can work

Microsoft documents that the guest uses VRD instead of the vendor KMD. Guest
Dxgkrnl forwards calls over VMBus, translates host UMD paths into HostDriverStore,
and offers registry queries marshalled to the host. The vendor KMD registry is
not simply mirrored into the guest. System32/SysWOW64 runtime placement is also
part of the documented GPU-PV model. See [GPU paravirtualization][ms-gpupv].

Thus copying `nvlddmkm.sys` is package closure, not evidence that the guest must
install/run that NVIDIA kernel service. The inbox `vrd.inf` binds PCI vendor
1414/device 008E to `VirtualRender` (`vrd.sys`). The inspected host INF uses
`%13%\nvlddmkm.sys` for the physical NVIDIA service and renames
`nvcuda_loader64.dll` -> System32 `nvcuda.dll`, `nvml_loader.dll` -> `nvml.dll`,
and `nvcuvid64.dll` -> `nvcuvid.dll` in `nv_system32_copyfiles__01` (11921-11939).
These installed names explain why package-only presence is insufficient for
normal loader lookup. INF `DriverSupportModules` and UMD path entries describe
host driver configuration; the copy module does not import their AddReg sections.

The updater assumes an already installed Windows guest with inbox GPU-PV support,
an existing configured VM/adapter and a compatible host driver. Initial creation
adds VM/resource settings, TPM and the adapter after offline copy. The GPU-specific
source trace contains no NVIDIA installer or additional required registry write.
This explains its design; it does not prove readiness on this exact driver/build.

## Existing-VM sequence

```text
Existing Windows 11 x64 Generation 2 Hyper-V VM, cleanly stopped
        |
Verify disposable VM/disk chain and preserve Secure Boot/TPM
Set low/high MMIO, cache types, static RAM and CPU virtualization settings
        |
Identify the partitionable RTX 5060 and its exact interface/device/service
        |
Copy service package + associated package trees into HostDriverStore
Copy associated non-DriverStore files to equivalent guest paths
(offline child volume in Easy-GPU-PV; finish and unmount before starting)
        |
Add-VMGpuPartitionAdapter with explicit -InstancePath
Set-VMGpuPartitionAdapter: VRAM, Encode, Decode, Compute triples
        |
Start-VM through normal Hyper-V lifecycle
        |
Windows initializes its virtual GPU and loads matching vendor components
        |
Observe sustained PnP readiness, then run checked NVIDIA/D3D11/D3D12/CUDA probes
```

This is an extraction for an existing VM, not a command to run
`CopyFilesToVM.ps1`: that script rejects an existing VM/disk. Its actual creation
order is **offline copy -> New-VM -> VM/memory/CPU/TPM settings -> Add GPU -> four
Set GPU calls -> Start-VM**. The update script supplies the existing-disk copy
path but does not set VM options or attach a GPU.

Our implemented sequence is **start an unattached guest -> PowerShell Direct
manifest/alias staging -> stop -> explicit GPU attach while off -> start**.
Both put the package in place before the next attached boot. The offline versus
online staging difference remains recorded; equivalence of resulting bytes and
destinations must be checked, not inferred from transport alone.

The last two arrows are validation obligations. These scripts do not check
sustained PnP health, render an output-verified D3D workload or prove CUDA on an
RTX 5060. Upstream source behavior is not hardware evidence for this target.

## VM settings: upstream versus initial observation

Read-only snapshot: **2026-10-05 01:03:30 UTC**, host Windows 11 Pro x64
`26300.9457`, NVIDIA RTX 5060, driver `32.0.16.1692` (616.92). VM GUID, GPU
interface and differencing child/parent paths were matched to
[`config/project.toml`](../../config/project.toml) before inspection. Raw output:
the initial `local/evidence/easy-gpu-pv-comparison.json` (later refreshed during B).
At that initial comparison, the established guest was Windows 11 Pro x64
`26200.9457`; the subsequent experiment below reread its OS/PnP state.

| Setting | Easy-GPU-PV path | Disposable VM before B | Initial assessment |
|---|---|---|---|
| Generation | `New-VM -Generation 2` | 2 | Matches; this source does not prove Generation 1 cannot work. |
| VM configuration version | Last supported version with major below 254 | 12.0 | Selection policy differs; exact upstream selection on this host not measured. |
| LowMemoryMappedIoSpace | `3GB` = 3,221,225,472 bytes | 134,217,728 bytes (128 MiB) | Untested difference. |
| HighMemoryMappedIoSpace | `32GB` = 34,359,738,368 bytes | 536,870,912 bytes (512 MiB) | Untested difference. |
| GuestControlledCacheTypes | true | false | Untested difference. |
| Dynamic memory | false | false | Matches. |
| Startup RAM / processors | Example 8 GiB / 4 | 8 GiB / 4 | Matches example. |
| ExposeVirtualizationExtensions | true on Windows 11; skipped only for AMD host builds below 22000 | false | Untested difference; upstream sets it, but does not prove it is required for GPU-PV. |
| Secure Boot | No explicit change in the traced code; relies on New-VM firmware defaults | On, MicrosoftWindows template | No basis to disable it. Upstream live firmware not measured. |
| TPM | New local key protector, then Enable-VMTPM | TpmEnabled=true | TPM presence matches. Preserve existing protector; do not replace it merely for parity. |
| CheckpointType | Disabled | Standard; automatic checkpoints false, prior verified baseline has no snapshots | Policy differs; retain in comparison. |
| AutomaticStopAction | ShutDown | ShutDown | Matches. |
| GPU identity | Named-GPU branch passes matched provider Name to `-InstancePath`; AUTO omits explicit path | One adapter, exact configured RTX interface | Named-GPU intent matches; ours rejects ambiguous/fallback selection. |
| Driver-copy timing | Offline before attached start | Online while unattached, then off/attach/start | Different transport/timing, needs destination equality evidence. |
| Windows build pairing | README recommends matching host/guest versions | Host 26300.9457 / last measured guest 26200.9457 | Unresolved compatibility variable; no new guest build claim. |

The exact live interface was:

```text
\\?\PCI#VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1#95B0EB63032DB04800#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV
```

Our fixed assignment operation in `src/bin/hyper-gpu-runner.rs` validates
target identity and calls Add with that path. It does **not** configure MMIO,
cache types, nested virtualization or call Set for GPU resources. The original
local VM preparation explicitly disabled nested virtualization and used static
RAM; successful attach/reapply consequently did not prove upstream settings parity.

### All four resource triples

Upstream computes `divisor = round(100 / percentage, 2)` in a PowerShell
`[float]` variable, then rounds each numerator/divisor. It uses the same value
for **minimum, maximum and optimal**, with numerator 1,000,000,000 for VRAM,
Decode and Compute, and 18,446,744,073,709,551,615 for Encode. With the shipped
top-level **50** input, local Windows PowerShell evaluation produced:

| Resource | Easy min / max / optimal | Adapter before B min / max / optimal | Initial CurrentPartition value | Host advertised min / max / optimal |
|---|---|---|---|---|
| VRAM | 500,000,000 / 500,000,000 / 500,000,000 | null / null / null | 1,000,000,000 | 0 / 1,000,000,000 / 1,000,000,000 |
| Encode | 9,223,372,036,854,775,808 for all three | null / null / null | 1,000,000,000 | 0 / 18,446,744,073,709,551,615 / 18,446,744,073,709,551,615 |
| Decode | 500,000,000 / 500,000,000 / 500,000,000 | null / null / null | 1,000,000,000 | 0 / 1,000,000,000 / 1,000,000,000 |
| Compute | 500,000,000 / 500,000,000 / 500,000,000 | null / null / null | 0 | 0 / 1,000,000,000 / 1,000,000,000 |

Null was independently observed through both Get-VMGpuPartitionAdapter and the
VM-correlated Msvm_GpuPartitionSettingData instance; it is **not zero**. Config
requests provider defaults. A prior 2026-10-04 experiment recorded explicit
host-advertised triples before later reset/reattach; that historical result is
not the current assignment and did not reproduce upstream's 50% triples.
CurrentPartitionCompute=0 is an observed field, not proof of CUDA impossibility.

These numbers must not be relabeled as 4 GiB, guaranteed half of physical GPU
resources, throughput, or working encode/decode. Test exact requests, readback
and workloads. The encode arithmetic result was System.Decimal before UInt64
conversion; rounding the half-integer yielded the value above. Do not use
floating-point JSON parsing to compare that large integer.

## Driver discovery and destination comparison

The copy module follows this path:

1. Resolve an OK PnP device by requested name (first match), or use the first
   partitionable adapter and a partial device-ID match for AUTO. Read its Service.
2. Query Win32_PNPSignedDriver by DeviceName. Resolve Win32_SystemDriver by that
   Service; derive the service binary's package directory from its path.
3. Recursively copy that whole service package to the corresponding
   HostDriverStore path if the destination directory does not already exist.
4. Match Win32_PNPSignedDriverCIMDataFile associations using the signed driver's
   escaped DeviceID/WMI Antecedent. Decode each associated file path.
5. For a DriverStore-associated file, copy its entire package directory to
   HostDriverStore if absent, including additional packages if associations
   reference them. For every other file, create its equivalent guest directory
   and copy with overwrite. It is not limited to a handwritten DLL list.
6. For NVIDIA, ensure `Windows\System32\drivers\Nvidia Corporation` exists.
   This is destination preparation, not an NVIDIA service/INF installation or
   special DLL rename routine. Renamed loader destinations already exist among
   the host-associated files. No hard-link, shim, registry or INF-install action
   appears in this module.

The service-path split assumes the conventional C:\Windows DriverStore depth;
other mapping uses literal C: paths. The comment about excluding Microsoft files
is not backed by a provider filter: selection is by GPU name/associations. These
are limitations to avoid in a Rust implementation, not extra requirements.

### What discovery returned on this machine

The selected PnP device resolved to **nvlddmkm**, whose running service binary is
inside `nv_dispi.inf_amd64_b20cc8aeaed64fc2`. One signed-driver row matched the
exact physical DeviceID: NVIDIA `32.0.16.1692`, published INF `oem59.inf`.
The service's package matches our configured manifest source.

There were **236 distinct associated files**: **182 within that one package**
and **54 outside DriverStore**. No second associated DriverStore package was
found. Our whole-tree manifest contains **217 files**, so its package portion
already includes those 182 plus the remaining package files. The independent
Rust manifest inspector passed the exact 217-file / 2,850,973,044-byte tree and
configured hashes. The demonstrated gap is destination coverage outside the
package, not an observed omitted file within that package.

| Outside-DriverStore destination group | Count | Maintained stager coverage |
|---|---|---|
| Windows\System32 | 20 | Only nvcuda.dll; 19 other destinations omitted. |
| Windows\SysWOW64 | 12 | None; recorded parity differences, outside v1 x64 application coverage. |
| Windows\System32\drivers\Nvidia Corporation | 1 | license.txt not placed here. |
| Same directory's DRS subdirectory | 2 | nvdrsdb.bin and dbInstaller.exe not placed here. |
| Windows\System32\lxss\lib | 18 | None; upstream copies Linux/WSL files even though this product targets Windows. |
| Windows\INF | 1 | oem59.inf not placed here. Copying it is not PnP installation. |

The 20 System32 entries are `nvcuda.dll`, `nvml.dll`, `nvapi64.dll`,
`nvcuvid.dll`, `nvEncodeAPI64.dll`, `nvofapi64.dll`, `OpenCL.dll`, `nvIFR64.dll`,
`nvFBC64.dll`, `nvcudadebugger.dll`, `nvcpl.dll`, `nvinfo.pb`, `nvidia-pcc.exe`,
`nvidia-smi.exe`, `NvDebugDump.exe`, `MCU.exe`, `vulkan-1.dll`,
`vulkan-1-999-0-0-0.dll`, `vulkaninfo.exe`, and `vulkaninfo-1-999-0-0-0.exe`.

The complete [54-file destination/hash table][destination-table] records every
source and equivalent guest destination, including SysWOW64 and LXSS entries.
`product_destination` means a destination managed by our stager, **not a fresh
guest existence check**. In particular, an earlier manual NVML copy may remain
in the guest; it does not make NVML part of our reproducible manifest.

Our stager generates hashes from the configured package, checks active
DeviceID/driver version and signatures, atomically stages the tree and copies
`nvcuda_loader64.dll` as System32\nvcuda.dll. It does not discover the full
association closure at apply time. A package-contained copy of nvapi64.dll or a
loader does not satisfy a different System32 destination. Hash verification of
the smaller destination set cannot establish parity with the larger one.

Fresh discovery at **2026-10-05 17:16:12 UTC** reproduced the same 236 associations
on Windows 11 Pro x64 26300.9457 / RTX 5060 / NVIDIA 616.92. The
[full copy inventory](GPU-009-easy-gpu-pv-full-inventory.tsv) expands whole-package
copying to **271 destinations**: 217 package files plus 54 external files. Each row
records source, association versus package closure, signed-driver DeviceID,
classification, package directory, upstream/current destination, manifest membership,
SHA-256, size and version. The 35 package files outside the 182 associations are
still copied by upstream's recursive package rule. No current logical destination
is extra relative to that rule; System32 nvcuda.dll is a separately managed alias,
whose host-associated bytes are the same loader bytes. The missing destinations
are the other 53 external paths, not a second DriverStore package.

The selected physical driver registry also contains `CopyToVmWhenNewer` and
`CopyToVmWhenNewerWow64` entries matching NVIDIA INF lines 8106 onward. Native
read-only evidence: `local/evidence/easy-gpu-pv-host-registry.json`; host class
`UserModeDriverName` identifies `nvldumdx.dll` in the package. Easy-GPU-PV copies
the associated external paths directly, without requiring these registry keys to
be recreated locally in the guest.

| Behavior | Easy-GPU-PV | Existing Rust stager/VM baseline | Baseline action |
|---|---|---|---|
| Driver identification | OK PnP GPU, service, signed-driver WMI row | Configured package with exact physical identity/version verification | Correlate the selected interface/PnP/service/signed row before discovery. |
| Associated files | All matching CIMDataFile associations | No association closure | Include all 236 association results and full package closure. |
| DriverStore | Whole service and associated package trees -> HostDriverStore | Same 217-file single package | Copy normally; verify every hash. No additional package discovered. |
| Non-DriverStore | Every equivalent Windows path, overwrite | CUDA alias only | Add all 54 paths, including all x86/WSL/INF/profile destinations. |
| CUDA | Installed System32/SysWOW64 loader names plus package binaries | x64 System32 alias plus package | Reproduce both installed loader paths and their exact host bytes. |
| NVML/NVAPI | Installed System32 loaders/API files plus package | Package only | Copy System32 nvml.dll and nvapi64.dll at their original names. |
| Kernel service | Copy nvlddmkm package, no guest service install | Copy same package, guest uses VirtualRender | Preserve inbox binding; no NVIDIA KMD/service installation. |
| MMIO/cache | 3 GiB/32 GiB; cache types true | 128 MiB/512 MiB; false | Apply/read back upstream settings before attached boot. |
| Memory/CPU | Static memory, virtualization extensions true on Win11 | Static; extensions false | Preserve 8 GiB/4 vCPU; enable extensions. |
| Checkpoints | Disabled | Standard, automatic checkpoints off | Disable checkpoints. |
| Resources | Exact effective 50% min/max/optimal triples | Provider defaults | Set/read back all twelve fields, including exact UInt64 encode arithmetic. |
| Startup/update | Offline copy before attached boot; copy-only update | Online staging then off/attach/start | Offline clean-child provisioning, unmount, attach/configure/start. |

### Updates and existing destinations

The upstream update script resolves the VM and its VHD, records whether it was
Running, issues Stop-VM -Force if not Off, polls for Off, mounts the disk,
invokes the same copy module, dismounts, and starts only if previously Running.
It leaves GPU assignment/resources/firmware unchanged. A new package directory
is copied; an already existing package directory is **skipped rather than
rehashed/repaired**. Associated files outside DriverStore are overwritten.
Old package directories are not removed. There is no bounded shutdown timeout,
transactional receipt, target enrollment or try/finally dismount protection.

Our matching reapply rehashes all managed destinations and is a verified no-op;
partial/mismatched state requires disposable recovery. Preserve those guarantees
while extending destination discovery. Do not execute the upstream updater
against the golden disk or adopt its unbounded lifecycle/error handling.

## Native reproduction order

GPU-009 used this reproduction order; the initial comparison alone did not complete its readiness
gate. Record preimages and exact effective settings for each bounded run. Keep
Secure Boot/TPM/isolation intact, reverify the configured disposable target before
effects, and use cold guest starts and the same observation interval/probes.

1. Preserve the prior 217-file/alias Code 43 baseline as historical A evidence.
   Recreate a clean disposable child through the fixed runner for B.
2. Copy the whole discovered package and every associated external file offline;
   verify all 271 destinations. Apply the upstream MMIO/cache/static-memory/
   virtualization-extension/checkpoint profile together, then attach the pinned
   GPU and set/read back the exact effective 50% resource triples.
3. Start through the normal runner. Read the actual guest binding, rehash the
   complete destination closure after boot and observe PnP for 120 seconds without
   a device cycle or further configuration mutation.
4. If Code 0 is sustained, immediately run nvidia-smi and the existing checked
   D3D11/D3D12/CUDA probes. Keep the working state and then refine Rust discovery.
   If Code 43 remains, establish filesystem/settings parity before pursuing a new
   cause. Build pairing, VM version and the global DXGK channel remain measured
   variables, not diagnoses or automatic blockers. Microsoft documents that VMMS
   host/guest builds can differ; our exact pair still needs qualification.

This combined reproduction is deliberately a baseline test, not an experiment
that can attribute success/failure to one individual file or setting. Only after
the full behavior works should controlled reduction identify unnecessary parts.

HCS guest creation/lifecycle, Plan9 transport and an AppSandbox replacement are
not fallback implementation work. A failed experiment is evidence to investigate
within the normal-VM objective, not permission to change that objective.

## Initial comparison checks

- `git ls-remote https://github.com/jamesstringer90/Easy-GPU-PV.git HEAD`, shallow
  clone and commit/tree inspection; only the named GPU/VM source paths reviewed.
- `local/scripts/inspect-easy-gpu-pv-comparison.ps1`, elevated read-only query
  through ordinary UAC because fixed-runner inspection lacks these fields.
  Second run added enum names, all adapter fields and VM-correlated WMI resource
  readback to distinguish actual nulls from formatting. No guest credential used.
- `local/scripts/summarize-easy-gpu-pv-comparison.ps1`, association accounting,
  full outside-destination table and exact upstream 50% arithmetic evaluation.
- `cargo run --locked --bin hyper-gpu-stage`: passed; manifest SHA-256
  `31cf877d2415ad686f34f6498ae4a08893cebdc699244be1649e5379e9516ea3`,
  tree SHA-256 `3662d618d9fb95d02a354077337d58dbc4c3ccf9db14714b1c5fbc4a675cf950`.

These initial checks contained no new guest workload result. Existing host probe
controls remain host-only evidence.

## Full-copy clean-child experiment (2026-10-05)

Historical A is the prior clean-child, verified 217-file package/CUDA-copy baseline
that reached Code 43, including the timestamped repeat linked above. B uses a new
child from the same protected parent; this is a combined reproduction test, not
an independently repeated A or an attribution test of individual deltas.

The fixed runner verified the parent hash and completed `remove-gpu`
(`1791220860-885207400`), `reset-slot` (`1791221110-948673200`) and exact RTX
`assign-gpu` (`1791221544-706329700`). Offline provisioning used
`local/scripts/provision-easy-gpu-pv-baseline.ps1`; the complete 271-file
closure was copied normally before any attached boot. Independent `-Phase Verify`
rehash/readback completed **17:31:23 UTC** with all 271 hashes matching. Evidence:
`local/evidence/easy-gpu-pv-Verify.json` and its `.files.json` output.

The first harness readback used cached Hyper-V child settings and reported a
mismatch after successful copies/settings. A fresh independent query proved the
settings had applied; the harness was corrected to use a fresh VM object. The
verification phase did not retry/overwrite the driver files. This was a harness
readback error, not an observed guest initialization result.

Verified VM profile: 3 GiB low / 32 GiB high MMIO, guest-controlled cache types true,
static 8 GiB / four vCPU, virtualization extensions true, checkpoints disabled,
Secure Boot On/MicrosoftWindows and TPM retained. VM configuration version remains
12.0. The resource phase applied and verified min=max=optimal **500,000,000** for
VRAM, Decode and Compute, and **9,223,372,036,854,775,808** for Encode. These are
opaque provider units, not an assertion of physical 50% performance. Evidence:
`local/evidence/easy-gpu-pv-Resources.json`.

Probe executables/FATBIN were copied separately to the protected guest staging
area and hash checked. They are test artifacts, not part of the 271-file driver
environment. `local/scripts/observe-easy-gpu-pv-baseline.ps1` reads the guest's
complete copy closure, inbox binding and 120-second PnP samples; it runs the existing
NVIDIA/D3D/CUDA checks only after sustained Code 0.

`start-slot` (`1791221779-842676600`) succeeded. Credentialed observation verified
all 271 guest file hashes and captured **116 consecutive Code 0 samples** from
17:41:11.2101053 to 17:43:10.699229 UTC. No problem devices remained, including the
previous Code 12 DXGK VMBus child. The GPU-PV device still binds inbox `vrd.inf`
10.0.26100.1150 / VirtualRender; no conventional NVIDIA INF installation or registry
import was performed. Evidence: `local/evidence/easy-gpu-pv-guest-baseline.json`.

The initial probe processes lacked the Microsoft VC runtime and terminated before
their main entry point. The test-only follow-up supplies the three imported x64
CRT DLLs from the installed Visual Studio REDIST directory beside the probe
executables, with Microsoft signatures and SHA-256 verification. This app-local
test setup is separate from GPU file provisioning. Microsoft's
[redistribution guidance](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files?view=msvc-170)
permits app-local runtime placement subject to the licensed Visual Studio terms
and distributable-code list; it recommends central installation for serviced
deployments. These local test artifacts are not committed or publicly distributed;
release packaging must review the applicable Visual Studio license/REDIST list.

### Checked guest workloads and remaining limitation

Host: Windows 11 Pro x64 **26300.9457**; guest: Windows 11 Pro x64
**26200.9457**; NVIDIA RTX 5060 8 GB, driver **616.92 / 32.0.16.1692**.
The final probe evidence is `local/evidence/easy-gpu-pv-guest-probes.json`
(17:54:28 UTC); the final device check remains Code 0 with no problem devices.

| Check | Actual result |
|---|---|
| nvidia-smi | Exit 0; RTX 5060, 616.92 |
| D3D11 offscreen | Exit 0; hardware GPU-PV, feature level 12_1, checked 256x256 frame |
| D3D12 offscreen | Exit 0; same hardware GPU-PV adapter, feature level 12_2, checked 256x256 frame |
| CUDA Driver API vector addition | Exit 0, `Result = PASS`; allocation, copies, sm_120 kernel and CPU output comparison |
| CUDA identity companion | Exit 2; CUDA LUID `00000000:00014b54` differs from guest D3D LUID `00000000:00005425` |

Both graphics probes produced SHA-256
`00f88da6c22b46ab45bfc5fbc6659e601ebcefe324d52a3302e614d4a7fb3de4`.
Initial graphics selection rejected zero subsystem/revision fields in the guest
DXGI descriptor. The Rust selector now queries D3DKMT physical device IDs for a
single-physical-adapter GPU-PV device, validates all configured PCI fields, and
selects the render adapter instead of the separately exposed display proxy.
Actual selected identity is `10DE:2D05`, subsystem `8A151043`, revision `A1`;
software and indirect rendering remain rejected. This fixes probe identity
selection; no GPU files or settings changed after stable initialization.

The CUDA companion's LUID equals the physical host control LUID. That observation
does not establish cross-namespace correlation, and the mismatch is retained as
a failure. CUDA computation success is separate from that unresolved identity
contract. No interoperability or broader CUDA capability is claimed.

Historical A reached Code 43; fresh B passed with settings and file placement
changed together. There was no fresh repeated A and no causal reduction. In
particular, the 53 missing external destinations are an observed provisioning
gap, not a claim that all are required to clear Code 43.

### Rust implementation and validation

Native COM/WMI discovery and a complete deterministic destination/hash manifest
are implemented as a read-only Rust inspector. The existing bounded guest writer
still supports only its package/CUDA-alias contract; full writer integration and
clean recreation remain GPU-006 work. Temporary experimental PowerShell procedures
are ignored local tooling, not an application backend.

The native inspector completed at **18:16:04 UTC**, independently resolving the
same **236 associations / 271 destinations / one package**. The comparison at
18:17:12 verified every source path, logical destination, byte count and SHA-256
against the experimental inventory. Evidence:
`local/evidence/easy-gpu-pv-native-environment.json`, its `.status.json`, and
`local/evidence/easy-gpu-pv-native-comparison.json`. No PowerShell subprocess is
used by the Rust discovery algorithm.

Native provider testing exposed null `__PATH` on projected signed-driver objects;
the reader therefore matches the same discovered-host/DeviceID association
Antecedent used by Easy-GPU-PV and resolves each Dependent through WMI. Complete
discovery exceeded the existing short inventory deadline; the driver manifest now
has its own validated, configurable discovery deadline. COM setup and individual
provider resolutions are synchronous, as documented on the adapter.

The final maintained check passed formatting, strict Clippy, 123 unit/integration/
doc tests, locked build, rustdoc and generated configuration checks (86 keys).
Documentation checks passed 41 Markdown files and diff whitespace checks passed.
Physical host D3D11/D3D12/CUDA controls passed one warmup and three measured runs
per API (`local/evidence/GPU-009-host-controls-after-selection.json`). One first
parallel full test run had seven PowerShell-startup deadline interruptions while a
release build competed for resources; the serial maintained rerun passed. Guest
workload results above were measured independently and were not inferred from
builds or DLL loading. Local app-runtime and probe hashes are retained in evidence.
An intermediate rebuild also encountered Windows' executable file lock while the
native inspector was running; checks were rerun after the reader completed.

[easy-commit]: https://github.com/jamesstringer90/Easy-GPU-PV/commit/2353d36325e18c759ca3888e6591e18e5f371011
[easy-vm]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/CopyFilesToVM.ps1#L4303-L4390
[easy-stage-call]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/CopyFilesToVM.ps1#L2484-L2486
[easy-files]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/Add-VMGpuPartitionAdapterFiles.psm1
[easy-update]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/Update-VMGpuPartitionDriver.ps1
[easy-prechecks]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/PreChecks.ps1
[destination-table]: GPU-009-easy-gpu-pv-driver-destinations.tsv
[ms-gpupv]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/gpu-paravirtualization
