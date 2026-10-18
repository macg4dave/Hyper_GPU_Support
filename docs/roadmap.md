# Core simplification roadmap

## Current scope

The CLI is now **defunct**. The GUI is the supported product frontend; CLI references below describe earlier implementation checkpoints and do not require retaining CLI functionality in the simplification work. Ordinary CLI inventory/planning/GPU/verification/credential dispatch is now removed. Administrator record/recovery commands remain for native evidence, legacy installation, import and recorded outcomes; their distinct contracts are documented in the [caller review](OPERATION_COMPLEXITY_REVIEW.md#frontend-caller-and-recovery-review).

The requested outcome is a simpler, readable application with cohesive responsibilities, one core operation path and the existing Slint design. There is no numerical source-line target or size-enforcement gate. The earlier target prompted a broader architectural review; changes are judged by reduced unnecessary complexity, clarity and preserved functionality. The [architecture review](OPERATION_COMPLEXITY_REVIEW.md) records current findings and implemented simplifications; engineering defines descriptive source accounting and test policy.

The first OPT-009 source slice implements shared prepared execution, Configure steps and completion merging. The guest progress source follow-up shares preparation/verification progress delivery. The latest frontend caller follow-up removes ordinary CLI dispatch and records dispositions for remaining administrator/recovery callers. Both add failure regressions and passed the integrated non-hardware gate. OPT-009 source review and automated validation are complete; installed/live acceptance remains separate. Further replacement candidates require caller and behavior evidence before implementation. Historical source inventories below provide context rather than a required reduction or a blocker to further work.

Keep the agreed workflow and protected architecture. No feature removal, dependency upgrade, transport replacement or recovery-format migration is assumed. Prefer straightforward implementations even when they are longer. If a simplification risks intended behavior, stop that change and explain the trade-off.

## The core

The frontends collect a request, display progress and present a result. The protected worker owns operational decisions and their completion. Use the existing workflow/backend, authenticated IPC, lock, observation bundle and persistence primitives; consolidate their callers rather than putting another service/framework around them.

| Request family | Essential work | Explicit limits |
| --- | --- | --- |
| Read | Fetch a revision-bearing observation for full refresh, poll, selected completion or preview | No effects; full and lightweight reads remain different scopes |
| Configure | Establish current facts; choose preparation, attachment/settings/allocation changes and any required graphics check; execute and confirm; publish exact intent | First setup, ordinary enable/VRAM, Disable and explicit Reinstall are variants of this operation |
| Verify | Check current guest files, device health and rendering; restore permitted power; record the check | No publication of GPU intent or acquisition of settings ownership |
| Recover | Load the original receipt; independently establish an allowed disposition; complete that disposition | Check outcome, retry saving and remove partial host setup are explicit variants; never replay Configure |
| Manage app records | Enrol a pair, delete a profile or import configuration through their existing domain-specific handlers | Distinct authority and publication rules; profile deletion never invokes Hyper-V or guest effects |

Bootstrap remains its fixed installation entrypoint. Credential management remains a local prompt/vault action. Neither needs to masquerade as a GPU workflow. One core means one implementation of each responsibility, not one giant function with every permission.

### One execution path

For Configure and Verify, the supported GUI uses the existing prepared-worker exchange:

```text
frontend request
  -> authenticated worker + common lock
  -> fresh facts and exact prepared scope
  -> local target/revision acknowledgement and credentials if required
  -> consume the worker's private admission token
  -> persist receipt, execute guarded steps, independently confirm
  -> publish only where the operation permits it
  -> terminal outcome
  -> protected completion observation, one frontend completion handler
```

The earlier CLI ordinary path used `apply_prepared`, matching the GUI and setup; that duplicate frontend dispatch is now removed. `OrdinaryPlan`, direct worker client Apply/Verify functions and optional backend/readmission branches have been removed. Retired direct reviewed-plan Apply/Verify wire variants are removed. Old submissions fail request decoding; terminal replies are checked directly against the authenticated prepared review. Eligible ordinary changes remain credential-free. The removed CLI `plan` granted no reusable execution authority; the public runner read contract remains available independently of that retired frontend. Audit public callers before removing exported helpers; this package is unpublished, but that alone is not proof of no consumers.

### One definition of the steps

The first source slice replaces parallel action construction in `workflow/planning.rs::plan_from_decision` and conditional sequencing in `workflow/apply.rs::execute_decision` with the fixed typed sequence in `workflow/steps.rs`. Displayed actions, required credentials/power transitions and execution consume that same sequence. Allocation steps carry the resource and exact value; settings steps carry their exact before/after facts.

The sequence is worker-owned, bound to fresh target/driver/payload/revision facts and cannot be supplied as arbitrary frontend instructions. A step is a domain action, not an arbitrary command, callback registry or persisted workflow language. Keep one guarded step runner: verify expected facts, durably mark an impending effect, invoke the native adapter, independently read back, then advance expected facts. Fresh final payload authentication still precedes success/publication.

Keep normal execution and failure restoration visibly separate. Restoration is based on recorded scope and fresh facts, not simply reversing the successful steps. Interrupted execution is never resumed by replaying the list. Reuse existing Verify/guest-check/restoration logic without forcing different final-power contracts into one conditional maze.

### One completion owner

Configure, save-only, Verify and workflow reconciliation now merge terminal results/readback through `gui/live/state_access.rs::complete_operation`. Remaining operation-specific presentation/state duplication is still to be removed. It receives the captured VM/GPU identity, request generation, authenticated terminal result and a separate completion-read result. It merges saved intent/recovery once, preserves newer drafts, and derives the dialog/model update once.

Terminal failure, verified-but-unsaved, successful publication and unavailable completion readback remain distinct. A failed readback must not erase an established terminal result; a successful read must not manufacture an operation success. Release of an exact backend instance remains distinct from receiving its reply. Read cancellation waits for release and never cancels effects.

GUI state has four clear owners: current observation, committed settings, editable draft, and one active foreground request. Recovery comes from the protected snapshot and remains VM-scoped; selection-specific fields are derived. Progress and result notices are presentation, never admission authority. Poll generations and bound foreground identity prevent stale delivery.

## Replacement and deletion map

These are replacement candidates, not a finding that the existing code can simply be deleted.

| Existing code | Reuse and simplify | Remove after replacement proves the contract |
| --- | --- | --- |
| `main.rs`, `worker/client.rs`, `worker/preparation.rs`, `worker/operations.rs` | Prepared exchange and consumed admission | Ordinary-plan execution detour, optional retained-backend branches, redundant approved-plan production entrypoints |
| `workflow/planning.rs`, `workflow/apply.rs`, `workflow/apply/facts.rs` | Decision rules, expected-fact guards, one typed sequence | Independently maintained preview/execution ordering and unused compatibility hooks such as `CheckOs` after caller/wire audit |
| `worker/operations.rs`, `worker/progress.rs`, `worker/dispatch.rs` | One receipt/effect/confirmation lifecycle, explicit publication policy | Repeated Apply/Verify lifecycle scaffolding; forwarding wrappers with no distinct contract |
| `gui/live/{apply,verification,recovery,partial_removal,profile_removal}.rs`, `gui/live_state/` | Existing actions and controls, one request binding and completion handler | Per-operation copies of acknowledgement/readback/state merging and redundant review/result fields |
| `configuration_store/`, `worker/runner/{storage,recovery,profile_removal}.rs`, `enrollment.rs` | One checked file read/write mechanism; explicit domain records | Repeated serialization/revision/readback plumbing, not distinct authority/effect/import/provider evidence |
| `windows_*.rs`, `guest*`, probe binaries | Existing Windows and transport adapters, one host adapter per effect and one guest command per task | Duplicate conversions/staging/session plumbing only after tracing concrete callers and failures |
| Rust tests under `src/` | Useful tests of current product behavior | Unused, obsolete or redundant tests/helpers; test-only architecture and compatibility code that no longer serve the program |

Keep the existing Slint component design and reuse shared controls across actual GUI tasks. The largest UI reduction must come from Rust coordination, not replacing the interface. Native API wrappers, cross-process authentication and the guest helper remain separate where they serve real boundaries. Repository tools/scripts are outside the requested simplification scope. No relocation into scripts, generated business logic, a new local crate or vendored project code counts as removal.

## Source measurements

These are descriptive implementation sizes at the listed checkpoints. They help locate maintenance cost; they are neither optimisation targets nor evidence that a responsibility should be removed. No per-area allocation or required reduction applies.

| Source area | Baseline | First core slice | GUI simplification | Worker lifecycle | Preview cleanup | Architecture review |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| GUI, including Slint | 8,652 | 8,638 | 8,241 | 8,241 | 8,241 | 8,204 |
| Worker and runner | 5,800 | 5,679 | 5,679 | 5,490 | 5,478 | 5,439 |
| Workflow | 1,905 | 1,858 | 1,858 | 1,852 | 1,816 | 1,798 |
| Native Windows modules | 3,803 | 3,803 | 3,803 | 3,803 | 3,803 | 3,796 |
| Guest, payload and probe modules | 3,199 | 3,201 | 3,201 | 3,201 | 3,201 | 3,201 |
| Other core and CLI under `src/` | 5,782 | 5,759 | 5,759 | 5,759 | 5,759 | 5,750 |
| Counted `src/` total | **29,141** | **28,938** | **28,541** | **28,346** | **28,298** | **28,188** |

Under `src/`, the first core slice changed 29,141 counted / 17,160 test lines to 28,938 counted / 17,341 test lines. Its 203 counted-line reduction includes direct-execution helpers becoming test-only; it is not 203 physical deletions. Four existing formatting differences were corrected for the gate.

The GUI batch including request-capture cleanup removes another **397 counted lines**, leaving **28,541 counted / 17,129 excluded test lines** and **45,670 physical lines** under `src/`. GUI code including Slint is 8,241 lines. It shares dialog actions, validates captured requests and removes hidden summary builders and obsolete review stores. The GUI checkpoint and request-capture checkpoint record complete passing gates. Earlier wider repository counts remain historical in engineering; tools/build/probes outside `src/` are excluded from this table.

The subsequent worker batch shares preparation exchange, receipt construction and the Apply/Verify execution/confirmation lifecycle, removes retired direct-plan wire variants and simplifies already-bound receipt paths. Its checkpoint records implementation and validation separately. At that checkpoint excluded Rust test/helper lines were 17,528, and gross `src/` source was 45,874 lines. Added meaningful failure tests account for the physical increase; no production code was reclassified as test-only.

The preview cleanup removes the three unused read RPCs, preliminary authentication mode and stand-alone reinstall/preliminary preview APIs. It reduces counted code by another 48 lines; useful tests use fresh preparation and retired frames/fields fail decoding. That checkpoint measured 28,298 counted / 17,542 excluded test / 45,840 physical `src/` lines. The complete gate and protected installation passed; see the linked checkpoint.

The architecture review follow-up simplifies exact record equality, GUID syntax and completion input ownership. It passed the complete non-hardware gate and leaves **28,188 counted / 17,688 excluded Rust test/helper / 45,876 physical `src/` lines**. The fresh baseline corrects a nine-line difference in the historical test inventory. No protected installation or live acceptance was run for this candidate.

The worker/native/guest areas total **12,436 lines** at the architecture-review checkpoint. Their size helps direct investigation but does not require replacing them. Simplify only demonstrated duplication or unnecessary indirection, preserving useful security and recovery boundaries. The source audit tables are in ignored `local/evidence/architecture-review-after.tsv` and `local/evidence/core-slice-count/`.

Rust tests have no line budget. Design the program around its actual tasks, then adapt useful tests to that design. Remove unused, obsolete and redundant tests and helpers when their behavior is retired. Existing tests do not require retaining unused production APIs, indirection or compatibility layers. Keep meaningful checks of retained product behavior and failures; test fixtures and implementation details are free to change.

Repository tools/build code outside `src/`, catalogue/configuration data, generated output and third-party source are disclosed separately in measurements. Existing English wording, native prompt bounds and typed placeholder safety still apply. Relocation, minification and generated business logic do not by themselves establish simplification.

## Delivery sequence

1. **OPT-009 - Review the simplification design.** Inventory production callers and retained requirement/test scenarios. The ordinary Configure slice already shares prepared execution, step definition and completion. Review remaining candidates against current code, dependencies, compatibility and failure contracts before wider changes; justify each replacement by a concrete maintenance benefit.
2. **OPT-010 - Replace operation coordination together.** Extend the proven slice across setup, Disable, Reinstall, Verify and explicit recovery. Integrate the thin GUI and CLI, common completion and domain-specific record operations in the same coherent batch. Delete superseded coordination as callers move; do not leave both engines active behind flags.
3. **OPT-011 - Consolidate remaining duplication.** Simplify demonstrated native/guest/storage/presentation duplication while preserving persisted data and product operation/failure contracts. Remove superseded paths and unused tests; adapt useful tests to the program's design. Validate clarity, responsibility ownership and compatibility without a required line count or size check.
4. **OPT-008 - Accept the simplified candidate.** Run the complete automated gate at coherent integration checkpoints; final acceptance uses the complete simplified candidate and retained behavior contracts. Separately prepare and authorize protected installation and live journeys. Prior candidate evidence remains historical and cannot qualify changed source/binaries.

The first slice couples worker execution and frontend completion because that is the complexity being removed. Splitting these into independent new frameworks would preserve the maintenance burden. Each batch includes meaningful behavior/failure regressions; do not run the entire build after each extracted function.

## Validation and acceptance

- Use before-and-after source measurements where useful, reporting tests and data separately. Assess actual removed duplication, clearer ownership and preserved behavior; a lower count alone is not acceptance evidence. Normal formatting, useful names and cohesive modules remain required.
- Show one production Configure/Verify admission route for GUI and CLI, one authoritative effect sequence and one frontend completion owner. Caller/dead-code searches must accompany actual deletion; file/function counts alone do not establish simplification.
- Preserve first setup from Off/Running, prepared VRAM/re-enable, Disable without guest access, explicit Reinstall and Verify with permitted final power. Keep changed-category allocation writes and independent final authentication. Never restart a guest stopped externally during Verify.
- Exercise cancellation before effects, exact backend release, stale selection/draft/revision, driver/payload drift, shutdown refusal, effect-stage failure, lost replies, readback failure, verified-unsaved publication and explicit save-only. Hold uncertainty and forbid automatic replay.
- Preserve VM-scoped recovery, global blocking for ambiguous/shared records, partial-removal ownership checks, native provider job evidence, additive enrollment, resumable profile deletion/import and local credential cancellation/opt-in. No stored record is discarded as a refactoring shortcut.
- Reuse the non-hardware gate, with useful isolated behavior/fault tests and strict binary/lint checks. Adapt tests to the chosen program design and remove unused tests. Validate retained permitted/forbidden effects, results, receipt transitions and independent readback without preserving obsolete test structures or production APIs solely for tests.
- Installed acceptance requires exact candidate/target identities, allowed effects, starting states and stop conditions. Confirm saved/current state, terminal outcome, final power/recovery and record preservation independently. Qualify retained polling/payload/helper/transfer optimizations at the candidate's configured chunk size using counts, timings, bytes, peak memory and slow/exact-tail failures; no source-derived speedup claims.

Planning and source work do not authorize launches, installation, protected writes, guest access or VM/GPU effects. Summarize results in chat; no new QA reports are required. The architecture/configuration/security handbooks continue to describe the current implementation until each replacement actually lands.
