# Backlog

## Next action

Source review and justified simplification are complete through 19 October 2026. The [architecture review](OPERATION_COMPLEXITY_REVIEW.md#current-repository-wide-follow-up) records the independent findings, changes, preserved contracts, measurements and verification. The [roadmap](roadmap.md) describes the current architecture and gates.

OPT-008 remains open for separately authorized installation/live acceptance of the exact candidate. No live GUI launch, installation, protected writes, guest access or VM/GPU effects are authorized by this source checkpoint. No further engine/controller replacement or dependency removal is established by the review; do not invent source work to satisfy a line target.

## Task register

| ID | Task | Implementation | Dependencies | Validation |
| --- | --- | --- | --- | --- |
| OPT-009 | Review the simplification design | Done | None | Source/caller review and automated checkpoint complete; live evidence separate |
| OPT-010 | Simplify operation coordination | Done | OPT-009 | Shared execution and foreground state/plumbing reviewed; non-hardware checks passed |
| OPT-011 | Consolidate demonstrated duplication | Done | OPT-010 | Focused changes and failure regressions passed; retained boundaries documented |
| OPT-008 | Simplified candidate acceptance | In progress | OPT-011 | Current source checks passed; candidate installation and live evidence pending |

## Tasks

### OPT-009 - Review the simplification design

Dependencies: None.

Completed: trace GUI workflows through protected admission, effects, readback and persistence; review administrator record callers, native/guest/security boundaries, configuration/recovery compatibility, diagnostics, tools, assets and dependencies. Record source-backed findings and uncertainties. Source measurements are descriptive. The architecture review distinguishes earlier checkpoints from this candidate.

### OPT-010 - Simplify operation coordination

Dependencies: OPT-009.

Completed: Configure variants share prepared execution and typed steps; Verify shares receipt/execution/confirmation mechanics while retaining non-publication and power-restoration rules. Foreground completion uses the existing merger where contracts match. Recovery has one receipt collection owner and GUID-derived selected access. Unused review/source arguments and the ordinary forwarding wrapper are removed.

Preserved: exact worker-owned scope, captured-request validation, cancellation/release, newer drafts, independent completion observations, verified-unsaved publication and distinct enrollment/import/profile/native recovery. Partial removal's clean-editor reset remains explicit; replacing these contracts with a generic controller would add policy without demonstrated benefit.

### OPT-011 - Consolidate demonstrated duplication

Dependencies: OPT-010.

Completed: earlier exact record equality, GUID syntax, protected-file read and guest-progress consolidation; this follow-up removes repeated vault lookup/prompt decisions using the existing credential owner. Tests use the sole receipt collection, cover unrelated/missing selection and preserve other VMs' holds and profile records. No tests, dependencies, native safeguards, supported options or persistent formats were removed in this follow-up.

The repository-wide review did not establish further safe deletion of guest/native trust, parity, rendering, provider recovery or peer/process/release checks. Exported compatibility hooks and stored audit values remain. These are recorded engineering decisions, not deferred implementation promises.

### OPT-008 - Simplified candidate acceptance

Dependencies: OPT-011.

Remaining acceptance:

- Match candidate binaries to the passing source inputs and disclose any exclusions.
- Prepare exact targets, starting states, allowed effects and stop conditions; obtain separate authorization before live launches, installation, protected writes, guest access or VM/GPU effects.
- Follow the current clean-reinstall tool/archival/reset contract. Verify installed artifact hashes, preserved archive evidence, fresh enrollment and separation from VM/guest/data-root state.
- Exercise the roadmap's normal/failure journeys. Independently confirm terminal result, saved/current state, final recovery/power and record handling.
- Measure transfer/polling/payload behavior at the configured 8 MiB chunk size, including memory, exact tails and slow/no-progress failures. Do not replay uncertain effects or discard evidence.
- Close only from matching candidate evidence and user acceptance. Summarize actual results and unrun scenarios in chat without creating QA reports unless requested.
