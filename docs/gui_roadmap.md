# Native Windows GUI plan

**Reviewed:** 8 October 2026. [Product roadmap](ROADMAP.md) owns milestones;
[GUI-001](BACKLOG.md#gui-001) owns status/dependencies. This is a written design
specification for the native presentation and supported interactions.

## Reuse and actual prototype state

`src/windows_gui.rs` already uses Win32 through existing `windows` bindings:
window/message loop, sidebar/report table/native checkbox/buttons, runner discovery,
one background request, credential prompt/vault and configuration save.
Keep this foundation and `src/bin/hyper-gpu-gui.rs`; no toolkit replacement or
duplicate Hyper-V implementation.

It currently requires `--config FILE` plus separate runner installation/enrollment.
The 8 October foundation implements sidebar/header/five-column table/panel/footer,
DPI-scaled resizing and minimum sizing, independent one-VM draft and retained
selection. Apply fetches the shared preview; Reapply / Update deliberately stages
current driver preparation even for an enabled VM. Protected enrollment is read
back from the runner. Historical inventory blocks actions until refresh succeeds.
Busy requests disable conflicting controls; close waits for their final response.
Save failures retain completed intent without replaying VM effects. Verification
confirmation uses the core's pending recovery power intent.

Native release smoke passed navigation, reapply staging, refresh retention,
discard, resize and idle close without provider/journal/configuration changes.
The selected-VM checkbox and native Details dialog are initial controls; per-row
switches, in-window Details/GPU selection, styling, 150/200% DPI, accessibility and
full GUI operation/recovery acceptance remain. The hang investigation is closed
by user direction; live GUI acceptance follows M2's affected preparation gate.

## Written layout

Keep **Hyper GPU Support**, standard Windows title bar/window controls and native UI
font. Use light surfaces, blue accent, soft borders, modest corners and whitespace.
Style incrementally with native controls/custom drawing only where needed. Preserve
keyboard focus/accessibility in drawn controls; avoid decorative animation.

| Region | Target |
|---|---|
| Sidebar | Approximately 280 logical pixels at design size; **Virtual Machines**, **System Information**, **Settings**, **About**; line icons, pale blue active row and slim blue indicator |
| Header | **Virtual Machines**; **Manage GPU support and related settings for your Hyper-V virtual machines.**; outlined **Refresh** at upper right |
| Table | **Virtual Machine**, **Status**, **GPU Support**, **GPU Memory Allocation**, **Details**; light header/separators and blue selection |
| Row | Real bold VM name/muted subline, generic VM icon unless OS is known, labelled power state, switch, allocation text and outlined **View Details** |
| Information panel | Light blue/grey with information icon and **GPU Support** explanation |
| Footer | Bottom-right **Discard Changes** and blue **Apply Changes**; disabled without a draft |
| Details | Selected-VM native pane/dialog with GPU selection, identities, eligibility, preparation/verification and recovery; technical details on demand |

Use roughly 1536 × 1024 for proportions, not fixed required dimensions.
Set a usable minimum window size, responsive layout/scrolling for many VMs and
DPI-aware sizing. Status is never colour alone. OS window rounding follows Windows.

## Supported behavior

- List discovered VMs, including readable ineligible/unenrolled entries. Discovery
  does not establish guest OS; never infer OS support/icons from VM names.
- Power state, GPU attachment, preparation and graphics health are separate.
  **Enabled** describes observed selected assignment, not successful rendering.
- Stage one VM draft with **Pending**; retain effective baseline. Toggle clicks
  have no effects. Changing rows cannot silently discard edits.
- Allocation text is **Provider default**, **Provider values**, or **Unknown** from
  readback. Existing raw/external values are not defaults. No GiB/percentage slider
  until GPU-010 establishes meaning/enforcement.
- Apply obtains the shared fresh plan, explains preparation/settings/downtime/
  restoration, then uses the existing runner. Never hash the payload on dashboard refresh.
- GPU selection must respect the protected enrolled VM/GPU pair. Explain changing
  config/re-enrolling; combo selection cannot grant privilege.
- Reuse CLI credential/vault semantics, with explicit storage only. Short-lived
  credentials never enter persistent draft/config, reports, logs or arguments.
- Use indeterminate progress/real exposed stage. Runner currently returns final
  results, not streamed stages; no progress protocol is needed just for a bar.
- Refresh readback after success/failure. Timeout/lost response/pending journal
  means inspect/reconcile, never automatic rollback or false success.
- Apply success and config-save success are separate. Persist safely; failed save
  does not undo Hyper-V or justify blind replay.
- Disable conflicting controls while busy. Closing/cancelling the UI does not
  prove native mutation stopped; prefer delaying close until bounded work finishes.
  A disconnected reply requires reconciliation before retry.
- Preserve selection/draft on Refresh; invalidate preview if identities/state
  changed. Discard clears UI intent only.
- Keep one-VM-at-a-time staged Apply. Disclose unqualified sharing; any concurrent
  admission policy belongs to the core, not UI-only warnings.

## Details and secondary pages

**View Details:** real name/VM ID/generation/power, enrollment/eligibility, selected
versus attached GPU, effective provider allocation, pending changes/recovery,
recorded prepared digest and discovered driver, last successful graphics timestamp.
Details do not authenticate a current guest receipt or run a fresh graphics check.
Provide deliberate **Verify** through the existing operation with credential/downtime
preview when needed.

**System Information:** existing GPU/driver/prerequisite facts; unknown where absent.
Add host queries only for a concrete useful fact.

**Settings:** configuration/enrollment location and real credential actions first;
no invented theme/refresh preferences or second policy authority.

**About:** actual version/build, purpose and included notices/documentation.

## Small steps under GUI-001

G1–G6 are checklist labels, not new backlog IDs.

### G1 — Native layout (available now)

Reuse current window/message loop. Add sidebar/header/table/panel/footer and move GPU
selector to Details. Small presentation fixtures cover empty/unknown/many-row cases
in development only.

**Acceptance:** written layout recognizable, four pages navigate; usable minimum/
resized window, 100/150/200% DPI, keyboard focus and readable text. No GPU run needed.

### G2 — Truthful read-only binding (CORE-012 integration)

Reuse runner discovery/journals/observed state. Preserve selection, show eligibility/
enrollment and detail fields; consume partial/denied results when core supports them.

**Acceptance:** UI matches CLI; unknown/denied never becomes Disabled or Healthy.
Refresh has no guest effects/full manifest hashing. Missing runner gives setup
guidance; ordinary launch is not elevated.

### G3 — Draft and fresh preview (CORE-006)

Add baseline/draft/dirty actions, stale-preview handling and shared effect summary.
Restrict GPU selection to enrollment; explain re-enrollment.

**Acceptance:** toggle/Discard/Refresh are effect-free; edits survive selection
safely; enable/disable/running-no-op plans match behavior. Apply unavailable for
invalid, unsupported or unenrolled pairs.

### G4 — Complete operation integration

Reuse background request/runner/credential UI. Handle channel disconnect, duplicates,
busy controls, close, vault reuse, safe config persistence and post-result readback.

**Acceptance:** responsive UI, redacted credentials, save failure distinguished from
apply failure, uncertain response preserved; focused checks of these failure paths.
No new GUI backend or unsupported cancellation claim.

### G5 — Setup and secondary pages (CORE-021)

Expose schema-2 selection/config and administrator enrollment instructions; add a
narrow guided route if needed for usability. Reuse `runner::install` with UAC and
fixed typed validation. GPU pair changes require re-enrollment.

**Acceptance:** packaged instructions or guided flow get an operator to enrolled VM
controls without lab paths/arbitrary elevated commands. UAC is for setup, not drawing
the dashboard. Secondary pages contain real supported facts/actions.

### G6 — Qualified journey and release

After M2 clearance, exercise select/preview/enable/verify/reapply/disable and one
representative failure using candidate artifacts. Check tab order, accessible
labels, High Contrast, scaling/resize and error readability.

**Acceptance:** CLI-equivalent readback; running no-op retains uptime; disable keeps
prepared files/restores attributable state; no false green result. Ships with CLI,
runner, guest worker and D3D11 probe under CORE-017/GPU-014.

## Dependencies and proportional checks

G1/G2 foundations and CORE-006 preview are available. G4 can use
hardware-free checks before live M2 clearance. G5 reuses completed M1 enrollment.
G6 closes M3 and feeds R1; M4 allocation/GPU-015 sharing stay separate.

Review materially changed privilege/credential boundaries independently. Routine
styling needs native usability checks, not an architecture review per control.
No pixel-test framework, stress campaign, scheduler, driver service or historical
lab migration is a prerequisite.
