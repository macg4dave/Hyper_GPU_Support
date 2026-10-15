# Release backlog

All tasks were closed and removed at the user's request on 15 October 2026, following their confirmation that all tests are complete.

## Next action

OS-001 is implemented; validation remains pending. No other task is authorised by this change.

## OS-001 - Fail-closed guest OS compatibility before GPU installation

- Implementation: **Done**. Shared installation admission requires Windows 11 client evidence for initial preparation, payload refresh and Repair / Reinstall. Conservative offline read-only inspection and authenticated queries of already running guests feed one explicit release policy. When primary inspection is unavailable, including locked offline BitLocker cases, freshly queried Hyper-V intrinsic OS metadata is the fallback; it requires complete consistent version/build/client-type facts and cannot override a positive rejection. Missing, malformed or conflicting metadata remains blocked. Guest bootstrap/installer also enforce current OS eligibility before writes. Discovery and ordinary management do not query compatibility.
- Validation: **Pending milestone gate**. Policy, fallback ordering, metadata completeness/conflicts, ambiguous/inaccessible layout and workflow refusal/management regression tests added; builds/tests and live guest/disk/BitLocker/GUI checks have not run. Static source/format checks are reported in chat. Prior accepted release evidence does not cover this addition.
- Contract and investigation: [security](SECURITY.md#guest-os-eligibility-before-installation).

See the [roadmap](roadmap.md) for the completed release checkpoint and general qualification guidance.
