# Hyper GPU Support v1.0 GUI specification

**Approved:** 9 October 2026. The completed Slint prototype is the authoritative
v1.0 GUI. Sources now live in [src/gui/ui/](../src/gui/ui/app.slint) and
[src/gui/](../src/gui/mod.rs). Preserve layout, navigation, styling, controls and
interactions. Backend integration may make existing controls functional; it must
not expand the interface without user approval.

## Approved interface

- **Dashboard:** VM cards, OS icons, state dots, selection highlight, ellipsized
  names and full-name tooltips; heading and Refresh in one row. Adjustable split,
  independent vertical scrolling and horizontal overflow at narrow widths.
- **Details:** observed attachment, saved desired state, graphics status; staged
  GPU switch, physical GPU selector, GPU Memory slider, expandable Advanced
  Allocation (VRAM/Compute/Encode/Decode Minimum/Optimal/Maximum), setup guidance,
  review/apply, discard, VM actions and technical details.
- **System:** read-only prerequisites and GPU capabilities.
- **Settings:** Follow Windows/Light/Dark, contextual help, mock scenarios/reset
  and credential information. Preferences currently last for the session.
- **About:** application information and shortcuts.
- **Dialogs:** dirty-VM switch, draft review, downtime consent, progress,
  verification, forget pairing, errors, recovery/reconciliation, save-only retry
  and dirty-close confirmation. Technical output remains selectable black text
  on white in both themes, with bounded scrolling and fixed-size disclosure.

There is no Activity page, VM search/filter control, separate setup wizard,
host GPU editor or diagnostic history. Exact source components determine the
details. Do not restore removed features or require Win32 parity.

## Implemented and simulated behavior

No arguments to `hyper-gpu-support.exe` open Slint. Explicit CLI commands remain
independent of GUI initialization. Navigation, themes, resizing, splitter, scrolling,
draft editing/validation, selection prompts and dialogs are real UI behavior.
Page navigation preserves the draft; switching VM or closing with edits prompts.
Normal close is deferred while simulated progress runs in mock mode.

**Normal startup is live mode.** It reads actual VM/GPU inventory in the background
through the shared core; Refresh repeats that read. Errors never fall back to
fixtures. Attachment does not imply enrollment, guest health or saved intent.
Unconnected configuration actions are unavailable, not simulated. The current
live slice does not yet apply, verify, enroll or save from the GUI; the CLI retains
its existing capabilities independently.

Enrolled pairs support in-memory toggles/raw VRAM drafts and fresh shared plan
previews; the native credential dialog can store an opt-in credential. Live
details distinguish unread, absent and previous preparation records from pending
recovery. Graphics timestamps are historical records for the enrolled pair,
never current health or current-driver parity. A failed Refresh marks retained
observations historical; mismatched durable records require inspection.

**`--mock-gui` explicitly selects rehearsal.** It currently uses deterministic
fixtures and simulates controls/review/stages/recovery in memory. No real effects
or persistent writes occur. Scenario/reset controls appear only in this mode.
The GPU Memory slider remains illustrative, independent of advanced allocation;
its GB label promises no real allocation or enforcement.

**`--mock-gui --snapshot FILE`** selects an existing JSON capture from CLI
`inventory`, instead of fixtures. It uses the existing real-data cards/System
view, labels observations historical and only rereads that file on Refresh.
Missing managed/enrollment data remains unknown; it is never inferred. Snapshot
contents cannot authorize live plans, credentials, verification or execution.
Editing and plan/stage rehearsal are not connected in this slice. The reader is
bounded to 8 MiB; invalid input reports an error without a fixture fallback.
This implementation remains untested/unqualified pending M3 validation.

## Mock mode development direction

As real bindings are added, mock mode should read the same actual inventory,
capabilities and existing configuration as live mode, validate the same draft and
show the same proposed plan, then rehearse stages without executing effects.
Reads are real; execution outcomes are explicitly simulated, never presented as
successful host/guest operations. Saving, forget, credentials and recovery clearance
remain session-only. Never modify Hyper-V, guest power/files/registry, GPU/driver
state, enrollment, configuration, vault entries, journals or audit files. Guest
verification that launches probes is an effect, not a read-only rehearsal.
Retain fixture scenarios for isolated/failure UI tests, not automatic fallback
after failed real discovery. No extra pages or second backend implementation.

**Current limitation:** ordinary-token live discovery uses the protected runner,
which writes mandatory security audit records even for reads. Mock mode therefore
does not call it. The optional snapshot reader needs no-write qualification;
authorized native discovery and recorded-plan rehearsal remain future work. Never suppress
required security audit to satisfy rehearsal mode.

## Backend integration

Reuse the shared Rust core, sound `gui_model`, workflow/planner, configuration,
credentials, protected runner, Named Pipe, lock and journals. Preserve CLI
capabilities absent from the GUI. No second GPU or security system in Slint.

Use stable identities and lightweight discovery/status for Refresh, full fresh
planning for Review & Apply. Report missing/denied/unsupported/unknown truthfully;
attachment does not prove graphics health. Validate provider units/bounds/readback
before using real values. Editing changes only an in-memory draft.

Protected effects require validated enrollment, fresh approved scope and separate
elevation, guest credentials and downtime consent where needed. Preserve audit,
security and uncertain recovery records. Real stages replace the demonstration
timer. Failed saving retries saving only; no GPU replay, automatic rollback or
blind retry. No host lifecycle action.

## Remaining work and retired requirements

[BACKLOG](BACKLOG.md) owns task status; [GUI_ROADMAP](GUI_ROADMAP.md) maps integration.
Bind discovery/status, supported selection/allocation, reviewed execution/progress/
recovery, persistence and credentials through existing controls. Then qualify
packaging, keyboard/DPI/accessibility and software rendering.

DEC-029's D01–D18/A18–A32 and former OPEN items are historical planning references.
GUI requirements absent from the completed prototype are retired from v1.0:
search/filters, extra pages/actions, persistent preferences and session activation.
Backend security/persistence plans stay on existing cards without authorizing
extra controls or weakening CLI behavior. Same-executable worker consolidation
is still planned and requires independent boundary review.
