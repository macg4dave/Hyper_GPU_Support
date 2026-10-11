# GUI Bug Report – Review & Apply / Main Window

## 1. Review & Apply Window

**BUG-01: Incorrect application status text**

The window currently displays:

"Live application information:
Waiting for the shared backend. No VM or guest changes are requested by this"

This is misleading. The purpose of **Review & Apply** is to review and confirm changes to the VM.

Replace this with clear, user-friendly information describing exactly what will change.

For example:
- **GPU Memory:** 4 GB → 8 GB
- **GPU Support:** Disabled → Enabled

The user should immediately understand what they are approving.

**BUG-02: Technical details clutter the main view**

The Review & Apply window currently displays excessive technical information.

- Keep the main view simple and user-friendly.
- Show a clear summary of the proposed changes.
- Move backend information, debugging output, and technical details into a separate, expandable **Technical Details** section.
- Display warnings prominently only when they require user attention.

**BUG-03: Important controls scroll out of view**

The following controls must remain visible regardless of scrolling:

- **Approve graceful guest shutdown and restoration** checkbox.
- **Apply Changes** button (or equivalent confirmation control).

Place these controls in a fixed footer so they cannot scroll out of view.

## 2. Main Window

**BUG-04: GPU Support section contains excessive diagnostic text**

The text beneath **GPU Support** contains unnecessary debugging information and warnings.

- Replace this with a clean, concise status message.
- Show only information relevant to the user.
- Move detailed diagnostics into a separate Technical Details section.
- Keep genuinely important warnings visible.

**BUG-05: Enable GPU switch becomes disabled unexpectedly**

The **Enable GPU** switch becomes greyed out after interacting with other windows.

- Investigate why unrelated UI interactions disable the switch.
- The switch should remain functional unless there is a genuine reason to disable it.
- Disabled controls should have a clear explanation.

**BUG-06: GPU memory slider does not work**

The GPU memory allocation slider is not functioning correctly.

- Restore slider functionality.
- Ensure changes update the displayed memory allocation.
- Ensure the selected value is retained in the draft configuration.
- Ensure Review & Apply accurately reflects the selected value.

**BUG-07: Remove internal development terminology**

Remove the following user-facing text throughout the GUI:

- "Live GUI"
- "Live workspace"
- "Actual inventory"
- "Test backend" (currently displayed as "Tusdt backend")

Replace these with appropriate user-friendly descriptions where necessary. Do not expose internal architecture or development terminology in the normal interface.

**BUG-08: Incorrect unapplied draft indicator**

The GUI displays an **Unapplied Draft** message even when **Review & Apply** is disabled.

- Investigate the draft-state handling.
- Only indicate unapplied changes when genuine configuration changes are pending.
- Ensure the Review & Apply button and draft status remain consistent.
- Do not treat opening or navigating windows as configuration changes.

**BUG-09: VM running/stopped status does not update**

The displayed VM power state does not update correctly.

- Ensure the GUI reflects the VM's actual running or stopped state.
- Refresh the status after VM power-state changes.
- Ensure the displayed state is not stale or incorrectly cached.
- Avoid requiring the user to restart the application to see an updated state.

## 3. Implementation Instructions

Address these issues as functional GUI fixes, not simply cosmetic changes.

Priorities:
1. Make the main window usable, with functioning controls and accurate VM state.
2. Fix draft tracking and Review & Apply behaviour.
3. Simplify user-facing language and remove development terminology.
4. Improve the Review & Apply layout and keep confirmation controls visible.

**Important:**
- Focus on implementing the fixes rather than expanding the test suite.
- Do not introduce additional diagnostic text into the main interface.
- Do not refactor unrelated components unless necessary to fix these bugs.
- Preserve existing safety checks and confirmation requirements for VM modifications.
- Do not perform extensive testing unless explicitly requested or required to establish that a major milestone is complete.
- Update the relevant backlog entries as the work progresses.
- Do not mark an issue complete merely because code has changed. Its expected GUI behaviour must be implemented.

The objective is a clean, functional, user-friendly GUI rather than an interface exposing backend implementation details.

## 4. Repair status — 10 October 2026

Source implementation is recorded separately from GUI acceptance. No tests,
builds, renders, UI launches or live operations were run for this repair.

| Issue | Source result | Acceptance status |
|---|---|---|
| BUG-01 | Review now summarizes fresh observed → planned GPU support/resource values, selected GPU and ordered VM steps; planner output remains expandable. | Pending GUI acceptance |
| BUG-02 | Review technical output is separate from its summary and expands inside the scroll area; actionable downtime/setup warnings remain visible. | Pending GUI acceptance |
| BUG-03 | Shutdown consent and approval are outside the scrolling content, including expanded technical output. | Pending layout/render acceptance |
| BUG-04 | Main GPU support shows concise attachment/saved/verification status; detailed observations and inventory diagnostics are expandable. | Pending GUI acceptance |
| BUG-05 | Replaced imperative modal-state updates with a reactive binding to dialog/operation state. Eligibility/recovery checks and their explanations remain. | Reported interaction needs reproduction/acceptance; root cause not established by a live check |
| BUG-06 | Subsequent user request authorized source mapping. Microsoft's documented VRAM byte contract now maps decimal GB/0.1 GB ticks to the existing Optimal draft field; Minimum/Maximum are retained (provider bounds initialize an omitted triple on the first edit). Advanced fields and Review share exact bytes; invalid/stale/historical/missing/precision-incompatible bounds remain unavailable. Sample preference stays explicitly simulated. See DEC-034. | Source implemented; builds/tests/render/provider/guest acceptance pending; not completed |
| BUG-07 | Removed the listed development labels from normal UI; explicitly identified demonstrations/snapshots remain distinguishable. Corrected obsolete About availability text. | Pending GUI acceptance |
| BUG-08 | Live and sample edits compare against their baseline; reverting values clears dirty state. Invalid changed text remains a draft with an explanation; Review requires changed, valid, eligible intent. | Pending GUI acceptance |
| BUG-09 | Installed inventory polls every 15 seconds when dialogs/operations are idle; manual and enrollment refreshes share presentation. Failed reads mark retained state historical. Snapshot/first-install paths do not poll. | Pending actual VM power-state acceptance |

Tracking owners: GUI-002 / GUI-003, with GPU-010.3 owning BUG-06 source mapping and pending acceptance.
No issue is marked completed solely from source changes.

## 5. Authorized build and installed update — 10 October 2026

- Fixed the Slint duplicate modal property binding found by the build; computed
  dialog/operation availability now synchronizes the shared modal flag.
- `cargo build --locked --all-features --bins --jobs 2` succeeded: Rust 1.94.0,
  Windows x64/MSVC debug, configured static CRT, all product binaries.
- Native Rust installation through the existing development elevation wrapper
  succeeded. All four source/installed/policy SHA-256 hashes match; the original
  operator and one enrolled pair were preserved.
- Evidence: `local/gui-update/0852293a78f94462a5a03e3ab3462074/result.json` and
  installer stdout/stderr in that directory. Installed GUI:
  `C:\Program Files\HyperGpuSupportProduct\hyper-gpu-support.exe`.
- Tests, GUI launch, control/layout acceptance and provider/guest behavior were
  not exercised. The per-issue acceptance statuses above remain pending.

## 6. Authorized acceptance attempt — 11 October 2026

The user authorized live testing, then separately included the second enrolled
VM (now `test-001`) for reconciliation/testing. Current source was built and
launched with the Slint development MCP feature. Installed discovery succeeded;
navigation stayed enabled and no false draft appeared across two automatic
polling intervals. Page navigation created no draft. The existing sample UI
passed toggle/slider reversal, label updates, discard and expanded Review footer
checks. Existing software layout/close qualification passed; narrow Review renders
at 100% and 200% were inspected with consent and confirmation visible.

These are scoped checks for BUG-03/07/08 and related sample interaction; they do
not accept the live Apply, memory editing or VM power-transition requirements.
BUG-05/06/09 and the live Review workflow remain pending. The protected installation
update was refused because `test-001` retains an `EffectsStarted` recovery record
and pending journal. Fresh inventory shows stopped/no attachment, differing from
the recorded enabled outcome. Two explicit reconciliation readbacks failed with
terminal audits; the hold remains. The second check verified the improved partial
state explanation and safe exact component message. No new Apply was requested.

Independent review rejected the current source's proportional card/provider VRAM
conversion. Restored DEC-034's exact native-byte requests and kept physical capacity
as separate inventory. Full provider-range editing still expands draft bounds only
on explicit selection. The reported 1 GB provider limit is not expanded using an
unqualified ratio; BUGS-002 BUG-02 remains open.

Validation: initial workspace suite passed 169 tests with four ignored after fixing
an outdated schema-3 rejection test. Later reporting (6), GUI/memory (40, one ignored),
worker/wire (2), and opt-in software layout (1) checks passed. Strict Clippy and
Windows x64/MSVC debug product/MCP builds passed. No guest rendering/allocation or
power-transition acceptance occurred. Evidence: `local/evidence/bugs-20261011/`,
`local/evidence/gui-003-layout/`, and blocked update
`local/gui-update/249de7859bf541f4bd17422d3ae9e92e/result.json`.
Tracking remains with GUI-002/GUI-003/GPU-010 and CORE-028 recovery; no M3 closure.
