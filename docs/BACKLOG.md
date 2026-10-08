# Backlog

## Resume

Active work is **ARCH-001**, implementing the user-approved architectural rebase.
**M1 is completed as of 2026-10-08**; its independent review and live
[installation/enrollment qualification](evidence/M1.md) passed. Active milestone
work is M2's small working GPU-PV core. NVIDIA one-VM functional checks passed,
and the user-authorized observed repeat on current revision `96152e7` passed
attachment/rendering/reapply/disable and graceful cleanup without host symptoms.
The user closed the hang investigation on 8 October; BLK-005 no longer blocks work
and no further hang testing/observation is scheduled. Its cause remains unknown.
Fresh preparation under the new child limits remains a separate qualification gap.
See [M2 results](evidence/M2.md).
Product configuration and native operations no longer require laboratory identities.
The old fixed-slot application is retained under `tools/lab/`, outside production.
The roadmap now orders contract separation → working core → native GUI → allocation
and vendors. Prior implementation cards/results remain historical; their old
dependency ordering does not override the new product goal. Materially changed
privileged boundaries require independent review and affected
qualification. Routine patches use proportional checks. Next: **CORE-006 shared
operator preview**, followed by GUI-001 integration; reuse the working core.

## Task register

This register alone owns current status, priority, milestone and dependencies.
Dependencies are real completion prerequisites, not an instruction to reread all
prior research. The completed foundation below already satisfies its dependencies.
The revised M1–M4 roadmap and ARCH-001 own product sequencing. Existing card
results are retained as research and reusable implementation history. Golden-image/disposable-target preparation is development
support, not shipped functionality. Qualification may reuse existing tooling.

### Current milestone acceptance

| Milestone | Status | Result |
|---|---|---|
| M1 | completed | Product/laboratory separation, existing-VM enrollment and rewritten privilege boundary reviewed and live-qualified on 2026-10-08; [acceptance](evidence/M1.md). |
| M2 | in progress | Observed current-build one-VM repeat passed; affected fresh preparation under new limits remains. Earlier hang cause unknown; [results](evidence/M2.md). Sharing unqualified. |
| M3 | in progress | GUI-001 native prototype; written layout/read-only work available now, live acceptance after M2. |
| R1 | planned | CLI + GUI candidate, tested guide and packaged acceptance; CORE-017/DOC-003/GPU-014. |
| M4 | deferred | Allocation meaning/enforcement and incremental vendor qualification after useful release. |

Legacy card milestone labels below retain implementation history; they do not
reopen the current roadmap's completed M1 acceptance.
Do not port laboratory setup merely to satisfy the Rust product migration gate.

| ID | Milestone/class | Priority | Status | Depends on |
|---|---|---|---|---|
| [ARCH-001](#arch-001) | M1/M2 | P0 | in progress; observed current-build repeat passed | M1 complete; affected fresh preparation |
| [GUI-001](#gui-001) | M3 | P1 | in progress | read-only/layout now; live acceptance after M2; CORE-006 preview |
| [CORE-022](#core-022) | M1 | P0 | completed | CORE-009 |
| [CORE-023](#core-023) | M1 | P0 | completed | CORE-002 |
| [CORE-003](#core-003) | M1 | P0 | completed | CORE-020 |
| [GPU-006](#gpu-006) | lab history | P2 | deferred | no product release dependency |
| [CORE-024](#core-024) | M1 | P0 | completed | CORE-020 |
| [CORE-025](#core-025) | lab history | P2 | deferred | product replacement owned by ARCH-001 |
| [CORE-026](#core-026) | M2 | P1 | merged | ARCH-001; DEC-028 |
| [CORE-027](#core-027) | M1 | P1 | completed | ARCH-001 M1 |
| [CORE-006](#core-006) | M2 | P1 | in progress | existing workflow; shared preview gap |
| [CORE-011](#core-011) | later | P2 | deferred | no standalone lifecycle release requirement |
| [CORE-010](#core-010) | M2 | P1 | merged | ARCH-001 recovery/disable |
| [CORE-012](#core-012) | M2/M3 | P1 | ready | existing observed/journal/error results |
| [CORE-021](#core-021) | M2/M3 | P1 | in progress | schema 2 and M1 enrollment implemented |
| [CORE-015](#core-015) | M2 | P1 | merged | ARCH-001 drift refresh |
| [GPU-012](#gpu-012) | M2 | P1 | in progress | affected ARCH-001 qualification |
| [CORE-017](#core-017) | R1 | P1 | planned | M2, GUI-001; packaging preparation may overlap |
| [DOC-003](#doc-003) | R1 | P1 | planned | stable CLI/GUI operator contract |
| [GPU-014](#gpu-014) | R1 | P1 | planned | M2, GUI-001, CORE-017, DOC-003 |
| [GPU-010](#gpu-010) | M4 | P2 | deferred | M2; existing raw allocation API |
| [GPU-007](#gpu-007) | post-v1 | P2 | deferred | M2; chosen workload |
| [GPU-013](#gpu-013) | post-v1 | P2 | deferred | M2; actual owner-controlled driver update |
| [GPU-015](#gpu-015) | conditional sharing | P2 | deferred | M2; two explicitly designated test VMs |
| [GPU-016](#gpu-016) | post-v1 | P2 | deferred | GPU-005 |
| [GPU-017](#gpu-017) | post-v1 | P2 | deferred | M2; concrete reduction hypothesis |

### Completed foundation

Completed results are condensed here; reuse them unless a change requires a new
check. Historical dependencies and research procedures are retired.

| ID | Status | Result |
|---|---|---|
| [DOC-001](#doc-001) | completed | Established owned documentation and task IDs. |
| [DOC-002](#doc-002) | completed | Established the original release plan; current sequencing replaces its research gates. |
| [DOC-007](#doc-007) | completed | Established shared [engineering standards](ENGINEERING.md). |
| [DOC-008](#doc-008) | completed | Established the one-VM/disposable-child architecture. |
| [DOC-009](#doc-009) | completed | Configured independent architecture review. |
| [DOC-010](#doc-010) | completed | Established autonomous development authorization. |
| [DOC-011](#doc-011) | completed | Established maintained script layout and physical-host session protection. |
| [CORE-019](#core-019) | completed | Created the Rust library/CLI, hardware-free tests and pinned Windows CI. |
| [GPU-001](#gpu-001) | completed | Initial recipe research completed; no future lookup dependency. |
| [HV-001](#hv-001) | completed | Target inventory completed; [evidence](evidence/HV-001.md). |
| [REF-001](#ref-001) | completed | Historical repository provenance captured; no active upstream workflow. |
| [REF-004](#ref-004) | completed | Established unmodified Windows media and protected parent/disposable-child layout. |
| [HV-003](#hv-003) | completed | Installed management/privilege contract measured; [evidence](evidence/HV-003.md). |
| [GPU-002](#gpu-002) | completed | Initial package manifest measured; superseded as a complete recipe by GPU-009, while package validation is reused. |
| [REF-002](#ref-002) | completed | Historical artifact inspection completed; no reference artifact is needed by the product. |
| [GPU-008](#gpu-008) | completed | Essential standalone probe inputs/oracles specified; [manifest](../probes/MANIFEST.md). |
| [GPU-003](#gpu-003) | completed | Targeted provisioning and child recovery procedure established. |
| [HV-002](#hv-002) | completed | Protected parent and fixed disposable child established; [evidence](evidence/HV-002.md). |
| [GPU-009](#gpu-009) | completed | Full normal-Hyper-V baseline and native discovery parity passed on 2026-10-05; [measured result/inventory](evidence/GPU-PV-BASELINE.md). |
| [GPU-005](#gpu-005) | completed | Sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA passed; [baseline](evidence/GPU-PV-BASELINE.md). |
| [CORE-001](#core-001) | completed | Rust inventory implemented; [evidence](evidence/CORE-001.md). |
| [CORE-004](#core-004) | completed | Strict typed configuration/error/report contracts implemented; [evidence](evidence/CORE-004.md). |
| [CORE-005](#core-005) | completed | Fixed target-bound runner inspect/reset/start/shutdown/attach/detach implemented and tested; [evidence](evidence/CORE-005.md). |
| [CORE-008](#core-008) | completed | Bounded authenticated guest transfer implemented and tested; reused by staging. |
| [CORE-009](#core-009) | completed | Package/alias writer, receipts, no-op and uncertain-child recovery tested; [evidence](evidence/CORE-009.md). Full associated-destination support remains CORE-022. |
| [CORE-002](#core-002) | completed | Exact attachment and verified no-op implemented/tested on 2026-10-04; [evidence](evidence/CORE-002.md). Corrected the stale pending card; full settings remain CORE-023. |
| [CORE-020](#core-020) | completed | Standalone checked D3D11/D3D12/CUDA probes built/tested; [evidence](evidence/CORE-020.md). |

### Retired task mapping

Retired IDs are never reused. These entries record consolidation, not scheduled work.
They are excluded from the dependency graph and release gates.

| ID | Disposition | Owning work | Reason |
|---|---|---|---|
| [GPU-004](#gpu-004) | cancelled | - | Reference comparison removed completely; no deferred replacement. |
| [CORE-007](#core-007) | merged | CORE-006 | Reuse fixed runner audit/locking within the public workflow. |
| [CORE-013](#core-013) | merged | implementation tasks | Existing Windows CI is maintained with each meaningful change. |
| [CORE-014](#core-014) | merged | CORE-022, CORE-010 | Focused interruption/recovery checks belong to the writer and recovery implementation. |
| [GPU-011](#gpu-011) | merged | GPU-012 | Live lifecycle checks belong to repeatability qualification. |
| [CORE-016](#core-016) | merged | milestone exits | Independent review remains at actual implementation/privilege gates; no separate review chain. |
| [CORE-018](#core-018) | merged | CORE-021 | Public schema/help/report stability belongs to config/CLI cleanup. |
| [DOC-004](#doc-004) | merged | CORE-017 | Resolve concrete payload/license/signing/publication choices at packaging, without blocking implementation. |
| [REF-003](#ref-003) | merged | CORE-017 | Actual shipped component notices belong to package acceptance. |
| [DOC-005](#doc-005) | merged | GPU-014 | Final checklist belongs to the packaged acceptance run. |
| [DOC-006](#doc-006) | merged | GPU-014 | Final artifacts and handover belong to the same acceptance/delivery result. |

## Tracking rules

Read the selected card and affected source, implement, run proportional checks,
record its short result, then move on. Claim ownership only for scheduled/shared
work or a handover; fix trivial drift in place. Normal statuses are planned, ready,
in progress, blocked and completed; deferred means outside v1, while cancelled/merged
IDs are retained only in the mapping. A ready task has satisfied prerequisites. Merged cards are excluded from release
dependencies; lab-history cards retain results without scheduling product work.
Create a blocker only for an observed impediment. Update only documentation affected
by the behavior; a new evidence file is optional unless a hardware report needs it.
Each implementation task maintains existing hardware-free Windows CI and meaningful
behavior tests; no separate task to check previous checks. Follow [engineering](ENGINEERING.md)
and [targeting/authorization](../AGENTS.md#development-and-test-authorization).

## Blocker register

**BLK-005 is closed by user direction on 8 October.** The current-build repeat
passed without symptoms. Historical cause remains unknown; there is no further
hang investigation or observation prerequisite for product work.

| ID | State | Resolution |
|---|---|---|
| BLK-001 | resolved 2026-09-25 | Approved elevated inventory established the management facts; [HV-001](evidence/HV-001.md). Use the approved runner for privileged work. |
| BLK-002 | resolved 2026-09-26 | Servicing/reboot impediment resolved; exact residual delete-only cleanup was reviewed. Unknown/active servicing is still rejected. |
| BLK-003 | closed 2026-10-05 | Optional historical guest-access comparison was discarded with GPU-004. Its transport was not repaired and has no reopening instruction or product dependency. |
| BLK-005 | closed by user direction 2026-10-08 | Current-build repeat passed without symptoms. Historical cause unknown; investigation/reproduction/observation work closed, not a product blocker. See [M2](evidence/M2.md). |
| BLK-004 | resolved 2026-10-04 | Verified stage/no-op accepted build drift as qualification context and only exact reviewed delete cleanup; [CORE-009](evidence/CORE-009.md), DEC-022/023. |

## Implementation cards and retained history

## CORE-022

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Write the complete discovered driver/runtime manifest**

- Objective: extend the existing Rust guest writer from package/alias staging to every DriverEnvironmentManifest destination.
- Read: src/driver_environment.rs, src/staging.rs, src/guest.rs, src/windows_guest.rs and [validated placement](ARCHITECTURE.md#driverruntime-manifest-and-guest-placement).
- Acceptance: consume the complete dynamically discovered driver/runtime manifest; preserve ordinary byte copies, discovered destination mapping and deterministic hashes. Validate roots, source identity, collisions and unsafe/reparse paths before effects. Verify every written length/hash and matching reapply; report partial or uncertain staging as requiring child recreation. Test mapping/write/no-op/interruption and different manifest lengths with artificial fixtures; compare historical mapping behavior with the [isolated inventory fixture](evidence/GPU-PV-BASELINE-INVENTORY.tsv). No fixed file count defines completeness or success.
- Reuse existing credential, receipt and target guards. Review the changed privileged writer boundary before deploying it. No new arbitrary guest-command transport.
- Owner: Codex; completed 2026-10-06.
- Result: complete Rust writer and CLI integration; native source/destination/digest authorization, protected ordinary copies, full receipt, verified no-op and retained-lock interruption recovery. Focused tests cover all 271 baseline mappings, real local publication, tampering and interrupted-copy refusal. Independent review closed the volume-root defect and has no remaining blockers.
- Live release-build Windows x64 qualification passed through PowerShell Direct on the recreated clean child: `applied`, then `already-applied`; both verified 271 files / 3,407,066,692 bytes and digest `6408de79388db75085c75c79fee9fdc376617349c11f89630615da2738f4d7f8`. Host build 26300.9457, guest build 26200.9457, RTX 5060 / driver 32.0.16.1692 (616.92). This qualifies file staging/reapply; VM settings and GPU workloads remain separate tasks. Local results: `local/evidence/core022-live.log` and `core022-live.status.json`.
- Preparation passed fixed-runner remove/reset/start after reconciling Hyper-V's session-lock shutdown refusal through exact-target forced shutdown and Off readback. Replaced inspection/reset's elapsed cutoff with read-activity supervision inside a shared finite task budget; updated release pins and installed the reviewed runner. A before-write servicing refusal was resolved by reviewing/configuring the exact absent Opera updater delete-only entry; unknown cleanup and replacement records remain refused.
- Validation: full Rust quality gate (143 tests, strict Clippy, formatting, build, rustdoc and configuration/policy drift), documentation checks, focused servicing/budget regressions and independent privileged-boundary review passed. Interactive credentials now precede driver discovery; use release builds for full-manifest hashing.

## CORE-023

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Apply and read back the validated Hyper-V profile**

- Objective: express the measured VM/GPU settings as typed configuration and apply them through the approved fixed runner.
- Read: config/project.toml, src/config.rs, src/runner.rs, src/windows_runner.rs and [validated settings](ARCHITECTURE.md#hyper-v-settings-and-gpu-partition-resources).
- Acceptance: configure MMIO, cache types, static memory, CPU/virtualization, checkpoint policy and all four GPU resource triples; retain Secure Boot/vTPM. Reuse exact attachment, require the safe VM state and read fresh effective values before reporting success or no-op. Refuse unsupported resource ranges and wrong/duplicate adapters. Test mismatch, partial update, stale readback and malformed configuration. Never change host partition count or reinterpret opaque resource units as physical percentages.
- Add the bounded operation and policy pins required for these settings; review that changed privileged boundary before deployment. Leave unrelated Hyper-V configuration to Windows.
- Owner: Codex; completed 2026-10-06 on the existing staged child, without recreation.
- Result: typed VM profile and all four explicit GPU resource triples are pinned in policy and applied through `configure-slot`. Off-state target/chain/security/range guards, durable preimage, stale-preimage refusal and independent fresh-process readback cover apply and matching no-op. Automatic checkpoints can be disabled; actual snapshots remain refused. Independent privileged-boundary review closed that guard defect and cleared the readback correction.
- Live release-build Windows x64 qualification passed on host build 26300.9457, RTX 5060 / NVIDIA 32.0.16.1692 (616.92): exact attachment, then `applied` (`1791282267-965953300`) and `already-applied` (`1791282472-671670700`). Both confirmed static 8 GiB/four CPUs, 3/32 GiB MMIO, cache/nested virtualization, disabled checkpoints, guest shutdown, retained Microsoft Windows Secure Boot/vTPM and all twelve GPU values, including encode 2^63. This qualifies Hyper-V settings; guest readiness and workloads were not run for this task.
- Initial live mutation reached the desired state but its setter-process verification failed; an independent diagnostic confirmed all settings. Removed that redundant verification and retained the independent reader as the success gate. Suspected Hyper-V object caching is not a proven diagnosis. Reviewed recovery restored only the recorded resource preimage with independent readback before clearing the exact marker and repeating Rust apply/reapply. Failure/preimage/audit remain retained.
- Validation: `scripts/testing/check.ps1` passed 151 tests, strict Clippy/compiler warnings, formatting, build, rustdoc and policy drift; release runner/client/rights build, all artifact-pin checks and documentation checks passed. Local test: `local/scripts/test-core023-live.ps1 -SettingsOnly`; results in `local/evidence/core023-live.status.json` and `core023-live-{0,1}-configure-slot.log`.

## CORE-003

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Automate GPU readiness and essential workload verification**

- Objective: invoke the existing known probes through bounded Rust orchestration, wire the public validate command, and reuse that path for reproduction.
- Read: src/probe.rs, src/windows_probe.rs, probes/MANIFEST.md and [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: verify transferred probe/runtime inputs; observe sustained Code 0 with a configured deadline; run nvidia-smi and checked D3D11/D3D12/CUDA. Return per-check pass/fail/blocked/untested and exact adapter, runtime and output identities. Reject software fallback, absent probes, wrong CUDA selection, malformed output and timeouts. Include the measured app-runtime prerequisites rather than assuming developer tools exist in the guest.
- Keep CUDA identity diagnostics separate: host/guest LUID equality and graphics/compute interop are not standalone computation gates. Verify the intended one-GPU CUDA device safely; do not weaken graphics identity checks.
- Use fixed project workloads, not an arbitrary privileged execution API. A live integration run belongs to GPU-006; no new feasibility investigation.
- Test public validate dispatch, result/exit propagation and missing prerequisites; the command must execute checks rather than remain an advertised stub.
- Owner: Codex; completed 2026-10-06.
- Result: public `validate` executes fixed Rust guest readiness and workloads, independently verifies all nine transferred inputs and successful report evidence, and returns per-check states with native identities and bounded output. CRT prerequisites are explicit; graphics partition identity remains strict while standalone CUDA safely accepts a distinct LUID on the sole configured device.
- Protection: exact compiled configuration/target, protected input ACLs, exclusive host/guest locks, finite remoting and independent worker deadlines, suspended children assigned to kill-on-close jobs before execution. Local credentials are excluded from command lines/reports. Independent architecture review closed unsafe-existing-file ACL, worker deadline and child containment findings.
- Validation: Windows x64 `scripts/testing/check.ps1` passed 166 tests, strict Clippy/compiler warnings, formatting, build, rustdoc and configuration drift checks. Tests include public dispatch/exit propagation, absent/tampered prerequisites, sustained/nonzero readiness, wrong hardware/CUDA selection, malformed reports, timeouts/output bounds, permissive ACL refusal and native worker/descendant termination. Release worker/CLI/probe builds, documentation checks and all four artifact pins passed. Public release `validate` preflight returned structured blocked/untested evidence and exit 1 for an unelevated token before privileged calls. No guest workload qualification is claimed here; GPU-006 owns that combined live run.

## GPU-006

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Reproduce the proven baseline from a clean child through Rust**

- Objective: exercise the three implemented pieces as one configured Rust workflow.
- Acceptance: verify the enrolled disposable target and protected parent; recreate its child, apply the full manifest, read back settings/adapter, start and reach sustained Code 0, nvidia-smi and checked D3D11/D3D12/CUDA passes. Verify reapply and graceful shutdown; retain one concise run report with configuration/manifest identity, effective settings, versions and outputs. Temporary experimental provisioning scripts must not supply missing application behavior.
- Obtain independent M1 implementation review using the actual diff and test results. Recipe minimality and interop are outside this gate.
- Owner: Codex; started 2026-10-06. Combined clean-child qualification through existing Rust staging, fixed-runner settings/lifecycle and public validation entry points.
- Result: pending; the experimental baseline and native discovery parity are established, full Rust reproduction has not passed.
- 2026-10-07 continuation: development-feature release build, default/development quality gates (`RUST_TEST_THREADS=4`), documentation checks and independent boundary review passed. Updated generated artifact pins and installed the reviewed development runner; enrolled-token `inspect` passed (`1791339041-828647900`), qualifying the stronger directory read/rename guard on the configured Off child with zero adapters. Preparation evidence: `local/evidence/gpu006-runner-preparation.json` and `gpu006-enrolled-inspect.log`. Combined qualification is running in the local interactive console; guest credentials and staging/workload results remain pending. Run report: `local/evidence/gpu006-20261007T021240630Z/report.json`. The initial default-concurrency test attempt hit six existing PowerShell fixture timeouts; the four-thread retry passed.
- Follow-up: that console exited before staging; `remove-gpu` failed preflight because the inspected target already had zero adapters (`1791340800-892922100`). No reset/staging/workload ran. Fresh native inspection (`1791340907-617321700`) confirmed the exact Off/zero-adapter target; quiescent, locked reconciliation archived and cleared only that failed preflight marker (`local/evidence/gpu006-preflight-reconciliation.json`). Combined qualification remains pending. User selected CORE-025 native backend migration as the active implementation step.

## CORE-006

**Finish the shared operator preview; reuse the working workflow**

- Owner: Codex; originally claimed 2026-10-07.
- Historical first slice: bounded native plan/status and versioned operator JSON
  passed focused operator/public-command tests, native Windows inventory checks,
  formatting, strict Clippy, workspace tests/doc-tests and rustdoc. Guest state was
  explicitly unobserved. That pre-rebase implementation now lives in tools/lab;
  its old pending-apply statement does not describe the current root product.
- Implemented under ARCH-001: public apply/enable/disable/verify, exact target guards,
  preparation, settings, initial-power handling and recovery. Do not rebuild these.
- Remaining: shared effect summary from target/observed/journal/preparation state;
  explain settings, detach/attach, driver refresh, credential need, downtime and
  restoration. Existing enable-plan hashes the payload; keep dashboard reads separate.
- Acceptance: enable/disable/running-no-op/pending previews agree with workflow;
  CLI/GUI share results; no VM/guest effects. Apply still rechecks identities/state.
- Read: src/main.rs, src/runner.rs, src/workflow.rs, src/model.rs.
- Result: workflow functionally passed M2; useful preview remains pending.

## CORE-024

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Replace PowerShell inventory with bounded native Rust discovery**

- Objective: remove the fixed script in `src/windows_inventory.rs` from public inventory, preflight and state comparison without rewriting native driver discovery.
- Owner: Codex; completed 2026-10-06. Existing GPU-006 qualification ownership is preserved.
- Priority: next native implementation slice. Read `src/inventory.rs`, `src/windows_inventory.rs`, existing native WMI/identity helpers and the [audit](../scripts/PRODUCT-MIGRATION.md).
- Replacement: Windows registry/system APIs for host facts; reuse native GPU/driver discovery and query registered VM/partitionable GPU state through native management interfaces. Preserve typed known/missing/denied/unavailable distinctions and configured exact-target selection. PowerShell module availability may remain a diagnostic fact, never a product prerequisite.
- Tests: missing providers, access denial, multiple/wrong identities, absent GPU, malformed provider values, timeout/output bounds and native-worker termination. Compare affected facts with current adapter output on the configured host; read-only parity does not claim workload support.
- Acceptance: public inventory does not launch PowerShell; potentially blocking COM/provider calls remain in a bounded fixed Rust worker. Errors retain useful native details and secret-free reports. Remove the production script only after demonstrated parity.
- Result: replaced the embedded inventory script with native registry/system APIs and reused WMI bindings under a fixed sibling Rust worker. Exact full GPU interface/PnP and VM GUID/name selection, effective management-rights checks, structured availability/native errors, bounded streams, suspended launch/job containment and independent watchdog preserve trustworthy read-only reporting. Parent checks the worker's compiled configuration digest, required fact keys and known identities; a stale worker fails with rebuild guidance. Exact detail queries isolate unrelated null provider properties; VM counting is separate. Removed the PowerShell-module diagnostic dependency.
- Windows x64 read-only parity passed for all 14 host/GPU/VM facts on host 26300.9457, RTX 5060 / driver 32.0.16.1692 (oem59.inf), configured running Gen2 VM / configuration version 12.0. Corrected run: `local/evidence/core024-parity.json`, 2026-10-06T16:24:57Z, retaining outputs and artifact/config hashes. No guest workload qualification is claimed by inventory parity.
- Validation: focused native identity/provider/selection, stale-worker, malformed-protocol, launch/exit/timeout/overflow/watchdog tests and public worker-argument rejection pass. Full `scripts/testing/check.ps1` passed formatting, strict Clippy/compiler warnings, all workspace tests/doc-tests, build, rustdoc and configuration/policy drift with `RUST_TEST_THREADS=4`; the initial default-concurrency run exhausted existing PowerShell fixture deadlines. Documentation checks passed. Independent architecture review closed stale-worker and unrelated-provider-field findings and reported no remaining blockers in implementation commit `5f4e514`.

## CORE-025

Historical standalone-lab card; original results below are retained. Root product
replacement/qualification belongs to ARCH-001, not this legacy procedure. Source
paths below refer to the pre-rebase tree (now tools/lab), unless stated otherwise.

**Port fixed Hyper-V and GPU-PV operations to native Rust**

- Owner: Codex; first native read slice delivered 2026-10-06; remaining port in progress.
- Active continuation 2026-10-07: qualify the native settings/attachment/lifecycle implementation already present in source, with independent milestone review before live mutations. Existing PowerShell bodies are now compiled only as test references; previous first-read-slice results do not qualify the full native mutation path.
- Objective: replace product cmdlet bodies in `hyper-gpu-runner.rs` and `hyper_gpu_runner/settings_adapter.rs`, retaining the enrolled fixed-operation protocol and existing Rust policy/settings contracts.
- Replacement: investigate/use Hyper-V WMI/COM for VM/GPU discovery, attachment/removal, resource triples, MMIO/cache/memory/CPU settings and guest lifecycle; native Virtual Disk APIs for read-only disk identity/chain safety where applicable. Differencing-disk creation/deletion and laboratory reset are development-only and do not gate this product port. Reuse provider bindings rather than recreate Hyper-V or reopen HCS-owned guests.
- Deliver incrementally: read/inspect first, adapter/settings next, product lifecycle last; disposable reset is optional test support. Preserve VM-Off requirements, exact GPU/VM/disk identity, other-VM assignment checks, golden-parent protection, Secure Boot/vTPM, snapshots refusal, durable preimages, independent readback and uncertain-state reconciliation.
- Tests: provider errors/job completion, denied access, stale preimages, wrong target/parent/adapter, resource limits including u64 values, no-op, partial mutations and bounded timeout/reconciliation. Run affected live apply/readback/reapply and child recovery on the verified disposable target; essential workloads qualify the changed configuration path.
- Acceptance: fixed operations no longer launch PowerShell for these capabilities; independent architecture review of changed privileged boundaries and actual tests passes before deployment. Any capability lacking a practical native interface gets a specific DECISIONS exception with alternatives and exact bounded command; no broad Hyper-V exemption.
- Implementation: fixed native Rust workers now own inspection, adapter/resource/profile reads and writes and guest lifecycle; Virtual Disk APIs guard the protected chain. Historical cmdlet bodies remain only in test compilation. Contained subprocesses and independent watchdogs bound COM, with configuration/enrollment/policy/identity guards, durable preimages, stale-state refusal and independent effective readback. CORE-002/023 retain their original cmdlet-backed evidence; full native mutation/workload qualification remains pending.
- Read-only release Windows x64 parity passed all 18 fields on the exact configured Off Gen2 VM (version 12.0), host build 26300.9457, RTX 5060 / driver 32.0.16.1692. Evidence: `local/evidence/core025-read-parity.json`. This is management-read parity, not guest workload qualification.
- After review, the existing maintained restore/install paths updated the controlled runner and artifact pins. Installed-runner `inspect` passed (`1791320566-017033300`), including contained native worker launch, observation comparison, protected parent hash/chain checks and successful audit/result publication; the exact Off VM had zero guest GPU adapters. Installed runner hash matched its release pin. Evidence: `local/evidence/core025-installed-inspect.json` and `core025-installed-inspect.log`; setup result in `core025-runner-update.json`.
- Validation: `scripts/testing/check.ps1` passed formatting, strict Clippy/compiler warnings, all workspace tests/doc-tests, build, rustdoc and configuration/policy drift (`RUST_TEST_THREADS=4`). Focused tests cover exact/wrong/missing/duplicate identities, unsafe states, missing/malformed/overflowing resource values, stale configuration, and retained inspection with host GPU cmdlets deliberately unavailable. Independent architecture review reported no blockers; release runner/client/rights build, four artifact-pin checks and documentation checks passed.

## CORE-026

**Guest provisioning port — absorbed by ARCH-001**

- Root Rust guest writer, native payload/trust and checked D3D11 are implemented.
  DEC-028 authorizes the fixed session/transfer/bootstrap/launch bridge only.
- Result: merged into ARCH-001; M2 one-VM preparation/reapply/rendering passed,
  current-build observed repeat subsequently passed; affected fresh preparation remains. Historical lab PowerShell migration is not a
  separate product prerequisite. Extended CUDA/D3D12 remain contributor diagnostics.

## CORE-027

**Native product installation/enrollment — completed under M1**

- Result: root runner native installation, exact multi-target VM/GPU enrollment,
  protected artifacts/state and interrupted-install admission/recovery qualified;
  independent review cleared M1. See [M1 acceptance](evidence/M1.md).
- Reuse src/runner.rs. GUI setup and runtime intent integration belong to CORE-021/
  GUI-001; packaged update/recovery instructions belong to CORE-017/DOC-003.
  No S4U/LSA/one-slot lab installer port or installer redesign is required.

## CORE-011

**Standalone guest lifecycle commands — deferred**

- Root apply/verify already use bounded start/graceful shutdown and power restoration.
  Standalone start/shutdown/restart commands are not required for the GPU-PV journey.
- Schedule only for a concrete operator need; preserve identity, finite waits and
  graceful refusal. Do not add lab reset/forced-stop commands to production.

## CORE-010

**Removal and recovery — absorbed by ARCH-001**

- Implemented: exact disable, settings attribution/restoration, preserved preparation,
  durable pending state and retry/reconciliation, including failed verification and
  stale-refresh recovery. Existing workflow tests cover meaningful interruption paths.
- Result: merged; M2 disable/reapply passed. Remaining recovery presentation belongs
  to CORE-012/GUI-001 and stability qualification to ARCH-001/GPU-012. No disk reset.

## CORE-012

**Finish useful diagnostics and truthful observed state**

- Reuse root observed state, journals, audit and existing errors. Distinguish intent,
  attachment, prepared digest, pending recovery and last successful graphics timestamp.
  Status does not freshly authenticate guest preparation or run graphics checks.
- Isolate per-VM discovery errors where practical; missing/denied/unavailable is not
  an empty successful result. Give concise stage/context and next safe action.
- Acceptance: focused partial-access, pending/stale and secret-redaction checks;
  CLI/GUI agree. Keep diagnostic details bounded/on demand, no telemetry service.
- Result: underlying state/errors implemented; human summaries/partial results pending.

## CORE-021

**Finish runtime configuration and enrollment UX**

- Implemented: schema 2 deserialized/validated at boundary, multiple exact targets,
  no lab paths/pins; actual CLI commands and native administrator enrollment.
- Read: src/model.rs, src/main.rs, src/runner.rs, src/windows_gui.rs and CONFIGURATION.md.
- Remaining: useful schema-2 selection/config flow, clear protected re-enrollment
  for VM/GPU changes, safe GUI persistence and credential/vault parity.
- Acceptance: packaged instructions or guided GUI flow reach an enrolled pair;
  unapproved pair cannot mutate, failed config save cannot cause blind replay;
  examples/help match inventory/install/plan/apply/enable/disable/status/verify/
  credentials/forget. No new settings authority or general elevated command endpoint.
- Result: core configuration complete; operator/GUI integration remains in progress.

## CORE-015

**Driver drift refresh — absorbed by ARCH-001**

- Implemented: dynamic current-driver payload, digest comparison, stale-receipt
  invalidation before refresh, completed-preparation retention and rollback/retry tests.
- Result: merged. Remaining drift explanation belongs to CORE-006/CORE-012; affected
  refresh qualification to ARCH-001/GPU-012. No fresh-child/reset prerequisite or
  manufactured host-driver transition; GPU-013 retains optional real-update testing.

## GPU-012

**Bounded core stability and maintenance qualification**

- BLK-005 investigation is closed by user direction; no further hang campaign.
  Reuse M1/M2 passes and existing workflow recovery tests.
- After clearance, verify exact designated target and run only affected NVIDIA
  preparation/default attachment/PnP/checked D3D11/reapply/disable/restoration.
  Include one representative recovery and refresh when the changed path needs it.
- Acceptance: bounded stable-host result, correct state/readback and no essential
  unresolved failure. Report actual environment, duration and limitations.
- No two recreated children, mandatory CUDA/D3D12/nvidia-smi campaign, prolonged
  stress, host reboot or review-of-review. Review materially changed boundaries.
- Result: current-build observed attach/render/reapply/verify/disable/cleanup passed;
  original hang cause unresolved; fresh preparation under new child limits remains.

## CORE-017

**Build the CLI/GUI Windows x64 candidate**

- Package CLI, GUI, protected runner, guest worker and checked D3D11 probe from a
  clean locked build; root workspace excludes the standalone lab/reset helpers.
- Include version/revision/checksums/notices, runtime and native install/update/
  re-enrollment/interrupted-install recovery instructions. Verify unpack/help,
  absent prerequisites and all fixed sibling artifacts. Product MSVC CRT is static.
- Exclude proprietary drivers, media/disks, secrets and keys. Signing/publication
  choices apply when concrete distribution needs them; packaging does not publish.
- Acceptance: usable candidate without developer paths or lab setup. Reuse native
  installer; no new service/installer framework. GPU-014 owns final acceptance.
- Result: pending; packaging preparation may overlap GUI work.

## DOC-003

**Write the tested CLI/GUI operator guide**

- Document actual schema-2 selection, administrator install/enrollment, plan/apply/
  enable/disable/status/verify, credentials/forget, drift refresh and pending recovery.
  Include GUI draft/preview/effective-state behavior and setup route.
- Acceptance: candidate instructions work without hidden developer paths; clear
  qualified combinations, sharing/allocation limits and host-lifecycle boundary.
  No product validate/reset/standalone lifecycle command assumptions.
- Result: pending; write with CORE-017 and exercise under GPU-014.

## GPU-014

**Accept the packaged v1.0 CLI/GUI journey**

- Dependencies: qualified M2, GUI-001 M3, CORE-017 and DOC-003.
- Use candidate artifacts/guide on the existing designated disposable test VM;
  verify setup/enrollment, preview/apply/PnP/checked D3D11/status, no-op and disable,
  plus representative interruption/recovery. Repeat only changed package paths.
- Acceptance: exact candidate/environment record, CLI/GUI parity, stable host,
  preserved unrelated state and no essential/security blocker. No mandatory reset,
  golden image, CUDA/D3D12 or manufactured driver update.
- Deliver local artifacts/notices/limitations/recovery; publication separately requested.
- Result: pending.

## Optional and later work

## GPU-010

**Qualify resource units and enforcement before ordinary controls**

- Class: M4. Root optional VRAM triples/provider range validation/native setter
  already exist; do not reimplement them or restore the historical 50% profile.
- Acceptance: establish actual units, bounds, effective readback and enforcement
  on selected hardware before UI controls. No fairness/performance SLA or host
  partition-count changes. Provider defaults remain the normal release path.
- Result: deferred; meaning/enforcement unqualified.

## GPU-007

**One optional API capability**

- Class: useful if cheap, otherwise post-v1. Select one concrete workload such as OpenGL/Vulkan/OpenCL or video before scheduling; do not add speculative capability fixes to v1.
- Acceptance: checked before/after results and essential regressions for that workload.
- Result: no optional workload selected.

## GPU-013

**A real signed host-driver transition**

- Class: useful if safe and cheap, otherwise post-v1. Qualify a different installed signed driver when an actual owner-controlled update occurs; do not manufacture host servicing for a release gate.
- Acceptance: new discovered inventory, explicit restage, essential passes and viable guest recovery. Host reboot always requires immediate explicit permission.
- Result: no transition run is required for v1.

## GPU-015

**Qualify conditional same-GPU sharing**

- Class: separate capability; multiple selected VMs are current product intent.
  Simultaneous sharing remains unqualified, not automatically supported.
- Acceptance: two explicitly designated test VMs, independent attach/render/reapply,
  disabling one preserves the other, stable host and exact tested combination.
  Record core admission/support policy before exposing concurrent operations.
- Reuse enrollment/per-VM journals/global serialization; no scheduler/fairness benchmark.
- Result: deferred until stable M2 and a second designated test VM.

## GPU-016

**CUDA/D3D identity correlation and interoperability**

- Class: post-v1. Define cross-namespace identity correlation and test interop only for a chosen workload. Standalone checked CUDA computation already passes.
- Result: the diagnostic reports different host/CUDA and guest graphics LUIDs; see the [baseline](evidence/GPU-PV-BASELINE.md). Promote only a demonstrated essential device-selection/computation failure to v1.

## GPU-017

**Reduce the complete provisioning/settings recipe**

- Class: post-v1 optimisation. Compare isolated reductions to the full recipe on recoverable clean children with sustained readiness and essential checked workloads.
- Result: historical full-recipe necessity is unisolated; root M2 already uses
  provider defaults and minimal attributable settings. No recipe-minimization
  campaign is needed to ship v1.

## Completed and retired ID anchors

These compact results preserve useful incoming task links without historical instructions.

### DOC-001

Established owned documentation and task IDs.

### DOC-002

Established the original release plan; current sequencing replaces its research gates.

### DOC-007

Established shared [engineering standards](ENGINEERING.md).

### DOC-008

Established the one-VM/disposable-child architecture.

### DOC-009

Configured independent architecture review.

### DOC-010

Established autonomous development authorization.

### DOC-011

Established maintained script layout and physical-host session protection.

### CORE-019

Created the Rust library/CLI, hardware-free tests and pinned Windows CI.

### GPU-001

Initial recipe research completed; no future lookup dependency.

### HV-001

Target inventory completed; [evidence](evidence/HV-001.md).

### REF-001

Historical repository provenance captured; no active upstream workflow.

### REF-004

Established unmodified Windows media and protected parent/disposable-child layout.

### HV-003

Installed management/privilege contract measured; [evidence](evidence/HV-003.md).

### GPU-002

Initial package manifest measured; superseded as a complete recipe by GPU-009, while package validation is reused.

### REF-002

Historical artifact inspection completed; no reference artifact is needed by the product.

### GPU-008

Essential standalone probe inputs/oracles specified; [manifest](../probes/MANIFEST.md).

### GPU-003

Targeted provisioning and child recovery procedure established.

### HV-002

Protected parent and fixed disposable child established; [evidence](evidence/HV-002.md).

### GPU-009

Full normal-Hyper-V baseline and native discovery parity passed on 2026-10-05; [measured result/inventory](evidence/GPU-PV-BASELINE.md).

### GPU-005

Sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA passed; [baseline](evidence/GPU-PV-BASELINE.md).

### CORE-001

Rust inventory implemented; [evidence](evidence/CORE-001.md).

### CORE-004

Strict typed configuration/error/report contracts implemented; [evidence](evidence/CORE-004.md).

### CORE-005

Fixed target-bound runner inspect/reset/start/shutdown/attach/detach implemented and tested; [evidence](evidence/CORE-005.md).

### CORE-008

Bounded authenticated guest transfer implemented and tested; reused by staging.

### CORE-009

Package/alias writer, receipts, no-op and uncertain-child recovery tested; [evidence](evidence/CORE-009.md). Full associated-destination support remains CORE-022.

### CORE-002

Exact attachment and verified no-op implemented/tested on 2026-10-04; [evidence](evidence/CORE-002.md). Corrected the stale pending card; full settings remain CORE-023.

### CORE-020

Standalone checked D3D11/D3D12/CUDA probes built/tested; [evidence](evidence/CORE-020.md).

### GPU-004

Cancelled: Reference comparison removed completely; no deferred replacement.

### CORE-007

Merged: Reuse fixed runner audit/locking within the public workflow. Owner: CORE-006.

### CORE-013

Merged: Existing Windows CI is maintained with each meaningful change. Owner: implementation tasks.

### CORE-014

Merged: Focused interruption/recovery checks belong to the writer and recovery implementation. Owner: CORE-022, CORE-010.

### GPU-011

Merged: Live lifecycle checks belong to repeatability qualification. Owner: GPU-012.

### CORE-016

Merged: Independent review remains at actual implementation/privilege gates; no separate review chain. Owner: milestone exits.

### CORE-018

Merged: Public schema/help/report stability belongs to config/CLI cleanup. Owner: CORE-021.

### DOC-004

Merged: Resolve concrete payload/license/signing/publication choices at packaging, without blocking implementation. Owner: CORE-017.

### REF-003

Merged: Actual shipped component notices belong to package acceptance. Owner: CORE-017.

### DOC-005

Merged: Final checklist belongs to the packaged acceptance run. Owner: GPU-014.

### DOC-006

Merged: Final artifacts and handover belong to the same acceptance/delivery result. Owner: GPU-014.

## ARCH-001

Runtime existing-VM architecture and small GPU-PV core.

- Owner: Codex; claimed 2026-10-07 for the explicit user-approved refactoring plan.
- M1: completed 2026-10-08; existing-VM enrollment, product/laboratory contracts
  and full rewritten runner boundary independently reviewed and live-qualified.
- Replace laboratory-bound product contracts, runtime enrollment, current-driver
  preparation, workflow/recovery and ordinary health-plus-graphics verification.
- Preserve the previous application as standalone contributor tooling. Production
  must not depend on its configuration, golden disks, reset or extended probe suite.
- Acceptance: default product checks; independent privilege-boundary review; live
  discovery/enrollment and NVIDIA/default-resource preparation, rendering, reapply,
  disable/restoration. Two disposable VMs must qualify sharing before it is advertised.
- Implementation result: runtime schema 2, native discovery/management, typed
  installed runner, per-VM journal, Rust guest writer, catalog-member trust and
  fixed transport bridge are implemented. Independent reviews cleared corrected
  boundaries for controlled qualification. Root strict checks passed; current
  focused library tests pass (29), CLI tests pass (3). Standalone lab compiled and
  its library/bin tests passed; the relocated CLI test path was corrected and all
  11 CLI tests passed. Native installation, enrollment, status and current-driver
  trust/preview passed on the designated existing VM (host 10.0.26300.0, RTX 5060,
  driver 32.0.16.1692). Initial apply retained pending state after WMI 0x8004101E;
  corrected GetMethod to read class metadata instead of an instance. Controlled
  guest retry remains M2 work; no new rendering/support claim yet. Host lifecycle
  remains outside authorization.
- M1 result (2026-10-08): installation validates runtime intent and Generation 2
  before effects; durable credential-free runner auditing closes the independent
  review finding. Product gate passed (39 library tests, 3 CLI tests, strict Clippy,
  formatting, build, rustdoc and documentation); production graph excludes the lab,
  which builds separately. Live native install/discovery/status/current-driver plan,
  invalid enrollment refusal, interrupted-install denial/recovery, task/artifact
  checks and ordinary-token authorization/audit/write-denial qualification passed.
  The independent reviewer cleared M1 closure. Exact environment and commands are
  in [M1 acceptance](evidence/M1.md). Pending preparation remains M2 work.
- M2 started (2026-10-08), owner Codex: running no-op verification now journals
  pending intent before checking; standalone verification restores initial power
  after an uncertain start, retains check and recovery errors together, and leaves
  failed work pending for retry. Graceful shutdown refusal stops apply before
  preparation or attachment. Independent bounded review found no blockers;
  11 focused workflow tests and the product gate passed (43 library, 3 CLI tests,
  strict Clippy/compiler warnings, formatting, build, rustdoc and documentation).
  Installed-runner `status` freshly confirmed the selected Generation 2 VM Off,
  no GPU attached, pending preparation and no prepared digest. Next: qualify
  current NVIDIA preparation, provider-default attachment and checked rendering,
  then running reapply and disable/restoration. No live guest qualification or
  deployment of this recovery change is claimed.
- M2 continuation (2026-10-08): current complete NVIDIA preparation, native default
  attachment, PnP health and checked D3D11 rendering passed on the designated VM.
  Off/running reapply, standalone verification and disable passed; running reapply
  retained uptime and disable retained preparation. Final guest Off, no adapter,
  no pending operation; CPU/RAM, disk identity, Secure Boot and TPM preserved.
  Fixed WMI host-resource identity/readback, nullable defaults and failed-job
  diagnostics. Recovery invalidates stale receipts before refresh, preserves
  completed preparation after later failure, and covers host-driver rollback.
  Product gate passed: 49 core and 3 CLI tests, strict Clippy/compiler warnings,
  formatting/build/docs; release x64 build passed. Focused independent reviews
  cleared deployment. Exact environment and scope: [M2 acceptance](evidence/M2.md).
  Sharing requires a second explicitly designated test VM before qualification.
- Host-failure follow-up: user reported a freeze/crash during this session; host
  rebooted at 14:08:37 UTC. Kernel-Power 41 reports bugcheck 0; no recent resource-
  exhaustion event was found. Pre-failure RAM/commit data was not captured, so
  memory pressure/process growth is unresolved. Live testing stopped; successful
  command results do not close host stability or M2 acceptance.
  User clarified progressive slowdown, mouse-movement beeps and manual hard
  power-off. Diagnose the hang; do not treat the restart event as its root cause.

- Observed follow-up (2026-10-08): user authorized bounded reproduction despite
  hang risk. Rebuilt/reviewed/installed revision `96152e7`; 52 core and 3 CLI tests
  plus strict product gates passed. Current-build default attachment, PnP/checked
  D3D11, Off/running reapply, verify, disable and graceful cleanup passed. Final
  Off/no GPU/pending=false; disk/CPU/RAM/Secure Boot/TPM preserved. User reported
  no slowdown/beeps; host resource capture showed no runaway or long sample stall.
  Fixed the temporary contributor helper's synchronous shutdown wait using a
  bounded asynchronous request and Off readback; current-build cleanup passed.
  Preparation was reused, so fresh transfer/writing under the new child limits
  remains unqualified. The original hang's cause is unknown; see [M2](evidence/M2.md).
- User closure (2026-10-08): close hang investigation/reproduction/observation
  work and remove BLK-005 as a blocker. Preserve results without claiming a cause
  or fix. Next active product task is CORE-006 shared preview.

## GUI-001

**Native Windows GUI over the shared core**

- Owner: Codex; prototype in progress since 2026-10-07. Win32 controls and background
  runner integration already exist in src/windows_gui.rs; reuse them.
- Plan: [written GUI specification](gui_roadmap.md), G1 layout → G2 truthful reads →
  G3 draft/shared preview → G4 operation/persistence/close handling → G5 setup/pages
  → G6 qualified usability/journey. These are substeps, not new task IDs.
- Layout/read-only work may proceed; live GUI acceptance depends on M2 closure.
  Fresh preview depends on CORE-006; enrollment/persistence integrates CORE-021.
- Acceptance: sidebar/header/five-column table/panel/footer, one-VM staged Apply/
  Discard, real observed versus desired/prepared/verified state, responsive native
  operation, DPI/resize/keyboard/accessibility and CLI parity after M2 clearance.
- Respect protected VM/GPU pair; no unqualified GiB sliders or implicit sharing.
  No toolkit replacement or duplicate backend.
- Result: functional scaffolding exists; layout and integration gaps remain.
