# Security and recovery

This page owns product authority/trust boundaries and recovery semantics. [AGENTS.md](../AGENTS.md) owns agent authorization rules; [testing](ENGINEERING.md#validation-and-live-acceptance) owns qualification scenarios. These are contracts to preserve and validate, not a claim that static review proves their enforcement.

## Authority and trust

- Protected enrollment binds the initiating operator, exact existing VM GUIDs/GPU interfaces, and installed artifact identities. Runtime intent, observed attachment, or a GUI draft cannot grant that authority.
- Routine artifact reinstall preserves the existing operator and exact pair identities under the common lock before changing existing permissions or task availability. It rejects authority replacement; a durable package-bound receipt preserves interrupted policy/pending-pair publication and archives readback before releasing installation admission.
- Additive enrollment reviews the current policy revision and operator, appends one pair, and preserves others. GPU replacement is a distinct unavailable transition. Initial setup reviews a fixed four-artifact package and keeps package inputs locked across elevation.
- Elevated setup and worker entrypoints accept fixed typed operations, validated session identities, and authenticated peer processes. Preserve operator SID checks, process binding, local pipe restrictions, frame/output limits, and deadlines. Do not substitute an owner-SID check for exact launched-process authentication.
- Privileged directories/files require trusted ownership, restrictive permissions, and reparse/path checks. Fixed destinations come from Windows known folders. A configuration import or save cannot name an arbitrary privileged destination.
- Installed artifacts are hash-bound to enrollment. Driver payload trust uses Windows signatures/catalog membership and per-file integrity; the mere presence of a catalog is insufficient.
- Effects revalidate fresh enrollment, target identity, provider limits, power, conflicts, driver/payload identity, and the reviewed plan under the protected operation lock. Never execute an old approved plan merely because its displayed summary has not changed.

The relevant implementations are [runner](../src/runner.rs), [worker](../src/worker.rs), [setup](../src/setup.rs), [session pipes](../src/windows_pipe/session.rs), [security](../src/security.rs), and [trust](../src/trust.rs). APP-007 audits concrete trust/recovery gaps in the agreed workflow; APP-008 validates meaningful failure coverage.

## Credentials and diagnostics

Credentials are obtained locally, retained only through explicit Windows Credential Manager opt-in, and kept out of desired configuration, journals, audits, public reports, and persistent recovery. Before the prompt, name the selected VM and clearly explain that its Windows credentials are needed, not the host computer's credentials. A credentials settings action does not establish guest connectivity or graphics health. Credential deletion does not revoke pairing.

The shared native prompt path requires acknowledgement of a named-VM explanation before collecting input. Acknowledgement does not authorize remembering; the native Remember checkbox starts unchecked. Cancelling either panel returns before worker submission, so it starts no guest work and does not clear an existing recovery hold. Vault errors remain failures rather than silently falling back to a prompt. Existing explicit opt-in entries may be reused by ordinary operations without prompting. Settings instead always offers a VM/GUID-named replace/forget/cancel choice. Replacement opens fresh native input; checking Remember saves the replacement, while successful input without Remember removes the old entry as disclosed. Cancelling retains the previous entry. Forget is per-user/per-VM and absence is idempotent. These settings perform no guest, power, enrollment or GPU work. Source implementation is complete under APP-005; runtime acceptance remains pending.

Treat adapter error text as untrusted and potentially secret-bearing. Public outcomes use bounded application-owned failure context. Preserve component/stage/numeric-code evidence where safe rather than copy arbitrary stderr into UI/audit output. Diagnostic tracing is opt-in through trusted fixed product paths and runs only in elevated processes; ordinary frontends skip trace-file creation so read-only artifact checks do not require protected write access. Local qualification records must exclude secrets.

## Operation recovery

The [store records](../src/configuration_store.rs) distinguish effect recovery from configuration publication.

| Phase | Meaning | Permitted response |
|---|---|---|
| `Admitted` | Bound operation persisted before effects | Establish current facts independently; do not assume it is safe to replay |
| `EffectsStarted` | Effects may have occurred; terminal outcome not established | Explicit reconciliation from fresh host/guest/recovery evidence; partial/mismatched state remains blocked |
| `Verified` | Independently verified outcome recorded | Retry only the bound save, after checking state/revision/driver/payload consistency |
| `Published` | Bound configuration publication recorded | Confirm the completed publication and safely retire the record |


A failed/interrupted first preparation on an initially unattached VM has a separate **Remove partial GPU setup** disposition. Its fresh review binds the original effect record, configuration revision, VM/GPU and pending journal. A running VM requires separate graceful-shutdown approval and ends off. An unprepared first attempt must still have the original unattached host settings: preparation precedes adapter/settings writes, so later host changes cannot be attributed to it. Other partial-enable dispositions restore only attributable attachment/allocations and recorded compatibility settings; guest files and original failure evidence remain, preparation eligibility is invalidated, and desired intent/graphics success are not published. Every resumed transition requires the bound journal and independent host/GPU readback; uncertainty retains the hold.

Native provider admission is separate from workflow outcome. New markers capture VM/GPU, exact stage, before facts and requested effect, workflow/configuration tokens, existing job identities and a typed COM failure where available. A missing job reference or unreadable/expired job never permits automatic marker removal. From an administrator console, correlate a surviving local terminal job with the original invocation, then run `hyper-gpu-support native-review --job 'Msvm_ConcreteJob.InstanceID="JOB-ID"'`. The supported review validates the job's affected VM, provider job type, new identity, terminal result, compatible before/after host facts and quiescent job inventory. Inspect its original/fresh facts and returned token before separately approving `hyper-gpu-support native-reconcile --review TOKEN`.

The confirmation uses protected captured terminal evidence if that job has since expired; provider access errors do not count as absence. It rechecks workflow/configuration bindings and exact fresh readback, archives the original marker and terminal result, and retires only provider admission. It does not certify workflow success, save desired intent or replay effects. Without sufficient independent completion evidence (including legacy records lacking scope), preserve the native marker, workflow records and Hyper-V event logs for administrator/provider support. Never manufacture a terminal receipt, delete an uncertain marker or infer completion from instantaneous matching state. Hyper-V job-type/terminal metadata follows the [Microsoft provider contract](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-concretejob).


## Qualification limits

Historical incident notes do not establish a current host-lockup cause or permanent fix. This reset imports no old milestone completion or active hold. Future live work needs its own scoped authorization and evidence. Automated IPC/fault tests validate selected software boundaries; they do not establish host stability or concurrent GPU sharing.
