# Testing and qualification

[Roadmap](ROADMAP.md) owns milestone gates; [backlog](BACKLOG.md) owns tasks. This page owns validation timing, scenarios, and evidence.

## Timing and authorization

Run builds and automated tests at agreed milestone completion or on explicit request. Individual tasks are not milestones. Test source may be added earlier and labeled unrun.

Documentation-only work may check links, anchors, task consistency, and diffs without compiling or launching the product. Live GUI launch, guest access, VM/GPU changes, privileged setup/enrollment, and protected writes require explicit authorization for a concrete sequence. The documentation Q&A authorizes none of those effects.

Retired mock/snapshot GUI tooling is not used or maintained. Meaningful isolated backend unit tests remain useful; inspect their effects before selecting them.

## Planned automated gate

The source-based [UI audit](BASELINE.md#ui-interaction-audit---12-october-2026) adds journey acceptance below; it does not require per-card builds or authorize a GUI launch. Focused regression coverage added during separately requested fixes runs at the agreed milestone/checkpoint.

APP-008 provides the narrow non-elevated [gate](../scripts/testing/check.ps1) against current source, using a local [documentation checker](../scripts/testing/check-docs.py). Run `powershell -NoProfile -File scripts/testing/check.ps1` at M1 completion. The gate stops at its first failed stage and records commands, revision, environment, timestamps, results and exclusions under ignored `local/evidence/m1-gate-*`.

The [APP-001 through APP-008 follow-up source review](evidence/m1-source-review.md) is complete after APP-006/APP-007 implementation. The gate passed on 11 October 2026; [closure evidence](evidence/m1-closure.md) records the executed commands, 192 passing tests and exclusions. Source fixes and added regressions alone do not constitute a passing gate.

| Order | Check | Intended command or behavior |
|---|---|---|
| 1 | Documentation | Local links/anchors, task references/dependencies, instruction consistency, diff whitespace |
| 2 | Formatting | `cargo fmt --all -- --check` |
| 3 | Non-hardware tests | `cargo test --locked --all-targets`, after inspection and explicit exclusion of any effectful suites |
| 4 | Lint | `cargo clippy --locked --all-targets -- -D warnings` |
| 5 | Product binaries | `cargo build --locked --bins` |
| 6 | API documentation | `cargo doc --locked --no-deps` |

The gate is non-elevated, fails on the first failed stage, reports what ran, and retains local evidence. It does not install software, launch the GUI, prompt for credentials, modify protected stores, access VMs, or execute hardware workloads.

## Non-hardware acceptance scenarios

- VRAM minimum is 1 GB; upper bound follows matched card capacity; units/provider mapping remain truthful; incompatible settings are not silently rewritten.
- Edits have no effects; review binds the target, proposed settings, and lifecycle; stale review is rejected.
- Initial setup includes verification and confirmed final-off intent; later toggles/VRAM changes do not repeat preparation or graphics verification and always finish off. Cover ordinary recovery without guest access, refused/unconfirmed shutdown, and explicit setup/update review for invalid preparation.
- Credential explanation names the VM and precedes the local prompt; cancelling either panel prevents worker submission and retains edits/recovery; remembering requires opt-in; explanation/vault failures remain explicit.
- Settings survive reload, drafts remain separate, and conflicts block stale execution.
- Additive enrollment and selected-VM operations preserve other targets.
- Publication failure permits bound save-only retry; uncertain effects retain recovery and never trigger blind replay.
- Unfinished operations replace blocked More actions with an explanation naming the affected VM and a Review recovery button. Opening it preserves edits and holds, uses the existing recovery review, and requires separate confirmation before any recovery action. Check both host-wide records and selected-VM pending journals, plus the normal menu after recovery is resolved.
- Reconciliation handles a start error after a power change, safely restores only unchanged approved facts, retains refused-shutdown failures, and blocks restoration after changed facts. Pending journals without a save record still block new controller reviews.
- Existing trust, IPC, locking, transport/output budgets, and interruption boundaries remain enforced.
- Checkpoint rollback follow-ups (APP-016 through APP-022): unchanged host receipts remain historical, current host/guest discrepancies and unavailable checks are distinct, explicit same-digest reinstall cannot skip preparation, and removal/forget preserve other targets and recovery evidence. Execute added isolated regressions at the agreed milestone gate; APP-016 regressions are currently unrun.
- APP-018 payload verification: complete, absent, partial and corrupted current files; missing/wrong-payload/wrong-guest receipts; invalid/wrong-GPU manifests; denied/unavailable/timed-out observations; changed payload with unchanged driver version; absent host attachment before effects; failed checks preserving recovery and restoring only reviewed power. Compilation with `cargo check --locked --all-targets` passed; regression execution and actual guest comparison remain pending. No GUI, guest, credential or VM/GPU run is included in that compilation check.

Add coverage for real missing behavior and failure boundaries. Reuse meaningful existing tests. Dedicated retired GUI simulation tests were removed through APP-001. The replacement startup-dispatch regression and retained live controller/state tests passed APP-008 execution; real interaction remains APP-009 acceptance.

APP-029 adds isolated compatibility-write regressions: matching-settings VRAM changes and same-digest reinstall skip the native write; re-enable after ownership release records the fresh matching baseline; required settings-write failure retains pending recovery and prevents attachment/graphics. These regressions remain unrun until the agreed automated checkpoint.

## Live qualification scenarios

APP-009 prepares the exact environment, targets, effects, workload interval, evidence locations, final power states, and stop conditions before requesting authorization.

The [M2 preparation record](evidence/m2-preparation.md) contains the proposed sequence and missing environment/mapping facts. It is preparation, not authorization or executed evidence. The accepted selected-VM setup is not repeated solely for qualification.

APP-023 adds separately prepared checkpoint rollback qualification after the source fixes. Name a disposable target and checkpoint, disclose checkpoint creation/application and subsequent setup/repair/removal effects, and obtain explicit authorization before execution. Record the actual Verify failure details, fresh mismatch detection, same-digest reinstall, removal and fresh setup results; preserve other VMs. The bug report does not authorize rollback of the designated test VMs below.

### Available Hyper-V test VMs

The following existing VMs are available for testing: `win_26300.9457  VM1` , `win_26300.9457  VM3`and `win_26200.9457 VM2`. Do not roll back either VM or add generic rollback warnings solely because these designated targets are used for testing; do not automatically reset or otherwise restore them. If a test requires a VM to be reset, stop and inform the user.

All tests can be unsafe to the VM and its contents.

| Scenario | Acceptance |
|---|---|
| Initial setup | Review explains lifecycle and time; credential explanation identifies guest sign-in; hardware verification reports pass/fail; completed setup ends with VM off |
| Ordinary changes | Later toggle and VRAM updates use reviewed effects without repeated full preparation or initial graphics verification |
| Lifecycle | Actual power transitions and final state match the review; failures report independently observed state |
| Saved state | Settings survive reopen; drafts and observations remain distinct |
| Concurrent use | Two guests identify the selected hardware and run checked rendering during an overlapping interval |
| Independent operation | Selected-VM changes preserve the other guest's attachment, settings, and rendering |
| Real GUI usability | Names, scrolling, keyboard, review, progress, cancellation, and failure presentation work in the authorized live workflow |
| Failure/recovery | Safely applicable approved scenarios retain evidence and holds; save-only does not replay GPU effects |
| Host observation | Record responsiveness and any failures during the defined interval |

Select exact host/guest builds, GPU/driver, two existing Generation 2 Windows 11 guests, and representative workloads before executing. Qualification checks can observe concurrent rendering without adding repeated rendering verification to ordinary product operations.

All Windows redraw bugs are closed by explicit user acceptance on 11 October 2026. The live check switched the partly off-screen/taskbar-covered window from Dashboard to About through Slint MCP, then the user restored it without another UI action and confirmed the latest content redrew correctly. The [local backend patch](../vendor/i-slint-backend-winit/LOCAL_PATCH.md) records that check, acceptance and two passing in-memory regressions with their separate test command; root-package tests do not execute dependency unit tests. Cover/uncover, minimize/restore, resize and fractional-DPI monitor transitions remain useful future regression scenarios, not open acceptance blockers. No VM/GPU effects are authorized by these window checks.

Do not force power-off, induce hangs, restart the host, or automatically retry. Stop on uncertain outcomes and retain evidence. Live fault injection requires specific authorization. APP-010 adds separately authorized package installation and upgrade trials.

## UI journey acceptance

APP-009 owns these interaction checks after the separately requested UI fixes and agreed automated gate. The [source audit](BASELINE.md#ui-interaction-audit---12-october-2026) is not interaction evidence. Prepare an exact artifact/revision, target VM(s), starting state, permitted actions/effects and evidence location before requesting live authorization; a GUI launch may elevate and is not implied by this checklist. Do not repeat the already accepted setup just to collect screenshots. Use a separately approved eligible target if a new setup run is needed.

For each exercised case, record starting state, user action, actual result, expected result, pass/fail and supporting screenshot or non-secret event evidence. Record unexercised cases as Not run, with the reason. Source and installed-artifact behaviour must not be conflated.

| Journey / case | Observable acceptance |
|---|---|
| Selection and drafts | Edit VM A, try switching to B: Stay keeps A/edits; Review names A; Discard & Switch changes to B without applying. Duplicate names remain distinguishable. Other profiles stay unchanged |
| Refresh and conflicts | Pending edits survive Refresh/failure; changed saved intent has an actionable conflict explanation. Missing VM/GPU or reordered GPUs do not redirect a draft/approval to another target |
| First setup | Follow authorization then effect review without ambiguous completion claims. Local credential cancellation performs no new guest work, retains the draft, and leaves existing recovery intact. Remember is opt-in |
| Ordinary change | Toggle/VRAM edit is pending until second Apply; Cancel retains it. Review names actual changes, transitions and final Off. Completion distinguishes host outcome, save result and historical guest verification |
| Action availability | APP-030 reasons match pending edits, active/inactive profile, power, authority, stale inventory and recovery; an available next step is reachable without first attempting a prohibited operation |
| Verify and repair | Verify explains driver-file/receipt and rendering checks plus actual lifecycle. Repair is a separate reviewed reinstall. Cancel and changed target/revision invalidate approval; negative or unknown results stay visible |
| Removal and deletion | APP-024 deletes app records only and never stages GPU removal or changes power. Active/unknown support blocks deletion under the existing inactive-profile contract. Cancel preserves records; completion keeps the Hyper-V VM and other profiles; partial failure retains uncertainty/recovery |
| Progress and failure | Long-running progress remains readable; scrolling up is retained; 100% transfer is not verification success. Failure names the outcome and next step; details add diagnostic value instead of repeating the summary |
| Recovery and save-only | Review recovery opens the bound review without clearing holds. Save-only never repeats GPU/guest effects. Denied/unavailable reconciliation preserves evidence and clearly says what remains blocked |
| Keyboard and layout | Tab/Shift+Tab, Enter, Escape, safe focus return, consent and disabled controls work through idle/running dialogs. Long names/content, supported minimum window, recorded scaling and both themes keep actions reachable |
| Close and reopen | Closing with a dirty draft offers safe choices; closing during work does not imply cancellation. Saved settings survive reopen; fresh/unknown observations and historical verification remain distinct |

APP-027/APP-028 recovery cases and APP-011 allocation behaviour retain their own prerequisites and evidence. No fault injection, credential prompt, guest access, protected write or VM/GPU effect is authorized by inclusion here. Close usability acceptance only for the actual cases exercised; do not mark all journeys passed from one successful setup.

## Evidence and claims

The requested build on 12 October 2026 fixed two Slint compilation errors
(a duplicate VM-card property and a conditional Verify-button focus reference)
and updated an outdated VRAM test fixture. Strict Clippy uses a documented,
function-scoped argument-count allowance for the existing background-operation
orchestrator. The locked development binary build, all-target tests (306 passed),
formatting, strict Clippy and API documentation generation passed; local logs are under
`local/evidence/build-20261012/`. The full documentation gate was not run because
the roadmap, backlog and several evidence documents were deliberately removed
pending replacement. These results do not close a milestone or establish live
GUI appearance, keyboard focus, installation, guest or VM/GPU behavior.

Record revision/working-tree context, commands, exact environment, authorized sequence, timestamped stages, outcomes, and exclusions in ignored local evidence storage. Flush operation-boundary records so crashes retain the last known stage. Exclude credentials.

The 13 October 2026 automatic-polling fix passed the 100 application tests,
including independent poll admission, obsolete-result rejection and panic cleanup;
strict Clippy, formatting and the locked binary build also passed. Evidence is
under `local/evidence/polling-fix-20261013/`. The installed package hash checks
passed with enrollment and saved state preserved. The GUI was reopened; visual
confirmation that controls stay available across timer ticks remains pending.

Report Passed, Failed, Not run, and Unknown honestly. A scoped pass applies to its recorded environment and interval; it does not prove universal stability or VRAM isolation. Readback or sequential verification alone does not prove concurrent sharing.

APP-033 review regressions cover twelve APP-028 combinations of Verify/reconciliation, payload/graphics failure and restoration-proof save/readback failure; combined public diagnostics; GUI primary failure retention when completion reads fail; and compact newer-adverse-check precedence with missing/mismatched records. Existing `SaveRecord::requires_graphics` regression covers ordinary Apply versus standalone Verify/legacy requirements reused by CLI recovery. Added tests remain unrun until the agreed automated gate. APP-034 adds delayed-effect drift/restoration coverage as a separate future implementation.
