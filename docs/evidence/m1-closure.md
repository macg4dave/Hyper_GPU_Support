# M1 workflow implementation closure

M1 closed on 11 October 2026 against the current [roadmap](../ROADMAP.md): APP-001 through APP-007 source implementation and APP-008 non-hardware validation. This is the agreed workflow milestone, not the retired enrollment milestone with the same name. Real GUI/VM qualification remains M2; packaging remains M3.

## Reviewed source and changes

Base revision: `4c293ec467586bc3b7502b90303cc4200f6f624a`, with the working-tree changes recorded in the gate report. The [follow-up source review](m1-source-review.md) confirmed existing APP-006/APP-007 implementation and inspected test effects. This closure work added a narrow gate/documentation checker, corrected task/status documentation and applied rustfmt to the milestone source. No further product behavior change or dependency update was needed. The pre-existing `docs/TESTING.md` test-VM edit was preserved.

## Executed gate

Command: `powershell -NoProfile -File scripts/testing/check.ps1`.

Final run: 11 October 2026, 17:23:56 UTC onward, ordinary non-elevated Windows x64 token, Windows `10.0.26300.0`, Rust/Cargo `1.94.0`, configured target `x86_64-pc-windows-msvc`, pinned Slint `1.18.1`. The existing environment directs Cargo output to `C:\Users\dave\.cargo-target\text-game`; no toolchain, dependencies or installed product were changed.

| Stage | Command | Result |
|---|---|---|
| Documentation | `python scripts/testing/check-docs.py` | Passed: 25 Markdown files, 10 task cards, local links/anchors, task references/dependencies and diff whitespace |
| Formatting | `cargo fmt --all -- --check` | Passed after `cargo fmt --all` corrections |
| Non-hardware tests | `cargo test --locked --all-targets` | Passed: 142 library/core and 50 frontend tests; no failures, ignored or filtered tests; three helper binary targets have no tests |
| Strict lint | `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| Product binaries | `cargo build --locked --bins` | Passed |
| API documentation | `cargo doc --locked --no-deps` | Passed |

All stages and the final gate process exited 0. The closure documentation was checked again after the gate; product source did not change after the passing run.

Raw incremental evidence is in ignored `local/evidence/m1-gate-20261011T172355932Z/`: `report.json` records exact commands, timestamps, revision, working-tree context, environment, source/tooling input SHA-256 hashes, results and exclusions; `stage-0.txt` through `stage-5.txt` retain output. Earlier evidence retains the initial formatting failure and gate stderr-reporting corrections. No earlier failed stage was relabeled as a pass.

## Acceptance and exclusions

Executed coverage includes startup dispatch without retired mock modes, slider bounds/exact byte endpoints, review binding/cancellation/stale inputs, setup/ordinary distinction, confirmed final-off lifecycle, credential explanation/cancellation/opt-in closures, saved-state reload/conflicts, other-VM preservation, actual progress records, missing terminal outcomes, publication failure/save-only and interrupted recovery. These are isolated non-hardware tests, as detailed in the source review.

Not run: GUI launch/render/interaction, native credential/vault interaction, installed worker integration, privileged setup/enrollment, protected writes, guest connections, VM/GPU effects, graphics workloads, concurrent sharing, package installation or upgrade. Responsiveness, visual acceptance, actual provider behavior and supported hardware remain unqualified until separately authorized APP-009 work. No live authorization is implied by M1 closure.
