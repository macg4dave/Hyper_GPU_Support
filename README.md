# Hyper GPU Support

A Windows GUI for turning GPU support on or off for supported existing Hyper-V VMs and choosing their GPU VRAM amount. Hyper-V handles VM creation, general management, and system RAM.

The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including multiple VMs sharing one GPU. Keep the existing Slint design and shared Rust backend.

Windows-version checking is deferred. GPU installation, driver refresh and Repair / Reinstall display a non-blocking compatibility warning without a new version probe or release allowlist. Authenticated guest transport, signed payload checks and recovery still apply; invalid replies are communication failures, not evidence of an unsupported OS.

## Agreed workflow

- Select a VM, set GPU support on/off, and choose VRAM from 1 GB to the selected card's capacity.
- Apply goes straight to the progress screen. Fresh checks prepare the exact request internally, then execution starts without a review dialog or second Apply click. Selecting an action includes its required VM power transitions.
- First-time setup prepares and saves the exact VM/GPU permission, independently reads it back, then continues into fresh GPU planning and Apply for the same unchanged request. Windows credential and UAC prompts remain.
- First GPU setup manages preparation and VM power, explains guest credentials before the local prompt, verifies GPU operation with a pass/fail result, then leaves the VM off after completed setup.
- Later toggle/VRAM changes prepare and apply directly without repeating unnecessary setup or initial verification.
- Remembering guest credentials is optional. Saved settings, current observations, drafts, and failures stay truthful.
- Setup refreshes permissions and VM status automatically. A failed operation keeps its original error and shows independently refreshed power/recovery facts when available; retrying the status update is necessary only when that read fails. An uncertain operation is never repeated automatically.

The VRAM range does not establish enforced isolation. Automatic balancing and full-capacity warnings are outside current scope.

## Current state

The main VM page omits the first-install inventory banner, Check recorded outcome and Technical details controls. Recovery remains available through More actions; diagnostic details remain in operation dialogs. These presentation changes await build and visual validation.

Settings contains the Appearance controls; the Credentials & diagnostics section has been removed from that page. System displays the GPU information boxes without the This computer box. The latest progress and frontend source follow-ups passed the integrated non-hardware gate; installed visual validation remains pending.

The next source priority is [core simplification](docs/backlog.md#next-action): a clearer architecture, less unnecessary complexity, one shared operation path and the existing Slint design. There is no numerical source-line target or size-enforcement gate. Unused tests should be removed; useful tests follow the program's design. The [roadmap](docs/roadmap.md) proposes the replacement/deletion sequence; the core slice, GUI simplification and captured-request cleanup passed complete non-hardware gates. The worker lifecycle follow-up also passed its complete gate. The runner preview cleanup passed its complete non-hardware gate; the protected update is installed. The [architecture review](docs/OPERATION_COMPLEXITY_REVIEW.md) records further implemented simplifications and their passing non-hardware gate. Source counts are descriptive measurements; changes are judged by clarity, maintainability and preserved functionality. Previous source optimizations are implemented; their recorded automated/install evidence is separate from pending installed journeys, measurements and VM/GPU/guest acceptance.

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
