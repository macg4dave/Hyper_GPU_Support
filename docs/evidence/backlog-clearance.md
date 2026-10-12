# Backlog clearance - 12 October 2026

Authorization: the user requested continuing through the backlog in order and explicitly permitted tests, live tests and builds. Current priority is APP-034, APP-027, then the remaining cards/dependencies. Live targets/effects/evidence are prepared before execution; failed/uncertain live effects stop that sequence and retain recovery. No force-off, host restart or blind replay is included.

## APP-034 automated checkpoint

Checkout: `b134cc2` plus the recorded working-tree changes. Input hashes, commands, environment and exclusions are captured by the non-elevated gate.

- Baseline gate `local/evidence/m1-gate-20261012T045702911Z`: failed at tests (195 library tests passed, one stale provider-VRAM-ceiling assertion failed); later stages Not run. The regression now rejects invalid triple ordering, preserving the approved byte-request contract. Removed an unused import.
- Interim gate `local/evidence/m1-gate-20261012T050149445Z`: all 287 tests passed; strict Clippy found a pre-existing nonminimal profile-removal boolean. Simplified that condition equivalently without weakening admission. Build/docs Not run by that stopped gate.
- Final gate `local/evidence/m1-gate-20261012T050319452Z`: Passed documentation, formatting, 199 library plus 88 frontend tests, strict Clippy, product binary builds and API documentation. All commands exited zero. APP-034 also passed all 80 isolated workflow tests.

APP-034 checks stage-specific full VM/GPU/journal expectations before and after delayed effects, captures only changed fields after our own writes, and refuses changed-fact power restoration. A changed journal during guest verification is retained. Existing previews and no-replay/ownership/lifecycle boundaries remain. New regressions include seven drift/unavailable cases during preparation, seven during failed verification-only Apply and changes between settings/resource writes.

This checkpoint also executed prior APP-028/APP-033 regressions and the accumulated source changes. It does not establish installed-artifact parity or live acceptance. No GUI launch/render, credential access, installed-worker integration, protected writes, guest access or VM/GPU effects ran in these gates.

Next: APP-027 failed verification-only Apply retirement, then remaining source/qualification work in dependency order. Later checkpoints below supersede this source checkpoint only for the commands/scenarios actually recorded.

## APP-027 source checkpoint

Implemented narrowly bound failed verification-only Apply retirement, allowing only unchanged non-power host facts and the reviewed final Off. Negative payload/check evidence and historical results remain; attempted configuration publication is cancelled by ending the archived operation without publication. Persisted before/after bindings preserve interrupted retirement. GUI/CLI prompt selection is read-only and the worker always revalidates.

Passed: all 82 isolated workflow tests and strict all-target Clippy. Regression matrices cover four original-power/success-history cases plus twelve refusal/persistence/interruption cases. Full automated gate follows; no live effects have run at this checkpoint.

APP-027 final gate: `local/evidence/m1-gate-20261012T050900261Z` passed documentation, formatting, all 289 tests (201 library, 88 frontend), strict Clippy, product binaries and API docs. No guest/GPU effects ran in this gate.

## Slider source and interaction checkpoint

APP-014 explicit edits adopt exact selected-card maximum, while loading does not migrate legacy triples. All 89 frontend tests passed. APP-026 then rounds only endpoint display half up and preserves exact selection; all 90 frontend tests passed. An MCP-enabled current-source build (`SLINT_EMIT_DEBUG_INFO=1`, `cargo build --locked --features slint/mcp --bins`) passed.

Current-source live GUI on Windows build 26300, 1240x860 at 100 percent scaling, was rendered and inspected through Slint MCP on local port 9315. It consumed installed-runner inventory and displayed the RTX 5060 physical capacity of 8,278,507,520 bytes as the rounded 8 GB endpoint. Screenshots/tree records are in `local/evidence/backlog-live-20261012`. No draft application, credentials, guest or VM/GPU effects were triggered by these UI checks. More actions displayed unavailable reasons and a reachable recovery route; its normal explanatory dialog exposed no duplicated Technical Details button. Actual complete journey acceptance remains pending.

## Live preflight blocker

The installed product has an existing VM1 receipt `58b2582461e984b86343587e83c069d5` in EffectsStarted, final Off, no graphics requirement, requesting VRAM bytes 0 / 8,000,000,000 / 8,278,507,520. VM1 GUID is `9aca1d8e-6e4e-44ea-ab6d-3776c66e005c`. Installed-runner read-only inventory shows it Off, GPU attached, pending managed state and null VRAM readback. A separate administrator read via `Get-VM`/`Get-VMGpuPartitionAdapter` confirmed Off/attachment and actual null VRAM properties (not merely missing property names). No power, allocation, driver or guest change was made. This is not proof that the requested allocation completed.

Retained the receipt, journals and all original state. Fresh effects and protected package replacement stay blocked while the partial operation has no safely established outcome. APP-009/APP-011 and dependent live qualifications cannot close from this preflight. Existing setup acceptance is not repeated.

## APP-025 diagnostics checkpoint

12 October 2026: the user renewed authorization for backlog work and any automated/live/read-write tests. This checkpoint exercised only current-source presentation and live inventory reads. The unrelated deletion of `docs/WORKFLOW_REFACTOR_PLAN.md` was preserved. Raw logs, UI trees, output properties and screenshots are under `local/evidence/app025-20261012`.

Audited all Technical Details inputs: VM panel, inspection, failure, Apply/permission/Verify reviews, save-only and reconciliation. Explicit additional details and resets already existed. Fixed duplicate-only summary subsets/reordered rows and missing inventory/GPU context; inspection now shares VM diagnostics. Tests cover absence/repetition, typed failure/completion context, redaction, scoped targets, exact capacity/provider facts, selection changes and stale observations.

- Passed: `cargo test --locked --bin hyper-gpu-support gui::diagnostics::tests` (6 tests), `cargo test --locked --all-targets` (203 library plus 92 frontend tests), MCP-enabled current-source build, targeted `rustfmt --edition 2024 --check src/gui/diagnostics.rs src/gui/live.rs`, and `git diff --check`.
- Initially failed: repository formatting on existing `src/workflow/tests.rs`; strict all-target Clippy on existing `src/gui/memory.rs:113` (`manual_is_multiple_of`); documentation checker on the deleted tracked plan and three incoming links. The user then requested fixing failures as encountered: formatted those tests, used the equivalent `is_multiple_of` predicate, replaced stale plan links with current architecture/evidence and made the documentation checker validate working-tree deletions while still rejecting dangling links. The plan deletion remains intact.
- Live scoped passes: current source at 1240x860, scale factor 1, dark theme; VM1 inspection displayed current inventory diagnostics and exact physical capacity 8,278,507,520 bytes in read-only selectable output with freshness/history wording. Opening/closing details did not submit an operation. Switching to VM3 collapsed old details; its More actions dialog exposed no Technical Details button. The app was closed normally after inspection.
- Interaction limit: VM-panel details button was partly clipped at the scroll area's bottom; accessibility activation worked, and the inspection dialog's output was readable. APP-032 owns layout/keyboard follow-up. Full clipboard, both-theme, failure-dialog and operation-transition interaction acceptance were not exercised; isolated tests/source inspection establish only their recorded scope.
- No credentials, guest checks, VM power/GPU changes, recovery retirement, protected writes, package replacement or milestone closure. The existing VM1 operation receipt was retained.

Final gate: `local/evidence/m1-gate-20261012T140341196Z`, revision `8dec678` plus recorded working-tree changes, passed all six stages: documentation, formatting, 203 library plus 92 frontend tests, strict Clippy, product builds and API documentation. A prior standalone standard build encountered Windows' running-executable lock; after closing the inspected GUI normally, its retry and the final gate build passed. The live screenshots precede only the equivalent lint predicate/test formatting/tooling/docs cleanup; no additional UI behavior changed afterward. Later documentation edits record these results and receive a separate documentation/diff check.

Next task: APP-032 dialog keyboard/layout usability, including the observed scroll-area clipping. APP-009/APP-011 live allocation qualification remains blocked by the existing unresolved receipt.
