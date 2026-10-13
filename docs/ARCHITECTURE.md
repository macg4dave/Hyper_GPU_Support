# Architecture

This page owns component and data-flow boundaries. The [roadmap](roadmap.md) and [backlog](backlog.md) record current delivery status; [security](SECURITY.md) owns trust/recovery rules; [configuration](CONFIGURATION.md) owns formats and persistence.

## Components

| Layer | Responsibility | Source |
|---|---|---|
| Frontend dispatch | Select the live GUI for argument-free startup, CLI for explicit commands, or fixed internal modes; retired simulation entrypoints removed under APP-001 | [main](../src/main.rs) |
| Slint presentation | Existing dashboard, VM details, System, Settings, About, dialogs, resource controls | [UI root](../src/gui/ui/app.slint), [GUI host](../src/gui/mod.rs) |
| Live controller/state | Asynchronous reads, draft/review state, operation callbacks | [live](../src/gui/live.rs), [live state](../src/gui/live_state.rs), [GUI model](../src/gui_model.rs) |
| Live presentation adapters | Slint model updates, selected-VM presentation and editor/resource properties | [presentation](../src/gui/live/presentation.rs) |
| Live source adapter | Protected discovery, planning, configuration/recovery reads and worker availability | [source](../src/gui/live/source.rs) |
| Live progress presentation | Ordered worker stages and in-place transfer counter updates | [progress](../src/gui/live/progress.rs) |
| Shared product model/workflow | Typed intent/observations, planning, lifecycle, selected-target effect decisions | [model](../src/model.rs), [workflow](../src/workflow.rs) |
| Setup and enrollment | Fixed package bootstrap, additive reviewed pair authority, interrupted setup/pair records | [setup](../src/setup.rs), [enrollment](../src/enrollment.rs) |
| Restricted elevated worker | Bind/recompute approval, serialize effects, persist recovery and intent | [worker](../src/worker.rs) |
| Protected runner | Installed enrollment/artifacts, authenticated requests, audit, native backend | [runner](../src/runner.rs), [runner binary](../src/bin/hyper-gpu-runner.rs) |
| Native Windows adapters | Hyper-V WMI/COM, driver inventory, path/security/trust, local pipes/processes | [Hyper-V](../src/windows_hyperv.rs), [WMI](../src/windows_wmi.rs), [pipe](../src/windows_pipe.rs) |
| Guest preparation/verification | Authenticated payload, bounded PowerShell Direct bridge, fixed guest helper and checked D3D11 probe | [guest](../src/guest.rs), [guest helper](../src/bin/hyper-gpu-guest.rs), [transport](../src/guest_transport.ps1) |
| Protected persistence | Revision-checked per-VM intent, publication receipts, distinct import records | [store](../src/configuration_store.rs), [protected writer](../src/configuration_store/protected.rs) |
| Diagnostics/reporting | Bounded public failure context and opt-in protected tracing | [reporting](../src/reporting.rs), [diagnostics](../src/diagnostics.rs) |

The default application is `hyper-gpu-support.exe`. Setup currently expects that executable plus `hyper-gpu-runner.exe`, `hyper-gpu-guest.exe`, and `d3d11-probe.exe` in the same package directory. Internal setup/worker modes reuse the application executable; they are fixed protocol entrypoints, not general command execution. The installed runner is separate from the initiating user frontend.

The first live-controller extraction separates presentation, protected source reads/planning, and progress into private child modules. `live.rs` still owns background scheduling, refresh completion, operation flows and callback registration; `live_state.rs` retains draft/review/conflict and completion-state decisions. The source adapter reuses the existing runner/worker contracts, and only event-loop callbacks update Slint models. Existing row/progress tests moved with their helpers. This structural pass is source implemented; compilation, test execution and runtime acceptance remain pending the agreed validation checkpoint.

The GUI keeps a VM-bound [result notice](../src/gui/result_notice.rs) during a completion status retry. It contains presentation text only; no reviewed effect authority is retained. Refresh invalidates effect reviews before restoring a result's continuation. Closing or starting a new review drops the notice.

The fixed guest executable streams bounded `HGPREPARATION` phase labels at actual copied-file check, installation, installed-file check, current guest-file/receipt check and graphics boundaries. The authenticated transport relays those fixed events separately from the final JSON reply. The host [transfer monitor](../src/guest_transfer.rs) validates operation-specific ordering, bounds the phase queue and requires all expected phase boundaries for a successful worker reply. Coalesced byte acknowledgements remain independent. Phase delivery cannot certify success; worker exit, receipt/file readback, graphics and workflow lifecycle/publication checks remain authoritative.


## Workflow module family

[Workflow root](../src/workflow.rs) retains the shared `Backend` trait and `OperationResult`, with explicit re-exports preserving public and crate-visible caller paths. Its private children own the operation stages:

| Module | Responsibility |
|---|---|
| [Journal](../src/workflow/journal.rs) | Durable state, setup eligibility, check persistence/readback and historical success timestamps |
| [Planning](../src/workflow/planning.rs) | Read-only decision gathering, reviewed effects and per-setting restoration calculation |
| [Apply](../src/workflow/apply.rs) | Consumed admission and ordered setup/update, ordinary change, reinstall and removal effects |
| [Verification](../src/workflow/verification.rs) | Explicit Verify admission, current payload checks and operation-specific restoration guards |
| [Reconciliation](../src/workflow/reconciliation.rs) | Independently prove achieved outcomes and recorded lifecycle without replaying Apply |
| [Guest checks](../src/workflow/guest_check.rs) | Shared check classification, restored-negative evidence and graceful power restoration/readback |

Enabled Apply writes compatibility settings only when admitted initial settings differ from the reviewed target. Matching settings still acquire/retain the recorded ownership baseline, and final independent readback remains mandatory (APP-029).

Operation modules depend on the shared contracts/helpers; Verify and reconciliation do not depend on each other's execution. Admission-token fields remain private to their operation modules; planning internals and shared primitives are visible only within the workflow family. The worker still owns authority, authentication, locks, protected receipts, publication and audit. Backend adapters and transport remain outside this family.

Focused tests live inline with their owning module. [Shared doubles/fixtures](../src/workflow/test_support.rs) and [cross-operation scenarios](../src/workflow/tests.rs) are compiled only under `#[cfg(test)]`. Current automated checkpoints and their exclusions are recorded in backlog clearance evidence (historical record; page retired).

## State separation

| State | Meaning | Does not establish |
|---|---|---|
| Observed inventory | Last read VM/GPU/provider state | Saved intent or verified rendering |
| Saved intent | Valid production TOML committed for one VM | Current attachment or operation authority |
| Draft | In-memory requested changes | Enrollment, publication, or effects |
| Enrollment | Protected operator and exact VM/GPU authority | Desired enablement or guest health |
| Plan | Reviewed proposed actions from current inputs | Permission to reuse stale inputs or proof of completion |
| Journal / recovery receipt | Durable ownership and operation progress | Success merely because the frontend exited |
| Verification result | Checked guest device/rendering outcome for a recorded context | Permanent health, concurrent sharing, or resource isolation |

Stable VM GUIDs and exact GPU interfaces identify targets. Display names are presentation only. Sharing means several VM targets can reference one GPU; each VM has one selected GPU. The current global operation lock serializes protected operations; concurrent guest workloads do not require concurrent mutation workers.

## Operation flow

1. Discovery provides inventory and protected authority/managed state. The frontend reads committed intent separately and preserves drafts where appropriate.
2. A missing installation/pair leads to separate package/enrollment review. Setup/enrollment establishes authority without applying GPU intent.
3. Review asks the shared planner for selected-target actions and required downtime/credentials. This can authenticate/hash driver payloads and should not be treated as a cheap polling operation.
4. Confirmation submits the reviewed operation to a fixed elevated worker. The worker authenticates the session, acquires the lock, validates current enrollment/revisions, and recomputes approved facts.
5. Durable admission precedes effects. The shared workflow uses native host adapters and the guest bridge, then independently reads effective state. Initial setup includes graphics verification and confirmed final shutdown under APP-004. Ordinary enabled changes still repeat graphics checks until APP-006 separates them.
6. Successful effects and configuration publication remain separate phases. Publication failure leaves a bound save-only path; uncertainty leaves an explicit reconciliation path.
7. The frontend receives bounded progress/outcome information and refreshes independent state. A missing terminal message does not authorize replay.

## UI and platform constraints

Blocking discovery, payload work, and effects belong off the Slint event thread. UI updates return through event-loop callbacks; preserve stable selection, drafts, and editor focus when updating models. Installed status polling does inventory reads only. First-install elevation is not a timer side effect.

Rust is the product implementation. PowerShell Direct remains the current guest transport bridge; replacing it requires its own scoped work. The fixed transfer stream now uses a Rust-owned 4 MiB chunk limit injected into the bridge and shared with acknowledgement validation. Partial chunks send only the bytes read. Ordering, exact totals, deadlines and independent preparation verification remain separate from transfer acknowledgements. APP-013 runtime throughput and process-memory acceptance are pending. The deleted laboratory/reset tooling is not a product dependency. Non-Windows code paths do not establish Hyper-V support.
