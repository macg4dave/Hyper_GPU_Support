# Hyper GPU Support

A Windows GUI for turning GPU support on or off for supported existing Hyper-V VMs and choosing their GPU VRAM amount. Hyper-V handles VM creation, general management, and system RAM.

The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including multiple VMs sharing one GPU. Keep the existing Slint design and shared Rust backend.

Windows-version checking is deferred. GPU installation, driver refresh and Repair / Reinstall display a non-blocking compatibility warning without a new version probe or release allowlist. Authenticated guest transport, signed payload checks and recovery still apply; invalid replies are communication failures, not evidence of an unsupported OS.

## Agreed workflow

- Select a VM, set GPU support on/off, and choose VRAM from 1 GB to the selected card's capacity.
- Apply goes straight to the progress screen. Fresh checks prepare the exact request internally, then execution starts without a review dialog or second Apply click. Selecting an action includes its required VM power transitions.
- First-time setup prepares and saves the exact VM/GPU permission, independently reads it back, then continues into fresh GPU planning and Apply for the same unchanged request. Windows credential and UAC prompts remain.
- First GPU setup manages preparation and VM power, requests guest credentials in a native sign-in prompt kept above the app, verifies GPU operation with a pass/fail result, then leaves the VM off after completed setup. Sign-in instructions and the optional Remember choice are in that prompt; there is no preceding OK/Cancel explanation dialog.
- Later toggle/VRAM changes prepare and apply directly without repeating unnecessary setup or initial verification.
- Remembering guest credentials is optional. Saved settings, current observations, drafts, and failures stay truthful.
- Setup refreshes permissions and VM status automatically. A failed operation keeps its original error and shows independently refreshed power/recovery facts when available; retrying the status update is necessary only when that read fails. An uncertain operation is never repeated automatically.

The VRAM range does not establish enforced isolation. Automatic balancing and full-capacity warnings are outside current scope.

## Current state

The main VM page omits the first-install inventory banner, Review recovery and Technical details controls. Recovery remains available through More actions; diagnostic details remain in operation dialogs. The current candidate builds; installed visual validation remains pending.

Settings contains the Appearance controls; the Credentials & diagnostics section has been removed from that page. System displays the GPU information boxes without the This computer box.

Setup progress reuses the VM-state and GPU/driver observation rows while retaining each check in Technical Details. Repeated power actions remain separate. Copy counters and the percentage follow acknowledged transfer progress and clear when the active stage moves to guest checks; they do not establish final success. Technical Details buttons use text without a chevron. These dialog changes await installed visual and setup validation.

The percentage text remains bound throughout the dialog's lifetime, including transitions into and out of file copying. Its colour also follows the copy fraction: during long copies Slint 1.18.1's text cache sweep can otherwise leave a same-width value unrepainted. Completed summary checks stay complete during later reads; their new execution is tracked separately in Technical Details, and a later failure remains visible.

Status polling explicitly refreshes its exact launched Task Scheduler instance before confirming release. A stale Running value after reader exit must not block Refresh or the next Apply; unknown states and failed release observations still block new work.

The repository-wide [architecture review and simplification](docs/OPERATION_COMPLEXITY_REVIEW.md#current-repository-wide-follow-up) is complete through its 19 October 2026 source checkpoint. Configure and Verify reuse the existing protected execution paths. Recovery has one receipt owner; credential lookup and request plumbing reuse existing components. Workspace tests, strict Clippy, binary builds and API documentation passed at that checkpoint. The [roadmap](docs/roadmap.md) describes current responsibilities and the [backlog](docs/backlog.md#next-action) retains the separate candidate installation/live-acceptance gate. Source counts describe maintainability changes, with no numerical reduction target. The dialog and reader-release follow-ups above are separate from that checkpoint; focused validation does not close the full acceptance gate.

## Project guidance

| Document | Purpose |
| --- | --- |
| [Roadmap](docs/roadmap.md) | Agreed scope and milestone gates |
| [Backlog](docs/backlog.md) | Simplification tasks, dependencies and next action |
| [Agreed workflow](#agreed-workflow) | User workflow |
| [Install](#install) | Clean reinstall of the built app through UAC |
| [UI text and languages](resources/i18n/README.md) | GUI wording catalogue and adding a language |
| [AGENTS.md](AGENTS.md) | Authoritative agent instructions (Codex and Claude Code) |

Build prerequisites come from the current manifests/toolchain. Keep passwords, driver payloads, VM disks and machine-specific evidence out of Git.

## Install

Install or update the app with the clean-reinstall tool, from the repository root, with the app closed and no operation running:

```powershell
cargo run -p app-install            # build, review the plan, type "yes", approve UAC
cargo run -p app-install -- --yes   # agents: skips the typed confirmation only
```

The tool builds the release binaries (`--skip-build` reuses the existing build) and refuses while any `hyper-gpu-*` process or product scheduled task is running. After the Windows UAC prompt it:

1. Copies the protected state (`%ProgramData%\HyperGpuSupportProduct` and legacy `%ProgramData%\HyperGpuSupport`) to `%USERPROFILE%\HyperGpuSupport-archives\install-<time>`, so audit and recovery evidence survives. If the copy fails nothing is removed; `--no-archive` skips it.
2. Removes the `\HyperGpuSupport-Product-v2` and legacy `\HyperGpuSupport\` scheduled tasks, `%ProgramFiles%\HyperGpuSupport`, `%ProgramFiles%\HyperGpuSupportProduct` and both state folders.
3. Copies the four artifacts to `%ProgramFiles%\HyperGpuSupport` and checks each SHA-256 against the build.

It then removes your preferences (`%LOCALAPPDATA%\HyperGpuSupport`) and remembered guest credentials (`HyperGpuSupport/guest/*`). VM, guest and `[paths] data_root` state are never touched. Launch `%ProgramFiles%\HyperGpuSupport\hyper-gpu-support.exe`; first-time setup re-enrolls the protected product. Declining UAC or any failed precondition leaves the installation unchanged.

Codex and Claude Code start with [AGENTS.md](AGENTS.md), which routes to short task prompts and focused [Rust](.github/prompts/rust.prompt.md) and [Slint](.github/prompts/slint.prompt.md) guidance. Read only the guidance relevant to the requested task; product requirements stay in project docs.
