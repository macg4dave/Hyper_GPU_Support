# Backlog

## Resume

The normal Generation 2 Windows 11 RTX 5060 GPU-PV baseline is proven: sustained
Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA computation. Native Rust discovery
matches the complete measured inventory. The product work remaining is integration,
usable operation and release; no feasibility or external-reference task remains.

CORE-022 complete-manifest staging and verified reapply passed on a clean child;
CORE-023 settings apply and verified no-op reapply passed on that existing child.
CORE-003 automated readiness/workloads is implemented and behavior-tested.
Next work is **GPU-006**, exercising their combined Rust workflow on a clean child.
The public CLI/config cleanup and reusable lifecycle/diagnostics work are also actionable.
**CORE-024** native inventory is complete with exact-target binding and read-only
parity. **CORE-025** has delivered its first exact-VM/host-GPU native read slice;
continue guest-adapter/profile reads and disk guards before native mutations,
followed by CORE-026 guest writer/transport and CORE-027
runner setup/recovery. GPU-006 retains its current owner and qualification work;
its result does not discharge these production PowerShell dependencies. See the
[complete script/backend audit](../scripts/PRODUCT-MIGRATION.md).
Current starting evidence: [project baseline](evidence/GPU-PV-BASELINE.md).
Do not assume the preserved experimental guest is a production Rust reproduction.

## Task register

This register alone owns current status, priority, milestone and dependencies.
Dependencies are real completion prerequisites, not an instruction to reread all
prior research. The completed foundation below already satisfies its dependencies.
M1/M2/M3 product tasks and qualification gates are required for v1; deferred classes
are outside the gate. Golden-image/disposable-target preparation is development
support, not shipped functionality. Qualification may reuse existing tooling.
Do not port laboratory setup merely to satisfy the Rust product migration gate.

| ID | Milestone/class | Priority | Status | Depends on |
|---|---|---|---|---|
| [CORE-022](#core-022) | M1 | P0 | completed | CORE-009 |
| [CORE-023](#core-023) | M1 | P0 | completed | CORE-002 |
| [CORE-003](#core-003) | M1 | P0 | completed | CORE-020 |
| [GPU-006](#gpu-006) | M1 | P0 | in progress | CORE-022, CORE-023, CORE-003 |
| [CORE-024](#core-024) | M1 | P0 | completed | CORE-020 |
| [CORE-025](#core-025) | M1 | P0 | in progress | CORE-024, CORE-023 |
| [CORE-026](#core-026) | M1 | P0 | ready | CORE-022, CORE-003 |
| [CORE-027](#core-027) | M2 | P1 | ready | CORE-005 |
| [CORE-006](#core-006) | M2 | P1 | planned | CORE-022, CORE-023 |
| [CORE-011](#core-011) | M2 | P1 | ready | CORE-005 |
| [CORE-010](#core-010) | M2 | P1 | ready | CORE-005 |
| [CORE-012](#core-012) | M2 | P1 | ready | CORE-020 |
| [CORE-021](#core-021) | M2 | P1 | ready | CORE-004 |
| [CORE-015](#core-015) | M2 | P1 | planned | CORE-022, CORE-006 |
| [GPU-012](#gpu-012) | M2 | P1 | planned | GPU-006, CORE-024, CORE-025, CORE-026, CORE-027, CORE-006, CORE-010, CORE-011, CORE-012, CORE-015, CORE-021 |
| [CORE-017](#core-017) | M3 | P1 | planned | CORE-027, CORE-006, CORE-010, CORE-011, CORE-012, CORE-015, CORE-021 |
| [DOC-003](#doc-003) | M3 | P1 | planned | CORE-006, CORE-010, CORE-011, CORE-015, CORE-021 |
| [GPU-014](#gpu-014) | M3 | P1 | planned | GPU-012, CORE-017, DOC-003 |
| [GPU-010](#gpu-010) | post-v1 | P2 | deferred | GPU-006 |
| [GPU-007](#gpu-007) | post-v1 | P2 | deferred | CORE-003 |
| [GPU-013](#gpu-013) | post-v1 | P2 | deferred | CORE-015 |
| [GPU-015](#gpu-015) | experimental | P2 | deferred | GPU-012 |
| [GPU-016](#gpu-016) | post-v1 | P2 | deferred | GPU-005 |
| [GPU-017](#gpu-017) | post-v1 | P2 | deferred | GPU-006 |

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
IDs are retained only in the mapping. A ready task has satisfied prerequisites.
Create a blocker only for an observed impediment. Update only documentation affected
by the behavior; a new evidence file is optional unless a hardware report needs it.
Each implementation task maintains existing hardware-free Windows CI and meaningful
behavior tests; no separate task to check previous checks. Follow [engineering](ENGINEERING.md)
and [targeting/authorization](../AGENTS.md#development-and-test-authorization).

## Blocker register

**No active delivery blocker is recorded.** This is a plan/evidence review, not a new
live-state inspection. Recheck actual identities/servicing before the next effect.
Missing implementation is represented by task dependencies. Publication/license
choices are handled when concrete package actions need them, not as a speculative
development blocker.

| ID | State | Resolution |
|---|---|---|
| BLK-001 | resolved 2026-09-25 | Approved elevated inventory established the management facts; [HV-001](evidence/HV-001.md). Use the approved runner for privileged work. |
| BLK-002 | resolved 2026-09-26 | Servicing/reboot impediment resolved; exact residual delete-only cleanup was reviewed. Unknown/active servicing is still rejected. |
| BLK-003 | closed 2026-10-05 | Optional historical guest-access comparison was discarded with GPU-004. Its transport was not repaired and has no reopening instruction or product dependency. |
| BLK-004 | resolved 2026-10-04 | Verified stage/no-op accepted build drift as qualification context and only exact reviewed delete cleanup; [CORE-009](evidence/CORE-009.md), DEC-022/023. |

## Required v1 tasks

## CORE-022

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

**Reproduce the proven baseline from a clean child through Rust**

- Objective: exercise the three implemented pieces as one configured Rust workflow.
- Acceptance: verify the enrolled disposable target and protected parent; recreate its child, apply the full manifest, read back settings/adapter, start and reach sustained Code 0, nvidia-smi and checked D3D11/D3D12/CUDA passes. Verify reapply and graceful shutdown; retain one concise run report with configuration/manifest identity, effective settings, versions and outputs. Temporary experimental provisioning scripts must not supply missing application behavior.
- Obtain independent M1 implementation review using the actual diff and test results. Recipe minimality and interop are outside this gate.
- Owner: Codex; started 2026-10-06. Combined clean-child qualification through existing Rust staging, fixed-runner settings/lifecycle and public validation entry points.
- Result: pending; the experimental baseline and native discovery parity are established, full Rust reproduction has not passed.

## CORE-006

**Connect public plan/apply/status to the proven Rust operations**

- Objective: replace public CLI stubs with one reviewable operator workflow; reuse the fixed runner's audit, serialization and reconciliation.
- Read: src/cli.rs, src/config.rs and existing client/runner operation contracts.
- Acceptance: plan shows exact target, settings, manifest changes and prerequisites without effects; apply rejects stale assumptions and wrong targets, orders staging/settings/start safely, and reports verified success or no-op. Status distinguishes VM state, staging, assignment and GPU readiness. Bind mutation to rechecked identities and existing locks/receipts. Test stale plans, concurrent/replayed requests and external state changes.
- Run full native manifest discovery behind the existing bounded process supervision: synchronous COM setup/object resolution can otherwise escape its enumeration deadline. Test timeout, child termination/reaping and bounded failure reporting; CORE-015 reuses this supervised path. No new resident worker or arbitrary privileged command interface.
- CORE-007 is absorbed here; no general transaction engine or second privilege channel.
- Own the ordinary staging/settings/start order currently exercised by `run-clean-child-qualification.ps1`; keep that script only as an optional acceptance harness. Integrate CORE-024/025/026 replacements as they land, never launch the harness from apply. Existing helpers allow integration to start before all backend ports finish; GPU-012 requires the native replacements.
- Result: pending; operational helpers exist, most public commands currently return exit 70.

## CORE-024

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

**Port fixed Hyper-V and GPU-PV operations to native Rust**

- Owner: Codex; first native read slice delivered 2026-10-06; remaining port in progress.
- Objective: replace product cmdlet bodies in `hyper-gpu-runner.rs` and `hyper_gpu_runner/settings_adapter.rs`, retaining the enrolled fixed-operation protocol and existing Rust policy/settings contracts.
- Replacement: investigate/use Hyper-V WMI/COM for VM/GPU discovery, attachment/removal, resource triples, MMIO/cache/memory/CPU settings and guest lifecycle; native Virtual Disk APIs for read-only disk identity/chain safety where applicable. Differencing-disk creation/deletion and laboratory reset are development-only and do not gate this product port. Reuse provider bindings rather than recreate Hyper-V or reopen HCS-owned guests.
- Deliver incrementally: read/inspect first, adapter/settings next, product lifecycle last; disposable reset is optional test support. Preserve VM-Off requirements, exact GPU/VM/disk identity, other-VM assignment checks, golden-parent protection, Secure Boot/vTPM, snapshots refusal, durable preimages, independent readback and uncertain-state reconciliation.
- Tests: provider errors/job completion, denied access, stale preimages, wrong target/parent/adapter, resource limits including u64 values, no-op, partial mutations and bounded timeout/reconciliation. Run affected live apply/readback/reapply and child recovery on the verified disposable target; essential workloads qualify the changed configuration path.
- Acceptance: fixed operations no longer launch PowerShell for these capabilities; independent architecture review of changed privileged boundaries and actual tests passes before deployment. Any capability lacking a practical native interface gets a specific DECISIONS exception with alternatives and exact bounded command; no broad Hyper-V exemption.
- Result: first native read slice implements exact VM identity/state/generation/version and all twelve host GPU capability values in the same pinned runner's fixed `read-hyperv` mode. A contained subprocess plus independent watchdog bounds COM; configuration/identity/unsigned-range validation fails closed. Runner `inspect` uses this native observation and omits the host GPU cmdlet, comparing it with the retained disk/snapshot/guest-adapter guard results. Full native inspection/settings/mutations remain pending; CORE-002/023 remain cmdlet-backed.
- Read-only release Windows x64 parity passed all 18 fields on the exact configured Off Gen2 VM (version 12.0), host build 26300.9457, RTX 5060 / driver 32.0.16.1692. Evidence: `local/evidence/core025-read-parity.json`. This is management-read parity, not guest workload qualification.
- After review, the existing maintained restore/install paths updated the controlled runner and artifact pins. Installed-runner `inspect` passed (`1791320566-017033300`), including contained native worker launch, observation comparison, protected parent hash/chain checks and successful audit/result publication; the exact Off VM had zero guest GPU adapters. Installed runner hash matched its release pin. Evidence: `local/evidence/core025-installed-inspect.json` and `core025-installed-inspect.log`; setup result in `core025-runner-update.json`.
- Validation: `scripts/testing/check.ps1` passed formatting, strict Clippy/compiler warnings, all workspace tests/doc-tests, build, rustdoc and configuration/policy drift (`RUST_TEST_THREADS=4`). Focused tests cover exact/wrong/missing/duplicate identities, unsafe states, missing/malformed/overflowing resource values, stale configuration, and retained inspection with host GPU cmdlets deliberately unavailable. Independent architecture review reported no blockers; release runner/client/rights build, four artifact-pin checks and documentation checks passed.

## CORE-026

**Move guest provisioning into a Rust writer and narrow remoting glue**

- Objective: replace product guest-side PowerShell in `windows_guest.rs`, `windows_environment_staging.rs` and `validation_transport.ps1`; reuse complete native manifest and Rust validation worker.
- Replacement: fixed Rust guest writer/bootstrap owns Windows-root destination calculation, filesystem/security/reparse/servicing validation, locks, byte copies, lengths/hashes/versions, atomic receipts and verified no-op; Rust owns fixed validation input preparation and launch supervision. Preserve current credential, guest-identity and restricted destination/source contracts.
- Investigate the supported session/transfer interface separately from writer logic. Prefer a practical native API; if PowerShell Direct is essential for authenticated bootstrap/transport, document the exact remaining session/transfer/launch calls and why investigated COM/WMI/Win32 routes cannot provide them. DEC-020 is historical rationale, not approval for retaining file/receipt/security logic in PowerShell. No arbitrary guest command channel or resident service.
- Tests: clean copy and discovered source/destination mapping with variable-size artificial manifests, tampered/missing files, ACL/reparse escapes, destination collisions, wrong guest, servicing, stale receipts, partial publication, lost remoting and timeouts. Retain the historical mapping fixture separately; no fixed file count is a general acceptance condition. Never infer cancellation from a killed host transport; retain uncertain-state recovery.
- Acceptance: complete fresh provisioning and verified reapply pass against the working baseline; Rust readiness/nvidia-smi/D3D11/D3D12/CUDA passes through the changed boundary. Preserve scripts until demonstrated. Independent security/architecture review passes; remove migrated embedded logic and keep only a specifically justified interface exception, if required.
- Result: pending; CORE-022 qualified the full recipe through a substantial PowerShell guest backend.

## CORE-027

**Provide Rust runner installation, enrollment and recovery**

- Objective: remove ordinary setup/recovery dependence on `install-runner-v1.ps1`, `restore-runner-v1.ps1` and the policy-generation portion of `update-project-pins.ps1`.
- Replacement: typed Rust installation/policy generation with native account/security and Task Scheduler interfaces, pinned artifact validation, administrator ACLs, exact-SID rights via the existing LSA helper, task quiescence and captured preimage restoration. Artifact pin maintenance and golden-image/disposable enrollment remain development tooling. Product enrollment pins an existing selected VM/GPU without requiring a golden image; preserve laboratory protections only in development setup. CORE-021 owns runtime configuration integration; CORE-017 owns packaging/prerequisites.
- Tests: bad pins/config, non-elevated refusal before effects, unrelated account/SID/task/path rejection, ACL escapes, in-flight runner, partial installation and interrupted restore. Demonstrate install/update/recovery through a bounded authorized elevation path with retained audit/preimages; no arbitrary elevated command input or host lifecycle.
- Acceptance: packaged Rust setup and recovery require no manual PowerShell; old scripts are optional developer recovery only after demonstrated replacement. Independent review of changed privileged/security boundaries passes. Do not redesign the runner or broaden its one-slot policy.
- Result: pending.

## CORE-011

**Expose bounded guest start/shutdown/restart**

- Objective: use the existing fixed runner lifecycle operations from the public CLI.
- Acceptance: start, graceful shutdown and guest restart have configured waits and useful timeout errors; report VM state separately from GPU health. Reject conflicting/saved state and unsafe target identities. Test running, off and unresponsive cases without a forced-stop default. GPU-012 owns live repeatability.
- Result: pending; fixed start/shutdown operations already work.

## CORE-010

**Expose safe removal and existing-VM recovery**

- Objective: integrate exact GPU detach and reconciliation/restaging of product-owned changes on the selected existing VM. Do not expose disposable disk reset as product recovery.
- Acceptance: remove only the exact project adapter in a safe state; recovery preserves the user VM/disk and reconciles observed state before retrying; uncertain guest provisioning requires explicit recovery guidance, not automatic disk replacement. Make repeated removal a verified no-op. Test wrong identity, partial assignment/staging, denied cleanup and representative interruption before/after effects; reconcile rather than report false success. Preserve useful recovery instructions when automatic completion is impossible.
- CORE-014 failure tests are incorporated here and in CORE-022; do not rebuild rollback machinery for the guest.
- Result: pending; fixed detach exists. Disposable reset remains reusable development tooling, excluded from the default build. Remove mandatory golden-parent/child configuration from ordinary use under CORE-021/027 without weakening the enrolled laboratory guards.

## CORE-012

**Report actionable failures and redacted diagnostics**

- Objective: explain failures at inventory, staging, assignment, guest readiness or workload using existing native results.
- Acceptance: provide concise human and versioned machine reports with observed/unknown distinctions, bounded relevant events, manifest/operation references and the next useful recovery action. Test partial access, missing logs, malformed guest output and secret redaction. Reuse current error categories; no automatic device cycling or broad telemetry service.
- Result: pending.

## CORE-021

**Finish runtime TOML configuration and the public CLI contract**

- Objective: make one operator-selected TOML file drive application behavior and stabilize help/config/report semantics.
- Read: docs/CONFIGURATION.md, src/config.rs, src/cli.rs and maintained script configuration readers.
- Acceptance: deserialize/validate once and pass typed values; discover Windows inventory and derive paths. Remove embedded development-machine target/path assumptions; avoid a second settings authority. Clarify installed runner policy regeneration/enrollment after configuration changes. Test examples, unknown fields/schema, missing inputs, exit codes and report redaction; help describes implemented commands accurately.
- CORE-018 schema/help stabilization is absorbed here. Fixtures, protocol values and fixed safety bounds stay in code. Finish affected command checks as those commands land.
- Result: pending; strict types/shared project configuration exist, product runtime selection and remaining cleanup do not.

## CORE-015

**Regenerate and restage after driver/environment drift**

- Objective: support an explicit driver update/re-stage workflow without installing or downgrading host drivers.
- Rediscover the current selected signed driver's packages and associated external files after updates. Generate fresh paths, lengths, hashes and manifest identity; the old manifest/count is comparison data, never the source for a new payload. Replace baseline package pins with generated per-environment integrity artifacts rather than silently bypass their checks or require an updated package to reproduce their old extent/digest. Keep operator target intent and signature validation separate from discovered inventory.
- Acceptance: detect changed driver/package/signature/manifest before apply/start; invalidate stale receipts and explain requalification. Regenerate the complete discovered inventory and create a reviewed fresh-child restaging plan; apply, verify and rerun essential probes. Report host/guest build drift as qualification context under DEC-022, not an invented build-equality prohibition. Active/unknown servicing still fails closed.
- Test changed-driver/build fixtures, stale/partial manifests and identity drift; rehearse regeneration/re-stage on the current signed host driver. A real different host-driver transition is optional GPU-013, not required to prove the workflow.
- Result: pending.

## GPU-012

**Qualify repeatable operation and maintenance**

- Objective: qualify the integrated one-VM workflow rather than repeat feasibility experiments.
- Acceptance: reproduce on two independently recreated children; verify no-op reapply, three guest lifecycle cycles including restart, representative interrupted-operation recovery, and a bounded sustained mixed essential-workload run. Put run durations/deadlines in configuration and report actual duration/correctness, device state and any host responsiveness problem. Stop on essential failure; fix and repeat only affected checks.
- Include the maintenance restage and user-facing diagnostics from owning implementation tasks. No mandatory host reboots, manufactured host-driver transition, resource-fairness benchmark or percentage performance SLA.
- GPU-011 lifecycle qualification and CORE-016 independent security review are absorbed here. Review actual privileged/credential/path boundaries at M2 closure; fix release-blocking findings. Reuse existing foundation tests instead of retesting each check for its own sake.
- Result: pending.

## CORE-017

**Build a traceable Windows x64 release package**

- Objective: produce a candidate from a clean locked Windows build with the tested CLI, examples and verification prerequisites.
- Acceptance: include revision/version, checksums, applicable project/dependency/probe notices and documented runtime requirements; test unpack/help and absent prerequisites. Exclude drivers, Windows images/disks, credentials and keys. Review actual CRT/probe redistribution rights or document legitimate local prerequisites; do not assume the developer runtime is installed.
- DOC-004 release/license/signing choices and REF-003 payload notice review are absorbed here. Prepare concrete local artifacts first; obtain owner choices only for unresolved licensing/signing/publication actions when they are actually needed. No installer or background service is required. Packaging and DOC-003 guide work may proceed together once commands are stable.
- Result: pending. Final acceptance/delivery belongs to GPU-014; packaging does not publish anything.

## DOC-003

**Write the tested operator and recovery guide**

- Objective: explain the actual one-VM workflow using runtime configuration and legitimate local OS/driver inputs.
- Acceptance: document prerequisites, approved runner setup, ephemeral guest credentials, config selection, plan/apply/status/validate, lifecycle, removal/recovery and driver re-stage. Commands match help and contain no hidden development-machine paths. State tested compatibility, known optional limits and host lifecycle permission. Reuse the architecture recipe and existing reports rather than create another evidence hierarchy.
- Result: pending; write alongside packaging, then exercise the guide in GPU-014.

## GPU-014

**Accept the packaged v1.0 workflow and deliver handover**

- Objective: use only candidate artifacts and the operator guide to reproduce on a clean disposable child.
- Acceptance: record candidate hashes and exact host/guest/driver environment; follow inventory/plan/apply/validate, restart, removal/recovery and current-driver re-stage. Require stable Code 0, nvidia-smi and checked D3D11/D3D12/CUDA with no undocumented developer setup. Verify applicable notices, complete required tasks and no open essential/security blocker.
- DOC-005's final checklist and DOC-006 handover are incorporated into this single acceptance run. Independent M3 review uses the candidate and actual results; no review-of-review task. Fix package/guide defects and repeat affected steps. Provide final local artifacts, compatibility/limitations, checksums and recovery instructions; publish only to a separately authorized destination if requested.
- Result: pending.

## Optional and later work

## GPU-010

**Resource-envelope and enforcement measurements**

- Class: post-v1. Measure allocation/headroom/enforcement only when changing the validated resource profile; no v1 fairness or performance SLA.
- Result: deferred; use the complete validated profile for v1.

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

**Multi-guest contention**

- Class: experimental and outside the one-VM product. Requires an explicit future scope change and separately verified targets; do not implement a scheduler.
- Result: deferred.

## GPU-016

**CUDA/D3D identity correlation and interoperability**

- Class: post-v1. Define cross-namespace identity correlation and test interop only for a chosen workload. Standalone checked CUDA computation already passes.
- Result: the diagnostic reports different host/CUDA and guest graphics LUIDs; see the [baseline](evidence/GPU-PV-BASELINE.md). Promote only a demonstrated essential device-selection/computation failure to v1.

## GPU-017

**Reduce the complete provisioning/settings recipe**

- Class: post-v1 optimisation. Compare isolated reductions to the full recipe on recoverable clean children with sustained readiness and essential checked workloads.
- Result: individual file/setting necessity is unisolated and is not needed to ship v1.

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
