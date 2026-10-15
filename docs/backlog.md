# Release backlog

Track remaining qualification work against the [roadmap](roadmap.md).

## Next action

QA-011 to QA-014 now have source implementations and isolated regressions, with no source deferrals. The integration non-hardware gate passed; next qualify the installed recovery, reinstall and native credential routes on explicitly authorised prepared targets. QA-009/QA-010 remain open: this source-only batch does not supply installed/live results or user acceptance. Candidate preparation still needs exact build identity, local guest credentials, allowed effects, evidence location and stop conditions.

## Release tasks

| ID | Priority | Task | Dependencies | Implementation | Validation |
|---|---|---|---|---|---|
| QA-009 | P1 | Candidate regression and automated gate | None | Ready: preparation only; live execution requires authorisation | Candidate-specific live regression pending |
| QA-010 | P1 | Release decision and user acceptance | QA-009, QA-011, QA-012, QA-013 | Blocked: candidate evidence and installed P1 qualification required | Not run |
| QA-011 | P1 | Recover interrupted first setup from a running VM | None | Source complete: reviewed first-setup host restoration | Passed: full non-hardware gate; installed qualification not run |
| QA-012 | P1 | Reconcile native operations without a usable job reference | None | Source complete: scoped administrator terminal-evidence disposition | Passed: full non-hardware gate; installed qualification not run |
| QA-013 | P1 | Preserve recovery authority during administrator reinstall | None | Source complete: authority-preserving artifact reinstall | Passed: full non-hardware gate; protected qualification not run |
| QA-014 | P2 | Replace or remove saved guest credentials through Settings | None | Source complete: selected-VM replace/forget settings | Passed: full non-hardware gate; native-dialog testing not run |

### QA-009 ? Candidate regression and automated gate

Dependencies: None

Record exact installed candidate/build/revision and working-tree context. Prepare resettable targets, allowed effects, stop conditions and evidence before live execution. Run the core journeys and failure/UI coverage in the [roadmap](roadmap.md#test-journeys). Record each result and exclusion.

At the agreed checkpoint run the existing non-hardware gate after reviewing its effects. Completion requires candidate-specific results, final state/readback and persistence evidence, and an accurate automated gate outcome. Handle any retained recovery records through supported authorised actions before affected live journeys.

### QA-010 ? Acceptance decision

Dependencies: QA-009, QA-011, QA-012, QA-013

Summarise candidate evidence and deferred/untested scenarios. Successful core journeys, usable critical recovery, automated evidence and explicit user acceptance are required by the roadmap. Record the user's decision; documentation or automated checks alone cannot close this task.

### QA-011 - Recover interrupted first setup from a running VM

Dependencies: None

Priority: P1; blocks release until installed qualification. Implementation: source complete. Validation: full non-hardware gate passed; installed qualification not run.

Original finding: start first-time enable with an unattached running VM and fail guest preparation. [Apply](../src/workflow/apply.rs) retains a pending journal and can leave the guest running without a GPU or preparation receipt. Outcome reconciliation requires the completed attachment/settings; [baseline retirement](../src/configuration_store.rs) requires the original VM to have been Off; [partial removal](../src/workflow/reset.rs) also requires original Off and refuses operations requiring graphics. Even manually stopping this VM does not satisfy the original-Off restrictions. The host-wide hold has no supported completion for this failed setup. QA-009 describes exercising recovery, but does not implement this missing route.

Source resolution: the existing **Remove partial GPU setup** review now admits a failed/interrupted first preparation from an originally running, unattached VM. It binds the exact recovery/configuration revisions, journal, VM/GPU and fresh host facts. A separate shutdown-consent flag is required before graceful shutdown; the durable review resumes only its stopped/detached/restored transitions. It retains unknown guest files and the archived original failure, releases host ownership and publishes no desired configuration or setup success. Unavailable or changed facts retain recovery.

Acceptance criteria:

- Provide a separately reviewed recovery disposition for this recorded first-setup failure, bound to its exact VM/GPU, journal, configuration revision and fresh provider facts.
- Disclose and separately approve any graceful shutdown. Restore only attributable host state; retain unknown guest files and failed preparation evidence without publishing success or replaying Apply.
- Cover preparation failure, interruption during the recovery disposition, changed host/journal facts and unavailable readback with focused backend regressions. Uncertain or conflicting facts keep the hold.
- Qualify the installed route on an explicitly authorised resettable target before closing its live validation.

### QA-012 - Reconcile native operations without a usable job reference

Dependencies: None

Priority: P1; blocks release until installed qualification. Implementation: source complete. Validation: full non-hardware gate passed; installed qualification not run.

Original finding: [native invocation](../src/windows_wmi.rs) durably writes `PendingOperation { job: None }` before `ExecMethod`, then records a job only for an asynchronous reply. A COM failure, lost output or process interruption before that second write leaves no job reference. `reconcile_pending_operation` unconditionally requires a readable recorded job; a missing job object also fails before any outcome reconciliation. Every subsequent mutation remains blocked, and the GUI/CLI offers no scoped administrator reconciliation for this state. This is separate from QA-011's workflow-state restrictions.

Source resolution: new native markers bind the exact VM/GPU, operation stage, full before facts, requested host effect, workflow/configuration revisions, pre-invocation job identities and any COM failure. Administrator `native-review --job LOCAL_JOB_REFERENCE` requires a new terminal job independently associated with the exact VM and matching provider job type, plus compatible before/after state and quiescent provider inventory. It saves protected terminal evidence and returns a review token. `native-reconcile --review TOKEN` rechecks the receipt and fresh facts, archives the original marker/result and retires only provider admission. Protected terminal evidence may survive job expiry; matching VM state alone never authorises retirement. Missing evidence, legacy records without scope, active jobs and conflicting/unavailable facts retain the hold with an explicit administrator/provider-support handoff.

Acceptance criteria:

- Bind native recovery to the exact VM, operation stage and reviewed before/after facts, rather than only the management-service object and method.
- Provide an explicit supported disposition for a missing/expired job reference using sufficient independent provider-completion and state evidence. Never infer completion merely from a matching instantaneous state or remove an uncertain marker automatically.
- Preserve the original failure and unresolved evidence when completion cannot be established; give a concrete supported administrator handoff instead of an action that can only fail on the same missing reference.
- Cover missing job, lost reply, absent job object, active job, conflicting state and interrupted reconciliation with isolated failure tests, then perform scoped installed qualification.

### QA-013 - Preserve recovery authority during administrator reinstall

Dependencies: None

Priority: P1; blocks release until installed qualification. Implementation: source complete. Validation: full non-hardware gate passed; installed qualification not run.

Original finding: the CLI `install --config FILE` calls [install_inner](../src/runner.rs) without bootstrap mode. Its recovery/authority checks are confined to bootstrap branches; the ordinary path publishes a new operator and `configuration.targets` as the entire enrollment. Reinstalling with a subset or different pair while a save/effect recovery record exists can remove that record's enrolled pair. The worker subsequently refuses reconciliation/save-only because the recorded pair is no longer enrolled. Pending enrollment/removal receipts can likewise lose both of their bound policy revisions. Protected records survive on disk but their supported recovery authority is stranded.

Source resolution: routine reinstall admits under the common lock before changing existing ACLs or task availability. It rejects subset/additive/GPU/operator replacement and retains the existing exact authority targets rather than copying desired configuration into policy. A durable package-bound receipt supports interrupted artifact/policy/pending-pair publication. Pending enrollment/removal retains a valid current policy token; historical original tokens stay historical after completed authority publication. Saved intent, managed history and separate effect/save/import/native holds are untouched. Readback and an archived reinstall receipt precede removal of the installation hold. Legacy CLI markers can upgrade only through the same authority checks; GUI setup remains a separate recovery domain.

Acceptance criteria:

- Admit routine artifact reinstall under the common lock before changing operator permissions, task admission or enrollment; preserve the existing operator and exact pair identities needed by all pending recovery records.
- Refuse incompatible configuration/operator replacement without altering existing authority, records or task availability. Keep additive enrollment, inactive-profile removal and recovery as their existing distinct reviewed operations.
- Verify same-scope reinstall preserves saved intent, managed history and effect/save/import/enrollment/removal holds, including interrupted installation; test subset, GPU-change and operator-change rejection before publication.
- Qualify accepted reinstall and rejected replacement through the supported protected installer on prepared targets; source tests alone do not establish installed acceptance.

### QA-014 - Replace or remove saved guest credentials through Settings

Dependencies: None

Priority: P2. Implementation: source complete, no deferral. Validation: full non-hardware gate passed; native-dialog testing not run.

Original finding: the existing Settings action calls [credential_dialog](../src/gui/live/source.rs), which calls [prompt_named](../src/credentials.rs). `prompt_if_needed` immediately returns an existing vault entry, so selecting Guest credentials never opens a dialog once credentials are saved. Apply, Verify and recovery keep reusing the same entry after an incorrect remembered password or a later guest password change. There is no GUI credential-removal action; the supported CLI `forget --config FILE --vm GUID` is the current workaround. Profile deletion is a different operation and does not delete the vault entry.

Source resolution: Settings always opens selected-VM credential management, naming both VM and GUID. The local native choice explicitly replaces, forgets or cancels; replacement always collects fresh input. Remember is unchecked by default; successful replacement without Remember explicitly removes the previous entry. Cancellation preserves it, and vault read/write/delete failures surface without guest or VM effects. Ordinary operation prompts retain their existing saved-entry reuse.

Acceptance criteria:

- Let the existing selected-VM credential settings flow explicitly replace or forget that VM's opt-in entry; ordinary operation prompts may continue to reuse valid stored credentials.
- Keep VM identity clear, replacement/retention explicit and passwords local. Cancellation preserves the previous entry and any recovery hold; vault failures remain visible.
- Changing credentials performs no guest connection, enrollment, power or GPU work. Validate replacement, removal, cancellation and vault failure without using real credentials in tests, followed by authorised native-dialog testing.

## Source batch validation, 15 October 2026

QA-011 to QA-014 source work is complete with no source deferrals. The final full non-hardware gate passed: 367 product tests, 12 catalogue validator tests, copy scanning, documentation, formatting, strict Clippy, all binary builds and API documentation. Evidence: `local/evidence/m1-gate-20261015T095733365Z/`. The built CLI `--help` returned exit 0 and exposed both administrator native-recovery commands and their options; output: `local/evidence/source-recovery-20261015T095902190Z-help.txt`. Final documentation and diff checks passed after recording this summary. No protected installation, live GUI/rendering, native credential-dialog/vault, guest, VM or GPU qualification ran. Those acceptance requirements remain explicit on each card; QA-009/QA-010 are the next qualification work, not source work deferred from this batch.

## Review fixes, 15 October 2026

Implemented: bounded preparation-manifest and enrollment reads now enforce their existing limits while reading; ordinary frontends skip privileged diagnostic-file creation; CLI help exposes supported import/recovery commands and power-consent options. These are contained fixes from the release review. Validation: all three focused regressions passed (bounded preparation input, bounded enrollment input, ordinary-token diagnostic opt-in); strict Clippy for all targets, all binary builds, formatting, copy scanning, documentation checks (39 Markdown files, 6 cards) and `git diff --check` passed. The built CLI `--help` returned exit 0 and exposed all supported recovery commands/options; raw output is in `local/evidence/release-review-20261015-help.txt`. The full non-hardware gate, protected installation and live GUI/guest/VM/GPU qualification were not run. This review checkpoint predates the QA-011 to QA-014 source batch described above; it is not validation of that batch.

## Dashboard improvements (UI-001 to UI-005)

Implementation: Done (15 October 2026). The dashboard Verify action appears for fresh
host configuration differences, with an explanatory warning; voluntary Verify remains
in More actions. Historical guest results never activate this warning. Failed or uncertain
operations retain their recovery flow. No new automatic guest checks run on refresh.

The permanent top notification area and routine refresh notice are removed. Inventory
warnings and recovery actions appear with the VM list. VM-card OS icons use host-readable
Hyper-V intrinsic KVP OS metadata, keyed by VM GUID; missing, ambiguous or unrecognised
metadata retains the generic icon. This does not start or enter guests.

Memory controls share one responsive slider/value/GB/Set row. Decimal filtering applies
to whole edits (including paste), and Set/Enter require an in-range value. Rust revalidates
on submission. Existing 0.1 GB slider steps, exact byte values, selected-card physical
capacity, Advanced Allocation, saved settings and review/Apply semantics are retained.
Redundant instructions and the separate fixed-capacity control were removed; precision
notes remain in Technical Details.

Validation: `cargo build --locked --bins` passed; 116 focused GUI/controller regressions
passed with `cargo test --locked --bin hyper-gpu-support gui::`. The isolated intrinsic-KVP
XML parser test also passed without a provider connection. Copy scan, formatting and diff
checks passed. Source
layout consistency was reviewed together; rendered resizing, keyboard/paste interaction,
OS detection on actual guests and installed GUI/VM/GPU acceptance remain untested.
Build/install checkpoint, 15 October 2026: the full non-hardware gate passed (347 product
tests, 12 catalogue tests, copy/docs/format checks, strict Clippy, binary builds and API
documentation). Corrected stale fixed-accessor assertions and recorded three deliberate
dashboard wording revisions in the test fixture; repaired two links without recreating
the removed report. Protected installation verified all four artifact hashes/pins,
preserved both enrolled targets and saved/configuration/managed/recovery state, and left
the runner enabled and Ready. Evidence: `local/evidence/m1-gate-20261015T025600563Z/`
and `local/evidence/install-20261015T025622Z/`. No live GUI/VM/GPU retest ran;
QA-009/QA-010 remain open.
