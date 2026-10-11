# M2 source review — 11 October 2026

Scope: source-only review of VRAM conversion/admission, Apply planning/execution/readback, native adapter binding, readable progress and bounded transfer acknowledgements. The user explicitly deferred tests and authorized small fixes plus backlog cards for larger findings. The working tree was clean at review start; the preceding M2 changes were already present in source.

## Small corrections

- `Allocation::validate` used the provider-range error for malformed ordering. It now identifies `minimum <= optimal <= maximum` violations before checking range bounds, so invalid VRAM triples do not suggest the removed metadata ceiling has returned. Regression source asserts the ordering explanation.
- `windows_hyperv::allocation` checked VM/GPU identity in an earlier inspection, then loaded the adapter to modify. It now binds that adapter's fresh HostResource against the selected provider's local relative/absolute paths before setting properties. Missing, unknown, replaced or ambiguous resources fail before the write. Isolated helper regression source covers matching paths, replacement, missing and unknown identity. This narrows the stale-object window; it does not provide atomic exclusion against external Hyper-V managers changing state after the check.

## Continued review

- Disable fetched adapters after its earlier identity inspection, then removed `adapters[0]`. It now requires exactly one adapter and binds its fresh HostResource to the selected provider before removal, reusing the allocation identity guard. Replacement, unknown identity or multiple adapters fail before the removal call. Guard regression source now also covers multiple host resources.
- Discard cleared Apply/enrollment approvals but left the separately reviewed graphics plan intact. It now uses the shared cancellation path. Cancellation also clears the reviewed enrollment-recovery revision. Loaded recovery records and saved configuration are retained; cancellation preserves the draft, while Discard clears it. Added isolated state regression source covers both paths and confirms saved/observed state is unchanged.
- Inspected background operation readback, save-only presentation, transfer-row coalescing, subprocess supervision, and transfer acknowledgement completion/deadline paths. No further concrete defect was established in those inspected paths. Existing APP-014 and APP-015 remain the larger pending work; no additional broad task was invented from a hypothetical failure.

## Larger findings retained

- **APP-014:** slider amount changes optimal but may retain a higher maximum, and omitted intent borrows provider defaults as its editing baseline. The user subsequently clarified that the maximum must follow detected physical card capacity; 2 GB was only an example. APP-014 now records this confirmed direction and pending implementation/coverage.
- **APP-015:** the finite offscreen graphics probe does not supply sustained concurrent-workload evidence for APP-009. Prepare exact workload/evidence commands before requesting the remaining live sequence.

## Reviewed boundaries and limits

Subsequent authorized build/install (11 October 2026): `cargo build --locked --bins --jobs 2` succeeded. Installed through the current native `install` command with the complete existing enrollment; independent readback confirmed all four built/installed/policy hashes match, enrollment is preserved, the runner task is enabled and the installation marker is absent. The temporary wrapper did not capture the GUI-subsystem install process exit code; completion was established from readback without rerunning installation. Evidence: ignored `local/evidence/build-install-20261011-202411/verified-result.json`. No tests or GUI/guest acceptance ran; this does not close M2. The source-review exclusions below describe the earlier review itself.

VRAM requests above reported partition metadata use shared category-specific admission and unchanged bytes in the native writer. Other resource range checks remain. Apply retains stale-plan comparison, pending-journal admission, final allocation/settings/power readback and recovery on failure. The transfer monitor retains bounded 4 MiB acknowledgement increments, file ordering/length/total checks, deadlines and completion requirements. No additional confirmed defect was identified in these inspected paths; this is a targeted review, not a repository-wide security or correctness audit.

Validation: static source/caller/diff review, targeted Rust formatting and `git diff --check`. New regression tests remain unrun. No build, compilation check, test execution, GUI render/launch, guest access, credential prompt, protected write, enrollment or VM/GPU operation ran. M2 and its live qualification remain open. Next source task is APP-014 using the confirmed selected-card maximum; APP-015 preparation can proceed independently.
