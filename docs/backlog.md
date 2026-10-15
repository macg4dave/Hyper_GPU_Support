# QA backlog

Track remaining qualification work against the [roadmap](roadmap.md).

## Next action

Prepare candidate-specific GUI qualification with explicit target/effect authorisation, local guest credentials, an evidence location and stop conditions. The [BUG-0002 recovery-preparation follow-up](../qa/inbox/2026-10-14-independent-recovery-progress-feedback.md#source-follow-up-15-october-2026) is source-complete with static checks; run its compilation/regressions and the non-hardware gate at the agreed checkpoint before installed retesting.

## Qualification tasks

| ID | Task | Dependencies | Implementation | Validation |
|---|---|---|---|---|
| QA-009 | Candidate regression and automated gate | None | Ready: preparation only; live execution requires authorisation | Candidate-specific live regression pending |
| QA-010 | Release decision and user acceptance | QA-009 | Blocked: candidate evidence required | Not run |

### QA-009 ? Candidate regression and automated gate

Dependencies: None

Record exact installed candidate/build/revision and working-tree context. Prepare resettable targets, allowed effects, stop conditions and evidence before live execution. Run the core journeys and failure/UI coverage in the [roadmap](roadmap.md#test-journeys). Record each result and exclusion.

At the agreed checkpoint run the existing non-hardware gate after reviewing its effects. Completion requires candidate-specific results, final state/readback and persistence evidence, and an accurate automated gate outcome. Handle any retained recovery records through supported authorised actions before affected live journeys.

### QA-010 ? Acceptance decision

Dependencies: QA-009

Summarise candidate evidence and deferred/untested scenarios. Successful core journeys, usable critical recovery, automated evidence and explicit user acceptance are required by the roadmap. Record the user's decision; documentation or automated checks alone cannot close this task.

## Dashboard improvements (UI-001 to UI-005)

Implementation: Done (15 October 2026). The dashboard Verify action appears for fresh
host configuration differences, with an explanatory warning; voluntary Verify remains
in More actions. Historical guest results never activate this warning. Failed or uncertain
operations retain their recovery flow. No new automatic guest checks run on refresh.

The permanent top notification area and routine refresh notice are removed. Inventory
warnings and recovery actions appear with the VM list. VM-card OS icons use host-readable
Hyper-V intrinsic KVP OS metadata, keyed by VM GUID; missing, ambiguous or unrecognised
metadata retains the generic icon. This does not start or enter guests.

Memory controls share one responsive slider/value/GB/Set row. Decimal filtering applies
to whole edits (including paste), and Set/Enter require an in-range value. Rust revalidates
on submission. Existing 0.1 GB slider steps, exact byte values, selected-card physical
capacity, Advanced Allocation, saved settings and review/Apply semantics are retained.
Redundant instructions and the separate fixed-capacity control were removed; precision
notes remain in Technical Details.

Validation: `cargo build --locked --bins` passed; 116 focused GUI/controller regressions
passed with `cargo test --locked --bin hyper-gpu-support gui::`. The isolated intrinsic-KVP
XML parser test also passed without a provider connection. Copy scan, formatting and diff
checks passed. Source
layout consistency was reviewed together; rendered resizing, keyboard/paste interaction,
OS detection on actual guests and installed GUI/VM/GPU acceptance remain untested.
The full non-hardware milestone gate was not run; QA-009/QA-010 remain open.

Documentation check: failed on two existing links to the missing independent recovery
QA report in this working tree. Those unrelated report changes/deletions were preserved.
