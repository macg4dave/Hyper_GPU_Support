# GUI guide

This page describes the agreed workflow from the user Q&A. [Baseline](BASELINE.md) records current implementation gaps. Keep the existing [Slint design](../src/gui/ui/app.slint) and wire behavior through the shared Rust core.

## Purpose and controls

Select a supported existing VM, toggle GPU support, and select GPU VRAM from 1 GB to the selected card's physical capacity. Hyper-V handles VM creation, system RAM, and general management. No automatic balancing or full-capacity warning policy is required now.

The existing Dashboard, System, Settings, About, and dialog design remain the foundation. Preserve stable selection, keyboard navigation, scrolling, themes, focus, and responsive progress.

APP-002 source now displays the 1 GB-to-card-capacity slider, with exact physical endpoints and separate driver-limit rejection. Unknown capacity or a range with no supported selection is unavailable. Existing requests below 1 GB remain labeled exactly; moving the slider is an explicit edit, not a rewrite on load. Advanced Allocation retains exact VRAM fields for compatibility and correction. Compute/Encode/Decode editors are removed from this GUI; saved requests are preserved and shown in review. Only the toggle disables support. Compilation, rendering, and real hardware acceptance remain pending.

## Review and Apply

Toggle/VRAM changes remain drafts. Apply opens a review panel naming the selected VM, showing what will change and relevant time warnings, and explaining every required start, shutdown, or restart. Show the expected final power state. A second Apply confirms; cancellation returns to editing.

Review must reflect a fresh backend plan. Changed target or inputs invalidate approval. Setup/enrollment authority remains distinct from GPU effects, even when presented as one understandable setup journey.

APP-003 source now provides first Apply → review → second Apply, with Cancel retaining the draft and clearing approval. Review shows ordered backend steps, changed compatibility values, final power, and relevant time warnings. It binds the original committed revision as well as the VM and draft; the protected worker independently rechecks backend state before effects. Initial setup now includes its final shutdown under APP-004; ordinary changes now skip guest work and finish off under APP-006. Source acceptance awaits APP-008 checks and APP-009 real interaction validation.

## First GPU setup

The application manages required VM power transitions, guest preparation, GPU configuration, and initial verification. Review explains the actual sequence before confirmation. Show a clear verification pass/fail for the selected VM. After completed setup and verification, gracefully shut the VM down and leave it off for the user to start through Hyper-V.

Before the local credential prompt, show a friendly explanation such as: “Setup needs the Windows sign-in for this VM. Use the account inside the VM, rather than the account for your host computer.” Include the actual VM name. Remembering credentials is optional and requires explicit opt-in.

APP-005 source now presents a blocking local Windows explanation panel before the native username/password dialog, naming the reviewed VM and explaining its guest administrator account. OK opens credential entry; Cancel stops the request before any guest operation starts. Remember begins unchecked and stores credentials only when explicitly selected, in this user's Windows Credential Manager for that VM. Existing stored credentials can be reused without opening either panel. Settings uses the same path and does not check guest connectivity. Cancel retains pending edits, clears an Apply/Verify review, and leaves any existing recovery hold intact. These dialogs and cancellation interactions await APP-008 checks and APP-009 live validation.

An interrupted or failed operation must not pretend setup completed or promise a final power state that has not been confirmed.

APP-004 source now identifies initial setup when completed preparation and graphics receipts are absent. Existing completed receipts remain valid historical setup evidence; an attachment or standalone graphics check alone does not establish preparation. Package setup/additive enrollment retain their existing separate reviews, followed by a fresh GPU effect review. Initial setup success requires graphics pass and confirmed final-off readback; its completion message names the VM and tells the user to start it through Hyper-V. A publication failure reports the guest setup result separately and offers save-only retry. Interrupted effects retain recovery with the reviewed final power; reconciliation does not repeat preparation/assignment/allocation. These source changes have not been compiled, rendered, or qualified on real VMs.

The first-time setup card now stages enabled support and opens the existing enrollment/setup review. It remains unavailable while eligibility or recovery blocks the selected VM; clicking it alone grants no effect authority. Real interaction is pending validation.

## Later changes

Enable/disable and VRAM changes follow review and Apply, without repeating full preparation or initial graphics verification. Retain necessary validation and readback, and show the actual result. Required shutdown/restart is managed by the application and disclosed in review. Successful ordinary changes always leave the VM off, whether it started running or stopped. The user starts it through Hyper-V.

If preparation becomes invalid or the signed driver payload changes, review explicitly identifies required setup/update, its preparation/graphics steps, credentials and time warnings before confirmation. Matching completed preparation/graphics receipts permit ordinary changes without repeating those steps. Ordinary success confirms host readback and final-off state; it does not claim a fresh rendering pass. Interrupted ordinary changes are reconciled from host readback without guest access or temporary boot; partial state retains recovery. APP-006 source is implemented; automated and live acceptance remain pending.

## State and failures

Keep saved settings, pending edits, actual observations, and the last initial verification result distinct. Preserve drafts through refresh conflicts and require resolution before effects. Busy/modal states prevent conflicting actions.

Progress names actual stages and explains delays plainly. Failed configuration publication offers bound save-only retry; uncertain effects require explicit reconciliation. Do not replay GPU work or clear recovery to make a status look successful.

Current-source review additionally blocks new editing/reviews when the selected journal requires recovery, even without a loaded save record. Failed controller state/recovery reads remain visible and disable execution instead of being replaced by a completion dialog. Installed worker availability is checked in background jobs; admission still performs fresh authentication. See the [review record](evidence/m1-source-review.md) for validation limits.

## Validation

Live launch can request elevation. Real GUI checks belong to the authorized APP-009 run, not a harmless visual launch. Legacy mock/snapshot tooling was removed from source under APP-001. Removal has not been compiled or visually validated; it is not a preview or testing path.
