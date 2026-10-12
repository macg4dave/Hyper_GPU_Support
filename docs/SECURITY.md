# Security and recovery

This page owns product authority/trust boundaries and recovery semantics. [AGENTS.md](../AGENTS.md) owns agent authorization rules; [testing](TESTING.md) owns qualification scenarios. These are contracts to preserve and validate, not a claim that static review proves their enforcement.

## Authority and trust

- Protected enrollment binds the initiating operator, exact existing VM GUIDs/GPU interfaces, and installed artifact identities. Runtime intent, observed attachment, or a GUI draft cannot grant that authority.
- Additive enrollment reviews the current policy revision and operator, appends one pair, and preserves others. GPU replacement is a distinct unavailable transition. Initial setup reviews a fixed four-artifact package and keeps package inputs locked across elevation.
- Elevated setup and worker entrypoints accept fixed typed operations, validated session identities, and authenticated peer processes. Preserve operator SID checks, process binding, local pipe restrictions, frame/output limits, and deadlines. Do not substitute an owner-SID check for exact launched-process authentication.
- Privileged directories/files require trusted ownership, restrictive permissions, and reparse/path checks. Fixed destinations come from Windows known folders. A configuration import or save cannot name an arbitrary privileged destination.
- Installed artifacts are hash-bound to enrollment. Driver payload trust uses Windows signatures/catalog membership and per-file integrity; the mere presence of a catalog is insufficient.
- Effects revalidate fresh enrollment, target identity, provider limits, power, conflicts, driver/payload identity, and the reviewed plan under the protected operation lock. Never execute an old approved plan merely because its displayed summary has not changed.

The relevant implementations are [runner](../src/runner.rs), [worker](../src/worker.rs), [setup](../src/setup.rs), [session pipes](../src/windows_pipe/session.rs), [security](../src/security.rs), and [trust](../src/trust.rs). APP-007 audits concrete trust/recovery gaps in the agreed workflow; APP-008 validates meaningful failure coverage.

## Credentials and diagnostics

Credentials are obtained locally, retained only through explicit Windows Credential Manager opt-in, and kept out of desired configuration, journals, audits, public reports, and persistent recovery. Before the prompt, name the selected VM and clearly explain that its Windows credentials are needed, not the host computer's credentials. A credentials settings action does not establish guest connectivity or graphics health. Credential deletion does not revoke pairing.

The shared native prompt path requires acknowledgement of a named-VM explanation before collecting input. Acknowledgement does not authorize remembering; the native Remember checkbox starts unchecked. Cancelling either panel returns before worker submission, so it starts no guest work and does not clear an existing recovery hold. Vault errors remain failures rather than silently falling back to a prompt. Existing explicit opt-in entries may be reused without prompting. Source implementation is complete under APP-005; runtime acceptance remains pending.

Treat adapter error text as untrusted and potentially secret-bearing. Public outcomes use bounded application-owned failure context. Preserve component/stage/numeric-code evidence where safe rather than copy arbitrary stderr into UI/audit output. Diagnostic tracing is opt-in through trusted fixed product paths; local qualification records must exclude secrets.

## Operation recovery

The [store records](../src/configuration_store.rs) distinguish effect recovery from configuration publication.

| Phase | Meaning | Permitted response |
|---|---|---|
| `Admitted` | Bound operation persisted before effects | Establish current facts independently; do not assume it is safe to replay |
| `EffectsStarted` | Effects may have occurred; terminal outcome not established | Explicit reconciliation from fresh host/guest/recovery evidence; partial/mismatched state remains blocked |
| `Verified` | Independently verified outcome recorded | Retry only the bound save, after checking state/revision/driver/payload consistency |
| `Published` | Bound configuration publication recorded | Confirm the completed publication and safely retire the record |


## Qualification limits

Historical incident notes do not establish a current host-lockup cause or permanent fix. This reset imports no old milestone completion or active hold. Future live work needs its own scoped authorization and evidence. Automated IPC/fault tests validate selected software boundaries; they do not establish host stability or concurrent GPU sharing.
