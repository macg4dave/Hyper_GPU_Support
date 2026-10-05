# GPU-009: Easy-GPU-PV versus the normal Hyper-V path

## Conclusion and scope

Comparison completed 2026-10-05. HCS-owned-guest work is paused. The product
remains Rust around an ordinary Generation 2 Windows 11 Hyper-V VM.
Easy-GPU-PV is the primary reference for VM settings, partition assignment and
driver destinations; AppSandbox is secondary for GPU-PV internals and measured
API compatibility issues. See [DEC-024](../DECISIONS.md#dec-024).

The native experiment has **not yet reproduced Easy-GPU-PV's configuration**.
Live low/high MMIO are 128 MiB/512 MiB rather than 3 GiB/32 GiB, guest-controlled
cache types and nested virtualization are false rather than true, and the
stager omits 53 of its 54 associated non-DriverStore destinations. This is a
concrete reason to return to the ordinary VM path. It does **not** establish
which difference causes Code 43, or prove that all 53 destinations are necessary.
None of these differences is dismissed as irrelevant without a test.

This session performed source inspection, read-only host/VM/driver inventory,
arithmetic evaluation and host manifest validation. It did not apply the upstream
settings, copy drivers into the guest, restart a VM, run a guest workload or
execute HCS operations. Earlier guest Code 43 evidence remains the last recorded
guest result, not a fresh guest measurement.

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

## VM settings: upstream versus fresh observation

Read-only snapshot: **2026-10-05 01:03:30 UTC**, host Windows 11 Pro x64
`26300.9457`, NVIDIA RTX 5060, driver `32.0.16.1692` (616.92). VM GUID, GPU
interface and differencing child/parent paths were matched to
[`config/project.toml`](../../config/project.toml) before inspection. Raw output:
`local/evidence/easy-gpu-pv-comparison.json`. The last established guest is
Windows 11 Pro x64 `26200.9457`; guest OS/PnP was not reread in this session.

| Setting | Easy-GPU-PV path | Current disposable VM | Assessment |
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

| Resource | Easy min / max / optimal | Current adapter min / max / optimal | CurrentPartition value | Host advertised min / max / optimal |
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

## Next native experiments, with causes kept separate

GPU-009 owns the remaining work; this comparison does not complete its readiness
gate. Record preimages and exact effective settings for each bounded run. Keep
Secure Boot/TPM/isolation intact, reverify the configured disposable target before
effects, and use cold guest starts and the same observation interval/probes.

1. Test low MMIO, high MMIO and cache types against their upstream values, with
   separate controlled changes and then the combined upstream memory profile.
   Measure sustained virtual-render PnP status and the previously failing DXGK
   VMBus channel; the existing Code 12 makes this a useful first hypothesis,
   **not a diagnosed MMIO cause**.
2. Test ExposeVirtualizationExtensions=true; retain the result even if unchanged.
   Test checkpoint policy separately. Record VM version and build-pair differences
   as open variables; do not silently upgrade VM configuration or change the parent.
3. Test upstream's exact 50% resource triples, all four classes, verifying native
   readback before/after start. This is distinct from the earlier full-host-range
   experiment and does not claim actual 50% enforcement.
4. Extend the reviewed Rust destination manifest to represent observed associated
   sources/destinations, starting with the Windows x64 runtime set. Compare actual
   guest paths/hashes, including preexisting files. Establish safe preimages for
   shared System32 files. Keep the complete 54-file delta visible; do not call a
   subset exact parity or conclude excluded x86/LXSS/INF/profile files irrelevant
   without a bounded test. Offline copy timing remains another controlled variable.
5. With settings and destination parity recorded, test a combined upstream-like
   baseline. Once PnP remains healthy, run the existing D3D11 and D3D12 output
   checks and CUDA allocation/transfer/kernel/CPU comparison. Isolate a remaining
   API-specific failure with AppSandbox's secondary reference knowledge.

HCS guest creation/lifecycle, Plan9 transport and an AppSandbox replacement are
not fallback implementation work. A failed experiment is evidence to investigate
within the normal-VM objective, not permission to change that objective.

## Checks actually run

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

No new guest GPU workload result. Existing host probe controls remain host-only
evidence. Documentation checks are reported with the completed task handover.

[easy-commit]: https://github.com/jamesstringer90/Easy-GPU-PV/commit/2353d36325e18c759ca3888e6591e18e5f371011
[easy-vm]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/CopyFilesToVM.ps1#L4303-L4390
[easy-stage-call]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/CopyFilesToVM.ps1#L2484-L2486
[easy-files]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/Add-VMGpuPartitionAdapterFiles.psm1
[easy-update]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/Update-VMGpuPartitionDriver.ps1
[easy-prechecks]: https://github.com/jamesstringer90/Easy-GPU-PV/blob/2353d36325e18c759ca3888e6591e18e5f371011/PreChecks.ps1
[destination-table]: GPU-009-easy-gpu-pv-driver-destinations.tsv
