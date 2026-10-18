# Architecture review and simplification

Reviewed: 18 October 2026. Source baseline: `10016629350876d7eb785fa5e8903d53af10f79e` with a clean working tree.

## Purpose and requirements

Repository-wide architectural and code review followed by justified source simplification. Preserve the capable application, persistent formats and supported workflows. Source size is a measurement, not this review's optimisation target. The separate [roadmap](roadmap.md) size goal remains open; this review does not establish its feasibility or authorise live qualification.

The application manages existing Hyper-V VM/GPU pairs. It must distinguish requested configuration, current host observations, guest preparation/check history, operator authority and unresolved operation evidence. These are different facts, not interchangeable copies of application state.

| User task | Required sequence and result |
| --- | --- |
| Open, Refresh and Poll | Read inventory, saved intent, authority and recovery; preserve drafts and stable identity; reject stale deliveries and wait for exact backend release. |
| Install or enrol | Authenticate the fixed package/operator and exact pair; persist and read back authority separately from GPU changes. Preserve existing pairs on update. |
| First setup or explicit Reinstall | Authenticate current files; prepare with GPU detached; stop, configure and attach; check guest files/graphics; finish Off and publish only the independently confirmed intent. |
| Ordinary allocation/re-enable | Authenticate eligible preparation and current facts; write only required host categories; finish Off, independently confirm and save. Omitted categories cause no write. |
| Disable | Gracefully stop, detach and restore only attributable settings; retain guest files and saved history; confirm and publish disabled intent without guest credentials. |
| Verify | Check current files/device/rendering; restore permitted power and record the check without publishing GPU intent. An external stop never permits a restart. |
| Reconcile or remove partial setup | Follow exact durable evidence and fresh facts; never replay Apply. Preserve uncertainty and other VMs' records. |
| Retry saving | Revalidate the bound verified outcome and publish that exact intent without repeating GPU work. |
| Delete profile or import | Perform reviewed record operations, with resumable progress and conflict checks; neither grants GPU effect authority. |
| Credentials and diagnostics | Prompt locally, save only on explicit opt-in, preserve cancellation and redact adapter prose. Display metadata never grants authority. |

[Architecture](ARCHITECTURE.md), [configuration](CONFIGURATION.md), [security](SECURITY.md) and the [agreed workflow](../README.md#agreed-workflow) own the product contracts. This document records findings and decisions rather than duplicating their detailed requirements.

## Review coverage and findings

The review inspected the repository inventory/manifests and searched maintained source for callers, repeated validation/comparisons, retired routes and compatibility scaffolding. It traced domain entrypoints and the affected failure paths; it is not a claim of exhaustive formal verification of every line or native runtime behaviour.

| Area and evidence | Independent finding and disposition |
| --- | --- |
| GUI and CLI dispatch: `main.rs`, `gui/live`, `gui/live_state`, shared Slint dialogs | Both already reach prepared execution. Foreground/poll admission, UI-thread delivery and draft/selection conflicts solve real races. Keep them. Completion still wraps a complete protected bundle in duplicate saved/recovery fields and an optional observation; only tests construct the incomplete variant. Remove that representation and fallback. |
| Shared workflow: `workflow/{planning,steps,apply,verification,reconciliation,reset,journal}` | Configure preview/execution already use one typed sequence; Apply/Verify already share worker receipt lifecycle. Earlier review proposals describing parallel production engines are obsolete. Keep operation-specific power, ownership and non-publication rules. |
| Exact record checks: workflow, configuration, worker recovery, reinstall and provider completion | Multiple helpers serialize already typed records to JSON solely for equality. Their fields have ordinary exact equality semantics. Replace these with derived typed equality and remove helper/error plumbing; retain actual reads and failure messages. Keep operational comparisons, serialization for hashes/wire/storage, and strict decoding. |
| VM identity: `model`, GUI selection, protected configuration, runner recovery, guest staging | Several implementations repeat the same 36-byte GUID grammar. Put syntax in the existing model and retain each caller's uppercase/normalisation policy. Do not broaden credential-vault key acceptance or change persisted filenames. |
| Protected worker/runner: preparation, dispatch, policy, pairing, storage, setup | Exact process/operator authentication, common lock, consumed admission, write-ahead receipts and final confirmation are distinct safeguards. Repeated reads at different boundaries are not automatically redundant. No new workflow framework or privilege path is justified. |
| Native Hyper-V/WMI/COM, pipes and process supervision | Batched reads share connections, while effects fetch fresh objects. Native provider holds cover jobs whose effects can outlive replies. Keep provider/workflow recovery separate, exact scheduler release, bounded waits and narrow FFI ownership. Removing them would require a different proven contract. |
| Guest/payload/transfer/probe/trust | Host authentication, guest installed-file checking and rendering establish different facts. Hash loops have different deadline/error/length policies; do not merge them merely because each uses SHA-256. Keep streaming bounds, ordering, signatures/catalog membership and independent final authentication. Actual transfer limit is 8 MiB; stale prose claiming 16 MiB is corrected. |
| Configuration/enrollment/import/removal and display caches | Schema-2/3 intent, legacy receipt handling and resumable record deletion remain compatibility requirements. Display caches and selection preferences are intentionally separate from protected records. No record migration or deletion is needed for the selected changes. |
| Diagnostics/reporting/work counters | Progress completion cannot certify terminal success; safe error envelopes and opt-in protected tracing remain necessary. No measured speedup follows from fewer conversions. |
| Build/text tooling, resources, dependencies, vendor patch and probes | Direct dependencies have production/build callers; no dependency removal is established. Catalogue contracts deliberately separate allowed keys/types/native bounds from editable wording. Vendored repaint fixes and inert probe binaries are not obsolete merely because they are outside the main Rust engine. Leave these interfaces and assets intact. |

## Prioritised source changes implemented

These findings and priorities were recorded before implementation. All four changes are implemented, reusing existing owners without a replacement framework.

1. **Exact typed equality:** remove JSON-only comparison helpers across the record lifecycle. Add field-drift regressions and preserve every read, transition and persisted schema. Low structural risk; broad removal of repeated plumbing.
2. **One GUID syntax owner:** reuse the existing model from target validation, preferences, protected store, receipts and staging. Preserve caller-specific canonical case rules and rejection boundaries. Low risk with malformed/uppercase/path tests.
3. **One completion observation:** use the existing protected `Bundle` directly. Remove duplicate cloned state, the uncalled incomplete-readback branch and repeat validation already performed by its display-merge method. Adapt useful tests to actual complete observations, including missing/wrong/stale identity and failed readback. Moderate integration risk because drafts, recovery and publication must remain distinct.
4. **Accurate current documentation:** replace obsolete route-count descriptions and correct the transfer limit without changing transport behaviour. Keep historical gate evidence explicitly historical.

Removed six JSON-only equality helpers/closures, the GUI-only GUID checker, duplicated GUID grammar and the `OperationReadback` wrapper/fallback. Added derived equality to existing record types, one model syntax function and four regressions. Reused protected observations, operational fact views, domain backends, persistence, UI controls and existing failure tests. No new module, dependency, workflow layer, locale or user-facing feature was introduced.

## Compatibility and unresolved concerns

No persistent configuration, journal, enrollment, recovery, transport frame or GUI text schema needs to change. Exact record equality includes display fields where the previous serialized comparison did; operational VM/GPU comparisons continue to exclude only their established presentation fields. The public reserved OS hook remains documented compatibility surface; no evidence establishes that deleting it is safe for consumers.

The selected completion change removes an internal representation that production never creates. A failed or incomplete protected observation still blocks completion and new review. Typed records cannot replace fresh independent reads or prove successful external effects.

Remaining architectural cost is concentrated in frontend coordination and the native/guest/security footprint. Broader replacement would need concrete equivalence evidence for cancellation/release, interruption, lost replies, record migration and external interference. This review finds no justification to weaken those contracts or to claim the separate 10,000-line goal feasible. Existing roadmap concerns remain open; no speculative backlog cards are added.

## Measurements and verification

Baseline uses tracked plus non-ignored untracked existing files, physical lines including comments/blanks, and the existing temporary `syn` source-span audit to separate inline Rust test items and external test-only module trees. Generated output, third-party `vendor/`, CUDA upstream source, docs and inert assets are excluded from handwritten source. Mixed files appear in production file counts and also contribute separately reported test lines; those counts are not additive unique-file totals.

| Category | Before files | Before lines | After files | After lines |
| --- | ---: | ---: | ---: | ---: |
| Product Rust, excluding Rust tests | 118 | 26,312 | 118 | 26,202 |
| Guest PowerShell source | 1 | 381 | 1 | 381 |
| Slint GUI definitions | 13 | 1,605 | 13 | 1,605 |
| Rust tests/helpers under `src/` (23 entirely test-only files, plus 70 mixed files) | 93 | 17,551 | 93 | 17,688 |
| Integration Rust tests | 3 | 1,642 | 3 | 1,642 |
| Build/developer/text/probe tooling excluding tests | 11 | 1,678 | 11 | 1,678 |
| Tool Rust tests/helpers | 1 | 662 | 1 | 662 |
| Python validation/rebuild tests | 3 | 312 | 3 | 312 |
| Configuration/manifests/active text/contract TOML or JSON | 24 | 11,492 | 24 | 11,492 |

Counted `src/` changes from **28,298 to 28,188** program lines, excluding Rust tests; physical `src/` changes from **45,849 to 45,876** lines. Production decreases by 110 lines while useful Rust tests/helpers increase by 137. No program logic moved to tests, generated output, data or another directory. The source-span tables are `local/evidence/architecture-review-before.tsv` and `architecture-review-after.tsv`.

Active catalogue data is 5,497 lines in 10 files; text-contract data is 5,627 lines in two files, included in the configuration/data row. Excluded unchanged third-party source: 9,194 lines in 33 vendored source files and 267 lines in two CUDA upstream files. These figures are source-extension inventories, not all third-party text/assets. Generated Cargo output is excluded.

The historical preview checkpoint reported nine fewer Rust test lines than this clean baseline; this table uses a fresh source-span inventory of the actual revision rather than carrying that old test total forward.

## Verification results and limits

The complete `powershell -NoProfile -File scripts/testing/check.ps1` gate passed all ten stages on 18 October 2026 for the baseline revision plus this working-tree diff: catalogue validation/failure tests, copy checks, documentation, formatting, non-hardware tests, strict Clippy, four binary builds and API documentation. Product Rust tests: **477 passed**; separate catalogue tests: **12 passed**. Evidence: `local/evidence/m1-gate-20261018T155728482Z/report.json`. All **217** recorded source/build/test input hashes still matched after the gate; `input-readback.json` records the comparison. Final documentation checks passed separately: `python -B scripts/testing/check-docs.py` checked 36 Markdown files and four task cards with zero errors; `git diff --check` passed.

New regressions reject drift in all 14 journal fields before prepared Apply/Verify effects, reject changed receipt bindings during migration/retirement, reject malformed GUIDs while preserving uppercase input, and reject invalid completion scope/revision/identity without replacing loaded state or releasing recovery. Existing tests retain lifecycle ordering, failed effects/reads, cancellation/release, payload drift, exact publication, save-only, negative completion, per-VM holds, resumable reinstall/deletion and newer drafts. Tests formerly using incomplete completion inputs now use the same validated bundle as production; a fresh observation can be known while an operation outcome remains uncertain.

Source tracing and the diff review retain the sequence, comparison fields, independent reads, case policies and persisted schemas. Tests support those findings but do not establish native external equivalence. No measured speedup is claimed. No live GUI launch, protected installation, credentials, guest access, VM/GPU changes, driver installation or host restart was performed. The changed candidate is not installed or live-qualified; the separate roadmap feasibility and acceptance gates remain open.
