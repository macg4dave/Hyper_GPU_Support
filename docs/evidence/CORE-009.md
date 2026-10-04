# CORE-009 evidence

## Implemented boundary

The Rust staging contract validates the complete configured NVIDIA DriverStore
package, required Authenticode signatures, active host GPU/driver identity, stable
servicing state, exact disposable VM/child/parent and authenticated guest identity.
It transfers only the encoded 217-file manifest through one PowerShell Direct
session, publishes through a protected partial directory, verifies every guest hash
and byte count, creates and verifies the CUDA loader alias, and writes an atomic
receipt. A matching receipt is fully rehashed and returned as `already-applied`.

Credential input is ephemeral and zeroized. The CLI now gives masked `*` feedback.
Fixed pre-mutation failures return one validated phase name rather than being
misclassified as an uncertain mutation; native error text and credentials remain
excluded. Once mutation may have begun, every unexpected failure still requires
disposable-child recreation.

## Live target proof (2026-10-04)

The configured target was Windows 11 x64 host build `26300.9457`, disposable guest
build `26200.9457`, VM `2627e735-5b33-4104-b739-622727dd3a40`, and NVIDIA RTX 5060
driver `32.0.16.1692`. The protected parent SHA-256 remained
`0fb4dfe6dd51eed64d36e482e4f58d67c19922802e34aa6ae2d1f4ccd5daeb07`;
the VM had zero checkpoints and zero GPU partition adapters during staging.

Read-only package inspection returned:

- manifest `nvidia-616.92-baseline-v1`;
- encoded manifest SHA-256
  `31cf877d2415ad686f34f6498ae4a08893cebdc699244be1649e5379e9516ea3`;
- package-tree SHA-256
  `3662d618d9fb95d02a354077337d58dbc4c3ccf9db14714b1c5fbc4a675cf950`;
- 217 files and 2,850,973,044 bytes.

Elevated live apply returned `status=applied` with the exact VM, `TESTVM` and
MachineGuid identities, all 217 files/bytes, final
`HostDriverStore\FileRepository\nv_dispi.inf_amd64_b20cc8aeaed64fc2`, and verified
`C:\Windows\System32\nvcuda.dll` hardlink. Matching reapply returned
`status=already-applied` with the same receipt. Both runs explicitly reported the
host qualification and host/guest build drift plus the two configured reviewed
delete-only cleanup records; no active servicing or unknown rename was accepted.

For recovery validation, an elevated, identity-checked test created one uniquely
named partial package directory only in the disposable guest. The Rust apply path
returned `guest staging state is uncertain; recreate the disposable child`. Fixed
runner shutdown and reset operation `1791097972-385962200` then discarded that
child and recreated its differencing disk against the exact protected parent. A
preceding runner shutdown timeout retained its reconciliation marker; a separate
elevated read-only inspection revalidated the running VM, disk chain, parent hash,
zero checkpoints and zero adapters before that exact marker was cleared and the
graceful shutdown was retried successfully. No physical-host lifecycle action ran.

The initially generic live failure was also reproduced and diagnosed: the
unelevated harness could not perform its fixed Hyper-V `host-target` query. The
corrected bounded phase report identified this pre-mutation failure without forcing
another child reset, and the documented elevated invocation then succeeded.

## Checks

The final candidate is validated with the repository's locked formatting, strict
Clippy, unit/integration/doc-test, build, rustdoc, generated policy/configuration and
documentation lanes. Focused coverage accepts only fixed preflight phase names and
continues to map malformed or caller-shaped diagnostics to uncertain state. The
existing missing, changed, partial-input, signature, ACL/reparse, timeout, malformed
protocol, credential-denied and integration-unavailable coverage remains enabled.
