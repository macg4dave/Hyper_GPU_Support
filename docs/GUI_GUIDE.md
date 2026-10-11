# GUI guide

This page owns the existing interface and workflow expectations. [Baseline](BASELINE.md) distinguishes source implementation from validation. The design in [Slint components](../src/gui/ui/app.slint) is retained; behavior work must reuse the Rust controller/core rather than redesign the application.

## Existing interface

The application has Dashboard, System, Settings, and About pages. Dashboard combines the VM list, selected-VM details, GPU enablement and selection, memory/resource controls, review actions, and status/recovery information. System presents discovered host/GPU details. Settings includes appearance, contextual help, and local credentials; fixture mode also includes scenario controls. Dialogs handle review, progress, dirty navigation, failures, and recovery.

Preserve selection by stable identity, meaningful keyboard navigation, scrollable long content, theme behavior, and focus during inventory updates. Busy/modal states must prevent conflicting actions while presenting useful progress. User-facing explanations should name the VM, proposed action, downtime, and outcome; low-level protocol details belong in expandable technical output.

## Modes

| Mode | Inputs | Effects |
|---|---|---|
| Live, no arguments | Current protected/native inventory, committed intent, recovery | Setup/credential/VM/GPU operations may occur only through their reviewed paths; launch can request first-install elevated discovery |
| `--mock-gui` | Explicit fixtures and scenarios | No backend effects or persistent writes; labels/results are simulated |
| `--mock-gui --snapshot FILE [--config GUID.toml]` | Bounded historical inventory and optional production-format candidate intent | No effects, credential access, publication, or recovery clearing; Refresh rereads files |

Snapshot input is capped at 8 MiB. Enabled shared-plan rehearsal needs a uniquely matching recorded payload digest from snapshot plans; it does not hash/discover the current host payload. Changed input bytes invalidate approval. A snapshot cannot establish current guest verification or physical VRAM provenance.

## Workflow contract

1. **Refresh and select:** show real inventory or a truthful read failure. Installed status polls every 15 seconds, skipping busy/modal work; it must not prompt for first-install elevation in the background. Select VM/GPU by identities, not display names.
2. **Setup/enrollment review:** for an uninstalled product or new pair, Review & Apply first reviews the fixed package/exact pair. Confirmation establishes authority only. Adding another VM preserves existing pairs. GPU effects require a subsequent review.
3. **Draft:** enable/disable and supported resource edits stay in memory. Show saved intent, last observation, and draft separately. Switching/closing a dirty draft asks whether to discard or keep editing. A refresh conflict preserves the draft and blocks stale approval until explicit discard/review.
4. **GPU review:** use the shared plan to explain changes, preparation, required guest downtime, power restoration, and credentials. Reapply/Update stages enabled intent and reviews a fresh plan; it is not an unconditional repeat command.
5. **Confirm:** use the fixed protected worker and local credential prompt where needed. Display bounded stages and real transfer progress. Cancellation or disconnect must not imply that effects were undone.
6. **Complete or recover:** refresh independent state. A successful effect with failed publication offers save-only retry; uncertain work offers explicit reconciliation. Inspection alone never clears recovery or establishes rendering.
7. **Verify:** review the separate health/rendering check and any temporary guest lifecycle. Successful standalone verification does not commit desired configuration.

## Resources and unavailable actions

The live memory slider uses the selected GPU's matched physical-capacity information and native VRAM byte requests; it does not reserve physical memory. Preserve invalid/out-of-range raw drafts rather than silently clamp or rescale them. Historical/unknown-capability paths use raw Advanced Allocation or explain why editing is blocked. Compute/Encode/Decode require supported provider capabilities and schema 3. See [configuration](CONFIGURATION.md).

GPU replacement and **Forget pairing** are not connected. Keep their unavailable state truthful. CLI `forget` deletes a stored credential and is unrelated to pairing removal. GUI import/export is deferred; pending import currently points to explicit CLI reconciliation.

## Appearance and interaction validation

The documentation reset did not render or launch the interface. V1-009 owns a later explicit mock preview, screenshots, keyboard/scroll/theme checks, and relevant milestone tests. V1-015 owns actual two-guest workflows. A fixture screenshot or mock success must never be labeled live GPU validation.
