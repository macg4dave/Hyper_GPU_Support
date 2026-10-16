# Release backlog

All tasks were closed and removed at the user's request on 15 October 2026, following their confirmation that all tests are complete.

## Next action

No open tasks. OS-001's design replacement passed its complete non-hardware gate on 16 October 2026. Protected installation passed on 16 October 2026. Live guest/GUI acceptance requires separately prepared authorization.

## OS-001 - Fail-closed guest OS compatibility before GPU installation

- Implementation: **Done (source)**, 16 October 2026. Replaced mandatory offline/KVP preflight with a read-only authenticated current-OS check during reviewed Apply, before GPU or installation changes. An off VM is temporarily started and gracefully stopped under durable recovery. Missing or unreliable facts block installation; old metadata cannot override the live result. Preparation session and worker retain checks before guest writes. Discovery, reviews and ordinary management remain independent. Existing permission-completion UI behavior is preserved.
- Review findings: evidence was required before opening the session capable of supplying it; disk/partition assumptions and optional metadata falsely denied supported guests; historical KVP could override failed live identification; guest-local OS reading depended on the host disk adapter. The replacement removes those boundaries and keeps release policy separate from observation and lifecycle.
- Validation: **Passed (complete non-hardware gate)** on 16 October 2026: 399 product tests, 12 catalogue tests, copy/docs/format checks, strict Clippy, product binary builds and API documentation. Evidence: `local/evidence/m1-gate-20261016T154402853Z/`. Protected installation passed: `local/evidence/install-20261016T154451876Z/verification.json`; all four source/installed/policy hashes match, enrollment and 953 persistent files were preserved, and the runner is enabled and Ready. Added isolated supported/unsupported and failure regressions, including the production PowerShell query/installation boundary with substituted guest entrypoints. Earlier gate/install evidence covers the previous implementation only. No live VM, guest, BitLocker, GPU or installed GUI acceptance has run for this replacement. Existing recovery evidence is preserved.
- Contract: [security](SECURITY.md#guest-os-eligibility-before-installation).

See the [roadmap](roadmap.md) for the completed release checkpoint and general qualification guidance.
