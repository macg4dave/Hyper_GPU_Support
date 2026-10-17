# Optimization roadmap

Updated 17 October 2026. The functional and architectural review is complete. OPT-001 through OPT-006 are implemented in source, including the protected read pipeline, R2 lifecycle/resource/payload changes and R3 helper/retention work. The integrated non-hardware gate and protected installation passed on 17 October 2026, covering R1/R2/R3 automated scope and the user-selected OPT-007 16 MiB change; see the [checkpoint](ENGINEERING.md#optimization-validation-checkpoint). Live acceptance and measurements remain pending under R4/OPT-008. Live performance validation has not started. The findings establish repeated work in execution paths, not measured speedups or new release failures. Previous accepted work remains historical context and is not reopened by this plan.

This page owns outcomes, delivery stages and gates. The [backlog](backlog.md) owns tasks, dependencies and implementation/validation status. The [agreed workflow](../README.md#agreed-workflow), [architecture](ARCHITECTURE.md), [security](SECURITY.md) and [engineering validation](ENGINEERING.md#validation-and-live-acceptance) remain authoritative.

## Objectives and scope

Reduce the work required to discover status, prepare an operation, apply a change and confirm its outcome. Prefer fewer provider calls, payload passes, guest writes and power transitions over small local code optimizations. Preserve the existing Slint design, shared Rust core, protected worker architecture and normal user-facing behavior.

The core scope is six cohesive work packages, OPT-001 through OPT-006, followed by candidate acceptance in OPT-008. OPT-007 includes the user-selected 16 MiB chunk size; broader transfer redesign is conditional on measurements from OPT-005. Dependency upgrades, a general service rewrite, UI redesign and broader guest support are outside scope.

| Outcome | Target and evidence |
| --- | --- |
| Lower idle cost | Routine polling uses bounded status reads instead of repeating complete inventory and package-availability work. Count provider calls, connections, hashes and process starts; explicit Refresh still obtains complete observations. |
| Cheaper fresh safety checks | Operational facts exclude optional display metadata. Reuse connections and batch inventory while fetching fresh mutable provider objects at safety boundaries. |
| Fewer setup power transitions | Combine OS identification and preparation for eligible unattached VMs. Target two boot cycles instead of three for first setup from Off; running and already-attached cases retain their own valid lifecycle. |
| Fewer payload passes | Target two complete host payload passes instead of three on eligible setup/reinstall/Verify paths, retaining independent final authentication. Eligible CLI ordinary changes should match the GUI's two-pass path. Disable remains payload-free. Prove feasibility and counts before claiming these targets. |
| Fewer mutations and repeated reads | Write only changed requested resource categories on retained adapters; apply all specified categories after recreation. Consolidate independent completion readback and reuse matching, freshly validated guest helpers. |
| Bounded waits and storage | Coordinate foreground work with polling; end read-only cancellation only after confirmed backend release. Bound completed audit/staging retention without deleting unresolved evidence. |
| Simpler orchestration | Extract affected worker handlers while changing their contracts. Preserve distinct Apply, Verify, reconciliation, save-only and authority operations. |

OPT-001 establishes reproducible baseline counts and measurement conditions. Set wall-clock targets from measurements rather than invented percentage gains. Distinguish source-backed reductions, isolated test results, installed timings and VM/GPU acceptance.

## Delivery stages

Stages group shared contracts and validation. Their preferred order minimizes rework; it is not a chain of prerequisites for every task.

| Stage | Joint delivery scope | Tasks | Exit gate |
| --- | --- | --- | --- |
| R0 - Baseline and contracts | Bounded counters/timing through existing diagnostics and fixtures; comparable journeys and safety boundaries. | OPT-001 | Original work counts are reproducible; unauthorized live measurements remain explicitly pending. |
| R1 - Protected read pipeline | Narrow operational facts, batched discovery, lightweight polling, foreground coordination, read cancellation/deadlines and completion observation. | OPT-002 | Freshness, contention, cancellation and partial-read regressions pass; reduced read/hash counts and the full non-hardware gate support the integrated batch. |
| R2 - Operation execution | Lifecycle planning/execution and per-category writes together; payload preparation/admission with extraction of affected worker handlers. | OPT-003, OPT-004 | Plan/effect agreement, payload binding, restoration and interruption regressions pass; the full non-hardware gate passes for the batch. |
| R3 - Guest and storage efficiency | Validated helper reuse, transport measurement and retention based on durable completion. Coordinate shared worker/guest acknowledgement fields with R2. | OPT-005, OPT-006 | Helper tampering, cleanup eligibility, interrupted deletion and uncertain-outcome regressions pass; the full non-hardware gate passes for the batch. |
| R4 - Candidate acceptance | Baseline comparison, protected installation/state preservation and affected user journeys under separately prepared authorization. | OPT-008; OPT-007 only if selected | Required scope meets the acceptance criteria below with actual results and disclosed exclusions. |

## Scheduling with few blockers

- OPT-001 through OPT-006 are ready for source work when requested. Baseline collection is the recommended first step, but live timing, installation and optional experiments do not block independent design, fixtures or implementation.
- Keep OPT-002 as one read-pipeline task. Agree its small shared read contract first, then integrate native discovery, polling, cancellation and GUI readback together instead of handing them through separate waiting tasks.
- Deliver OPT-003 and OPT-004 as the R2 batch when requested. They share workflow decisions, admission, worker handlers and failure coverage. Their independent work need not wait for R1.
- OPT-005 and OPT-006 can be prepared alongside R1/R2. Coordinate overlapping worker/transport edits and integrate a shared protocol change once. File overlap alone is not a hard dependency.
- The user selected 16 MiB chunks for OPT-007 before measurement, retaining the existing transport. Include this source change in the candidate validation gate; broader transport redesign remains deferred pending OPT-005 measurements.
- OPT-008 is the single required fan-in from OPT-001 through OPT-006. Scenario preparation and source review can start earlier; acceptance cannot close before the integrated scope and evidence are complete.

Independent preparation does not require concurrent edits to the same files or delegation. Keep one requested task or explicitly requested cohesive batch active under [AGENTS.md](../AGENTS.md). Resolve shared contract decisions inside the owning task rather than adding separate refactor or approval prerequisites. This plan authorizes no automatic progression into implementation.

Run the full non-hardware gate once per agreed coherent integration checkpoint, not after every substep. Adjacent stages may share a checkpoint when delivered as one requested batch. Reuse a passing gate for the exact unchanged candidate; rerun after changes that invalidate it. Source tasks can finish with installed/live validation pending; that work belongs to OPT-008 and does not block other source tasks.

## Safety and design constraints

The [security contract](SECURITY.md) remains in force: exact operator/VM/GPU/revision binding, fresh capability/conflict checks, artifact authentication, independent terminal readback, durable recovery, credential handling and publication ordering must survive every optimization.

- Package attestation reuse requires protected file identity/handles and enrollment-revision binding. Presentation caches never grant effect authority.
- Keep VM/GPU mutations serialized. Batch reads first; introduce bounded read parallelism only if measurement and partial-failure handling justify it.
- OS eligibility precedes installation writes. Retain actual preparation-session and guest-local OS checks across reboot/session boundaries.
- Cancel only read-only preparation through this work. Hold admission until confirmed backend release. Timeout, disconnect or matching instantaneous state never establishes completion or permits replay.
- Retain independent final payload/guest checks. Profile deletion stays record-only, save-only never replays Apply, and recovery remains an explicit foreground action.
- Cleanup needs positive durable completion and exact ownership/scope. Age alone cannot authorize staging or recovery deletion.

R3 automated status: helper tampering, cleanup eligibility, interrupted deletion and uncertain-outcome regressions passed with the complete non-hardware gate. Protected installation preserved enrolled authority and persistent state. OPT-007 retains the current protocol with 16 MiB chunks and passed isolated transport tests; native throughput/memory and live guest cleanup remain pending. R4 candidate acceptance is still open.

## Validation and acceptance

Each source package adds meaningful regression and failure coverage and updates the owning architecture/security/configuration documents when its contract changes. Tests should prove reduced work counts and unchanged permitted/forbidden effects.

At an integration checkpoint run `powershell -NoProfile -File scripts/testing/check.ps1` after inspecting its effects, following [engineering](ENGINEERING.md#validation-and-live-acceptance). Fix failures within the requested batch and rerun the complete gate. A passing gate establishes non-hardware validation only.

Candidate acceptance requires:

1. Exact revision, working-tree/build identity and passing automated scope. Existing accepted work is not relabeled as a new optimization result.
2. Comparable before/after VM states, GPU/driver/payload identities, VM/file counts, cache conditions and diagnostics settings. Account for durable diagnostic-flush overhead. Record work counts, whole-operation/stage durations, bytes, power transitions and terminal outcomes; disclose variability and regressions.
3. Authorized installed journeys covering open/poll/Refresh, first setup from Off and Running, ordinary VRAM/toggle changes, disable, Verify, repair, enrollment continuation, recovery, save-only and profile deletion. Include cancellation, stale state, changed payload/helper, publication failure and interruption scenarios where safely prepared.
4. Independent saved/current-state readback, final power and retained recovery where applicable. Completed progress stages are insufficient.
5. Protected installation preserving operator/pair authority, configuration, managed history and unresolved recovery; no failed safety regression or unexplained critical workflow regression.
6. Explicit unrun scenarios and conditional scope decisions, followed by the user's acceptance. Implementation, automated validation, installation and live acceptance remain separate statuses.

No live launch, guest access, protected installation/write or VM/GPU operation is authorized by this roadmap. Prepare exact targets, effects, stop conditions and evidence before that work. Summarize results in chat and task status; create separate QA reports, cases or archives only when requested.

## Review traceability

| Review finding | Delivery task |
| --- | --- |
| 1 - Narrow routine refresh | OPT-002 |
| 2 - Operational GPU facts and batched inventory | OPT-002 |
| 3 - Merge eligible identification/preparation boots | OPT-003 |
| 4 - Reduce payload passes | OPT-004 |
| 5 - Foreground priority, cancellation and deadlines | OPT-002 |
| 6 - Skip unchanged resource writes | OPT-003 |
| 7 - Consolidate completion observations | OPT-002 |
| 8 - Reuse helpers and measure transfer | OPT-005; conditional redesign in OPT-007 |
| 9 - Bound completed audit/staging retention | OPT-006 |
| 10 - Simplify worker orchestration | OPT-004, limited to affected handlers |

OPT-001 supplies the baseline and OPT-008 supplies integrated acceptance for all findings. Source entrypoints, detailed acceptance criteria and current status live in the [backlog](backlog.md).
