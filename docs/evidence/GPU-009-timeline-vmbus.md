# GPU-009: Code 0 timeline and unknown VMBus channel

All timestamps below are UTC on 2026-10-04; local UK time was BST (UTC+1).
The target was the pinned disposable VM `2627e735-5b33-4104-b739-622727dd3a40`
with the RTX 5060 GPU-PV interface and its verified child/parent disk chain.
Evidence is in ignored `local/evidence/gpu009-*.json`, the fixed runner's
`C:\ProgramData\HyperGpuSupport\Runner\results` and `audit/events.jsonl`, and
the scripts linked below. The guest was not reset or reconfigured during this
timeline investigation.

## Original sequence

| UTC | Evidence and action |
|---|---|
| 19:58:42 | Last GPU assignment runner result saved: `assign-gpu` succeeded while VM Off. |
| 20:02:21.685–21.687 | Kernel-PnP records configured the unknown VMBus child with no driver (400), then a start problem (411), Code 12 / `0xC0000018`. |
| 20:02:22.535–22.536 | Virtual PCI Bus child configured and started with `wvpci.inf` / `vpci`. The fixed runner's last operation, `start-slot`, completed at 20:02:22. No subsequent runner result or audit entry exists through the investigation. |
| 20:02:24.931–24.940 | `PCI\VEN_1414&DEV_008E` configured and started with `vrd.inf` / `VirtualRender` (Kernel-PnP 400/410). |
| 20:02:52.818 | First fresh diagnostic output saved; GPU Code 43. |
| 20:04:13.285 | One-file **copy** of `nvml_loader.dll` to `System32\nvml.dll`, followed by `nvidia-smi`; GPU Code 43 both before and after. |
| By 20:05:17.811 | [`cycle-gpu009-copy-layout.ps1`](../../local/scripts/cycle-gpu009-copy-layout.ps1) had disabled the exact `vrd.inf` devnode, waited one second and enabled it; its result reports 43 before and 0 after. The commands and status reads were not individually timestamped. `DEVPKEY_Device_LastArrivalDate` later reported guest-local 13:05:17 for the GPU, consistent with this enable, but has only second-level display precision. Thus the exact original 43→0 instant is **not recorded**. |
| By 20:05:51.329 | [`inspect-disposable-gpu-runtime.ps1`](../../scripts/diagnostics/inspect-disposable-gpu-runtime.ps1) saved a Code 43 readout. It queries the GPU's PnP state **before** launching `nvidia-smi`. Its internal read was not timestamped, so the exact original 0→43 instant and whether it preceded the script's launch are **not recorded**. |

The later diagnostic verifies VM/disk/adapter identity, opens a PowerShell
Direct session, reads guest identity, signed driver, PnP properties, video
controllers, files and event logs, then launches `nvidia-smi` and writes/removes
temporary output files. It contains **no** VM lifecycle command, GPU-PV
reassignment, `Disable/Enable-PnpDevice`, driver install/reload, registry write,
system file replacement or `VirtualRender` restart. Since it reads Code 43
before `nvidia-smi`, that process cannot explain the already observed return.
The runner audit confirms no runner operation after the 20:02:22 start.

## Controlled repeat without a follow-up diagnostic

One narrow repeat used
`local/scripts/observe-gpu009-cycle-without-followup.ps1`: the same exact
`vrd.inf` device was disabled, held one second and enabled. The script then
only read `Win32_PnPEntity.ConfigManagerErrorCode` in the **same** guest session;
it did not run `nvidia-smi` or start a second script. Its timestamped result is
`local/evidence/gpu009-cycle-without-followup.json`.

| UTC | Observation |
|---|---|
| 20:19:49.1382549 | Pre-cycle GPU Code 43. Unknown DXGK VMBus child Code 12 at 20:19:49.1797139. |
| 20:19:49.1797139–49.4747378 | `Disable-PnpDevice` ran. |
| 20:19:50.4864574–50.5379727 | `Enable-PnpDevice` ran. |
| 20:19:50.5749567 | First observed GPU Code 0. The transition occurred after enable started and no later than this read; finer timing was not instrumented. |
| 20:19:51.6260814–20:20:06.1319571 | Every roughly one-second read remained Code 0. |
| 20:20:07.1604248 | First observed Code 43. The return occurred in **(20:20:06.1319571, 20:20:07.1604248]**, without the later diagnostic or any further mutating command. |
| 20:20:07.1994762 | Unknown DXGK VMBus child still Code 12. |

The repeat establishes that the later diagnostic was **not necessary** for
the return to Code 43. It does not retrospectively recover the precise instant
of the first run's return. The periodic WMI status reads are read-only; the
evidence does not identify why `VirtualRender` later reports Code 43. The guest
Kernel-PnP Configuration, DeviceSetupManager and System logs have no matching
events in the repeat's 20:18–20:22 window. `setupapi.dev.log` was last modified
on 2026-09-26, so it contains no entry for either cycle.

### Elevated host window timing

The original one-off cycle script returned and closed its elevated host window
soon after saving the Code 0 result. The later Code 43 observation therefore
looked close to that window closing, but its exact onset was not measured.
The controlled repeat distinguishes those events: the elevated host PowerShell
was still blocked in `Wait-Job` while the guest sampled Code 43 at
**20:20:07.1604248**. The guest also read the unknown VMBus status at
20:20:07.1994762; only after the job returned did the host write its result,
stamped **20:20:07.2872301**. The host process could exit only after that
write. No second script or GPU command ran between Code 0 and Code 43.
Thus closing the administrator window is **not required** for the regression
under the repeated conditions. Device Manager's visual refresh time in the
original run was not logged, so UI lag remains a possible explanation of the
reported coincidence; it is not itself proven.

## Unknown VMBus device

Read-only identity-pinned collection in
[`inspect-gpu-pv-timeline.ps1`](../../scripts/diagnostics/inspect-gpu-pv-timeline.ps1)
captured the following at 20:15 UTC:

| Property | Observed value |
|---|---|
| Instance ID | `VMBUS\{DDE9CBC0-5060-4436-9448-EA1254A5D177}\{711DAD3A-73CE-468B-90A9-EDE6906841B2}` |
| Hardware IDs | `VMBUS\{dde9cbc0-5060-4436-9448-ea1254a5d177}`; `VMBUS\{711dad3a-73ce-468b-90a9-ede6906841b2}` |
| Compatible ID | `VMBUS\{dde9cbc0-5060-4436-9448-ea1254a5d177}` |
| Class / class GUID | Unset; Kernel-PnP records all-zero class GUID. |
| Parent | `ACPI\MSFT1000\0` |
| Location path | `ACPI(_SB_)#ACPI(VMOD)#ACPI(VMBS)#VMBUS({dde9cbc0-5060-4436-9448-ea1254a5d177}#{711dad3a-73ce-468b-90a9-ede6906841b2})` |
| Problem | Code 12 (`CM_PROB_NORMAL_CONFLICT`); ProblemStatus `0xC0000018` (`STATUS_CONFLICTING_ADDRESSES`). |
| Driver / INF / service | None matched; no driver provider, version or service. |
| PnP history | 20:02:21.686 configured with null driver/INF (record 204); 20:02:21.687 problem starting, Code `0xC`, status `0xC0000018` (record 205). |
| DeviceSetupManager | 20:03:01.241, event 127: no non-optional driver update matched (record 186). |

The GPU device is distinct: `PCI\VEN_1414&DEV_008E` is a child of the **working**
`VMBUS\{44C4F61D-4444-4400-9D52-802E27EDE19F}\…` Virtual PCI Bus and binds
`vrd.inf` / `VirtualRender`. The unknown child's parent is the VMBus ACPI root.
The unknown child was already present before the GPU's 20:02:24 configuration.
It was Code 12 immediately before and after the repeat's 16-second Code 0
interval; it was not sampled during that interval.

This is **GPU-PV transport**, not an unrelated integration device: the
[Microsoft-authored Linux DXGK VMBus patch](https://lists.openwall.net/linux-kernel/2022/03/01/1464)
names `{DDE9CBC0-…}` the GPU-PV **global DXGK channel** and `{6E382D18-…}`
the per-vGPU DXGK channel. Pinned AppSandbox source
[`dxgkrnl_compat.h`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/tools/linux/dxgkrnl/src/dxgkrnl_compat.h)
defines the same two channels for its Linux guest support. The present Windows
guest's PnP list contains the global channel but no `{6E382D18-…}` child.
That absence is an observation, not proof that a working Windows guest must
enumerate the per-vGPU channel as a PnP device.

AppSandbox's Windows path uses an
[HCS GPU update after VM start with `AllowVendorExtension=true`](https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/src/backend_win/hcs_vm.c#L1393).
It does not explicitly create this global channel or install a Windows driver
for it in the traced code. Our VMMS assignment already exposes the global
channel, so its **existence** is not unique to AppSandbox/HCS. Whether the
Code 12 state is expected on Windows, whether a working AppSandbox Windows
guest binds it, whether a per-vGPU channel is required, and whether HCS's
vendor extension changes either state remain unverified. The recorded Code 12
state at both ends of the GPU's Code 0 interval does not by itself explain
the GPU's later Code 43, though a role in sustained initialization or workloads
is still possible.

[Microsoft's Code 12 reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/cm-prob-normal-conflict)
describes insufficient/conflicting resources; [NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/596a1078-e883-4972-9bbc-49e60bebca55)
names `0xC0000018` `STATUS_CONFLICTING_ADDRESSES`. Those definitions classify
the reported status, not its underlying cause in this VM.
