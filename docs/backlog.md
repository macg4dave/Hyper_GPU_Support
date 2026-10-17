# Optimization backlog

Updated 17 October 2026. These tasks implement the [optimization roadmap](roadmap.md). The earlier release backlog was closed by user confirmation on 15 October; the optimization tasks are new work, not reopened QA cases. OPT-001 is implemented in source with validation pending the milestone gate. Other implementation has not started. Source work does not authorize installation or live effects.

## Next action

The next source task is **OPT-002**, using OPT-001's [measurement contract and frozen expectations](ENGINEERING.md#optimization-measurements). OPT-001 automated execution remains pending the coherent integration checkpoint. Then deliver **OPT-003 + OPT-004** together and **OPT-005 + OPT-006** together when requested; adjacent batches can be combined when their shared contracts are ready.

OPT-001 through OPT-006 have no hard source prerequisites. Work on their independent parts can be prepared without waiting for live measurements or another stage to close. Coordinate shared-file changes within the requested batch. OPT-007 is an optional measurement-driven extension; OPT-008 is the required final integration/acceptance task.

## Task register

Priority expresses delivery order/value, not defect severity. Dependencies are hard completion prerequisites; preferred sequencing and shared-file coordination are described separately. Source completion and automated/installed/live validation follow [engineering status](ENGINEERING.md#status-and-evidence).

| ID | Stage | Task | Priority | Implementation | Dependencies | Validation |
| --- | --- | --- | --- | --- | --- | --- |
| OPT-001 | R0 | Baseline, counters and comparison contract | High | Done (source) | None | Pending milestone gate |
| OPT-002 | R1 | Protected read pipeline and GUI coordination | High | Ready | None | Not run |
| OPT-003 | R2 | Setup lifecycle and changed-resource writes | High | Ready | None | Not run |
| OPT-004 | R2 | Payload admission and focused worker handlers | High | Ready | None | Not run |
| OPT-005 | R3 | Validated helper reuse and transport measurement | Medium | Ready | None | Not run |
| OPT-006 | R3 | Completed audit and staging retention | Medium | Ready | None | Not run |
| OPT-007 | R3 extension | Conditional bounded transfer optimization | Conditional | Deferred | OPT-005 | Not run |
| OPT-008 | R4 | Integrated candidate validation and acceptance | Required gate | Blocked | OPT-001, OPT-002, OPT-003, OPT-004, OPT-005, OPT-006 | Not run |

OPT-008 is blocked only from closing acceptance before its core prerequisites finish; its review/scenario preparation can start earlier. OPT-007 is excluded from core completion while deferred. If selected for the release candidate, its changes and validation also become part of OPT-008.

## Tasks

### OPT-001 - Baseline, counters and comparison contract

Dependencies: None.
Stage: R0. Implementation: Done (source). Validation: Pending milestone gate.

**Outcome:** reproducible before/after work counts and a small measurement contract shared by every optimization, without requiring a new benchmark framework or live access before source work.

**Scope:** [diagnostics](../src/diagnostics.rs), existing backend doubles in [workflow test support](../src/workflow/test_support.rs), provider/runner/guest instrumentation at the affected boundaries. Reuse existing opt-in diagnostics and ignored local evidence conventions.

Acceptance:

- Capture original work counts before each affected path changes: provider queries/connections, artifact hashes and payload passes, native mutations by category, helper/payload bytes, power transitions and completion reads. Include startup/poll, setup from Off/Running, ordinary Apply, disable, Verify, repair and save-only; document path eligibility.
- Add only bounded, secret-free aggregate counters/timing needed by these tasks. Preserve existing diagnostic durability and account for its overhead; avoid per-file logging expansion or an always-on telemetry service.
- Define comparable run conditions and the workflow/failure matrix for OPT-008. Counts obtained through isolated doubles are identified as such; current machine timings require separately authorized collection.
- Add meaningful isolated checks for counter attribution, terminal outcomes and unchanged effects when diagnostics are disabled. Capture baseline fixture expectations before updating them for optimized behavior.
- Source completion requires instrumentation, reproducible isolated baseline counts and the comparison procedure. Missing live timing stays pending under OPT-008 and does not hold this source task open.

Integration: establish counter names/attribution at the start of the first batch; other tasks can prepare their own fixtures before this card finishes. Run the full non-hardware gate at the agreed batch checkpoint.

Source delivered 17 October 2026: fixed aggregate counters on existing opt-in diagnostic summaries; native provider/hash/mutation and acknowledged transfer attribution; original workflow call/effect fixtures; and the [comparison/failure procedure](ENGINEERING.md#optimization-measurements). Fixture numbers are source-derived expectations, not executed results. Builds/Rust tests and the full gate have not run for this source package. No installed/native timing, guest, VM or GPU validation ran; those remain pending under OPT-008.

### OPT-002 - Protected read pipeline and GUI coordination

Dependencies: None.
Stage: R1. Implementation: Ready. Validation: Not run.

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

### OPT-003 - Setup lifecycle and changed-resource writes

Dependencies: None.
Stage: R2. Implementation: Ready. Validation: Not run.

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

### OPT-004 - Payload admission and focused worker handlers

Dependencies: None.
Stage: R2. Implementation: Ready. Validation: Not run.

**Outcome:** reduce duplicate complete host payload processing while keeping independently authenticated execution and final confirmation. Simplify the worker only where needed for this change, covering review findings 4 and 10 together.

**Scope:** [GUI preparation](../src/gui/live/review.rs), [CLI effect dispatch](../src/main.rs), [worker](../src/worker.rs), [native payload adapter](../src/worker/runner/backend.rs), [Apply admission](../src/workflow/apply.rs), [Verify admission](../src/workflow/verification.rs), payload/trust and affected protocol callers.

Acceptance:

- Use the existing preliminary-plan mechanism for eligible ordinary CLI effect commands. Explicit CLI plan remains a complete read-only plan; missing/adverse/pending history still forces the appropriate authenticated path.
- Specify the exact request, expected revision, permitted effect scope, credential boundary and in-process payload lifetime before changing the worker handoff. Consolidate expensive preparation/admission in one locked worker session where that binding can be proved.
- Preserve consumed non-serializable admission tokens, current identity/capability/journal checks, signature/catalog membership, guest integrity checks and independent final full payload authentication. No persistent digest cache or driver-version-only shortcut grants authority.
- Demonstrate reduced payload-pass counts on eligible setup/reinstall/Verify paths, targeting three to two without increasing the ordinary GUI path. Payload or scope drift must reject before effects rather than silently escalate guest/power work. Resolve any infeasible path explicitly before claiming completion.
- Extract affected private Apply/Verify handlers and identical publication/finalization helpers as part of the same change. Keep recovery, save-only, import and enrollment domains distinct; no generic command framework or unrelated worker rewrite.
- Test same-version byte changes, changed manifest membership/catalogs, stale request/revision, scope expansion, lost connection, journal/effect admission ordering, final authentication failure and verified-unsaved recovery. Existing four-to-three-pass admission work must not be duplicated.

Integration: combine with OPT-003; agree any completion acknowledgement needed by OPT-006 in this batch. Protocol consumers migrate together, without requiring another work package to close first.

### OPT-005 - Validated helper reuse and transport measurement

Dependencies: None.
Stage: R3. Implementation: Ready. Validation: Not run.

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

### OPT-006 - Completed audit and staging retention

Dependencies: None.
Stage: R3. Implementation: Ready. Validation: Not run.

**Outcome:** bound storage growth from completed discovery audits and obsolete guest staging without discarding recovery evidence. Review finding 9 includes the retention contract and its implementation in one task.

**Scope:** [runner audit/admission](../src/worker/runner/audit.rs), [runner storage](../src/worker/runner/storage.rs), [worker finalization](../src/worker.rs), [guest staging](../src/guest_transport.ps1) and the owning [security](SECURITY.md)/[configuration](CONFIGURATION.md) contracts.

Acceptance:

- Define terminal eligibility, age/count/byte bounds, ownership and scope, preservation rules and cleanup-error behavior before enabling deletion. Record the chosen policy and rationale in the owning handbook; do not infer permission from file age alone.
- Bound completed discovery/status audit retention while preserving required admission/terminal auditing, failed/started/uncertain records, administrator-retained evidence and records referenced by active work.
- Remove guest staging only with a durable host completion/retirement acknowledgement bound to the exact operation, VM, manifest and bundle. Interrupted or unacknowledged preparation is ineligible; installed payloads, receipts, unrelated bundles and unresolved recovery remain untouched.
- Cleanup uses fixed protected paths, exact identities and the relevant locks; reject unsafe files, links and ambiguous references. Make interruption/resume and partial failure safe without replaying VM/GPU work or changing the established operation outcome.
- Test bounds, active references, failed/unknown operations, malformed receipts, duplicate/mismatched acknowledgements, unsafe paths, deletion failure and interruption between cleanup steps.
- Source completion requires deterministic cleanup eligibility/failure coverage. Live cleanup/guest writes remain pending under OPT-008; no existing operational records are removed during source validation.

Integration: policy and eligibility tests can begin immediately. Design the shared acknowledgement alongside OPT-004/OPT-005 and integrate overlapping worker/guest changes once; this is contract coordination, not a dependency on unrelated task completion.

### OPT-007 - Conditional bounded transfer optimization

Dependencies: OPT-005.
Stage: R3 extension. Implementation: Deferred. Validation: Not run.

**Entry condition:** OPT-005 shows material chunk/remoting overhead and identifies a concrete benefit worth the protocol complexity. Until then, this is outside the required core scope and does not block OPT-008.

**Outcome:** reduce measured transfer overhead within the existing fixed bridge, only if justified.

Acceptance if selected:

- Select the smallest measured improvement, such as bounded batching or streaming; avoid a general transport replacement, speculative concurrency or increased CPU/memory limits.
- Preserve exact file/chunk acknowledgements, membership, source/destination integrity, bounded memory/output, cooperative deadlines, phase/receipt semantics and retained partial-state evidence.
- Test reordered/duplicate/missing acknowledgements, truncation/growth, slow/no-progress transfer, cancellation/disconnect and worker failure. Confirm no concurrent VM/GPU mutations.
- Show comparable transfer and whole-workflow measurements, including instrumentation overhead, with no correctness/resource regression. Run the non-hardware gate at the integration checkpoint and include selected changes in OPT-008.

If evidence does not justify the change, retain Deferred with the decision rationale. Do not manufacture implementation work merely to empty this card.

### OPT-008 - Integrated candidate validation and acceptance

Dependencies: OPT-001, OPT-002, OPT-003, OPT-004, OPT-005, OPT-006.
Stage: R4. Implementation: Blocked. Validation: Not run.

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
