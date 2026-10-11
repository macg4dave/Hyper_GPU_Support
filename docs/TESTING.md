# Testing and qualification

This page owns validation timing, intended checks, scenarios, and evidence. [Roadmap](ROADMAP.md) owns milestone gates; [backlog](BACKLOG.md) owns implementation work. Commands below describe the intended gate and are not a record that they ran.

## Timing and authorization

Normally run builds and automated tests when the agreed milestone is ready to close, or when the user explicitly requests them sooner. A task/subtask is not a milestone. Test code may be added earlier; report it as unrun or pending the gate. Do not validate every small edit or silently redefine a milestone.

Documentation-only work may check links, anchors, task/dependency consistency, instruction conflicts, and `git diff --check` without building or launching the application. This reset is limited to those checks.

Live setup/enrollment, VM/GPU effects, guest access, privileged writes, and hardware probes require explicit authorization for a concrete target and run sequence. Mock GUI preview must also be explicit; a live launch can prompt for elevation. Automated tests using temporary files, fake backends, or local IPC are distinct from hardware trials, but inspect their effects before admitting them to the default suite.

## Planned automated gate

V1-002 will rebuild a small non-elevated `scripts/testing/check.ps1` orchestrator and separate documentation checker. Neither currently exists after the user's deletions. The default gate must fail at the first failure, retain useful local output, and report skipped later stages accurately.

| Order | Check | Intended command/behavior |
|---|---|---|
| 1 | Documentation | Local links/anchors, fresh IDs/dependencies, required prompt metadata, and diff whitespace; no external link crawler required |
| 2 | Formatting | `cargo fmt --all -- --check` |
| 3 | Non-hardware tests | `cargo test --locked --all-targets`, after source inspection confirms the selected tests cannot access/mutate VMs or run GPU workloads; split/exclude unsafe suites explicitly if discovered |
| 4 | Strict lint | `cargo clippy --locked --all-targets -- -D warnings` |
| 5 | Product binaries | `cargo build --locked --bins` |
| 6 | API documentation | `cargo doc --locked --no-deps` |

The manifests select Windows x64 and the pinned toolchain. The future gate must verify prerequisites, not install software or request elevation. Tests must be meaningful for decisions and failure boundaries rather than mirror implementation. Retained inline tests cover models/plans/enrollment/recovery, reporting, parsing, transfer state, and IPC; their existence is not a recorded pass. Deleted integration suites and CI are reassessed individually under the backlog.

This gate does not launch the GUI, install the package, store credentials, modify protected configuration/enrollment, operate VMs, execute D3D/CUDA probes, or claim rendering/host stability. If a selected test unexpectedly needs such effects, stop and separate it before gate execution.

## Non-hardware acceptance scenarios

| Scenario | Required observation | Tasks |
|---|---|---|
| Intent/schema compatibility | Schema 2 loads without rewrite; schema 3 preserves supported optional categories; malformed/partial/unknown inputs are rejected | V1-004, V1-008 |
| Additive enrollment | Second reviewed pair preserves first; stale revision/operator, duplicate VM, and replacement attempts are refused | V1-006, V1-010 |
| Plan freshness and target binding | Changed identity/power/provider/driver/payload/intent/authority cannot reuse stale approval | V1-007, V1-010 |
| Draft and refresh conflicts | Selection/drafts survive appropriate refresh; external edits/authority changes block execution until explicit resolution | V1-008 |
| Resource display | Supported categories validate; omission means no explicit write; native VRAM conversion stays exact within supported bounds; historical inputs never imply current GB provenance | V1-008 |
| Publication failure | Verified-but-unsaved result remains distinct; save-only retry publishes only bound intent, with no GPU effect replay | V1-011 |
| Interrupted effects | Missing terminal outcome retains recovery; only independently established state permits reconciliation/retirement | V1-011 |
| Setup/enrollment/import recovery | Domain-specific receipts and stale/conflicting inputs remain separate; import cannot grant effect authority | V1-011 |
| IPC and transport containment | Wrong peer/session, oversized frames, output/byte budgets, partial transfer, timeout, and cancellation lifetime are bounded and truthful | V1-010, V1-012 |
| Mock/snapshot | No persistent writes, credential access, or effects; changed inputs/missing recorded payload block rehearsal | V1-009 |

At G1 completion, an explicit fixture preview also checks keyboard navigation, long names, many/no VMs, scrolling, themes, modal progress, dirty navigation, and failure/recovery presentation. Record screenshots as simulation, never live evidence.

## Live qualification scenarios

Before Q1 choose exact host/guest builds, GPU/driver, two existing Generation 2 VM identities, checked workloads, observation duration, package revision/hashes, and stop conditions. Put mutable values in local configuration/evidence. V1-014 prepares the reviewable run sheet; V1-015 executes only separately authorized steps.

| Scenario | Acceptance |
|---|---|
| Initial setup and second pairing | Setup/package review is separate from GPU review; adding the second pair preserves the first pair and its settings |
| Concurrent rendering | Both guests run checked hardware workloads during a recorded overlapping interval; each identifies the intended GPU-PV hardware; sequential Verify passes alone are insufficient |
| Independent enable/disable | Operating one selected guest preserves the other guest's attachment/settings and checked rendering results |
| Reapply/Update while sharing | Reviewed selected-guest preparation/settings work completes or reports its true outcome while the other guest's workload is checked continuously |
| Saved settings and reopen | Committed per-VM intent survives application reopen; drafts are not silently saved; actual observation and last verification remain distinct |
| Guest lifecycle and Verify | Shutdown/start/restore matches approval and original power; checked D3D11 device/rendering result is bound to the selected guest/GPU |
| Failure and recovery | Safely applicable, explicitly approved scenarios preserve evidence and holds; save-only does not replay GPU work; changed/partial state stays blocked |
| Host observation | Host remains responsive and no lockup is observed during the defined run interval; failures retain the last operation record |

Do not induce host hangs, crash VMs, force power-off, restart the host, or run additional stress/fault trials merely to fill this table. Software fault injection should cover unsafe cases. Any live fault scenario needs its own concrete effect review and authorization. Stop on uncertain outcome; no automatic retry or unrequested post-run investigation.

P1 adds separately authorized clean package setup and upgrade trials: required artifacts, hash/trust checks, configuration/enrollment preservation, truthful missing-artifact failures, and accurate operator instructions.

## Evidence and claims

Use unique ignored local evidence directories. Flush timestamped records at operation boundaries so a crash retains target, stage, durations, result/error, and the last known operation. Include revision/working-tree context, exact environment, approvals/sequence, workload overlap, and terminal results. Exclude credentials and untrusted secret-bearing output.

Record **Passed**, **Failed**, **Not run**, and **Unknown** honestly. Failure retains its evidence and leaves the relevant gate open. A scoped pass supports the recorded combination and interval, not universal stability. Attachment, Code 0 alone, mock success, allocation readback, or sequential rendering cannot prove simultaneous sharing or resource isolation.
