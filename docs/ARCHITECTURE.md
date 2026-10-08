# GPU-PV management architecture

## Current state

The target is a reusable Rust core, thin CLI, installed bounded runner and native
Windows GUI. The proven NVIDIA/Windows 11 baseline remains research evidence.
The rewritten product is not hardware-qualified by inheritance. Status and remaining
acceptance work belong to [ARCH-001](BACKLOG.md#arch-001).

## Product versus development tooling

**Test-environment automation may support development but must not become production application functionality unless it is explicitly required by the user-facing product. Production code may be used by test tooling; production code must not depend on test tooling.**

The default package does not load contributor configuration or import reset helpers.
The previous application is preserved in the independent `tools/lab/` package.
Historical driver pins, package/alias staging and extended probes remain laboratory
reference material, excluded from the shipped graph.

## Components

| Responsibility | Product modules |
|---|---|
| Runtime intent and observations | `model` |
| Enable/disable, ownership and partial-state reconciliation | `workflow` |
| Native discovery, assignment, settings, allocation and lifecycle | `windows_hyperv`, `windows_wmi`, `windows_com` |
| Installed driver associations, complete payload and trust | `windows_driver`, `payload`, `trust` |
| Fixed transfer/launch and Rust guest preparation | `guest`, `hyper-gpu-guest` |
| Checked hardware rendering | `probe`, `windows_probe`, `d3d11-probe` |
| Protected enrollment, authentication and typed requests | `runner`, `windows_pipe`, `security` |
| Opt-in Windows vault credentials | `credentials` |
| Bounded execution, independent of verification | `process` |
| Presentation only | CLI and `hyper-gpu-gui` |

CLI/GUI → workflow → discovery/Hyper-V/preparation/verification → Windows APIs,
guest transport and privilege runner. Production never invokes the laboratory.

## Working recipe

### GPU discovery and VM identification

Enumerate normal Hyper-V VMs and partitionable GPUs. Select stable VM GUIDs and exact
GPU interfaces; friendly names are display data. Enroll multiple existing VMs, one
GPU each. No golden image or prescribed VHDX is required. Serialize conflicting
mutations; sharing does not promise fairness and needs separate qualification.

### Hyper-V settings and GPU partition resources

Require Generation 2 and stable Running/Off state before mutations. Preserve disks,
CPU/RAM quantities, Secure Boot and security devices. Capture preimages of compatibility
MMIO/cache/checkpoint settings and independently read back effects. Conservative NVIDIA
MMIO values retain the measured recipe pending affected qualification. First-core
attachment omits explicit resources; optional raw VRAM triples require provider limits.
Attach using the selected GPU's discovered WMI object path, then map full/relative
host-resource references against fresh local GPU inventory for interface readback.
All-null adapter allocation fields mean provider defaults, not zero capacity.

### Driver/runtime manifest and guest placement

Discover current service/INF/associated files and package trees through Windows.
NVIDIA is the first preparation adapter. Build the complete deterministic manifest,
map DriverStore to HostDriverStore, and retain external Windows associations.
Reject unsafe/reparse paths and conflicting destinations. Authenticate payload bytes
using embedded signatures or signed catalog membership. Counts/hashes describe this
operation; there are no static driver versions, signer thumbprints or payload counts.
Windows-generated `.PNF` caches are the narrow data exception: protected installed
DriverStore provenance, a same-stem INF authenticated in this manifest, and operation
hashes authenticate their transfer. Runtime/executable files remain signature-bound.

### Staging and guest startup

Prepare a running guest without a newly attached GPU. The fixed Rust guest worker
validates transferred inputs, derives Windows destinations, verifies copies and
publishes a receipt. A narrow PowerShell Direct bridge supplies session, transfer,
bootstrap integrity and fixed worker launch only; DEC-028 records its exception.
Then shut down gracefully, configure/attach, start and verify. Restore initial power
state. A running unchanged target is verified without a restart.

### GPU readiness and workloads

Require guest Code 0 and checked D3D11 rendering on the selected hardware. Loading
DLLs or enumerating adapters does not prove support. CUDA/D3D12/stress remain optional
contributor diagnostics. No interop or enforced VRAM ceiling is claimed.

## Configuration and recovery contract

Parse runtime schema 2 once. Protected administrator enrollment independently
authorizes VM/GPU pairs. The runner accepts fixed typed operations; no arbitrary
commands, caller file sources or host lifecycle operations are exposed.
Global mutation serialization and per-VM journals retain original/applied settings,
prepared digest and pending state. Publish before effects with atomic replacement.
The protected runner writes `audit/<nonce>.json` under product state before
dispatch and publishes `Succeeded` or `Failed` before replying. `Started` without
a terminal outcome means the operation needs inspection and reconciliation, not
that it failed before effects. Records contain operation/target intent and exclude
credentials, raw errors and response payloads; detailed errors stay in the
authenticated reply. Admission publication failure prevents dispatch; terminal
publication failure reports uncertainty and preserves the unfinished admission.
Audit records are retained for administrator inspection.
Reconcile fresh state before retry; preserve externally changed settings. Recovery
never replaces a user disk. Disable retains guest driver files; stale preparation
refreshes on apply. Credentials are ephemeral or explicitly stored per-user/per-VM.
Invalidate the previous prepared digest durably before a stale refresh starts;
publish the new digest only after complete preparation and graceful shutdown.
Keep completed preparation after later settings/attachment failure for retry.
Verification also records pending intent before checks on a running unchanged VM.
Standalone verification reconciles power after an uncertain start and retains both
the verification error and any restoration failure. Failed checks retain pending
intent for retry; they do not replace the last successful verification timestamp.

## Development strategy: guest image and disposable VM

Reuse the [standalone laboratory](../tools/lab/README.md) for designated-target tests.
Keep its separate artifacts, fixed parent protection and reset safeguards. Production
has no reset command. Physical-host lifecycle still needs explicit permission;
guest lifecycle does not.

## Display and presentation boundary

Native controls consume actual discovery and the shared workflow. The
[written GUI specification](gui_roadmap.md)
defines layout and interaction. The current prototype uses Win32 controls through
the existing `windows` crate. Reuse its background runner calls; add an explicit
observed-state/draft split, shared fresh effect preview and bounded close/disconnect
handling. GPU selection cannot broaden administrator enrollment. GiB sliders remain
deferred until units/enforcement are established; no duplicate GUI backend is needed.

Inventory/status are dashboard reads, not guest verification. Status reports fresh
Hyper-V state plus recorded journal state; a prepared digest or last-success timestamp
does not authenticate the current guest receipt or prove current graphics health.
Enable-plan currently validates/hashes the payload, so do not use it for periodic
refresh. Runner requests write protected audit records even for reads. Per-VM partial
discovery errors and a human-readable effect summary remain integration work.

## Native Windows boundaries

Keep COM/VARIANT/job handling and native security in focused adapters. Supervised
process deadlines bound synchronous-provider/remoting limitations. Timeout means
uncertainty, not rollback. Record exact retained external interfaces in DECISIONS.

## Validation contract

Review privileged changes before deployment. Qualify rewritten NVIDIA preparation,
provider-default attachment and two-VM sharing separately. Old laboratory/cmdlet
passes do not close these gates. Report actual environment and checked workloads.

## Test lanes

Hardware-free tests cover identities, mapping, provider-range validation, workflow
ordering, reapply, interruption and protocol refusal. Native tests cover safe OS
primitives. Hardware tests explicitly target designated disposable VMs with immediate
identity checks; do not run repeated feasibility campaigns.

Product host artifacts/state use separate `HyperGpuSupportProduct` known-folder
directories. Interrupted installation disables admission and requires rerunning
administrator install; it never kills an active guest operation. Guest runtime
files retain vetted Windows destination read/execute inheritance. Private worker
bundles and state remain restricted to SYSTEM/Administrators.
