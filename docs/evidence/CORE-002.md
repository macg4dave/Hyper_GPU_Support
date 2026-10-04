# CORE-002 GPU attachment evidence

## Implemented boundary

The normal-token `hyper-gpu-client ensure-gpu` command submits only fixed runner
operations. It first inspects and identity-checks the enrolled VM, child/parent
chain, protected parent hash and configured GPU interface. An off VM with zero
adapters proceeds to the runner's fixed `assign-gpu`; an exact existing adapter is
a verified no-op. Running, foreign, duplicate, malformed and failed states are
rejected. Output keeps `staging_readiness=not-checked` separate from attachment.

The client now queries the registered task through the native Task Scheduler COM
API and waits for `Ready` before every trigger. This prevents a second operation
from being silently ignored while the one-shot task remains `Running` under
`TASK_INSTANCES_IGNORE_NEW`, even after its pipe response was published.

## Live target proof (2026-10-04)

The same-day qualified target recorded by CORE-009 was Windows 11 x64 host build
`26300.9457`, disposable Windows 11 x64 guest build `26200.9457`, NVIDIA RTX 5060
driver `32.0.16.1692`, VM `2627e735-5b33-4104-b739-622727dd3a40` and protected
parent SHA-256
`0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07`.
This task exercised adapter management only; it did not boot the guest or claim a
D3D/CUDA workload result.

The first end-to-end client trial exposed the task-teardown race: inspection
operation `1791121800-022229200` observed the VM `Off` with zero adapters, and
attach operation `1791122003-112753100` successfully changed the configured RTX
interface from zero to one, but the client timed out before receiving the second
response. The audit and result were successful and no reconciliation marker
remained, so the native mutation itself was certain while the client report was
wrong.

After the readiness fix, inspection operation `1791122716-782133800` returned the
matching one-adapter state and the client reported `attachment_status=already-assigned`
without mutation. A bounded detach operation `1791122930-218464500` then returned
one-to-zero. The repaired two-request path completed inspection operation
`1791123137-422691000` followed by attach operation
`1791123338-413748700`; the client validated the published zero-to-one transition
and reported `attachment_status=assigned`, `state=Off`, `gpu_adapters=1`, the exact
configured GPU-PV interface and `staging_readiness=not-checked`. The runner task
returned `Ready`, its last result was zero, and no host lifecycle action ran.

## Checks

Focused validation passed six `hyper-gpu-client` tests, nine Windows runner
transport tests, formatting and strict locked Clippy. The final full repository
and documentation checks are recorded on the task card.
