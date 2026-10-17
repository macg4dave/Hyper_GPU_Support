# Optimization backlog

Updated 17 October 2026. These tasks implement the [optimization roadmap](roadmap.md). The earlier release backlog was closed by user confirmation on 15 October; the optimization tasks are new work, not reopened QA cases. OPT-001 through OPT-006 and the user-selected OPT-007 16 MiB scope passed the complete non-hardware gate and protected installation on 17 October 2026. Live acceptance and performance measurements remain pending. Source work does not authorize installation or live effects.

## Next action

The next task is **OPT-008 live qualification**, after exact targets/effects are prepared and authorized. The [integrated checkpoint](ENGINEERING.md#optimization-validation-checkpoint) passed 471 product tests and 12 catalogue tests, strict lint/build/docs, and protected installation with matching artifact hashes and preserved state. R1/R2/R3 automated scope is established; installed GUI, native measurements, VM/GPU/guest acceptance and 16 MiB throughput/memory qualification remain pending. Include the selected OPT-007 chunk size; broader transport redesign stays deferred.

OPT-001 through OPT-006 have no hard source prerequisites. Work on their independent parts can be prepared without waiting for live measurements or another stage to close. Coordinate shared-file changes within the requested batch. OPT-007 is an optional measurement-driven extension; OPT-008 is the required final integration/acceptance task.

## Task register

Priority expresses delivery order/value, not defect severity. Dependencies are hard completion prerequisites; preferred sequencing and shared-file coordination are described separately. Source completion and automated/installed/live validation follow [engineering status](ENGINEERING.md#status-and-evidence).

| ID | Stage | Task | Priority | Implementation | Dependencies | Validation |
| --- | --- | --- | --- | --- | --- | --- |
| OPT-001 | R0 | Baseline, counters and comparison contract | High | Done (source) | None | Passed (non-hardware); live pending |
| OPT-002 | R1 | Protected read pipeline and GUI coordination | High | Done (source) | None | Passed (non-hardware); live pending |
| OPT-003 | R2 | Setup lifecycle and changed-resource writes | High | Done (source) | None | Passed (non-hardware); live pending |
| OPT-004 | R2 | Payload admission and focused worker handlers | High | Done (source) | None | Passed (non-hardware); live pending |
| OPT-005 | R3 | Validated helper reuse and transport measurement | Medium | Done (source) | None | Passed (non-hardware); live pending |
| OPT-006 | R3 | Completed audit and staging retention | Medium | Done (source) | None | Passed (non-hardware); live pending |
| OPT-007 | R3 extension | Conditional bounded transfer optimization | User selected 16 MiB | Done (source: chunk size) | OPT-005 | Passed (non-hardware); measurements pending |
| OPT-008 | R4 | Integrated candidate validation and acceptance | Required gate | In progress | OPT-001, OPT-002, OPT-003, OPT-004, OPT-005, OPT-006, OPT-007 | Passed (gate/install); live pending |

OPT-008 has passed its integrated non-hardware and protected-install portions; final acceptance still requires authorized live evidence and the user's decision. The user selected the narrow OPT-007 chunk-size change for the candidate, so its changes and validation are part of OPT-008. Broader transfer redesign remains deferred.

## Tasks

### OPT-001 - Baseline, counters and comparison contract

Dependencies: None.
Stage: R0. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** reproducible before/after work counts and a small measurement contract shared by every optimization, without requiring a new benchmark framework or live access before source work.

**Scope:** [diagnostics](../src/diagnostics.rs), existing backend doubles in [workflow test support](../src/workflow/test_support.rs), provider/runner/guest instrumentation at the affected boundaries. Reuse existing opt-in diagnostics and ignored local evidence conventions.

Acceptance:

- Capture original work counts before each affected path changes: provider queries/connections, artifact hashes and payload passes, native mutations by category, helper/payload bytes, power transitions and completion reads. Include startup/poll, setup from Off/Running, ordinary Apply, disable, Verify, repair and save-only; document path eligibility.
- Add only bounded, secret-free aggregate counters/timing needed by these tasks. Preserve existing diagnostic durability and account for its overhead; avoid per-file logging expansion or an always-on telemetry service.
- Define comparable run conditions and the workflow/failure matrix for OPT-008. Counts obtained through isolated doubles are identified as such; current machine timings require separately authorized collection.
- Add meaningful isolated checks for counter attribution, terminal outcomes and unchanged effects when diagnostics are disabled. Capture baseline fixture expectations before updating them for optimized behavior.
- Source completion requires instrumentation, reproducible isolated baseline counts and the comparison procedure. Missing live timing stays pending under OPT-008 and does not hold this source task open.

Integration: establish counter names/attribution at the start of the first batch; other tasks can prepare their own fixtures before this card finishes. Run the full non-hardware gate at the agreed batch checkpoint.

Source delivered 17 October 2026: fixed aggregate counters on existing opt-in diagnostic summaries; native provider/hash/mutation and acknowledged transfer attribution; original workflow call/effect fixtures; and the [comparison/failure procedure](ENGINEERING.md#optimization-measurements). Isolated fixture expectations passed at the [integrated checkpoint](ENGINEERING.md#optimization-validation-checkpoint), including builds/Rust tests and the complete gate. Native timing and installed GUI/guest/VM/GPU acceptance remain pending under OPT-008.

### OPT-002 - Protected read pipeline and GUI coordination

Dependencies: None.
Stage: R1. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** cheap, bounded routine status updates and fresh operational checks, with one coherent completion read and foreground priority. Review findings 1, 2, 5 and 7 are implemented together because they share the same read contracts.

**Scope:** [native Hyper-V reads](../src/windows_hyperv.rs), [WMI sessions](../src/windows_wmi.rs), [native backend](../src/worker/runner/backend.rs), [runner IPC](../src/worker/runner/ipc.rs), [Apply facts](../src/workflow/apply/facts.rs), [GUI source](../src/gui/live/source.rs), [refresh](../src/gui/live/refresh.rs), [polling](../src/gui/live/polling.rs) and [background coordination](../src/gui/live/background.rs). Adapt other workflow consumers to the same facts contract where required.

Acceptance:

- Separate operational identity/capability/driver facts from optional display metadata. Failed optional DXGI/KVP reads cannot by themselves invalidate operational equality; unavailable required facts still fail closed.
- Batch inventory relationships and reuse operation-scoped connections. Re-fetch mutable provider objects at effect boundaries; retain fresh VM/GPU/journal checks, exact adapter binding and partial-error attribution.
- Implement a bounded lightweight polling path and explicit full Refresh, with defined invalidation for topology, display metadata and package observations. Avoid redundant availability hashing; any attestation reuse has protected-handle/file identity and enrollment-revision binding, while worker admission remains authoritative.
- Coordinate foreground requests with in-flight polls before protected effect admission. Propagate cancellation and an operation-wide deadline through read-only work, retaining admission until confirmed backend release. Do not cancel or retry effects.
- Consolidate post-operation configuration, recovery, authority and selected-target observations. Include revisions/provenance, preserve independent publication readback and enrollment continuation, and refresh unrelated inventory separately.
- Tests cover query/hash-count reduction, added/removed/reordered VMs/GPUs, optional/required read failure, drafts/selection/conflicts, stale poll suppression, foreground races, cancellation/deadline expiry, disappearing receipts and failed completion readback.
- Explicit Refresh, discovery before installation, VM-scoped/global recovery blockers and final result truthfulness retain their existing authority boundaries.

Integration: agree the operational facts/status/observation bundle first, then deliver its native and GUI consumers as one task. OPT-001 supplies comparison counts but is not a prerequisite for independent implementation.

Implemented 17 October 2026: operational equality excludes optional names/capacity/OS metadata; operation-scoped connections re-query current objects; inventory batches exact provider relationships. Full Refresh obtains display metadata; polling retains it only for matching identities and driver versions. One locked observation supplies current authority, saved intent, recovery and selected completion facts. Foreground requests cancel polls and wait for confirmed release of the exact runner instance; read budgets/cancellation reach provider, hash and pipe waits without cancelling effects. Authenticated reads hash the current enrolled artifacts once under the common lock, and readiness comes from that response without a separate GUI hash pass. Completion preserves drafts/conflicts, marks unrelated rows historical until a later poll, and never substitutes an old selected row for a missing current read. See [architecture](ARCHITECTURE.md#protected-read-pipeline) and [security](SECURITY.md#protected-read-admission).

Regression source covers operational/display drift, fixed batch-query counts and relationship failures, artifact-authentication counts/failure, topology/display invalidation, cancellation/deadline/release tracking, stale poll races, disconnected clients, selected completion failures and disappearing save receipts. Static formatting, UI-copy and documentation/whitespace checks pass. Compilation, regressions, the complete non-hardware gate and protected installation passed at the integrated checkpoint. Native measurements and installed/live acceptance remain pending.

### OPT-003 - Setup lifecycle and changed-resource writes

Dependencies: None.
Stage: R2. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** remove an unnecessary setup power cycle on eligible unattached VMs and skip proven unchanged resource writes. Review findings 3 and 6 share workflow planning, execution and effect-order tests.

**Scope:** [planning](../src/workflow/planning.rs), [Apply](../src/workflow/apply.rs), [stage facts](../src/workflow/apply/facts.rs), [native allocation](../src/windows_hyperv.rs) and existing lifecycle/GUI progress consumers.

Acceptance:

- Plan and execute OS identification followed by preparation within one running, unattached boot. An initially Off VM targets two setup boot cycles instead of three; a running unattached VM avoids a shutdown/restart solely between identification and preparation.
- Keep already-attached/stale-driver cases explicit. No GPU/installation writes precede supported current OS evidence; retain preparation-session/guest-local checks, journaling before starts/writes, graceful restoration and the agreed final power.
- Compute a per-category write set for retained adapters. A VRAM-only change does not rewrite matching Compute/Encode/Decode requests. Newly created/recreated adapters receive every specified category; omitted categories retain their current contract.
- Shared previews/progress and executed effects agree on lifecycle and resources. Preserve settings ownership, disable behavior, current capability checks and exact independent readback.
- Regression/failure tests cover Off/Running and attached/unattached setup, unsupported/unknown OS, uncertain start, failed preparation/shutdown/restoration, external state drift, retained/recreated adapter defaults and partial category failure.
- Count the removed transitions/writes and prove unchanged forbidden effects. Do not combine native mutations into a new recovery unit merely to reduce calls.

Integration: deliver with OPT-004 as a requested R2 batch to update shared workflow/worker tests once. Existing backend contracts suffice to begin before OPT-002 closes; coordinate any later facts-type migration.

Implemented 17 October 2026 with OPT-004. Successful OS identification keeps an unattached guest running for preparation; Off setup requests two starts/two shutdowns, and Running setup one start/two shutdowns. Attached cases retain the shutdown/detachment boundary. Identification refusal/uncertainty still uses guarded restoration, and actual preparation/guest-local eligibility checks remain. Retained adapters write only differing requested categories; recreated adapters write all specified categories. Previews, the authenticated progress checklist and execution use the same decisions. Regression source covers these reductions, required/omitted categories, existing failure/restoration cases and prepared admission drift. Candidate comparison assertions explicitly replace the affected original counts; the original table remains in engineering documentation.

Validation: static formatting, copy and documentation/whitespace checks pass. Compilation, Rust regressions, the complete non-hardware gate and protected installation passed at the integrated checkpoint; installed/live acceptance remains pending. No transition, timing or GPU acceptance is claimed from these source expectations.

### OPT-004 - Payload admission and focused worker handlers

Dependencies: None.
Stage: R2. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** reduce duplicate complete host payload processing while keeping independently authenticated execution and final confirmation. Simplify the worker only where needed for this change, covering review findings 4 and 10 together.

**Scope:** [GUI preparation](../src/gui/live/review.rs), [CLI effect dispatch](../src/main.rs), [worker](../src/worker/mod.rs), [native payload adapter](../src/worker/runner/backend.rs), [Apply admission](../src/workflow/apply.rs), [Verify admission](../src/workflow/verification.rs), payload/trust and affected protocol callers.

Acceptance:

- Use the existing preliminary-plan mechanism for eligible ordinary CLI effect commands. Explicit CLI plan remains a complete read-only plan; missing/adverse/pending history still forces the appropriate authenticated path.
- Specify the exact request, expected revision, permitted effect scope, credential boundary and in-process payload lifetime before changing the worker handoff. Consolidate expensive preparation/admission in one locked worker session where that binding can be proved.
- Preserve consumed non-serializable admission tokens, current identity/capability/journal checks, signature/catalog membership, guest integrity checks and independent final full payload authentication. No persistent digest cache or driver-version-only shortcut grants authority.
- Demonstrate reduced payload-pass counts on eligible setup/reinstall/Verify paths, targeting three to two without increasing the ordinary GUI path. Payload or scope drift must reject before effects rather than silently escalate guest/power work. Resolve any infeasible path explicitly before claiming completion.
- Extract affected private Apply/Verify handlers and identical publication/finalization helpers as part of the same change. Keep recovery, save-only, import and enrollment domains distinct; no generic command framework or unrelated worker rewrite.
- Test same-version byte changes, changed manifest membership/catalogs, stale request/revision, scope expansion, lost connection, journal/effect admission ordering, final authentication failure and verified-unsaved recovery. Existing four-to-three-pass admission work must not be duplicated.

Integration: combine with OPT-003; agree any completion acknowledgement needed by OPT-006 in this batch. Protocol consumers migrate together, without requiring another work package to close first.

Implemented 17 October 2026: the process-bound worker now prepares an authenticated Apply/Verify plan, binds its exact review revision into audit, waits for a matching session/scope acknowledgement and local credentials, then consumes the retained in-process admission token and manifest under the same common lock. Source handles deny write/delete sharing from hashing through final confirmation, and package membership is re-enumerated before effects. Final authentication remains a separate complete pass. GUI setup/reinstall/Verify and CLI setup/Verify therefore request two complete host passes; ordinary GUI remains two, eligible CLI uses the existing preliminary mechanism and also requests two. Explicit CLI `plan` remains a full read-only pass. Cancellation/disconnect closes the preparation connection and retains admission until exact launched-process exit; effect approval disables read cancellation. Apply/Verify handlers and their common final confirmation/publication code are private focused modules; recovery, save-only, import and enrollment admission remain separate. See [architecture](ARCHITECTURE.md#prepared-worker-execution) and [security](SECURITY.md#prepared-payload-lifetime).

Regression source covers scope/session/revision/credential mismatches, same-version byte replacement locks, changed package/catalog membership, cancelled overlapped reads, final authentication failures, draft binding, checklist continuity and existing verified-unsaved recovery. `Complete` remains a session-bound terminal reply; process exit is release proof, not a client/guest cleanup acknowledgement. OPT-006 supplies separate durable completion/readback criteria and preserves unresolved records. Compilation, regression execution, the complete non-hardware gate and protected installation passed at the integrated checkpoint; installed/live qualification remains pending.

### OPT-005 - Validated helper reuse and transport measurement

Dependencies: None.
Stage: R3. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** avoid transferring/replacing already matching guest helpers and determine whether chunk/remoting overhead warrants OPT-007. Review finding 8's helper optimization is core scope; transfer redesign is conditional.

**Scope:** [guest bridge](../src/guest.rs), [fixed transport](../src/guest_transport.ps1), [transfer monitor](../src/guest_transfer.rs) and isolated production-transport tests.

Acceptance:

- Reuse the guest helper/probe only after fresh protected path, owner, ACL, length and hash checks. Retain the locked-handle integrity check at launch. Missing, mismatched or untrusted leaves take the existing safe replacement/refusal path.
- Matching helpers cause zero helper transfer/publication writes during preparation and Verify; preparation still performs its required installation work and Verify still checks current files and graphics.
- Preserve staged atomic publication, backups on uncertain replacement, exact byte accounting, phase order and receipt validation. Monitor totals reflect only files actually selected for transfer.
- Test matching/missing/changed helpers, unsafe leaf/parent, publication and ACL failure, mixed reuse/replacement, launch-time drift, partial transfer and invalid terminal reply.
- Measure or prepare authorized measurement of session setup, chunk round trips, serialization, file writes, hashing and graphics separately. The current chunk limit is defined by code; increasing it or resource caps is not a default optimization.
- Record a supported decision for OPT-007: proceed with a specific measured bottleneck, or retain the current transport. If live measurement is unavailable, mark the decision pending without delaying helper source completion or the core candidate.

Integration: independent of R1/R2 source completion. Share transport fixtures/counters with OPT-001 and protocol edits with OPT-006; validate once for the requested guest/storage batch.

Source outcome: helpers are selected only after fresh path/owner/ACL/length/hash checks; unsafe leaves/parents refuse replacement. Matching helpers skip transfer/publication, while launch still holds both independently checked files against writing/deletion. The transfer monitor validates one selection before ordered byte acknowledgements and reports only selected files. Existing preparation/Verify receipt and phase requirements remain mandatory. Isolated production-script and monitor regression source covers reuse, missing/changed/mixed helpers, unsafe ownership/writers/links/directories, publication/ACL failure, launch drift, zero-transfer Verify and existing partial/invalid terminal cases. Transport timing preparation is in [engineering](ENGINEERING.md#helper-and-transport-measurement-preparation); OPT-007 now includes the user-selected 16 MiB chunk size, with the existing transport retained and measurements pending. Compilation, regressions and protected installation passed at the integrated checkpoint; live timing remains unrun.

### OPT-006 - Completed audit and staging retention

Dependencies: None.
Stage: R3. Implementation: Done (source). Validation: Passed (non-hardware); live pending.

**Outcome:** bound storage growth from completed discovery audits and obsolete guest staging without discarding recovery evidence. Review finding 9 includes the retention contract and its implementation in one task.

**Scope:** [runner audit/admission](../src/worker/runner/audit.rs), [audit retention](../src/worker/runner/audit/retention.rs), [runner storage](../src/worker/runner/storage.rs), [staging acknowledgements](../src/guest/staging.rs), [worker finalization](../src/worker/dispatch.rs), [guest staging](../src/guest_transport.ps1) and the owning [security](SECURITY.md)/[configuration](CONFIGURATION.md) contracts.

Acceptance:

- Define terminal eligibility, age/count/byte bounds, ownership and scope, preservation rules and cleanup-error behavior before enabling deletion. Record the chosen policy and rationale in the owning handbook; do not infer permission from file age alone.
- Bound completed discovery/status audit retention while preserving required admission/terminal auditing, failed/started/uncertain records, administrator-retained evidence and records referenced by active work.
- Remove guest staging only with a durable host completion/retirement acknowledgement bound to the exact operation, VM, manifest and bundle. Interrupted or unacknowledged preparation is ineligible; installed payloads, receipts, unrelated bundles and unresolved recovery remain untouched.
- Cleanup uses fixed protected paths, exact identities and the relevant locks; reject unsafe files, links and ambiguous references. Make interruption/resume and partial failure safe without replaying VM/GPU work or changing the established operation outcome.
- Test bounds, active references, failed/unknown operations, malformed receipts, duplicate/mismatched acknowledgements, unsafe paths, deletion failure and interruption between cleanup steps.
- Source completion requires deterministic cleanup eligibility/failure coverage. Live cleanup/guest writes remain pending under OPT-008; no existing operational records are removed during source validation.

Integration: policy and eligibility tests can begin immediately. Design the shared acknowledgement alongside OPT-004/OPT-005 and integrate overlapping worker/guest changes once; this is contract coordination, not a dependency on unrelated task completion.

Source outcome: the owning [retention contract](SECURITY.md#completed-record-retention) defines eligibility, bounds and safety exclusions. Successful read audits have conservative retention under the common lock after terminal publication. New unique guest bundles have protected pending descriptors before transfer; only independently verified, durably audited operations with exact recovery retirement can publish completion acknowledgements. A later already-authorized session delivers VM-bound acknowledgements; guest cleanup holds the preparation lock, preflights the complete deletion set and removes identity last. Exact cleanup readback retires host acknowledgements; partial failure/lost replies remain resumable without replaying effects. Legacy, unacknowledged, uncertain, retained and unrelated evidence remains protected. Added source regressions cover bounds, terminal eligibility, scope/duplicates, preflight failure and interruption/resume. Compilation, regressions, the complete gate and protected installation passed at the integrated checkpoint; live deletion remains pending. Isolated cleanup validation removes no operational records.

### OPT-007 - Conditional bounded transfer optimization

Dependencies: OPT-005.
Stage: R3 extension. Implementation: Done (source: 16 MiB chunk size). Validation: Passed (non-hardware); measurements pending.

**Selected scope:** on 17 October 2026 the user explicitly chose 16 MiB chunks before measurement, retaining the option to return to 4 MiB if qualification finds problems. This overrides the measurement-first entry condition for this bounded constant change. Broader batching, streaming or transport replacement still requires evidence of a material bottleneck.

**Outcome:** use 16 MiB chunks within the existing fixed bridge and qualify throughput, memory and reliability under OPT-008. No performance improvement is claimed from source changes.

Acceptance if selected:

- Select the smallest measured improvement, such as bounded batching or streaming; avoid a general transport replacement, speculative concurrency or increased CPU/memory limits.
- Preserve exact file/chunk acknowledgements, membership, source/destination integrity, bounded memory/output, cooperative deadlines, phase/receipt semantics and retained partial-state evidence.
- Test reordered/duplicate/missing acknowledgements, truncation/growth, slow/no-progress transfer, cancellation/disconnect and worker failure. Confirm no concurrent VM/GPU mutations.
- Show comparable transfer and whole-workflow measurements, including instrumentation overhead, with no correctness/resource regression. Run the non-hardware gate at the integration checkpoint and include selected changes in OPT-008.

Source outcome: the shared Rust constant now selects 16 MiB for both sender buffering and acknowledgement validation. Boundary/tail and failure fixtures follow the constant; deadlines, integrity checks and remoting caps are unchanged. Compilation, regressions, the complete gate and protected installation passed at the integrated checkpoint; live qualification remains pending. Use qualification evidence to retain or downgrade this candidate size without blindly retrying an interrupted transfer. Broader optimization remains deferred.

### OPT-008 - Integrated candidate validation and acceptance

Dependencies: OPT-001, OPT-002, OPT-003, OPT-004, OPT-005, OPT-006, OPT-007 (selected 16 MiB scope).
Stage: R4. Implementation: In progress. Validation: Passed (non-hardware gate and protected installation); live acceptance pending.

**Outcome:** one reviewable candidate with demonstrated reductions in work and preserved product behavior, using the [roadmap acceptance gate](roadmap.md#validation-and-acceptance). Prerequisites block final closure, not early scenario preparation.

Acceptance:

- Review the combined source changes and trace each optimization to its count/behavior tests. Run the complete non-hardware gate on the integrated candidate, or reuse its exact unchanged passing checkpoint; retain actual evidence and exclusions.
- Prepare exact installed candidate identity, targets, starting states, allowed effects, stop conditions and before/after conditions. Obtain applicable authorization before live launches, credentials/guest access, protected installation/writes or VM/GPU effects.
- Verify protected installation and preservation of operator/pair authority, configuration, managed history and unresolved recovery independently of GUI acceptance.
- Execute the roadmap's normal/failure journeys when authorized. Record durations, counts, bytes, transitions, terminal outcomes, independent readback and final recovery/power state. Isolated tests, installed observations and hardware results stay distinct.
- Include OPT-007 only if selected. Disclose unavailable timings, unrun scenarios, conditional decisions and regressions. Never mark installation or live acceptance complete from source/gate evidence.
- Close acceptance only from actual candidate evidence and the user's decision; summarize results in chat and status fields. Do not create QA reports/cases/archives unless requested.

## Prior checkpoint

The following completed OS-001 record is retained for provenance. It is not a dependency of the new optimization tasks and does not establish validation for their future changes.

## OS-001 - Fail-closed guest OS compatibility before GPU installation

- Implementation: **Done (source)**, 16 October 2026. Replaced mandatory offline/KVP preflight with a read-only authenticated current-OS check during reviewed Apply, before GPU or installation changes. An off VM is temporarily started and gracefully stopped under durable recovery. Missing or unreliable facts block installation; old metadata cannot override the live result. Preparation session and worker retain checks before guest writes. Discovery, reviews and ordinary management remain independent. Existing permission-completion UI behavior is preserved.
- Review findings: evidence was required before opening the session capable of supplying it; disk/partition assumptions and optional metadata falsely denied supported guests; historical KVP could override failed live identification; guest-local OS reading depended on the host disk adapter. The replacement removes those boundaries and keeps release policy separate from observation and lifecycle.
- Validation: **Passed (complete non-hardware gate)** on 16 October 2026: 399 product tests, 12 catalogue tests, copy/docs/format checks, strict Clippy, product binary builds and API documentation. Evidence: `local/evidence/m1-gate-20261016T154402853Z/`. Protected installation passed: `local/evidence/install-20261016T154451876Z/verification.json`; all four source/installed/policy hashes match, enrollment and 953 persistent files were preserved, and the runner is enabled and Ready. Added isolated supported/unsupported and failure regressions, including the production PowerShell query/installation boundary with substituted guest entrypoints. Earlier gate/install evidence covers the previous implementation only. No live VM, guest, BitLocker, GPU or installed GUI acceptance has run for this replacement. Existing recovery evidence is preserved.
- Contract: [security](SECURITY.md#guest-os-eligibility-before-installation).
