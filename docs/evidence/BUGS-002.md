# GUI Bug Report – Main Window, GPU Enrollment & Apply Workflow

## Authorized acceptance attempt — 11 October 2026

Live testing is authorized for the configured disposable VM and, by subsequent
explicit instruction, the second enrolled VM now named `test-001`. Protected
discovery succeeded. Two explicit reconciliation attempts ended in Failed audits
and retained its existing `EffectsStarted` recovery record/pending journal. Fresh
inventory does not match the recorded enabled outcome: VM stopped, no GPU attached,
original settings. The update wrapper refused installation before effects. Further
live Apply/enrollment/memory editing/VM-power acceptance is blocked by supported
recovery; deleting records or replaying Apply is not a resolution.

| Issues | Actual result / remaining acceptance |
|---|---|
| BUG-01 / BUG-11 | Navigation stayed enabled with no false draft through two live polling intervals and page navigation. Sample toggle/slider reversal and discard passed; normal/dirty close passed in existing software qualification. Live edits after recovery remain pending. |
| BUG-02 | Independent review rejected the unqualified physical/provider ratio in committed source. Restored exact native-byte requests; physical memory remains independent inventory. The discovered provider maximum is 1,000,000,000. Physical-card-sized requests remain open until a supported mapping is established. |
| BUG-03 / BUG-04–08 | Source/software checks passed; narrow Review renders and expanded fixed footer inspected. Installed advanced defaults, actual enrollment wrapping, fresh live plan, credentials and Apply stages remain pending behind recovery/update. |
| BUG-09 / BUG-10 | Reproduced vague Unknown reconciliation failure. New live source now explains the unmatched interrupted outcome, retains the hold, and exposes the exact safe application component message in Technical Details. Arbitrary adapter messages remain withheld; other Apply/component failure paths remain pending. |

Exact safe application messages use a complete-match allowlist; added untrusted
text is never copied. This is bounded coverage of known application failures,
not a general structured adapter-diagnostic solution. Main/technical text and
Review now consistently describe native decimal-byte VRAM requests, following
DEC-034. Full provider-range editing and explicit-only default/draft behavior remain.

Initial workspace tests passed 169 tests/four ignored after correcting stale schema
expectations. Later reporting, GUI/memory and wire-compatibility tests passed; strict
Clippy and Windows x64/MSVC debug/MCP builds passed. Existing software layout/close
qualification and sample interactions passed. Host: Windows 26H2 build 26300.9457;
GPU inventory: RTX 5060, driver 32.0.16.1692. No guest GPU workload, allocation
enforcement or live power-transition pass is claimed. Evidence and screenshots:
`local/evidence/bugs-20261011/`; narrow renders: `local/evidence/gui-003-layout/`.
Owners: GUI-002/GUI-003/GPU-010; CORE-028 owns the partial-state recovery blocker.

## Source repair status — 10 October 2026

Implementation only; tests, builds, renders, GUI launches, installed updates and
live operations remain deferred. No bug is marked runtime-accepted.

- **BUG-01:** automatic polling no longer sets modal/running state. Backend work
  remains serialized; editing/navigation stay available. Inventory models update
  changed rows in place where their count is unchanged, preserving UI elements.
- **BUG-02 — partial / provider contract limit:** slider endpoints now use the
  selected GPU's full reported allocation range rather than the draft's narrower
  Minimum/Maximum. Explicit movement expands the draft bounds only as necessary.
  There is no hard-coded 1 GB cap. If Hyper-V reports a 1 GB maximum, it remains
  the validated request limit. Physical VRAM discovery and a qualified mapping
  to provider requests are still missing; DEC-034 forbids substituting physical
  card memory or multiplying by a card/provider ratio. This part remains open.
- **BUG-03:** valid reported defaults appear as labelled suggestions in omitted
  allocation fields. Editing an omitted group fills the remaining defaults;
  viewing it creates no intent. Suggestions follow GPU selection. Saved explicit
  values and incomplete user edits remain intact; unsupported defaults stay unknown.
- **BUG-04:** enrollment identifiers now live in Technical Details, the heading
  wraps, and the modal surface can shrink. Existing constrained body wrapping
  is retained. Resizing/render acceptance remains pending.
- **BUG-05–08:** enrollment explains permission and separate GPU approval; fresh
  planning explains its potentially lengthy VM/GPU/driver checks; credential
  text explicitly names the guest administrator account rather than the host;
  Apply progress uses actual stages and plain status labels, without estimates.
- **BUG-09:** Apply failure preserves the last reported step, classified reason,
  retained draft, uncertain outcome and Refresh/recovery guidance.
- **BUG-10 — partial / safe diagnostic contract:** Technical Details retains
  reported stages, error classification and available native hexadecimal codes.
  Arbitrary adapter error text remains omitted under `reporting.rs`'s existing
  credential-protection contract. Full component messages need structured safe
  diagnostics from the failing adapter; generic raw-string echoing is not a fix.
- **BUG-11:** presentation/model updates are guarded from edit/selection callbacks;
  slider callbacks ignore unchanged/programmatically presented positions. Existing
  baseline comparison, revert, discard and verified-Apply clearing remain in place.

Focused existing memory regressions were adjusted for range expansion; a native
error-code/secret-omission regression was authored. Neither was executed.

## 1. Main Window

**BUG-01: Entire interface becomes disabled every 10–15 seconds**

Approximately every 10–15 seconds, the entire interface briefly greys out, as though the application is refreshing or updating its state.

- Investigate the cause of this periodic UI disruption.
- Background refreshes must not disable or visually interrupt the interface.
- Keep controls responsive during routine status updates.
- Only disable individual controls when genuinely necessary.
- Avoid rebuilding or resetting the entire UI during routine refreshes.

**BUG-02: GPU memory slider limited to 1 GB**

The maximum VRAM allocation available through the slider is currently only 1 GB.

- The slider's maximum must reflect the actual VRAM capacity of the selected GPU.
- Update the maximum dynamically when a different GPU is selected.
- Ensure the displayed allocation and underlying configuration remain consistent.
- Do not use a fixed 1 GB maximum.

**BUG-03: Advanced Allocation fields missing default values**

The **Advanced Allocation** section does not automatically populate its input fields with the selected GPU's default allocation values.

- Populate the fields with the appropriate values for the selected GPU.
- Ensure the values update when switching GPUs.
- Keep the advanced fields and main allocation slider synchronised where applicable.
- Do not leave fields empty when valid defaults are available.

## 2. Review GPU Enrollment

**BUG-04: Text does not wrap correctly**

Text in the Review GPU Enrollment window extends beyond the available window width.

- Enable proper text wrapping.
- Ensure all content remains readable when resizing the window.
- Prevent text clipping and horizontal overflow.

**BUG-05: Technical information displayed in the main interface**

The Review GPU Enrollment window displays excessive technical information and uses language unsuitable for ordinary users.

- Rewrite the main text in clear, user-friendly language.
- Explain what GPU enrollment will do and what the user is approving.
- Move implementation details, backend information, and diagnostics into an expandable **Technical Details** section.
- Keep important warnings visible without exposing unnecessary technical complexity.

## 3. Building Fresh Plan

**BUG-06: Insufficient progress information**

The **Building Fresh Plan** stage can take some time, but the interface does not adequately explain this or indicate what the application is doing.

- Display a clear message explaining that preparing the changes may take some time.
- Show the current operation or stage where that information is available.
- Provide useful progress information without inventing percentages or progress estimates.
- Ensure the interface remains responsive during this process.

**BUG-07: Guest credentials prompt is unclear**

The application requests a username and password without adequately explaining why they are needed or which system they belong to.

- Clearly state that the credentials are for the **guest VM, not the Windows host**.
- Explain why the application needs them and what operation will use them.
- Make the distinction between host and guest credentials immediately obvious.
- Do not request credentials without explaining their purpose.

## 4. Apply Approved Changes

**BUG-08: Application messages are not user-friendly**

The **Apply Approved Changes** workflow displays messages that are too technical or unclear.

- Rewrite the messages in plain language.
- Clearly explain what changes are being applied.
- Display meaningful progress or operation stages where possible.
- Keep backend diagnostics and implementation details in Technical Details.
- Clearly distinguish between preparing, applying, completing, and failing an operation.

## 5. Operation Did Not Complete

**BUG-09: Failure reason is not explained**

When an operation fails, the application displays **Operation Did Not Complete** without explaining why.

- Display a meaningful, user-friendly error message describing the failure.
- Identify which stage failed, where possible.
- Explain whether any changes were applied or whether the outcome is uncertain.
- Do not imply that the VM is unchanged unless this has been confirmed.
- Provide an appropriate next action when one can be determined safely.

**BUG-10: Technical Details contains no useful failure diagnostics**

The **Technical Details** section does not provide meaningful diagnostic information about the failed operation.

- Include the actual error message returned by the failing component.
- Identify the failed operation or stage.
- Include relevant error codes and diagnostic context where available.
- Preserve useful backend error information instead of replacing it with generic failure text.
- Do not expose passwords, tokens, or other sensitive information in diagnostic output.

The main interface should explain the failure to the user, while Technical Details should provide enough information for troubleshooting.

## 6. Unapplied Draft

**BUG-11: False unapplied changes warning when quitting**

The application displays an **Unapplied Draft** warning every time I attempt to close it, even when I have made no changes.

- Investigate why the draft is incorrectly marked as modified.
- Only display the warning when genuine, unapplied configuration changes exist.
- Do not count automatic refreshes, initial configuration loading, or ordinary window navigation as user modifications.
- Ensure applying or discarding changes correctly clears the dirty state.
- Allow the application to close normally when no unapplied changes exist.

## 7. Implementation Instructions

Fix these issues as part of the existing GUI development work.

**Priorities:**

1. Fix the periodic interface greying-out and restore stable UI responsiveness.
2. Correct GPU VRAM detection, slider limits, and Advanced Allocation defaults.
3. Fix false unapplied draft detection.
4. Improve enrollment, planning, and apply workflow messages.
5. Provide meaningful failure explanations and useful technical diagnostics.
6. Correct text wrapping and layout issues.

**Development requirements:**

- Focus on functional fixes rather than additional test infrastructure.
- Do not introduce unnecessary refactoring or unrelated features.
- Keep the main interface simple and user-friendly.
- Keep technical details accessible but separate from normal user-facing content.
- Preserve existing VM safety checks, approvals, and recovery handling
