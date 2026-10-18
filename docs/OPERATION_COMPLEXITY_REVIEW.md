# Architecture review and simplification

Reviewed: 18 October 2026. Source baseline: `10016629350876d7eb785fa5e8903d53af10f79e` with a clean working tree.

## Purpose and requirements

Repository-wide architectural and code review followed by justified source simplification. Preserve the capable application, persistent formats and supported workflows. Source size is a measurement, not this review's optimisation target. The [roadmap](roadmap.md) follows this architectural objective without a numerical size target. This review does not authorise live qualification.

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

Remaining architectural cost is concentrated in frontend coordination and the native/guest/security footprint. Broader replacement would need concrete equivalence evidence for cancellation/release, interruption, lost replies, record migration and external interference. The user has withdrawn the numerical line target; source size remains a descriptive measurement. Simplification must preserve those contracts. Existing roadmap concerns remain open; no speculative backlog cards are added.

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

Source tracing and the diff review retain the sequence, comparison fields, independent reads, case policies and persisted schemas. Tests support those findings but do not establish native external equivalence. No measured speedup is claimed. No live GUI launch, protected installation, credentials, guest access, VM/GPU changes, driver installation or host restart was performed. The changed candidate is not installed or live-qualified; remaining roadmap simplification work and live acceptance remain separate.

## Protected record-read consolidation

A subsequent caller review at base `2d52b0b` found seven copies of the same optional protected-file read: setup and pair recovery, configuration-import recovery, operation recovery, profile history, managed journals and saved configuration. They all inspect metadata, accept only NotFound as absence, reject non-files, verify ownership/permissions, and read at most the input limit plus one byte. Those mechanics now live in `security::read_optional`; each caller keeps its existing size limit, exact diagnostic wording and domain decoding/validation. This removes the repeated I/O implementations without a new storage framework, module, dependency or record format.

The reinstall receipt, enrollment policy, audit-retention and guest-receipt readers remain separate: they have different file-type, open-error, single-link, sharing or typed-failure contracts. No obsolete public API or compatibility record was established as safe to delete. Recovery admission, publication, independent readbacks and the native effect sequence are unchanged.

Regression coverage checks exact and excessive input lengths at both retained limits, bounded consumption of oversized streams, failure after partial input, missing/non-file distinction and refusal of operator-writable files before reading. On 18 October 2026, the complete non-hardware gate passed all ten stages for this follow-up: **480 product Rust tests**, **12 catalogue tests**, copy/docs/format checks, strict Clippy, four binary builds and API documentation. Evidence: `local/evidence/m1-gate-20261018T161417144Z/report.json`. Earlier attempts stopped on formatting and an unused import; both were corrected before the complete passing run. No live application or protected-state work is part of this follow-up.

## Guest progress consolidation

A source-only OPT-009 follow-up at base `247591a` traced `ObservedBackend::prepare` and `ObservedBackend::verify` in `worker/progress.rs`. Both separately marked effects, delivered the parent start, ran diagnostic instrumentation and backend phase callbacks, finished the current phase, attempted the parent finish and combined backend/delivery errors. Their backend operations and return values differ; their progress lifecycle does not. One private `guest_effect` method now owns those mechanics, using the existing phase and failure helpers. The two Backend entrypoints retain their fixed stage names and distinct native calls. No new module, protocol, persisted format or execution engine is introduced.

The order remains receipt before progress before guest work. Failed receipt/start delivery prevents dispatch. Parent terminal delivery is attempted even after phase delivery fails; the first delivery error and any backend error are retained. A successful backend result with failed terminal delivery remains uncertain. Diagnostics still wrap backend work, and structured failures receive the original preparation/verification stage. Existing phase transitions, final authentication and operation receipt/publication owners are unchanged.

Two added isolated regressions cover both Backend entrypoints and the shared phased lifecycle: receipt refusal, start refusal, successful return values, phase-start failure, backend failure, phase/parent terminal failures and simultaneous terminal failures. Existing phase-order and operation-lifecycle tests remain useful and are retained. Regression execution and the complete gate were **Not run** at this source-only checkpoint; the subsequent [integrated checkpoint](ENGINEERING.md#frontend-and-progress-integration-checkpoint) qualifies this follow-up.

The same caller review found CLI-only native recovery, import continuation and legacy-install handling in `main.rs` and the runner. Although the CLI is defunct by product decision, wholesale dispatch deletion would remove those remaining record-resolution routes. The subsequent caller review below resolves the scope of ordinary frontend deletion while retaining those record contracts. The progress follow-up does not close OPT-009 or authorize OPT-010, installation or live acceptance.

## Frontend caller and recovery review

The next source-only OPT-009 follow-up reviewed remaining frontend callers against the defunct-CLI decision. The supported GUI already owns Configure/setup/Disable/Reinstall through `gui/live/apply.rs` and `worker::apply_prepared`; Verify uses `gui/live/verification.rs` and `worker::verify_prepared`. Both consume worker-owned preparation. Save-only and workflow reconciliation join the existing `complete_operation` in `gui/live/state_access.rs`. There is no second production GPU engine to replace. Deleting distinct recovery, admission or publication boundaries would not simplify a duplicate engine.

| Caller or contract | Source-backed disposition |
| --- | --- |
| `main.rs` inventory, plan, status, apply, enable, disable, verify, credentials and forget | Remove the duplicate ordinary frontend dispatch, per-target iteration, option scanner and frontend credential/power approval branches. The GUI retains its existing supported actions and protected execution. |
| `main.rs` native-review/native-reconcile; runner and `windows_wmi/recovery.rs` | Retain administrator review and separate token approval. The GUI does not perform terminal provider-job correlation; stored evidence cannot be discarded or retired through workflow reconciliation. |
| `main.rs` install; runner `installation.rs::pending_setup` and `reinstall.rs` | Retain administrator installation/update for legacy records. GUI setup explicitly rejects legacy CLI markers because they lack its original binding. Preserve the original installer/reinstall validation and records. |
| `main.rs` import/import-reconcile; `gui/live/recovery.rs::review_records` | Retain the complete-bundle import and exact recorded continuation. The GUI presents the pending import but has no import execution callback. Neither import route grants enrollment or GPU authority. |
| `main.rs` save-only/reconcile; GUI Apply/recovery callbacks | Retain explicit recorded-outcome administration without introducing another Configure route. Keep verified-unsaved publication, credential-free negative completion, explicit permitted power and uncertainty retention. |
| Public runner Operation/Request and audit records | Retain the exported API and enum values. `audit.rs::AuditRecord` serializes Operation, and retention decodes historical records. Removing Status/Plan variants merely because CLI callers are gone would also change stored-audit compatibility. No format migration is needed for frontend deletion. |
| Public reporting helpers and local console credential prompt | Retain exported helpers pending a consumer contract, and keep the credential prompt used by retained reconciliation. No dependency removal is justified by this dispatch change. |

`main.rs` now rejects removed ordinary commands before configuration reads, runner calls or GUI startup. Its two remaining configuration commands accept only `COMMAND --config FILE`; target selection, repeated options and power flags are rejected before file access. Import still checks every absent destination and expected revision, and configuration input remains bounded to 1 MiB. Fixed internal setup/worker modes and argument-free GUI startup are unchanged. Existing administrator command spellings, native review tokens, recovery operation identities and effect approval checks are retained.

Two added dispatch regressions cover complete-bundle parsing and forbidden ordinary routes/options through the production validation helper, without filesystem or protected effects. Existing startup-dispatch coverage is retained. Builds, regression execution and the complete gate subsequently passed at the [integrated checkpoint](ENGINEERING.md#frontend-and-progress-integration-checkpoint). Earlier candidate evidence remains historical.

The integrated source follow-ups are validated. Next is to assess concrete foreground request/completion duplication under OPT-010. Keep pair authority, profile deletion, imports and native provider recovery as distinct domain handlers. Replacing those handlers with one configurable workflow or removing legacy records has no demonstrated maintenance benefit. A complete GUI replacement for administrator-only record resolution would be new product work, not a prerequisite for this bounded ordinary frontend deletion.
