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
