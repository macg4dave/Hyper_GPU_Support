# Backlog

## Next action

Continue **OPT-009** by completing the measured replacement/budget design before wider migration under the [core simplification roadmap](roadmap.md). The goal is 10,000 code lines under `src/` only, excluding Rust tests, with one coherent operation path. Remove unused tests; useful tests follow the program's design. The corrected `src/` baseline is 29,141 counted lines excluding 17,160 Rust test/helper lines. Repository tools are outside the goal. The first source slice is implemented and passed its complete non-hardware gate. The GUI simplification and request-capture cleanup also passed complete gates. The worker lifecycle follow-up also passed its integrated gate. Current counted `src/` code is 28,188 lines; complete-budget feasibility remains pending.

The requested [architecture review and simplification](OPERATION_COMPLEXITY_REVIEW.md) is complete through its [non-hardware checkpoint](ENGINEERING.md#architecture-simplification-checkpoint). Exact record equality, GUID syntax and completion input ownership are simplified; useful tests follow the production path. No new cards are needed. This source follow-up closes neither the budget design nor installed/live candidate acceptance.

Preview cleanup is implemented: `PreliminaryPlan`, `ReinstallPlan` and `VerifyPlan` runner reads had no in-repository GUI/CLI submission callers and are removed, together with deferred-authentication scaffolding and stand-alone reinstall/preliminary preview APIs. Useful tests now use current preparation; obsolete deferral-only cases are removed. The integrated gate passed and the protected update is installed; see the [checkpoint](ENGINEERING.md#runner-preview-cleanup-checkpoint).

The prior optimization candidate's recorded automated/install checkpoint is retained in [engineering](ENGINEERING.md#optimization-validation-checkpoint). Its pending live acceptance is carried by OPT-008 for the eventual simplified candidate; it does not qualify future changes. This plan starts no live qualification.

## Task register

| ID | Task | Implementation | Dependencies | Validation |
| --- | --- | --- | --- | --- |
| OPT-009 | Prove the reduction design | In progress | None | Core, GUI and worker slices passed complete gates; full-budget feasibility remains open |
| OPT-010 | Replace operation coordination | Not started | OPT-009 | Pending milestone gate |
| OPT-011 | Reach the 10,000-line goal excluding Rust tests | Not started | OPT-010 | Size and complete gate pending |
| OPT-008 | Simplified candidate acceptance | In progress | OPT-011 | Prior candidate gate/install recorded; new candidate and live evidence pending |

## Tasks

### OPT-009 - Prove the reduction design

Dependencies: None.

**Outcome:** a working, deletion-backed vertical slice and measured evidence that the retained product can fit the roadmap's strict size budget, or a concrete account of the incompatible requirement/budget.

Source checkpoint: GUI/CLI share prepared execution; the ordinary prediction/admission detour is removed; Configure preview/execution share typed steps; Configure/save-only/Verify/reconciliation share completion merging. An obsolete prediction test was replaced with ordinary prepared-path coverage. Newer same-VM drafts survive completion. Stored-record and protected-install formats are unchanged; the runner no longer accepts `OrdinaryPlan` and old direct reviewed-plan effect submissions are rejected. The complete non-hardware gate passed; [checkpoint and size evidence](ENGINEERING.md#core-simplification-checkpoint) record the exact scope. The 10,000-line goal is not yet demonstrated: the worker/native/guest areas alone currently exceed it. Finish the measured replacement/budget design before OPT-010.

GUI checkpoint: both dialog families now share action/footer rules, focus and scrolling; prepared Apply/Verify acknowledgement is pure validation rather than a duplicate GUI plan store; unused running-body summary builders and their obsolete tests are removed. Enrollment, profile deletion and partial removal also pass checked requests directly into existing callbacks; their redundant global caches are removed. Existing resource fields, copyable logs, action callbacks and the Slint design are reused. See [GUI evidence](ENGINEERING.md#gui-simplification-checkpoint) and the [captured-request checkpoint](ENGINEERING.md#captured-gui-requests-checkpoint); this bounded follow-up does not close the full-budget design.

Worker follow-up: one Apply/Verify preparation exchange, receipt builder and persisted execution/confirmation lifecycle replace the parallel paths. Apply publication remains explicit; Verify never publishes intent. Retired direct-plan command variants are removed and terminal validation uses the prepared review directly. Bound receipt paths no longer use impossible optional-path branches. See the [worker checkpoint](ENGINEERING.md#worker-lifecycle-checkpoint) for validation status.

Acceptance:

- Establish the revised [size accounting](ENGINEERING.md#size-accounting) baseline excluding Rust tests and test-only helpers; inventory actual production callers, persisted/wire compatibility and retained product behavior/failure scenarios. Count adapters under `src/` explicitly; tools outside `src/` are outside the goal. Report Rust tests separately.
- Route ordinary GUI and CLI Configure through the same prepared worker path. Retain a credential-free ordinary path and CLI power approval; remove the replaced ordinary-plan execution/admission detour for this slice.
- Use one worker-owned typed step sequence for preview/execution and one bound completion handler. Reuse native/backend/storage code. A frontend plan cannot supply executable instructions or forge admission.
- Preserve fresh target/driver/revision checks, effect receipts, independent final payload/readback, exact publication and verified-unsaved recovery. Add regressions for ordinary change, no-op/omitted categories, stale acknowledgement, payload drift, cancellation/release and failed completion readback.
- Measure the remaining native/security/guest/presentation footprint under `src/` and replace speculative allocations with implementation evidence. Rust tests do not consume the program budget or dictate its architecture; remove unused tests and adapt useful ones. If the full scope cannot fit, identify exact retained product requirements and the shortfall before proceeding to wholesale migration. A partial slice is not proof that 10,000 lines is feasible.
- At the coherent slice checkpoint, run the existing complete non-hardware gate after inspecting its effects. Report count, deletions, passing/failing checks and unchanged live-evidence limits.

### OPT-010 - Replace operation coordination

Dependencies: OPT-009.

**Outcome:** one readable production orchestration path shared by GUI and CLI, with one sequence owner and one frontend completion owner.

Acceptance:

- Extend the proven Configure path to first setup, prepared enable/VRAM, Disable and explicit Reinstall; integrate Verify with explicit non-publication and power-restoration rules.
- Consolidate common worker admission/receipt/effect/final-confirmation mechanics while keeping authority, import, effect and native provider recovery domains explicit. Recovery dispositions never replay Configure.
- Move all foreground callbacks to one captured request binding and typed completion; observation, committed intent, draft and active request each have a clear owner. Preserve stale/poll suppression and drafts.
- Integrate enrollment continuation, save-only, partial host removal, profile deletion and import without alternate GPU engines or a configurable workflow framework. Credentials/bootstrap retain their existing distinct boundaries.
- Delete replaced entrypoints, state fields and forwarding layers as callers move. Preserve exported contracts until their consumers and compatibility are accounted for; do not keep a second engine behind a switch.
- Test the roadmap's retained normal/failure behavior using the simplified program interfaces. Remove unused or obsolete tests/helpers and update useful tests; do not retain production layers solely to satisfy existing tests. Run the complete non-hardware gate at the integrated checkpoint; disclose count and remaining budget.

### OPT-011 - Reach the 10,000-line goal excluding Rust tests

Dependencies: OPT-010.

**Outcome:** the complete maintained codebase fits the agreed count and retained product contracts.

Acceptance:

- Reach the **10,000-line code goal**, excluding Rust tests and test-only helpers. Count program code under `src/`, comments, blank lines, Slint and guest PowerShell. Exclude repository tools, root build code and other code outside `src/`; do not move program logic outside `src/` to evade the count. Report excluded Rust tests and data/generated/third-party totals separately; no relocation of program logic or minification to pass.
- Simplify actual duplicate native/guest/storage/presentation mechanisms under `src/`; keep exact ownership, trust, bounded transport, failure evidence and recovery semantics. Preserve `en-GB` wording, typed substitutions, escaping/native bounds and the existing Slint design.
- Remove unused, obsolete and redundant Rust tests/helpers; adapt useful tests to the program's design. Tests have no line budget and must not dictate production interfaces or preserve retired code. Keep meaningful checks of retained product behavior/failure contracts. Make count enforcement part of the existing gate with focused accounting regressions.
- Remove all superseded paths and temporary migration scaffolding. Preserve stored authority, configuration, managed history and unresolved recovery; any necessary migration needs an explicit format/compatibility design and interruption tests first.
- Run the complete non-hardware gate for the final integrated inputs. Report the measured count, source deletions, coverage changes and any unproven requirement. Failure to reach the count leaves this task open.

### OPT-008 - Simplified candidate acceptance

Dependencies: OPT-011.

**Outcome:** accept the final simplified product from matching automated, installation and live evidence. The previous optimization candidate's gate/install checkpoint remains historical; no live acceptance is claimed.

Acceptance:

- Match the simplified source and binaries to a complete passing non-hardware gate and the strict size count. Trace retained operation/failure contracts to meaningful isolated assertions and disclose exclusions.
- Prepare exact candidate identity, targets, starting states, allowed effects and stop conditions. Obtain authorization before live launches, guest access, protected installation/writes or VM/GPU effects.
- Independently verify protected installation and preservation of operator/pair authority, configuration, managed history and unresolved recovery.
- Execute the roadmap's normal/failure journeys. Record terminal outcome, saved/current readback, final recovery/power state, durations, counts, bytes and transitions. Distinguish isolated tests, installed observations and hardware results.
- Qualify the candidate's configured transfer chunk size for throughput, peak memory, exact tails and slow/no-progress guests. Retain interrupted evidence; do not retry effects for a better measurement.
- Disclose unrun scenarios, unavailable timings and regressions. Close only from candidate evidence and the user's acceptance; summarize in chat without creating QA reports unless requested.
