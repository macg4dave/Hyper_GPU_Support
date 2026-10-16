# Hyper GPU Support

A Windows GUI for turning GPU support on or off for supported existing Hyper-V VMs and choosing their GPU VRAM amount. Hyper-V handles VM creation, general management, and system RAM.

The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including multiple VMs sharing one GPU. Keep the existing Slint design and shared Rust backend.

GPU installation, driver refresh and Repair / Reinstall identify the **current running guest OS** through a read-only authenticated session before GPU or installation changes. Only recognised Windows 11 client releases can proceed. For an off VM, the review includes a temporary start for identification and graceful shutdown; missing or unreliable identity blocks installation. Detection never mounts or unlocks disks and never uses historical Hyper-V metadata as installation authority. BitLocker may unlock normally during guest boot; recovery prompts must be resolved locally. Discovery, ordinary management, disable and verification-only work do not run this check. See [guest OS eligibility](docs/SECURITY.md#guest-os-eligibility-before-installation). This replacement passed the complete non-hardware gate on 16 October 2026 (399 product tests and 12 catalogue tests, strict lint, binary builds and API documentation). Protected installation passed on 16 October 2026 with all four artifact hashes matching and enrollment/persistent state preserved. Live qualification remains pending.

## Agreed workflow

- Select a VM, set GPU support on/off, and choose VRAM from 1 GB to the selected card's capacity.
- Apply opens a review showing actual changes, required VM power transitions, and relevant time warnings. A second Apply confirms.
- After first-time GPU management permission is saved, fresh status readback opens the GPU setup review directly. There is no separate permission-saved dialog; Apply still requires confirmation.
- First GPU setup manages preparation and VM power, explains guest credentials before the local prompt, verifies GPU operation with a pass/fail result, then leaves the VM off after completed setup.
- Later toggle/VRAM changes follow review and Apply without repeating full setup or initial verification.
- Remembering guest credentials is optional. Saved settings, current observations, drafts, and failures stay truthful.

The VRAM range does not establish enforced isolation. Automatic balancing and full-capacity warnings are outside current scope.

## Current state

Settings contains the Appearance controls; the Credentials & diagnostics section has been removed from that page. System displays the GPU information boxes without the This computer box. These page changes await build and visual validation.

The previous APP backlog is complete by user confirmation on 13 October 2026. Current work follows the consolidated [QA roadmap](docs/roadmap.md). Fixed GUI copy and the [editor handoff](docs/UI_TEXT_GUIDE.md#editor-handoff) are implemented; the catalogue contains 408 of 696 planned keys with generated Rust/Slint bindings. LANG-006 dynamic/progress/failure extraction was recorded as in progress before the QA planning transition. Full migration, rendered acceptance and the later editorial pass remain pending. Earlier check summaries are historical context and do not establish new test results; [backlog](docs/backlog.md) now tracks QA tasks and their actual results.

Plan preparation, Apply/Verify reviews and the GPU permission review share the operation dialog with execution and results. The four-step header highlights the current phase, and reviews retain their exact VM/GPU/memory details and explicit power consent. Permission review keeps uniformly spaced identity/allocation lines and its bold Windows permissions notice. Apply uses a fixed checklist in reviewed order; completed milestones retain their ticks during repeated checks, while the current-stage heading and expandable, copyable log show every actual check. A later failure overrides a completed tick. Dialogs and logs use the existing neutral light/dark surfaces. Cancellation abandons read-only plan preparation and keeps the draft; cancellation remains disabled during Apply. Result summaries still require the confirmed terminal outcome and saved readback. Footer actions remain on one row, scrolling horizontally at smaller sizes. More Actions retains its existing layout. This correction passed the complete non-hardware gate and protected update on 16 October 2026. Headless interaction/render checks cover both themes at normal and minimum sizes; installed interaction and VM/GPU acceptance remain pending. VM OS icons use transparent monochrome SVGs that follow the current theme. Technical Details supports text selection, Copy/Select All and keyboard shortcuts.

Legacy mock/snapshot GUI tooling has been removed under APP-001 and passed the M1 automated gate. Its flags are unsupported; backend unit test doubles remain.

Running the executable without arguments opens the live GUI and can request elevated first-install discovery. CLI and fixed internal modes share the backend; source entrypoints are not authorization to perform live operations.

## Project guidance

| Document | Purpose |
| --- | --- |
| [Roadmap](docs/roadmap.md) | Agreed scope and milestone gates |
| [Backlog](docs/backlog.md) | QA tasks, dependencies and next action |
| [UI text inventory](docs/UI_TEXT_INVENTORY.md) | Presentation consumers and retained input exceptions |
| [Agreed workflow](#agreed-workflow) | User workflow |
| [Architecture](docs/ARCHITECTURE.md) | Shared Rust components and boundaries |
| [Configuration](docs/CONFIGURATION.md) | Intent formats and storage |
| [Security](docs/SECURITY.md) | Protected operations and recovery |
| [Validation](docs/ENGINEERING.md#validation-and-live-acceptance) | Check effects and live acceptance boundaries |
| [Engineering](docs/ENGINEERING.md) | Status and documentation ownership |
| [AGENTS.md](AGENTS.md) | Authoritative Codex instructions |

Build prerequisites come from the current manifests/toolchain. Run the non-elevated gate with `powershell -NoProfile -File scripts/testing/check.ps1`; [testing](docs/ENGINEERING.md#validation-and-live-acceptance) defines its effects and exclusions. Keep passwords, driver payloads, VM disks and machine-specific evidence out of Git.

Codex starts with [AGENTS.md](AGENTS.md), which routes to short task prompts and focused [Rust](.github/prompts/rust.prompt.md) and [Slint](.github/prompts/slint.prompt.md) guidance. Read only the guidance relevant to the requested task; product requirements stay in the handbook.
