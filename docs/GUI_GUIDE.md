# Hyper GPU Support v1.0 GUI specification

**Approved:** 9 October 2026. The completed Slint prototype is the authoritative
v1.0 GUI. Sources now live in [ui/](../ui/app.slint) and
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
Normal close is deferred while simulated progress runs.

All inventory, eligibility, enrollment, provider values, graphics results, Apply
stages, recovery, credentials and configuration saves remain simulated. Visible
demo/sample labels remain. The GUI performs no Hyper-V calls, privileged launch,
file saving or credential collection. The GPU Memory slider is a visual mock
preference independent of advanced allocations: its GB label is no allocation or
enforcement promise. Real mapping requires provider evidence.

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

