# Current source review - 12 October 2026

User scope: review accumulated code changes for gaps and logic errors, fix small/medium findings and create tasks for larger findings. Review started at clean revision `733438d`; fixes are working-tree changes. No sub-agents were used.

## Findings and disposition

| Severity | Location / trigger | Consequence and disposition |
|---|---|---|
| High | [Apply](../../src/workflow/apply.rs), compatibility writes after slow shutdown/preparation and verification-only failure restoration | Enabled Apply uses initial settings to decide/write compatibility values without the changed-settings check already used by removal. Failure restoration uses only power readback, unlike Verify/reconciliation's full host/GPU checks. External drift may be overwritten or followed by power changes before final readback. New APP-034 owns stage-specific drift safeguards and failure coverage; not implemented here |
| High | [Apply](../../src/workflow/apply.rs) and [negative completion](../../src/configuration_store.rs), matching preparation with invalid/missing check evidence followed by failed verification-only Apply | Pending publication cannot take the standalone Verify negative-retirement path. A proven payload mismatch can strand the profile behind recovery and prevent reviewed repair. Existing APP-027 retained; no duplicate task and no hold cleared |
| Medium | [Diagnostics](../../src/gui/diagnostics.rs) calls `ComponentFailure::decode` across the library/binary crate boundary | Method was `pub(crate)` in the library and inaccessible to the binary. Exposed the existing bounded typed parser with an explicit diagnostics-only contract. Confirmed source visibility mismatch; compilation remains unrun |
| Medium | [Restoration proof](../../src/workflow/guest_check.rs), proof save/readback fails after a negative payload/graphics check | `?` hid the original guest failure. Fixed APP-028 by retaining both errors, resetting unconfirmed in-memory proof, and preserving pending recovery. [Reporting](../../src/reporting.rs) presents both contexts with independently redacted technical facts |
| Medium | [GUI completion](../../src/gui/live.rs), failed Apply/Verify/reconciliation/save-only followed by failed completion records/state load | Completion callback returned after the read error and never presented the original operation error. Added a shared completion wrapper that retains both redacted contexts and the recovery next step. Success remains unconfirmed when reads fail |
| Medium | [Compact VM summary](../../src/gui/live/presentation.rs), reopened/restored negative check alongside historical `last_verified` | Summary used old success instead of the newer failure/mismatch/unavailable/interrupted outcome. Reuses [shared recorded summary](../../src/gui_model.rs), with invalid/missing records and recovery taking precedence |
| Medium | [CLI reconciliation](../../src/main.rs), enabled ordinary Apply receipt has `graphics_required = false` | CLI requested guest credentials despite guest-free recovery. Now follows the immutable receipt's `requires_graphics`, matching the workflow requirement and existing ordinary/legacy/Verify regression contract |

APP-032 retains real keyboard/focus/layout acceptance; source alone does not reproduce those interaction failures. APP-025/APP-032 documentation still describes earlier audit starting points while current source contains further diagnostics/focus/progress changes; those cards need acceptance against their complete criteria, not a claim that the current implementation is absent. APP-011 retains live VRAM allocation qualification; physical capacity and provider metadata were not conflated or remapped in this review.

## Review coverage and limits

Inspected the accumulated-change history and current callers through GUI action eligibility, stable VM/GPU/draft/revision binding, background completion/readback, diagnostic redaction, progress model updates, slider exact values, workflow admission/preview/effect order, latest check/preparation eligibility, failed-check restoration and retirement, worker receipt/audit/publication/finalization, configuration conflict tokens/save-only, guest manifest/file/receipt parity and transfer bounds, inactive-profile removal, and process-bound IPC cancellation. Inspected startup/CLI recovery routing and applicable agent/handbook guidance.

The review is source-based and focused on product flows and changed integration points. It is not a line-by-line audit of every vendored dependency, proof against all concurrent administrator changes, or independent security certification. Existing locks, authentication, protected stores and final readback remain required; this review does not certify the unresolved Apply drift paths as safe.

## Checks and acceptance

Added isolated regression source covers twelve proof-persistence failure combinations, compact negative-history precedence/missing/mismatched records, GUI combined failure presentation and secret redaction. Existing durable graphics-requirement coverage is reused for CLI recovery. No new effectful tests were added.

Checks actually run: targeted `rustfmt` with child traversal disabled, targeted formatting check, local documentation/link/task consistency checker and `git diff --check`. These parse/format source or inspect documentation; they do not compile or execute regressions. Passed: targeted formatting check, documentation checker (35 Markdown files, 34 task cards, zero errors), and diff whitespace check.

Not run: Cargo compilation/test/Clippy/build/API-documentation gate; Slint rendering; GUI launch or interaction; credential prompts; installed-worker IPC probes; protected writes; guest access; VM/GPU or driver effects. These remain subject to [testing timing and authorization](../TESTING.md#timing-and-authorization). No milestone or live acceptance closed.

Next safety tasks: APP-034 and APP-027. Then run the agreed non-hardware gate against the final source checkpoint before preparing separately authorized live qualification.
