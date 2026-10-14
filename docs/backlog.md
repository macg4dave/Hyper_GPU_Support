# QA backlog

Track confirmed reports and remaining qualification work against the [roadmap](roadmap.md). BUG IDs retain the identities in the [case index](../qa/INDEX.md); an implemented patch or automated pass does not close a bug. The source batch below completes the remaining confirmed fix; it does not authorise installation or live effects.

## Next action

Prepare a scoped BUG-0001 installed Verify/recovery retest, with explicit target/effect authorisation, local guest credentials, evidence location and stop conditions. Preserve the retained VM1 recovery hold until that sequence is authorised. Then retest BUG-0002 and BUG-0003 through the normal GUI and complete the remaining BUG-0004/BUG-0005 scenarios. The source batch passed the non-hardware gate; prepare installation and GUI retests as the next checkpoint.

## Bug register

| Case | Severity | Symptom | Implementation | Validation / remaining work |
|---|---|---|---|---|
| BUG-0001 | P1 | Verify ends Unknown and leaves a recovery hold | Done: reported source patches; original live cause remains unconfirmed | Current non-hardware gate passed; installed Verify/recovery retest pending; release blocker |
| BUG-0002 | P2 | Ordinary VRAM change review is slow and preparation progress/cancellation is unclear | Done: preliminary review, cancellation admission and nested read progress patches reported | Current non-hardware gate passed; installed timing, progress and cancellation retests pending |
| BUG-0003 | P3 | Memory-only draft wording suggests GPU support will be enabled/disabled | Done: baseline-aware summary and regression coverage reported | Current non-hardware gate passed; installed editing/review retest pending |
| BUG-0004 | P3 | Non-first VM selection is lost after reopening | Done: persistence/GUID normalisation and regression coverage reported | One independent installed reopen scenario passed; remaining scenarios and user acceptance pending |
| BUG-0005 | P2 | Recovery wording identifies the wrong VM when another VM is selected | Done: recovery identity wording and regression coverage reported | Independent read-only recovery review passed; remaining scenarios and user acceptance pending |
| BUG-0006 | P3 | GB-labelled memory slider exposes 60 while visible request is 6 GB | Done: GB accessibility metadata/actions and disabled-edit guards | Current non-hardware gate and compiled headless regression passed; not installed |

### BUG-0001 — Verify Unknown and retained recovery

Evidence: [existing case index and source follow-up](../qa/INDEX.md#intake-notes); the [independent session](../qa/inbox/2026-10-14-independent-results.md) still observed the hold and did not execute Verify/reconciliation (R09).

Expected: Verify reports a truthful result with useful failure facts and a supported recovery route. Uncertain outcomes preserve evidence and the hold. Source follow-ups report typed guest/device failures, separate power-restoration diagnostics and recovery regressions; they do not establish the original live cause.

Remaining: on an authorised prepared target, exercise Verify and supported recovery, inspect the final result/details, guest health, final power state and persistence after reopen. Record failures without replaying effects or clearing uncertainty to manufacture a pass. Keep P1 open until candidate-specific evidence and user acceptance exist.

### BUG-0002 — Slow VRAM review and preparation feedback

Evidence: [case index source follow-ups](../qa/INDEX.md#intake-notes). Independent session R08 was blocked by the retained recovery hold; no timing result exists.

Expected: ordinary-change review presents actual changes promptly, preparation exposes useful progress, and Cancel does not permit late results to reopen review or start effects. Source patches report preliminary read-only review, payload authentication at admission, nested read progress and retained busy state while cancelled reads finish.

Remaining: time review, admission and final checks separately for a configured stopped VM's VRAM change. Cancel during preparation, reopen review, observe responsiveness and verify the final saved/readback state after an authorised Apply. Record measured waits and any stale results; do not infer a performance pass from unit tests.

### BUG-0003 — Misleading memory-only draft wording

Evidence: [case index source follow-up](../qa/INDEX.md#intake-notes). The original draft-wording report is deleted in the current working tree; this task preserves that deletion. Independent session R08 could not exercise editing.

Expected: a memory-only edit says GPU support stays enabled/disabled; only an actual toggle change says enable/disable. Source follow-up reports a shared saved-state baseline and scenario regressions.

Remaining: retest Set request and Advanced Allocation against saved enabled and disabled support. Refresh with a retained draft, toggle away/back and discard. Confirm the draft and review describe only actual changes, including provider-default and incomplete/invalid input paths.

### BUG-0004 — VM selection after reopen

Evidence: [source follow-up](../qa/INDEX.md#intake-notes) and [independent session R06](../qa/inbox/2026-10-14-independent-results.md). Selecting non-first VM3, gracefully closing and reopening retained VM3 with the VM1 hold intact.

Expected: graceful reopen restores a still-present selected VM by identity; missing identities fall back safely without changing recovery state.

Remaining: repeat without a recovery hold on a prepared target; cover missing/renamed/reordered VMs and verify saved settings and warnings. The single observed pass is bounded evidence, not user acceptance or case closure.

### BUG-0005 — Recovery identity on an unrelated VM

Evidence: [source follow-up](../qa/INDEX.md#intake-notes) and [independent session R02](../qa/inbox/2026-10-14-independent-results.md). With VM2 selected, banner, panel and read-only dialog correctly identified VM1 and GPU verification; closing retained the hold.

Expected: recovery names the affected VM and operation consistently, including when an unrelated unconfigured VM is selected. Host-wide blocking remains intact.

Remaining: inspect the affected VM's recovery presentation, current/recorded-name fallback and GPU-change versus verification wording. Preserve the hold. Independent read-only evidence does not validate reconciliation effects or close the case.

### BUG-0006 — Accessible slider units disagree with visible GB

Evidence: [original defect report](../qa/inbox/2026-10-14-independent-slider-accessible-units.md) and [independent session R07](../qa/inbox/2026-10-14-independent-results.md). Installed About version 0.1.0; correspondence to a source revision was not independently established.

Reproduction: select VM1 with its visible 6 GB request and disabled editing controls. Windows accessibility exposes the slider named `Requested GPU memory in GB` with value `60`, while the edit field exposes `6` and the displayed endpoint is 8 GB. Repeated twice, including after graceful restart. This establishes accessible-value misinformation, not an incorrect saved allocation.

Expected: the accessible value/range or an unambiguous value description communicates the same GB request as the visible controls.

Source fix: accessible value/range/step use GB rather than internal tenth-GB positions, and assistive set/increment/decrement actions translate back to those positions. The exact visible request is retained as the accessible description. Disabled controls and presentation reject edits. A compiled headless regression covers values/ranges, 0.1 GB actions, native byte conversion, invalid input, endpoints and recovery/unavailable/modal/presentation guards.

Remaining: repeat both read-only observations after installation; separately check enabled keyboard increments and screen-reader output with appropriate authorisation. The source cause was internal step positions exposed under the GB label. Native screen-reader output and installed enabled-control behaviour remain untested.

## Qualification tasks

The roadmap's candidate and acceptance checkpoints remain open. Earlier exploratory results do not qualify an unidentified future candidate.

| ID | Task | Dependencies | Implementation | Validation |
|---|---|---|---|---|
| QA-009 | Candidate regression and automated gate | None | Ready: preparation only; live execution requires authorisation | Pending milestone gate; current non-hardware gate passed; candidate-specific live regression pending |
| QA-010 | Release decision and user acceptance | QA-009 | Blocked: candidate evidence and unresolved P1 | Not run |

### QA-009 — Candidate regression and automated gate

Dependencies: None

Record exact installed candidate/build/revision and working-tree context. Prepare resettable targets, allowed effects, stop conditions and evidence before live execution. Run the core journeys and failure/UI coverage in the [roadmap](roadmap.md#test-journeys), including all bug retests above. Record each result and exclusion; unresolved failures remain visible.

At the agreed checkpoint run the existing non-hardware gate after reviewing its effects. Completion requires candidate-specific results, final state/readback and persistence evidence, and an accurate automated gate outcome. Prerequisites for affected live journeys include resolving or safely handling the retained recovery hold through supported authorised actions.

### QA-010 — Acceptance decision

Dependencies: QA-009

Summarise unresolved defects, deferred/untested scenarios and candidate evidence. No unresolved P0/P1, successful core journeys and fix retests, usable critical recovery, automated evidence and explicit user acceptance are required by the roadmap. Record the user's decision; documentation or automated checks alone cannot close this task.

## Evidence and validation checkpoint

The [case index checkpoint](../qa/INDEX.md#intake-notes) records 14 October 2026 results: 340 product tests, 12 standalone catalogue tests, validation of 416 messages, copy scanning, formatting, strict Clippy, binary build and API documentation passed; protected installation hashes matched. The full gate failed at documentation. These are existing reported results, not checks rerun for this backlog update.

The later [independent installed session](../qa/inbox/2026-10-14-independent-results.md) supplies bounded BUG-0004/BUG-0005 passes and the new BUG-0006 report. It did not independently map the installed executable to the source revision, execute VM/GPU effects or establish release acceptance. No bugs are closed.

Some historical reports referenced by the existing index/roadmap are absent from the current tree. Use the surviving index summaries with that limitation; do not invent or restore reports as part of this task.

## Source batch - 14 October 2026

Audited existing BUG-0001 through BUG-0005 code and regressions: Verify retains typed guest failures, independent restoration errors and pending recovery; ordinary review defers authentication only where admitted execution can authenticate before effects; cancelled reads discard late completion; draft wording shares the saved baseline; selection uses canonical VM identity; recovery uses the affected record identity. No further confirmed source defect was identified in these scoped paths. The original live Verify failure still needs diagnosis from an authorised retest.

Changed `src/gui/ui/details.slint` for BUG-0006 and added `src/gui/accessibility_tests.rs` using the existing headless test platform. No dependency, privilege, allocation or recovery policy changed. The focused compiled accessibility regression and full non-hardware gate passed. Source review and headless tests do not establish rendered, installed or live acceptance; BUG-0006 is not installed and all six cases remain open.

Validation: `powershell -NoProfile -File scripts/testing/check.ps1` passed, with 341 product tests, 12 catalogue tests, 416-message validation, copy scanner tests/scan, documentation, formatting, strict Clippy, locked binary builds and API documentation. Evidence: `local/evidence/m1-gate-20261014T094403107Z/report.json`. The initial attempt stopped at module ordering in formatting; the order was corrected before the complete passing run. Historical missing report links were replaced with surviving backlog/index summaries while retaining the missing-report caveat and preserving the deleted original. No installation, native GUI launch/render, guest access or VM/GPU effects ran. QA-009/QA-010 and bug closure remain pending their live/candidate/user gates.
