# Hyper GPU Support

A Windows GUI for turning GPU support on or off for supported existing Hyper-V VMs and choosing their GPU VRAM amount. Hyper-V handles VM creation, general management, and system RAM.

The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including multiple VMs sharing one GPU. Keep the existing Slint design and shared Rust backend.

Windows-version checking is deferred. GPU installation, driver refresh and Repair / Reinstall display a non-blocking compatibility warning without a new version probe or release allowlist. Authenticated guest transport, signed payload checks and recovery still apply; invalid replies are communication failures, not evidence of an unsupported OS. See [guest OS eligibility](docs/SECURITY.md#guest-os-eligibility-before-installation).

## Agreed workflow

- Select a VM, set GPU support on/off, and choose VRAM from 1 GB to the selected card's capacity.
- Apply goes straight to the progress screen. Fresh checks prepare the exact request internally, then execution starts without a review dialog or second Apply click. Selecting an action includes its required VM power transitions.
- First-time setup prepares and saves the exact VM/GPU permission, independently reads it back, then continues into fresh GPU planning and Apply for the same unchanged request. Windows credential and UAC prompts remain.
- First GPU setup manages preparation and VM power, explains guest credentials before the local prompt, verifies GPU operation with a pass/fail result, then leaves the VM off after completed setup.
- Later toggle/VRAM changes prepare and apply directly without repeating unnecessary setup or initial verification.
- Remembering guest credentials is optional. Saved settings, current observations, drafts, and failures stay truthful.

The VRAM range does not establish enforced isolation. Automatic balancing and full-capacity warnings are outside current scope.

## Current state

Settings contains the Appearance controls; the Credentials & diagnostics section has been removed from that page. System displays the GPU information boxes without the This computer box. The current source passed the non-hardware build/test gate; installed visual validation remains pending.

The next source priority is [core simplification](docs/backlog.md#next-action): a 10,000-line goal for `src/` only, excluding Rust tests, one shared operation path and the existing Slint design. Unused tests should be removed; useful tests follow the program's design. The [roadmap](docs/roadmap.md) proposes the replacement/deletion sequence; the core slice, [GUI simplification](docs/ENGINEERING.md#gui-simplification-checkpoint) and [captured-request cleanup](docs/ENGINEERING.md#captured-gui-requests-checkpoint) passed complete non-hardware gates. The [worker lifecycle follow-up](docs/ENGINEERING.md#worker-lifecycle-checkpoint) also passed its complete gate. The corrected `src/` count is 28,346 lines. Repository tools are outside the goal; full-budget feasibility remains pending. Previous source optimizations are implemented; their [recorded automated/install evidence](docs/ENGINEERING.md#optimization-validation-checkpoint) is separate from pending installed journeys, measurements and VM/GPU/guest acceptance. The [UI text guide](docs/UI_TEXT_GUIDE.md) describes the current 497-key compiled catalogue and remaining migration limits.

## Project guidance

| Document | Purpose |
| --- | --- |
| [Roadmap](docs/roadmap.md) | Agreed scope and milestone gates |
| [Backlog](docs/backlog.md) | Simplification tasks, dependencies and next action |
| [Agreed workflow](#agreed-workflow) | User workflow |
| [Architecture](docs/ARCHITECTURE.md) | Shared Rust components and boundaries |
| [Configuration](docs/CONFIGURATION.md) | Intent formats and storage |
| [Security](docs/SECURITY.md) | Protected operations and recovery |
| [Validation](docs/ENGINEERING.md#validation-and-live-acceptance) | Check effects and live acceptance boundaries |
| [Engineering](docs/ENGINEERING.md) | Status and documentation ownership |
| [AGENTS.md](AGENTS.md) | Authoritative Codex instructions |

Build prerequisites come from the current manifests/toolchain. Run the non-elevated gate with `powershell -NoProfile -File scripts/testing/check.ps1`; [testing](docs/ENGINEERING.md#validation-and-live-acceptance) defines its effects and exclusions. Keep passwords, driver payloads, VM disks and machine-specific evidence out of Git.

Codex starts with [AGENTS.md](AGENTS.md), which routes to short task prompts and focused [Rust](.github/prompts/rust.prompt.md) and [Slint](.github/prompts/slint.prompt.md) guidance. Read only the guidance relevant to the requested task; product requirements stay in the handbook.
