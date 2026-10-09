# GPU-PV management architecture

## Current state

`hyper-gpu-support.exe` now opens the approved Slint GUI with no arguments and
runs explicit CLI commands through the existing headless path. Root `build.rs`
compiles `src/gui/ui/app.slint` with Fluent style; `src/gui/` owns the moved controller and
explicit mock fixtures. Slint/slint-build are pinned to 1.18.1 with winit/software
rendering and accessibility. The old Win32 entry/presentation was deleted.
Normal startup uses background native discovery when already elevated or the
existing runner's Discover operation with an ordinary token. Refresh/System show
actual data or an explicit failure, with no fixture fallback. Live mutation
controls remain unavailable until binding. `--mock-gui` uses isolated simulated
callbacks and fixtures with no persistent writes. Future real-data rehearsal needs
an authorized no-write read route; audited discovery cannot satisfy that contract.
No protected backend/security boundary changed. Runner and guest/probe payloads remain
separate; same-executable restricted worker and console packaging are pending.
[GUI_GUIDE](GUI_GUIDE.md) freezes v1.0 scope; [BACKLOG](BACKLOG.md) owns integration.
The console subsystem is retained to preserve CLI stdout/stderr and exit status.

The proven NVIDIA/Windows 11 baseline remains research evidence.
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
| Presentation only | `src/main.rs` GUI/CLI dispatch, `src/gui/`, `src/gui/ui/` |

This table describes current modules. Consolidation replaces host presentation/
runner entries, not the bounded ancillary guest preparation/probe payloads.

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
Initial release additionally requires physical GPU selection and all four resource
triples; compute/encode/decode are not implemented or qualified yet (GPU-010).
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

The following describes the implemented baseline. Planned worker-owned per-VM
configuration is defined in [CONFIGURATION](CONFIGURATION.md#approved-per-vm-contract-planned).
CORE-028 replaces implicit pending-Apply recovery with explicit manual reconciliation
and a host-wide unresolved-operation hold, extending existing journals/native guards.

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

The completed prototype sources are the official GUI; preserve their existing
Dashboard/System/Settings/About and controls. Navigation, drafts, dialogs, themes,
splitter and scrolling work. In mock mode, inventory, allocations, enrollment, graphics checks,
Apply/progress/recovery, credentials and saves are simulated only in `--mock-gui`;
live mode does not claim those bindings. Inventory/System use actual discovery in
normal mode. Keep mode/data provenance clear. No additional GUI feature.
Reuse sound `gui_model` state/validation during GUI-002 binding. The GPU Memory
slider is a separate visual mock preference; no physical GB/enforcement meaning.
Provider mapping for selector/allocation controls belongs to GPU-010.

CORE-006's typed plan shares apply's read-only validation and decision calculation.
It reports ordered semantic actions, settings before/after, raw allocation writes,
preparation drift, credential need, downtime and recorded restoration power. Pending
verification alone does not imply preparation or restart. The CLI consumes this
shared summary. GUI-002 must bind the same fresh plan and independent execution
rechecks; the promoted Slint review/apply is currently mocked.
Plan never saves a management journal or calls guest/VM mutators; protected audit
records still apply. Enabled plans authenticate the payload, while status/inventory
remain separate. Slint binding and process/UX acceptance belong to GUI-002/GUI-003.

Inventory/status are dashboard reads, not guest verification. Status reports fresh
Hyper-V state plus recorded journal state; a prepared digest or last-success timestamp
does not authenticate the current guest receipt or prove current graphics health.
Enable-plan currently validates/hashes the payload, so do not use it for periodic
refresh. Runner requests write protected audit records even for reads. Per-VM partial
discovery errors and a human-readable effect summary remain integration work.

## Repository audit — 9 October 2026

Historical pre-promotion source snapshot: inspected only; no builds, tests or hardware queries for that audit. Current presentation/dispatch is described above; retain this matrix as history, not current GUI status.
Historical passes remain in existing cards/evidence and do not qualify new boundaries.

| Existing implementation / evidence in source | Reuse and remaining work |
|---|---|
| `src/model.rs`: schema-2 targets, GUID normalization, strict fields, `Allocation` ordering/provider bounds; `windows_hyperv.rs`: exact GPU WMI selection, optional VRAM readback and native setter with independent readback | GPU-010 extends these to four categories, new pair enrollment/reassignment and typed capabilities. Compute/encode/decode and partition-count display are missing; units/enforcement remain unverified. |
| `src/workflow.rs`: shared decision/plan/apply/verify, typed effects, pre-effect journals, prepared digest invalidation, attributable settings restoration, fresh readback and power restoration; fake-backend ordering/failure tests | CORE-028 adds approved-plan binding, consent, real stage events and manual reconciliation. Current Apply can resume pending work and restores power on some failures; it is not the new host-wide manual-recovery contract. |
| `src/runner.rs`: protected installed artifact hashes, separate enrollment, typed requests, nonce expiry/replay rejection, durable Started/Succeeded/Failed admission records, atomic JSON and no-reparse checks | SEC-001 adapts fixed SYSTEM scheduled-task execution to same-executable per-operation elevation. Current requests have no reviewed-plan revision, progress stream or SaveOnly operation. A worker flag alone must never grant authority. |
| `src/windows_pipe.rs`: local-only first pipe instance, restrictive ACL excluding client server-instance creation, client token SID check via impersonation/reversion, client check of pipe owner SID, bounded framing/client deadlines; round-trip/wrong-peer/partial/oversized-frame tests | Preserve protections; adapt endpoint/identity and handshake for elevated user rather than SYSTEM, add session/operation binding and bounded progress/server waits. Owner SID alone does not authenticate a particular same-user worker. |
| `src/runner.rs::execute`: exclusive `operation.lock` file handle (`share_mode(0)`) across requests, also used during install | OS-backed cross-process serialization already exists; extend its host-wide scope across new entry modes/sessions. It currently serializes reads too. Crash handle release does not resolve journals/audits. |
| `src/gui_model.rs`: observed/saved/draft split, eligibility, single-draft retention, unsaved/readback gates, deliberate reapply, expected-content checks and flush/rename save; focused state/conflict tests | Reuse presentation state. CFG-001 moves persistence to protected per-VM worker path and makes pending-save recovery durable. GUI-002 adds first-time enrollment, GPU editor and switch confirmation. |
| `src/main.rs`, `src/bin/hyper-gpu-gui.rs`, `src/bin/hyper-gpu-runner.rs`: separate CLI/Win32/SYSTEM entries; `tests/cli.rs`, opt-in `tests/m1_enrollment.rs` | APP-001 introduces dispatch/console handling and per-session activation. Existing tests are regression starting points, not Slint or consolidated-worker acceptance. |
| `windows_driver`, `payload`, `trust`, `guest`, `probe`, `credentials`, `security`, `process`, `windows_wmi`: dynamic signed preparation, bounded bridge/guest logic, checked graphics, vault, protected paths and supervised native effects | Preserve working boundaries, DEC-028 and M1/M2 evidence. Qualify only affected paths. Discovery aborts on a failed VM inspection; CORE-012 adds partial unknown/denied results. |

Recommended allocation extension: a typed resource category (VRAM/compute/encode/
decode) with reusable available/unsupported/unknown capability results, provider
provenance/freshness, raw bounds/defaults, unit evidence and readback support. Keep
requested and observed triples separate; represent absent/default/partial values
explicitly, not as zero. Reuse `Allocation` validation where appropriate; add category
errors/provider constraints only when evidenced. No speculative vendor framework.

Consolidation preserves enrollment, artifact integrity, replay/admission checks,
ACL/reparse rules and bounded execution. SEC-001 independently reviews the changed
trust boundary before live deployment. CFG-001 distinguishes saving from successful
VM effects; save-only rechecks success/identity and stale file/provider state without
GPU mutation. CORE-028 retains recovery holds across restarts and opens the normal
dashboard with a persistent warning. GUI-003 defers normal close while work is active;
crashes/disconnects stay uncertain. No automatic GPU rollback, blind retry or second
IPC/locking/journaling stack.

## Native Windows boundaries

CFG-001 / OPEN-01's [selected configuration contract](CONFIGURATION.md) retains schema-2
TOML with one target per GUID-keyed file. `Configuration::read_vm_file` is a
read-only syntax/filename boundary; `View::from_vm_file` keeps saved intent
separate from observations/drafts. Neither authenticates protected storage or
grants enrollment. Synthetic files use the production parser, not a mock schema.
Live `State::refresh` still substitutes enrollment for configuration and the old
mock `Saved` representation remains transitional; GUI-002 must replace those
when binding saved configuration. `Configuration::vm_documents` prepares an
explicit schema-2 split without publication. Protected TOML publication/import and full-resource
extension are unimplemented; the new parser/fixtures are untested.

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
