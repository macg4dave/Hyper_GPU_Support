# CORE-005 fixed runner progress

## Implemented reset slice

The first reviewed runner slice implements only the operation required to prove
HV-002 disposable recovery. Repository sources are `src/runner.rs` and
`src/bin/hyper-gpu-runner.rs`.

Installed state:

| Component | Identity |
|---|---|
| Executable | `C:\Program Files\HyperGpuSupport\Runner\hyper-gpu-runner.exe` |
| Executable SHA-256 | `e0d93aed5b595475a4918eb0f4013583b2e36ec2811b19f7bd58860150345c17` |
| Policy | `C:\ProgramData\HyperGpuSupport\Runner\policy-v1.json` |
| Policy SHA-256 | `45b700f6d6ea31bda9d72cc000da3ca1788ba0c62961b2bd36178da5c1795ef6` |
| Scheduled task | `\HyperGpuSupport\ResetSlot-v1` |
| Fixed action | installed executable plus literal `reset-slot` |
| Audit | `C:\ProgramData\HyperGpuSupport\Runner\audit\events.jsonl` |
| Results | `C:\ProgramData\HyperGpuSupport\Runner\results\<operation-id>.json` |

The executable/policy/state are administrator-owned outside the repository. The
interactive user has read/execute rather than modification rights. The on-demand
task runs at highest privilege under the interactive user and was successfully
triggered from the normal token; no elevated editor or repository binary ran.

The compiled policy pins the fixed VM GUID/name, parent and child paths, parent
SHA-256 and the sole `reset-slot` allowlist entry. The runner rejects other/extra
arguments and policy byte drift, uses an exclusive create-new lock, bounds its
fixed adapter to 300 seconds/64 KiB per stream, appends audit state, and atomically
publishes a structured result. Reset revalidates elevation, VM off/Gen2/version,
no checkpoints/GPU adapter, parent ACL attribute/hash/VHD state, exact attachment,
reparse absence and child-parent identity before deleting only the fixed child.
It handles the safe partial cases where the exact child is detached or absent.

Operation `1790391920-577596700` succeeded from an unelevated scheduled-task
trigger. It recreated the enrolled child against parent SHA-256
`0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07`;
the resulting child subsequently booted and passed the HV-002 guest identity check.

## Validation and remaining work

Before edits, all 16 library, 2 main-binary, 4 integration and 1 doc tests passed.
After the slice, formatting, strict locked Clippy, 19 library, 2 runner-binary,
2 main-binary, 4 integration and 1 doc tests, locked build and rustdoc passed with
warnings denied. The release runner was built separately and its installed hash
was checked before task registration.

CORE-005 is not complete. Start/shutdown, GPU attach/remove, staging/probe, a normal
client/result command, broader replay/request handling, minimum-rights principal
measurement and the full fake failure matrix remain. Updating or broadening the
installed binary, policy or task requires new exact approval.

## Repository-only protocol and least-privilege candidate (2026-09-27)

The library recognizes the fixed version-one operation inventory and parses a
strict typed request containing only schema, request ID, operation, fixed slot/VM,
nonce and plan fingerprint. It rejects unknown/duplicate fields, alternate target
identities, paths, credentials, noncanonical identifiers, substituted operations,
stale plans and reused nonces. The candidate runner and client now connect that
contract over a local-only, length-bounded named pipe whose endpoints verify the
administrator-enrolled peer SIDs. The runner verifies exact installed policy bytes,
uses create-new persistent nonce records before effects, serializes operations with
an exclusive kernel-held file lock, and records request/result audit data. A crash
cannot leave a stale lock that permanently blocks recovery.

The candidate installation creates a dedicated random-password local principal,
adds it only to the built-in Hyper-V Administrators group by fixed SID, runs the
one-shot task with a limited token, grants read access to the exact parent and
modify access only to the exact child directory, and retains an explicit rollback
to the installed reset-only task. It captures file, ACL, task-definition and task-
security preimages before effects; stages and hashes artifacts under an
administrator-only ACL; disables and checks the old task before publication; and
registers the replacement disabled until installed hashes, file ACLs and task ACLs
are verified. Recovery also registers the old task disabled, restores its captured
security descriptor, and only then restores its prior enabled state. The fixed reset adapter accepts Administrators or
that Hyper-V group, while continuing to revalidate the enrolled VM, parent hash,
child chain, state and absence of a GPU adapter before replacing the child. Inspect
and reset now share one operation lock, and inspect audit records identify the
correct operation instead of being mislabeled as reset. Snapshot and GPU-adapter
provider failures propagate instead of being interpreted as empty results; durable
audit records correlate request and operation identifiers before effects.

Repository validation passed formatting, strict Clippy, 39 library tests, all
binary/integration/doc tests, locked build/rustdoc, the release build, documentation
checks and script syntax/hash checks. Independent review identified and drove the
fail-closed query, staging race, partial-recovery, file/task ACL and audit fixes;
the focused re-review found no remaining installation blocker. Reviewer runtime
model metadata was unavailable. The reviewed candidate hashes are runner
`8d001f660529b6287d6d9f10769448fde693e906ece4ff48737ab2c6235fb2ca`,
client `649eccc2c937f5baa3ad838dbbcc3c0e8966f6246e69aae5b5ba278cde2b4a92`
and policy `b4178d0a5b72520c3769cf371a0da824d1434c909df4c25b769b7a26e8f5916f`.
No installed executable, policy, scheduled task,
account, ACL, host/VM or guest state was changed. The host still has only the old
`ResetSlot-v1` task. Installing this candidate is a new privileged operation and
requires exact owner approval. Lifecycle, GPU, staging/probe operations and their
failure matrix remain required before CORE-005 can complete.

## First limited-principal installation trial and recovery (2026-09-28)

Under the separately approved installation/read-only-inspection scope, the staged
candidate hashes and task/account/ACL identities were verified and the S4U task was
registered. Task Scheduler did not start an instance (`0x41303`, task has not run),
so the named pipe never appeared and read-only `inspect` could not connect. A
read-only elevated local-policy export then showed that the new runner SID was not
a member of `SeBatchLogonRight`. No VM, GPU, guest or host-lifecycle operation ran.
The approved recovery path removed the candidate task/account and restored the old
enabled `ResetSlot-v1` task, installed runner/policy hashes and ACL preimages.

The corrected repository candidate replaces the rejected whole-template policy
edit with `src/windows_account_rights.rs` and the administrator-only
`hyper-gpu-rights` binary. The helper enumerates the exact SID's current LSA rights,
adds or removes only the five fixed batch/deny rights, and verifies that unrelated
rights are unchanged. Installation records the created SID in the protected backup
before enrollment; recovery treats it as authoritative, rejects name/SID drift,
removes the same fixed rights, deletes by SID and enables the restored old task only
after all other recovery steps.

The new release candidate hashes are runner
`69f2de301f28e2f2f7d64ed579c94e3abb34fc15db4af588b9f2d6e64f52351c`,
client `0a93a01262e37bd63d8251e3d6b486d9662adbec42a937530a81b081fc4901a1`,
rights helper `62de77ef8c9f307b36a3341b7de64f521dfa4e68029c62ec98db2d6e06280f52`
and policy `b4178d0a5b72520c3769cf371a0da824d1434c909df4c25b769b7a26e8f5916f`.
The full repository check passed with 45 library tests plus all binary, integration
and doc tests. Applying these account rights is a new protected host-policy change
and has not been performed. Independent focused re-review found no remaining
installation/recovery blocker after idempotent pre-grant rollback, canonical SID
validation and retry-safe task restoration were added. Reviewer runtime model
metadata was unavailable. Native LSA behavior and cross-principal execution remain
target-validation requirements.

## Second limited-principal installation trial and recovery (2026-09-28)

With explicit approval for the reviewed exact-SID rights delta, installation
succeeded for generated runner SID
`S-1-5-21-2102502009-691714006-1044501546-1011`. The enabled S4U/limited task had
the fixed `serve-once` action, six-minute limit and ignore-new instance policy;
account/enrollment SIDs and installed runner/client/policy hashes matched. Unlike
the first trial, Task Scheduler launched the runner, proving the batch-logon change
effective. The read-only client still failed before a request audit or Hyper-V query
with Windows access denied; Task Scheduler recorded runner exit `1`.

The approved recovery removed the five rights and exact-SID account, restored the
enabled old task and original runner/policy hashes, and removed the client. It also
exposed that the helper's administrator read/execute-only file ACL prevented its
post-restore deletion while that failure was silenced. The one leftover helper was
given administrator full control and deleted under the approved rollback; temporary
diagnostics were removed. Final verification found no runner account/new task/client/
helper/active backup, while `ResetSlot-v1` was ready and enabled with original runner
`e0d93aed5b595475a4918eb0f4013583b2e36ec2811b19f7bd58860150345c17`
and policy `45b700f6d6ea31bda9d72cc000da3ca1788ba0c62961b2bd36178da5c1795ef6`.

The repository fix gives Administrators full control only on the administrator-only
helper, checks removal instead of suppressing errors, and writes a sanitized bounded
startup failure under the read-only-to-client audit directory before runner exit.
Code-path review then identified the immediate access denial: both endpoints tried
to open the other account's process token, while the same-user transport test could
not represent the installed cross-account boundary. The server now reads a bounded
frame and impersonates only that connected pipe client to check its token SID; the
client verifies the pipe object's explicit enrolled owner SID. The pipe remains
local-only, first-instance and single-instance. The client requests identification-
only SQOS, and its exact access mask allows read/write data and security-owner
inspection without `FILE_CREATE_PIPE_INSTANCE`. A failed `RevertToSelf` terminates
the runner process before the handler can run.

The next candidate hashes are runner
`1fcefaf754c33fbe7e636f69475965d8a95fb26cc7e10ca74609f6c3960b3f95`,
client `a43d12f195dfee5f136e2eb94fa2c4c27ed108da48e55faae6dc9f7e941f28a3`,
rights helper `ff22fa9b0dc82b03c65f835e4857d826e7e4db04ecfe807fca93b4965918a41d`
and unchanged policy
`b4178d0a5b72520c3769cf371a0da824d1434c909df4c25b769b7a26e8f5916f`.
Another protected installation is required to validate cross-account transport.
Fresh independent architecture/security review found no remaining pre-installation
code or security blocker and independently passed all four Windows transport tests,
diff whitespace and the four pinned hashes. Reviewer runtime model metadata was
unavailable. Distinct-account S4U exchange and elevated recovery fault injection
remain target evidence gaps; synchronous pipe connection/frame deadlines remain
existing CORE-005 debt.

## Repository-only lifecycle slice (2026-09-28)

The candidate policy and runner now authorize fixed `start-slot` and
`shutdown-slot` requests through the same authenticated local pipe, policy
fingerprint, persistent nonce ledger, operation lock and audit/result path as
inspection and reset. The adapters accept no target or command parameters. Each
revalidates the enrolled GUID/name, Generation 2/version, exact child attachment
and differencing parent, read-only parent hash, checkpoint absence and zero-or-one
GPU adapter with the exact pinned instance path before the effect. Start requires `Off` and verifies `Running` within
180 seconds, rejects any other VM GPU assignment and requires at least 12 GiB of
reported available host RAM. Shutdown requires `Running`, invokes ordinary guest-integrated
`Stop-VM`, and verifies `Off` within 120 seconds. It never selects `-Force`,
`-TurnOff` or `-Save`; those remain outside the reviewed operation. The operation
result records the exact before/after state and preserved pinned adapter count.

Before either lifecycle effect the runner durably creates an administrator-owned
`state\reconciliation-required-v1` marker containing the operation and operation
ID. It removes the marker only after strict native-output parsing, atomic result
publication and terminal operation audit all succeed. Any timeout, adapter failure,
malformed result, full/unavailable result path or audit failure retains the marker,
returns the operation ID where one was allocated, and blocks every later mutating
request before nonce consumption; fixed read-only `inspect` remains available.
Clearing the marker is deliberately not exposed to the unelevated protocol and
requires a separately reviewed administrator reconciliation after native state is
known. Fake-cmdlet tests execute the actual fixed scripts across success, wrong GPU,
another-VM assignment, low-memory and denied-query cases; marker tests cover
create-new persistence, mutation rejection and read-only inspection allowance.

This follows the documented [`Start-VM`](https://learn.microsoft.com/powershell/module/hyper-v/start-vm)
and [`Stop-VM`](https://learn.microsoft.com/powershell/module/hyper-v/stop-vm)
object parameter sets. Repository validation passed formatting, strict locked
Clippy, 46 library tests, all binary/integration/doc tests, locked build/rustdoc,
documentation checks and a locked release build. The release hashes are runner
`f90f172162bcf1367064a9db965796e0ec069c470a0655abbefc166d902d8d62`,
client `eca369b52bd6a9bed68ec6c08799302f9130aea4483d479d9456e1b75e01f0c0`,
rights helper `ac757f1598b31763616471bfe1ac439d1609963cb64decce2cdcd12128ecd84f`
and policy `2889996ab6f035ae21c4c76c54146007369a704eb77aa884d78e9cb6b37dff91`.
No installed file/task/account/policy/ACL, VM, GPU, guest or host state changed.
Installation and any lifecycle run need a separately reviewed protected scope;
target rights, state transitions and timeout reconciliation remain unproven.

Independent final focused review drove the pinned-adapter, other-assignment,
12 GiB admission, durable reconciliation, operation-ID/failure-result and
fail-closed marker-lookup fixes. The reviewer found no remaining blocking code
issue and independently verified `git diff --check` plus all four hashes above;
it did not rerun the full suite. Reviewer runtime model metadata was unavailable.
Cross-account installation, limited-principal lifecycle rights/transitions,
real timeout reconciliation and injected publication/audit failures remain gaps;
this repository-only slice establishes no hardware capability.

## Third limited-principal installation trial and recovery (2026-09-28)

The owner approved installation of the reviewed lifecycle candidate and only a
read-only `inspect` invocation. The first installer attempt stopped before effects
because its constants still pinned the prior binary hashes. After mechanically
pinning the already reviewed artifacts, installation succeeded with runner SID
`S-1-5-21-2102502009-691714006-1044501546-1012`; installed runner, client and policy
hashes matched, `Runner-v1` was enabled with fixed `serve-once`, and no startup or
reconciliation marker existed.

The normal-token client request `4e97e5962423b74bea29e91aa3083e16`
successfully crossed the distinct-account boundary. The runner authenticated it,
recorded the exact policy fingerprint and started read-only operation
`1790616106-754611000`. This proves the DEC-018 pipe-owner and impersonated-client
SID design on the target. The inspect adapter then timed out after 60 seconds while
hashing the 24,767,365,120-byte parent. It recorded bounded terminal failures and
published no result; no VM lifecycle, GPU, disk or guest mutation ran. Both processes
exited, but the client remained blocked beyond its nominal 15-second timeout until
the runner responded, reproducing the synchronous response-deadline defect.

Approved recovery completed successfully. `ResetSlot-v1` is enabled/ready with
runner `e0d93aed5b595475a4918eb0f4013583b2e36ec2811b19f7bd58860150345c17`
and policy `45b700f6d6ea31bda9d72cc000da3ca1788ba0c62961b2bd36178da5c1795ef6`;
the candidate task/account/client/helper/enrollment/active backup and both processes
are absent. Audit/results were retained.

The corrected repository candidate gives parent-hash inspection a candidate
300-second adapter limit pending a passing measurement, plus operation-specific client deadlines. The client now
uses `PeekNamedPipe` to wait for and consume the header before waiting for the
declared payload, avoiding maximum-frame buffer backpressure. Native pipe tests
cover a 100 ms no-response deadline, split header/payload delivery, the legal
65,536-byte maximum and an oversized declaration while preserving the authenticated
owner check, including an empty frame after writer close. Full validation passed 51 library,
9 runner, 3 client, all other binary/integration and doc tests, formatting, strict
locked Clippy/build/rustdoc, documentation checks and a locked release build. New
hashes are runner `555e8b017d62b6941347e7990c65c1161175e92883fa7d79607cea4e4b05f80d`,
client `a977bfee9fd17457f2ceba07442064117afdb2b79665303fb78ee10f80fd4bc8`,
rights helper `d1340e514c42925e891ab951904c9f10284d3b232a5bb335d333ab8e47ef7a77`
and unchanged policy
`2889996ab6f035ae21c4c76c54146007369a704eb77aa884d78e9cb6b37dff91`.
Server-side connection/request deadlines remain debt. At this checkpoint no corrected
candidate was installed; another protected installation/read-only run needed fresh approval.
Independent focused review accepted the final header-first reader and its partial,
maximum, oversized, deadline and closed-writer empty-frame tests with no findings,
and independently verified all four hashes plus diff whitespace. Reviewer runtime
model metadata was unavailable; the reviewer did not rerun the full suite.

## Corrected read-only target validation (2026-09-28)

The owner approved installation of the exact reviewed hashes above and one read-only
`inspect`. Preflight verified those four source hashes, the enabled/ready legacy
`ResetSlot-v1`, absence of `Runner-v1` and reconciliation marker, and the recovered
legacy runner/policy hashes. The installer exited `0`; its elevated post-copy checks
verified the hardened rights helper that is intentionally unreadable to the normal
client account. Normal-account checks independently verified the installed runner,
client and policy hashes, enabled/ready `Runner-v1`, removal of `ResetSlot-v1`, no
startup/reconciliation marker, and consistent task/account/enrollment runner SID
`S-1-5-21-2102502009-691714006-1044501546-1013`.

The client launched request `abdd204e56936c6332bab98ee94d9e6e` and exited normally
after about 222 seconds with empty stderr. Operation `1790617624-800511500` durably
reported `succeeded`, proving that the 24,767,365,120-byte parent hash completes
within the candidate 300-second adapter bound and that the polled operation-specific
client deadline permits the complete response. The result pinned VM
`2627e735-5b33-4104-b739-622727dd3a40` in `Off`, found zero assigned GPU adapters,
verified parent SHA-256
`0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07`, and found
the configured NVIDIA `VEN_10DE&DEV_2D05` GPU-PV interface. Matching received,
started and succeeded audit records carry policy fingerprint
`2889996ab6f035ae21c4c76c54146007369a704eb77aa884d78e9cb6b37dff91`;
no reconciliation marker exists and the runner returned to `Ready`.

The corrected candidate remains installed. This was read-only validation: no VM,
GPU, disk, guest or host-lifecycle mutation ran. Fixed start/graceful-shutdown and
later GPU/staging/probe execution remain protected and unperformed.

## First lifecycle target trial and fail-closed reconciliation (2026-09-28)

The owner approved starting and then gracefully shutting down only pinned disposable
VM `2627e735-5b33-4104-b739-622727dd3a40`, with no GPU attachment/removal, force-stop
or host lifecycle action. Exact installed runner/client/policy hashes, absence of
failure markers, ready runner task and an elevated read-only `Off` VM identity check
passed before the request.

Start request `89f7a47ab9bee9f2684bdb34ecf2005b`, operation
`1790618530-782157600`, failed closed with `fixed Hyper-V adapter timed out` and
durably published `operation-failed-reconciliation-required`. The marker contains
`start-slot 1790618530-782157600`; received, started and failed audit records match
the request, operation and policy fingerprint. Shutdown was not sent. Elevated
read-only reconciliation found the exact VM still `Off`, `Operating normally`, with
zero uptime and zero GPU adapters. Thus no VM transition or GPU mutation occurred.

The cause is a composed-deadline defect: `START_SCRIPT` performs the complete parent
hash before calling `Start-VM`, but the whole script had only the 180-second start
allowance. The immediately preceding target inspection measured that same hash path
at about 222 seconds. The repository correction therefore runs a separately bounded
300-second inspection phase before a freshly revalidated 180-second start or
120-second shutdown phase; client responses allow the corresponding aggregate plus
30 seconds. Independent review found the installer's six-minute Task Scheduler cap
was shorter than those composed paths. It is now ten minutes, verified after
registration, and cross-layer tests bind that cap above runner and client deadlines
without allowing a fast inspection to lend time to a transition. Focused runner/
client tests and the full formatting, strict locked Clippy, locked workspace tests,
locked build, warning-denied rustdoc, documentation and whitespace checks passed.
The corrected release hashes are runner
`eb8f724adc94446867b9ca759024d464fb982cf07bc909f2f4399f9f52217080`, client
`bef2e0d03fa5f4497635bfd59f628d686d3cf3ecbc88d9d5defe4ef91d9f6391`, unchanged
rights helper `d1340e514c42925e891ab951904c9f10284d3b232a5bb335d333ab8e47ef7a77`
and unchanged policy
`2889996ab6f035ae21c4c76c54146007369a704eb77aa884d78e9cb6b37dff91`.
The installed prior candidate remains mutation-blocked by its reconciliation marker;
clearing/recovering it, reinstalling these corrected hashes and retrying lifecycle
are protected operations requiring independent review and explicit approval.

Independent re-review found no remaining blocker after phase separation and the
verified ten-minute task cap. It independently matched all four final hashes to the
artifacts, installer and evidence and confirmed clean diff whitespace; the reviewer
relied on the supplied full-suite results and runtime model metadata was unavailable.
The reviewer considered recovery/reinstall/retry safe to present only with runner
quiescence, exact-VM reconciliation, clearance of this specific marker, final hash/
task-setting scope and fail-closed inspection on any uncertain result. Corrected
lifecycle target execution remains unproven, and concurrent administrator changes
to the protected parent remain outside the runner lock's guarantee.

## Corrected lifecycle proof and repository-only GPU assignment slice (2026-09-29)

Retained installed-runner evidence showed that the corrected candidate had already
been recovered, installed and exercised after the preceding handover. Start request
`960d33f579508e7e7564583347563704`, operation
`1790619724-408164600`, succeeded from `Off` to `Running`; graceful-shutdown request
`46807c8a0462b9b3a79e3c8ff6affe0`, operation
`1790620213-020445200`, succeeded from `Running` to `Off`. Both results pin VM
`2627e735-5b33-4104-b739-622727dd3a40`, the exact child/parent chain and parent
SHA-256, and preserved zero GPU adapters. A fresh read-only inspect operation
`1790701914-505693000` then passed on 2026-09-29 with the VM `Off`, zero GPU
adapters, the exact RTX 5060 GPU-PV interface and no reconciliation marker. The
installed runner/client/policy hashes were respectively the corrected
`eb8f724adc94446867b9ca759024d464fb982cf07bc909f2f4399f9f52217080`,
`bef2e0d03fa5f4497635bfd59f628d686d3cf3ecbc88d9d5defe4ef91d9f6391` and
`2889996ab6f035ae21c4c76c54146007369a704eb77aa884d78e9cb6b37dff91`.

The repository candidate now policy-enables only `assign-gpu` and `remove-gpu` in
addition to the already proved operations. Both accept no target/resource input,
run a locked full inspection first, require the enrolled VM to be `Off`, validate
the exact child/parent/GPU identities and checkpoint state immediately before the
effect, and publish the verified zero-to-one or one-to-zero adapter transition.
Attach rejects any existing target adapter or adapter assigned to another VM and
uses provider-default resource values; detach removes only the exact enrolled
adapter. Both retain the durable reconciliation marker on any uncertain outcome.
Hardware-free validation passed 52 library tests, 10 runner tests, all other
binary/integration/doc tests, formatting and strict locked Clippy. The release
candidate hashes are runner
`691c45e9eb4bb234323dd8247f1c88e5505d2f9fa499377d81d5f865fd6aed2`, client
`31c38a75d3cdf1a82dce26fdf4441feddab0811ca77ea350097db17e9ebf53af`, rights
helper `f8ee9570cfbf25a6889b7111f3bd76004edaaebf4cdd67ffc06cc0a6be308563`
and policy `63430c1d9b059a0d750ebf6fab214ad6d7d66654510a85a217021a7fc74707e5`.
No installed file/task/account/policy/ACL, VM, GPU, disk, guest or host state was
changed by this repository slice. Installation and target attach/detach require a
new exact approval and independent review.

## Pre-deployment reconciliation correction (2026-10-01)

Independent privileged-boundary review found that `reset-slot`, unlike lifecycle
and GPU assignment, did not retain a reconciliation marker or operation ID when a
timeout, result-publication failure or final-audit failure followed a possible child
disk mutation. Deployment was held. The corrected dispatcher allocates the ID
before reset, creates the durable marker before launching the destructive phase,
returns that ID on failure, and clears the marker only after result publication and
the terminal operation audit succeed. Inspection now also rejects a single attached
adapter whose instance path is not the configured RTX 5060 interface.

Focused tests inject reset timeout, publication and terminal-audit failures and
verify that the exact marker remains; the success path verifies result/audit output
and marker clearance. Typed PowerShell fakes verify mismatched-adapter inspection.
After correction, `scripts/testing/check.ps1` passed formatting, strict locked
Clippy, 79 unit/integration tests, the doc test, locked build, warning-denied rustdoc
and the generated configuration check; `scripts/testing/check-docs.ps1` passed all
36 Markdown files. A locked release build and pin refresh produced runner
`e37400cf4d4ced13c28702bdcdfdd355694458c6fce694fc917673bb97a43275`, client
`221dd0972fa917e8d5274b3655c36d69a4796cd428fcaec37556cd40546bf380`, rights
helper `47eb09dc0a4e6f14360327763010f85478bab26dcd61983f842d801689e2579f`
and policy `12a532f188356fa72e04376329986d8dc68579449b0288406dd61c93c8889b5c`.
The generated pin check passed. Independent re-review found no remaining blocker
to exact restore/reinstall and the attach/detach target trial.

At that point native attach/detach and failure recovery remained unproved; the next
step was to install the candidate and run that trial on the designated disposable VM.

## Installed attach/detach proof (2026-10-01)

The reviewed candidate was restored over the prior lifecycle build and installed
with runner `e37400cf4d4ced13c28702bdcdfdd355694458c6fce694fc917673bb97a43275`,
client `221dd0972fa917e8d5274b3655c36d69a4796cd428fcaec37556cd40546bf380`,
rights helper `47eb09dc0a4e6f14360327763010f85478bab26dcd61983f842d801689e2579f`
and policy `12a532f188356fa72e04376329986d8dc68579449b0288406dd61c93c8889b5c`.
The first install attempt failed closed with both tasks disabled because Task
Scheduler returned the requested `PT600S` execution limit in equivalent canonical
form `PT10M`. After restoring the captured preimage, the installer was corrected to
compare parsed XML durations and installation succeeded with runner SID
`S-1-5-21-2102502009-691714006-1044501546-1016` and an enabled `Runner-v1`.

Inspection operation `1790860264-920153800` succeeded. Attach operation
`1790860476-127613900` then verified the off VM, exact child/parent chain, parent
hash and RTX 5060 interface and changed the adapter count from zero to one. Detach
operation `1790860695-250930400` changed that same adapter count from one to zero.
Both results pin VM `2627e735-5b33-4104-b739-622727dd3a40`, child
`Z:\HyperGpuSupport\images\disposable\gpu-pv-slot-01\child.vhdx` and the immutable
parent hash. The runner task is enabled with canonical limit `PT10M`, and no
reconciliation marker remains. These results complete CORE-005's native target
acceptance. After the installer correction, `scripts/testing/check.ps1` passed
formatting, strict locked Clippy, 80 unit/integration/doc tests, locked build,
warning-denied rustdoc and generated configuration checks; documentation checks
passed all 36 Markdown files.
