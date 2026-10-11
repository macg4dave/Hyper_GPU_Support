# First-release backlog

This is the task order and status register. [Roadmap](ROADMAP.md) owns requirements and milestone gates. [Engineering](ENGINEERING.md#status-and-evidence) defines status and evidence conventions. Fresh `V1-*` IDs replace the old planning scheme; old IDs in retained code/configuration are historical references.

## Next implementation action

**V1-002 — Rebuild the non-hardware quality gate.** Ready for a separate implementation request. Create a small non-elevated gate for the commands specified in [testing](TESTING.md#planned-automated-gate), verify that selected tests have no VM/GPU effects, and separate documentation checks from compiled checks. Do not run builds/tests merely because the gate script has been written; its first full run belongs to V1-005 at B1 completion, unless the user requests an earlier run. Do not restore the deleted CI or scripts wholesale.

## Task register

All validation entries below describe this new plan, not inherited historical passes. Source exists for many workflows; the open tasks are audits, targeted corrections, and evidence collection rather than requests to rebuild those workflows from scratch.

| ID | Milestone | Task | Depends on | Implementation | Validation |
|---|---|---|---|---|---|
| V1-001 | B1 | Rebuild planning and Codex guidance | None | Done | Static documentation checks passed; product not run |
| V1-002 | B1 | Rebuild non-hardware quality gate | V1-001 | Ready | Not run |
| V1-003 | B1 | Select qualification environment and review development config | V1-001 | Ready | Not run |
| V1-004 | B1 | Correct stale source comments and CLI help | V1-001 | Ready | Not run |
| V1-005 | B1 | Validate and close baseline milestone | V1-002, V1-003, V1-004 | Blocked | Not run |
| V1-006 | G1 | Audit and complete two-pair GUI setup/enrollment | V1-005 | Blocked | Not run |
| V1-007 | G1 | Audit and complete reviewed Apply/Reapply/Verify | V1-006 | Blocked | Not run |
| V1-008 | G1 | Audit saved intent, drafts, and resource presentation | V1-007 | Blocked | Not run |
| V1-009 | G1 | Validate the existing GUI workflow | V1-006, V1-007, V1-008 | Blocked | Not run |
| V1-010 | S1 | Review trust, privilege, and IPC boundaries | V1-009 | Blocked | Not run |
| V1-011 | S1 | Cover interrupted effects and publication recovery | V1-010 | Blocked | Not run |
| V1-012 | S1 | Review containment, progress, and retained diagnostics | V1-010 | Blocked | Not run |
| V1-013 | S1 | Validate and close reliability milestone | V1-011, V1-012 | Blocked | Not run |
| V1-014 | Q1 | Prepare bounded concurrent-sharing qualification | V1-003, V1-013 | Blocked | Not run |
| V1-015 | Q1 | Run authorized two-VM qualification | V1-014 | Blocked | Not run |
| V1-016 | P1 | Prepare package, notices, and installation trials | V1-015 | Blocked | Not run |
| V1-017 | P1 | Validate release and publish support documentation | V1-016 | Blocked | Not run |
| V1-018 | Deferred | GPU replacement and Forget pairing | New requirement needed | Deferred | Not run |
| V1-019 | Deferred | GUI import/export and import recovery | New requirement needed | Deferred | Not run |

## Baseline tasks

### V1-001 — Rebuild planning and Codex guidance

Deliver the source baseline, README, roadmap, backlog, focused handbook, decision log, authoritative `AGENTS.md`, and six supporting prompts. Repair stale surviving sample/probe guidance without restoring deleted evidence/tools.

Acceptance: local documentation links and anchors resolve; task IDs/dependencies are consistent and acyclic; release requirements have owning tasks; agent prompts agree with the authoritative policies. Existing application, manifests, binaries, configuration, and unrelated deletions remain unchanged. Validation is documentation-only; this task does not close B1.

### V1-002 — Rebuild non-hardware quality gate

Create `scripts/testing/check.ps1` as a narrow non-elevated orchestrator and a separate documentation check entrypoint. Reassess each old helper only if needed; write replacements against current source rather than copying the old workflow. Inspect test bodies before selecting the default suite. CI is optional at this stage: document the decision and leave its deletion intact if it adds no immediate value.

Acceptance: the gate checks documentation, formatting, meaningful non-hardware Rust tests, strict Clippy, all current product binaries, and rustdoc as specified in testing; it fails at the first failed stage, reports exactly what ran, and never installs the product, launches the GUI, prompts for credentials, touches VMs, or runs GPU probes. Evidence output stays under ignored local storage. Execution remains deferred to V1-005 or an explicit earlier request.

### V1-003 — Select qualification environment and review development config

Identify the host edition/build, NVIDIA GPU/driver, and at least two existing Generation 2 Windows 11 guests. Ask the user to choose the guest editions/versions and the representative concurrent workload. Record a proposed observation duration and supported combination in local environment configuration/evidence, with a non-secret example if a tracked template is needed. Live guest queries or modifications require authorization.

Acceptance: all environment choices needed for Q1 are explicit; obsolete disposable-slot assumptions, old IDs, paths, and pins in retained development configuration are identified and either retained with a clear purpose or changed within authorized scope. Product runtime intent remains separate. Unknown facts are marked unknown; no environment is silently provisioned.

### V1-004 — Correct stale source comments and CLI help

Check `src/main.rs` and the module comment in `src/gui/live.rs` against actual dispatch/callback behavior. Update help to describe supported import/recovery commands and the live GUI accurately, including credential `forget` versus unavailable Forget pairing.

Acceptance: help documents accepted command forms and relevant options; comments accurately describe existing paths; no operation semantics change. Use existing dispatch tests where useful and add coverage only for a real parsing/documentation regression. Run no builds/tests until the agreed checkpoint.

### V1-005 — Validate and close baseline milestone

At B1 completion, run the new non-hardware gate, fix failures within this task, and record revision, commands, outcomes, and exclusions. Inspect current test selection before running it. Do not qualify hardware or launch the GUI.

Acceptance: all intended gate stages pass, the environment proposal is recorded, and baseline status reflects actual evidence. A failure leaves B1 open; a successful build does not validate GPU sharing.

## GUI tasks

### V1-006 — Audit and complete two-pair GUI setup/enrollment

Trace first-install inventory, fixed-package review, initial pairing, pending setup resume, and subsequent additive pairing for a second VM using the same GPU. Reuse `setup`, `enrollment`, `runner`, and `worker`; correct concrete gaps found rather than implement alternate authority.

Acceptance: initial setup review remains separate from GPU effects; adding the second pair preserves the first; rejected elevation, stale policy/operator/package identity, interrupted authority publication, and duplicate/replacement attempts have truthful GUI outcomes. Useful fixture coverage exists; runtime qualification is deferred.

### V1-007 — Audit and complete reviewed Apply/Reapply/Verify

Trace selected-VM review, fresh-plan binding, shutdown consent, credentials, worker progress, enable/disable, Reapply/Update, and separate Verify. Preserve the shared workflow and independent readback.

Acceptance: approval binds the reviewed target and effects; stale inputs cannot execute; standalone Verify does not publish desired configuration; original guest power is restored only through approved safe lifecycle handling. UI reports failure/uncertainty honestly and cannot act on a different selected VM during completion. Fix identified gaps and add only meaningful regression coverage.

### V1-008 — Audit saved intent, drafts, and resource presentation

Trace per-VM committed files, reopen/refresh, dirty drafts, switch/discard behavior, external file/policy changes, resource triples, and the VRAM slider. Reuse production schema parsing and existing model/controller tests.

Acceptance: saved and observed state stay separate; conflicts preserve drafts and block stale approval until explicitly discarded/reviewed; omitted categories make no explicit write; schema 2 remains readable and extended requests use schema 3. Snapshot/raw-unit paths cannot acquire current physical-capacity provenance. Displayed GB/GiB values never imply reservation or isolation.

### V1-009 — Validate the existing GUI workflow

At G1 completion run the non-hardware gate and inspect an explicitly authorized fixture/mock render and interactions. Exercise no-VM/many-VM layouts, long names, dirty navigation, modal progress, failure, uncertain outcome, and save-only presentation. Screenshots must identify simulation mode.

Acceptance: existing design is retained; appearance and keyboard/interaction checks have recorded outcomes; regression coverage passes; source baseline distinguishes mock interaction checks from live support. Live setup/effects remain Q1 work.

## Reliability tasks

### V1-010 — Review trust, privilege, and IPC boundaries

Review fixed setup/worker launch, source/package locking, hashes, ACL/owner/reparse checks, operator and peer-process authentication, bounded frames, stale plan rejection, and operation locking. Record concrete findings with source references and resolve release blockers within the shared architecture.

Acceptance: no generic elevated commands or caller-selected privileged destinations are introduced; authority comes from protected enrollment, not intent; competing operations are bounded/refused safely. Test source exists for meaningful negative cases or missing cases are identified for V1-011/V1-012. Security review is not live qualification.

### V1-011 — Cover interrupted effects and publication recovery

Reassess deleted recovery/integration tests selectively, reuse surviving inline tests, and add fault injection for real gaps. Cover interruption before/after effect admission, completed-but-unsaved results, changed revisions, explicit readback, and distinct setup/enrollment/import recovery.

Acceptance: uncertain work is never blindly replayed; save-only retries perform no GPU effects; partial or externally changed state stays blocked; standalone verification cannot publish intent; a second VM's state is preserved. Keep automated scenarios local/fake; destructive live fault tests are not authorized by this card.

### V1-012 — Review containment, progress, and retained diagnostics

Inspect WMI deadlines, guest bridge/transfer bounds, byte/output budgets, cancellation lifetime, timeout handling, and per-file progress. Reassess deleted diagnostic helpers only where necessary. Prepare crash-persistent operation-boundary records with fixed trusted privileged paths and secret-free public summaries.

Acceptance: relevant bounded failure cases have test coverage; incomplete transfer progress never reports preparation success; timeout does not imply no mutation; logs record operation/target/stage/time without credentials. No speculative live stress test is run, and source containment does not establish a host-lockup fix.

### V1-013 — Validate and close reliability milestone

At S1 completion run the non-hardware gate and the selected IPC/fault/containment tests. Review findings and recovery receipts against the acceptance scenarios in testing.

Acceptance: S1's automated scenarios pass, blockers are resolved, and remaining live uncertainties are explicit. Qualification cannot begin with unresolved trust or replay risks.

## Qualification and release tasks

### V1-014 — Prepare bounded concurrent-sharing qualification

Prepare narrowly scoped tooling and a concrete reviewable run sheet for the selected two existing guests. Reassess deleted qualification scripts/probes individually; current product D3D11 verification is the starting point. Define checked workloads that remain active during an overlapping interval; sequential Verify successes alone are insufficient.

Acceptance: targets, exact effects, workload duration, evidence locations, stop conditions, guest-power restoration, and per-step approval boundaries are explicit. Include R1-R5 scenarios, independent operation while the other guest renders, and safe recovery inspections. Tooling has no automatic retry, host restart, forced power-off, or arbitrary VM creation/reset. Prepare package files as needed without claiming release readiness. No live trial runs during preparation.

### V1-015 — Run authorized two-VM qualification

Obtain authorization for the prepared run sheet, then execute only its scope. Record concurrent checked hardware rendering, independent enable/disable/reapply, saved settings across reopen, reviewed verification, and safe applicable recovery scenarios.

Acceptance: R1-R5 have revision/environment/time-bound evidence, hardware identity in both guests, overlapping workload results, and the unaffected guest's results during the other guest's operation. Failed/disconnected runs retain logs and stop; no automatic retry or extra investigation. Unknown outcomes keep Q1 open. No resource-enforcement guarantee is inferred.

### V1-016 — Prepare package, notices, and installation trials

Package the four required product artifacts from a qualified revision; assess licenses/notices, update behavior, artifact trust, and clean setup instructions. Reassess deleted build/CI helpers only where required. Prepare installation/upgrade trials and obtain authorization before privileged execution.

Acceptance: package inventory/hashes and notices are complete; missing/changed artifacts fail truthfully; old protected enrollment/configuration is not silently replaced during upgrade; intended setup and subsequent GPU review remain distinct. No old laboratory binaries become implicit release dependencies.

### V1-017 — Validate release and publish support documentation

At P1 completion run the final non-hardware gate and separately authorized package trials. Refresh support, limitations, troubleshooting, and source baseline using recorded outcomes.

Acceptance: R1-R6 link to concrete evidence; unresolved failures block release; instructions work from a complete package on the selected environment; only qualified combinations are described as supported. Provide a reviewable release result; publishing or pushing is a separate user instruction.

## Deferred capabilities

### V1-018 — GPU replacement and Forget pairing

Await a separate requirement. Design explicit authority transition and recovery semantics before connecting unavailable controls. Credential deletion remains distinct. No first-release dependency.

### V1-019 — GUI import/export and import recovery

Await a separate requirement. Existing CLI import/resume remains documented; any future GUI route must reuse production parsing, per-file publication receipts, and destination conflict checks. Import must not authorize GPU effects. No first-release dependency.
