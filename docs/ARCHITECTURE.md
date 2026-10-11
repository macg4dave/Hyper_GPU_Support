# Architecture

This page owns component and data-flow boundaries. [Baseline](BASELINE.md) owns current implementation findings; [security](SECURITY.md) owns trust/recovery rules; [configuration](CONFIGURATION.md) owns formats and persistence.

## Components

| Layer | Responsibility | Source |
|---|---|---|
| Frontend dispatch | Select GUI, explicit mock/snapshot, CLI, or fixed internal modes | [main](../src/main.rs) |
| Slint presentation | Existing dashboard, VM details, System, Settings, About, dialogs, resource controls | [UI root](../src/gui/ui/app.slint), [GUI host](../src/gui/mod.rs) |
| Live controller/state | Asynchronous reads, draft/review state, progress, callbacks, model updates | [live](../src/gui/live.rs), [live state](../src/gui/live_state.rs), [GUI model](../src/gui_model.rs) |
| Shared product model/workflow | Typed intent/observations, planning, lifecycle, selected-target effect decisions | [model](../src/model.rs), [workflow](../src/workflow.rs) |
| Setup and enrollment | Fixed package bootstrap, additive reviewed pair authority, interrupted setup/pair records | [setup](../src/setup.rs), [enrollment](../src/enrollment.rs) |
| Restricted elevated worker | Bind/recompute approval, serialize effects, persist recovery and intent | [worker](../src/worker.rs) |
| Protected runner | Installed enrollment/artifacts, authenticated requests, audit, native backend | [runner](../src/runner.rs), [runner binary](../src/bin/hyper-gpu-runner.rs) |
| Native Windows adapters | Hyper-V WMI/COM, driver inventory, path/security/trust, local pipes/processes | [Hyper-V](../src/windows_hyperv.rs), [WMI](../src/windows_wmi.rs), [pipe](../src/windows_pipe.rs) |
| Guest preparation/verification | Authenticated payload, bounded PowerShell Direct bridge, fixed guest helper and checked D3D11 probe | [guest](../src/guest.rs), [guest helper](../src/bin/hyper-gpu-guest.rs), [transport](../src/guest_transport.ps1) |
| Protected persistence | Revision-checked per-VM intent, publication receipts, distinct import records | [store](../src/configuration_store.rs), [protected writer](../src/configuration_store/protected.rs) |
| Diagnostics/reporting | Bounded public failure context and opt-in protected tracing | [reporting](../src/reporting.rs), [diagnostics](../src/diagnostics.rs) |

The default application is `hyper-gpu-support.exe`. Setup currently expects that executable plus `hyper-gpu-runner.exe`, `hyper-gpu-guest.exe`, and `d3d11-probe.exe` in the same package directory. Internal setup/worker modes reuse the application executable; they are fixed protocol entrypoints, not general command execution. The installed runner is separate from the initiating user frontend.

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
5. Durable admission precedes effects. The shared workflow uses native host adapters and the guest bridge, then independently reads effective state and graphics results.
6. Successful effects and configuration publication remain separate phases. Publication failure leaves a bound save-only path; uncertainty leaves an explicit reconciliation path.
7. The frontend receives bounded progress/outcome information and refreshes independent state. A missing terminal message does not authorize replay.

## UI and platform constraints

Blocking discovery, payload work, and effects belong off the Slint event thread. UI updates return through event-loop callbacks; preserve stable selection, drafts, and editor focus when updating models. Installed status polling does inventory reads only. First-install elevation and snapshot rereads are not timer side effects.

Rust is the product implementation. PowerShell Direct remains the current guest transport bridge; replacing it requires its own scoped work. The deleted laboratory/reset tooling is not a product dependency. Non-Windows code paths do not establish Hyper-V support.
