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
