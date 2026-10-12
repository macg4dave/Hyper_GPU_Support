# Workflow modularization plan

Priority: **P0**, as recorded in the [roadmap](ROADMAP.md#p0-workflow-modularization).

Status (12 October 2026): **Source implementation complete; automated validation pending milestone gate.** The user subsequently requested implementation of this plan. The module extraction and test relocation are complete; builds/tests and live operations have not run.

## Objective

Break [workflow.rs](../src/workflow.rs) into a small, idiomatic Rust module family while preserving all existing functionality and behavior. Keep the current public API, shared backend boundary, protected operation architecture and Slint design. This is an organizational refactor, not a redesign of planning, execution or recovery.

This file owns the extraction instructions. [Architecture](ARCHITECTURE.md), [security](SECURITY.md), [configuration](CONFIGURATION.md) and [testing](TESTING.md) remain authoritative for product contracts; [AGENTS.md](../AGENTS.md) owns agent behavior.

## Implementation record

The implemented layout matches the target below. `src/workflow.rs` is now an 82-line shared contract and explicit re-export entry point. The six private production modules own journal state, planning, Apply, Verify, reconciliation and guest-check primitives. Admission-token fields remain private; internal decisions and shared helpers use only the necessary workflow-family visibility. Caller paths remain unchanged.

The fresh starting inventory confirmed 3,665 lines and 74 workflow tests. All 74 scenario names and bodies were retained: 1 journal test, 7 planning tests, 36 Apply tests, 13 Verify tests and 6 reconciliation tests are inline; 11 cross-operation/other-VM scenarios remain in `workflow/tests.rs`. The shared isolated backend and six fixture/support items are test-only. No test-filter consumers requiring changes were found in scripts, Cargo configuration or GitHub guidance.

Structural comparison against the captured starting tree accounted for all 39 production items, all 74 test bodies, the backend double and every shared fixture. Comparison preserved string literals and expressions, normalizing only whitespace, formatting trailing commas, necessary sibling visibility and admission type qualification. Serde attributes/defaults and caller exports were also inspected. The APP-017 through APP-022 fixes remain part of the extracted source; no behavior or product policy was intentionally changed.

Validation: targeted `rustfmt --edition 2024 src/workflow.rs` (including its children), the structural comparison, documentation checks and diff whitespace checks. These are static checks, not compilation or executed behavioral evidence. All-target compilation/discovery, isolated regression execution, lint, binary build and API documentation remain pending the agreed automated checkpoint. No GUI launch, guest access, credentials, protected writes, VM/GPU effects or installed-artifact update ran. No milestone gate is closed by this extraction.

Subsequent review follow-up: APP-029 changes enabled Apply to skip unchanged compatibility writes, with three added regressions (77 workflow tests total). This is a separately requested behavior improvement after extraction; the original structural comparison above describes the extraction snapshot. APP-027 and APP-028 track the two remaining review findings without implementation. Compilation and regression execution remain pending.

## Starting point

The source reviewed on 12 October 2026 contains 3,665 lines in `src/workflow.rs`: approximately 1,300 production lines followed by 74 tests and their backend doubles. Production responsibilities include journal state, read-only planning, Apply admission/execution, explicit Verify admission/execution, reconciliation and shared guest-check helpers.

The APP-017 through APP-022 review changes are already present in the working tree. They are part of the refactor's starting behavior, not changes to discard or reimplement. Compilation and test execution of those changes remain pending. The earlier APP-018 compilation and M1 gate do not establish that this starting tree passes current checks.

Before implementation, inspect fresh source and working-tree changes. Refresh the function/test inventory if it has changed; the counts above are a planning snapshot, not a target file-size limit.

## Target layout

Keep `src/workflow.rs` as the module root; do not also create `src/workflow/mod.rs`.

```text
src/
├── workflow.rs
└── workflow/
    ├── journal.rs
    ├── planning.rs
    ├── apply.rs
    ├── verification.rs
    ├── reconciliation.rs
    ├── guest_check.rs
    ├── test_support.rs
    └── tests.rs
```

`test_support.rs` and `tests.rs` are compiled only under `#[cfg(test)]`. Focused unit tests stay inline in the module that owns their behavior. `tests.rs` retains scenarios spanning multiple operations.

## Responsibility and function mapping

| File | Move or retain | Responsibility and visibility |
|---|---|---|
| `workflow.rs` | Retain `Backend` and `OperationResult`; declare private child modules and explicit re-exports | Small shared entry point. Preserve existing `workflow::...` caller paths and signatures, including crate-visible worker entry points. No operation implementation or broad helper collection. |
| `workflow/journal.rs` | `Journal`, `GuestCheck`, `Journal::setup_eligible`, `persist_check`, `verified_at` | Durable per-VM state and its independent persistence/readback. Retain current Serde attributes/defaults and historical-success semantics. Storage effects remain behind `Backend`. |
| `workflow/planning.rs` | `PreparationPreview`, `PreviewAction`, `SettingsPreview`, `EffectPreview`, `Plan`, `Decision`, `decision`, `decision_for_operation`, `plan_from_decision`, `plan`, `plan_reinstall`, `restore_owned_settings` | Read-only decision gathering and review construction. Keep settings restoration calculation private here: it currently has one consumer. Planning may inspect/authenticate payloads but must not save journals or mutate host/guest state. |
| `workflow/apply.rs` | `AdmittedApply`, its `driver_version` method, `admit_approved`, `apply_admitted`, `apply`, `apply_approved`, `execute_decision` | Admission and execution of ordinary changes, setup/update, explicit reinstall and host GPU removal. Preserve the shared enable/disable sequence and failure handling. |
| `workflow/verification.rs` | `VerificationPlan`, `verification_power`, `verification_plan`, `plan_verification`, `AdmittedVerification`, `admit_verification`, `verify_admitted`, `verify_approved`, `verify`, `verify_current_payload` | Explicit guest verification, reviewed lifecycle policy, consumed admission and operation-specific restoration guards. Keep fresh-review and interrupted-verification paths distinct. |
| `workflow/reconciliation.rs` | `reconcile_observed`, `reconcile_with_graphics` | Check already-achieved outcomes and safely complete the recorded lifecycle. Preserve its independent preparation, host/GPU/payload and journal proof requirements. Reconciliation must not call Apply to repeat effects. |
| `workflow/guest_check.rs` | `check_guest`, `record_restored_check_failure`, `restore_guest_power` | Small shared primitives for guest checks, outcome classification, restored-negative evidence and graceful power restoration/readback. Operation-specific authorization and restoration guards remain in their callers. |
| `workflow/test_support.rs` | Shared `Fake` backend/implementation and fixture builders such as `fake`, `target`, `extended_target`, `recovery_record`, plus genuinely shared test trace helpers | Test-only support with the smallest necessary visibility. Keep specialized doubles, such as the multi-VM backend, beside their scenario unless actually reused. |
| `workflow/tests.rs` | Existing tests that span planning/execution, verification/reconciliation/repair/removal, reenrollment or other-VM preservation | Workflow-level integration regressions using isolated doubles. Keep existing scenario names and assertions; module-qualified test paths may change. |

## Dependencies and interaction

Use explicit imports, not broad `use super::*` throughout production modules.

1. All operation modules use the root's `Backend` and `OperationResult` plus existing model types.
2. Planning supplies its internal `Decision` and plan-construction functions to Apply. These internals are visible only within the workflow module family; they do not become public caller contracts.
3. Apply, Verify and reconciliation use the journal module and shared guest-check primitives. The helpers do not depend on the higher-level operation modules.
4. Journal persistence uses `Backend`; guest-check classification uses existing reporting types. Neither module imports the native runner, worker, GUI, credential or transport implementation.
5. The worker continues to own enrollment checks, authentication, the operation lock, protected recovery receipts, configuration publication and audit. Native adapters and guest-file/receipt hashing remain in their existing modules.

The dependency direction is operations toward shared contracts/helpers. Verify and reconciliation share primitives without depending on each other's operation implementation. Do not introduce a generic operation engine, new backend traits, service containers, new dependencies or a general-purpose `utils.rs` module.

Use private functions for local helpers and `pub(super)` for sibling access where necessary. Preserve current `pub` and `pub(crate)` entry points through explicit root re-exports. Admission-token fields remain private to their owning operation module; tokens remain non-serializable and consumed in the same authenticated backend context.

## Implementation sequence for the AI agent

Complete this as one cohesive refactor when implementation is requested. The stages below are work steps, not authorization to continue into unrelated backlog items.

### 1. Establish the source baseline

- Read root guidance and applicable Rust, implementation and security/recovery guidance.
- Inspect `git status` and relevant diffs. Preserve existing review fixes and unrelated edits/deletions.
- Inventory all exported workflow symbols, Serde contracts, internal helpers and tests. Trace their callers in the worker, runner, GUI model/controller, configuration store and enrollment code.
- Record pending validation honestly. If a pre-refactor check is separately requested, record any existing failure before moving source; do not silently combine a behavioral fix with extraction.

### 2. Extract the shared state

- Move journal types and helpers to `journal.rs` without changing fields, defaults, serialized enum values, eligibility logic, persistence order or messages.
- Keep `Backend` and `OperationResult` in the root.
- Add explicit root re-exports so callers keep their current paths.

### 3. Extract planning and Apply together

- Move preview/plan types, decision gathering and review construction to `planning.rs`.
- Move Apply admission and execution to `apply.rs`.
- Expose only the internal decision fields/functions Apply actually needs, limited to the workflow family.
- Preserve shared decision computation for preview and execution; do not create separate planners for reinstall or removal.

### 4. Extract guest checks, Verify and reconciliation

- Move shared primitives to `guest_check.rs` and update explicit imports.
- Move Verify admission and execution together to `verification.rs`, preserving private token construction and fresh mutable-state checks.
- Move reconciliation to `reconciliation.rs`, preserving its distinct proof and restoration requirements.
- Keep restoration guards with each operation. Similar-looking branches can authorize different effects; do not merge them during mechanical extraction.

### 5. Relocate existing tests

- Move journal/schema tests beside journal state, planning tests beside planning, Apply/removal/reinstall tests beside Apply, and Verify/reconciliation tests beside their respective operations.
- Retain multi-operation and multi-VM scenarios in `workflow/tests.rs`.
- Move only reusable fixtures to `test_support.rs`; compile all fixtures and doubles only for tests.
- Account for every starting test. Preserve scenario names, assertions, failure injection and effect-order expectations. Add coverage only for a real extraction gap.

### 6. Finish the structural review

- Inspect the diff for accidentally omitted branches, changed expressions, reordered effects, broadened visibility or altered persistence/readback boundaries.
- Initially keep operation bodies intact. Extract short, private helpers only where they clarify a meaningful stage without changing its inputs, errors or order.
- Update the owning architecture/source-index pages to describe the implemented layout. Update this plan and the roadmap's implementation/validation status separately.
- Follow the existing validation timing. Stop after this refactor; report actual changes, checks, outstanding failures and untested behavior.

## Behavior that must remain unchanged

- Existing public and crate-visible entry points and their signatures; plan, journal and receipt serialization, including backward-readable defaults.
- Fresh exact-target admission, consumed in-process tokens, authenticated payload reuse and independent final checks.
- Ordered effects and journal writes, including before-effect invalidation, guest-check outcomes, final-off requirements and ownership acquisition/release.
- Ordinary changes without guest work; explicit same-digest reinstall with preparation/verification; host-only removal independent of guest setup health.
- Failed or interrupted verification, restored-negative evidence, historical rendering timestamps and payload-mismatch preparation invalidation.
- Refused shutdown, externally stopped guests, changed/unavailable observations and persistence/readback failure retaining their existing recovery behavior.
- Reconciliation without effect replay; save-only publication remaining a worker/store responsibility.
- Preservation of other VMs, settings, authority, guest files and existing deletion/publication boundaries.

Do not use this refactor to change policy, error text, serialized equality checks, schemas, allocation semantics, credential flows or the UI. A newly discovered defect or material architectural uncertainty should be reported separately and clarified before changing behavior.

## Acceptance and validation

Source acceptance requires the target responsibility split, a small root entry point, preserved caller paths and contracts, no production dependency on test support, and no lost operation branch or regression scenario. File size alone is not acceptance.

At the agreed automated checkpoint, use the existing [testing gate](TESTING.md#planned-automated-gate) after inspecting its current commands and effects. Check all-target compilation/test discovery, execute applicable isolated regressions, and run the required formatting/lint/build/documentation checks. Do not run builds/tests after each extraction stage merely because source moved.

Compare the starting and final test inventories: the reviewed starting tree has 74 workflow tests, but a fresh inventory takes precedence if source has changed. Test module paths may change; update any actual filter consumers found during implementation rather than dropping tests to preserve a count. Verify schema/plan round trips and ordered-effect/failure assertions, including the APP-017 through APP-022 workstream regressions.

Document failures as pre-existing only when baseline evidence supports that claim. Passing source checks or compilation alone does not establish behavioral acceptance. Source completion may be recorded with validation pending, but no milestone gate closes without executed evidence.

This plan requires no live GUI launch, credentials, guest access, protected writes, VM/GPU effects or installed-artifact updates. Such work remains subject to separately prepared authorization under root guidance.
