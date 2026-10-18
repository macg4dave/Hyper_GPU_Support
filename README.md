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

Settings contains the Appearance controls; the Credentials & diagnostics section has been removed from that page. System displays the GPU information boxes without the This computer box. These page changes await build and visual validation.

The previous APP backlog is complete by user confirmation on 13 October 2026. Current work follows the [optimization roadmap](docs/roadmap.md), with grouped implementation tasks and separate validation status in the [backlog](docs/backlog.md). Fixed GUI copy and the [editor handoff](docs/UI_TEXT_GUIDE.md#editor-handoff) are implemented; the catalogue contains 408 of 696 planned keys with generated Rust/Slint bindings. LANG-006 dynamic/progress/failure extraction was recorded as in progress before the QA planning transition. Full migration, rendered acceptance and the later editorial pass remain pending. Earlier check summaries are historical context and do not establish new test results.

All in-app dialogs share one neutral light/dark modal frame, spacing, headings, fixed footer and expandable/copyable log. Separate review dialogs have been removed. This includes operation progress/results, read progress, recovery, permission setup, profile deletion, draft-switch/exit choices, notices, errors, help and More Actions. More Actions retains its vertical choices. Workflow headers follow typed operation identities: reads, plan building, app installation, permission updates/reconciliation, Apply, Verify, recovery, save-only and removal have their own labels. Readback appears only when the controller actually starts it; failed or cancelled work cannot retain a successful Result header. Informational dialogs show their actual title without an invented Apply flow. Permission preparation distinguishes first installation from granting an additional VM/GPU pair. Apply retains its stable reviewed checklist and confirmed outcome/readback requirements. Cancellation can abandon read-only plan preparation and keeps the draft; cancellation remains disabled during protected effects. Fresh preparation retains exact identity/allocation facts; the foreground action is the approval to proceed. Recovery is still entered only through an explicit foreground action and never replays Apply. The main page layout is unchanged; action captions describe the operation they start. Windows authentication and UAC prompts retain their native Windows UI. The direct-execution update passed the full non-hardware gate and protected installation on 16 October 2026: 408 product tests and 12 catalogue tests passed; all four installed artifact hashes match, both enrolled targets and 1,223 persistent files were preserved, and the runner is enabled and Ready. Headless interaction/render checks cover both themes at normal and minimum sizes; installed GUI and VM/GPU acceptance remain pending. VM OS icons use transparent monochrome SVGs that follow the current theme. Technical Details supports text selection, Copy/Select All and keyboard shortcuts.

Legacy mock/snapshot GUI tooling has been removed under APP-001 and passed the M1 automated gate. Its flags are unsupported; backend unit test doubles remain.

Running the executable without arguments opens the live GUI and can request elevated first-install discovery. CLI and fixed internal modes share the backend; source entrypoints are not authorization to perform live operations.

The cancellation/startup feedback follow-up shows a modal "Cancelling, please wait..." dialog until an abandoned read releases its block, then closes it automatically, and names each startup read. Validation passed on 17 October 2026: the complete non-hardware gate passed 410 product tests and 12 catalogue tests, strict lint, binary builds and API documentation (`local/evidence/m1-gate-20261017T102122120Z/`). Protected installation passed with all four build/installed/enrollment hashes matching, both enrolled targets and all 1,239 existing persistent files preserved, and the runner enabled and Ready (`local/evidence/install-20261017T102257896Z/verification.json`). Installed GUI and VM/GPU/guest acceptance remain pending.

## Project guidance

| Document | Purpose |
| --- | --- |
| [Roadmap](docs/roadmap.md) | Agreed scope and milestone gates |
| [Backlog](docs/backlog.md) | Optimization tasks, dependencies and next action |
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
