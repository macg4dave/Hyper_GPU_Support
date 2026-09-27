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
