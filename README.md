# Hyper GPU Support

A Windows GUI for turning GPU support on or off for supported existing Hyper-V VMs and choosing their GPU VRAM amount. Hyper-V handles VM creation, general management, and system RAM.

The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including multiple VMs sharing one GPU. Keep the existing Slint design and shared Rust backend.

## Agreed workflow

- Select a VM, set GPU support on/off, and choose VRAM from 1 GB to the selected card's capacity.
- Apply opens a review showing actual changes, required VM power transitions, and relevant time warnings. A second Apply confirms.
- First GPU setup manages preparation and VM power, explains guest credentials before the local prompt, verifies GPU operation with a pass/fail result, then leaves the VM off after completed setup.
- Later toggle/VRAM changes follow review and Apply without repeating full setup or initial verification.
- Remembering guest credentials is optional. Saved settings, current observations, drafts, and failures stay truthful.

The VRAM range does not establish enforced isolation. Automatic balancing and full-capacity warnings are outside current scope.

## Current state

The source already contains discovery, setup/enrollment, reviewed GPU effects, saved settings, recovery, and guest verification. It still needs targeted changes to match the agreed workflow. [Baseline](docs/BASELINE.md) records the differences. This documentation rewrite ran no product build, test, GUI launch, or live operation.

Legacy mock/snapshot GUI tooling has been removed from source under APP-001. Its flags are unsupported; backend unit test doubles remain. Removal validation is pending the milestone gate.

Running the executable without arguments opens the live GUI and can request elevated first-install discovery. CLI and fixed internal modes share the backend; source entrypoints are not authorization to perform live operations.

## Project guidance

| Document | Purpose |
|---|---|
| [Roadmap](docs/ROADMAP.md) | Agreed scope and milestone gates |
| [Backlog](docs/BACKLOG.md) | Source tasks and next action |
| [Baseline](docs/BASELINE.md) | Current source gaps and validation limits |
| [GUI guide](docs/GUI_GUIDE.md) | User workflow |
| [Architecture](docs/ARCHITECTURE.md) | Shared Rust components and boundaries |
| [Configuration](docs/CONFIGURATION.md) | Intent formats and storage |
| [Security](docs/SECURITY.md) | Protected operations and recovery |
| [Testing](docs/TESTING.md) | Validation timing and scenarios |
| [Engineering](docs/ENGINEERING.md) | Status and documentation ownership |
| [Decisions](docs/DECISIONS.md) | Confirmed product choices |
| [AGENTS.md](AGENTS.md) | Authoritative Codex instructions |

Build prerequisites come from the current manifests/toolchain. The previous gate script is absent; APP-008 prepares its replacement at the implementation milestone. Cached binaries are not current validation evidence. Keep passwords, driver payloads, VM disks, and machine-specific evidence out of Git.

Codex starts with [AGENTS.md](AGENTS.md), which routes to short task prompts and focused [Rust](.github/prompts/rust.prompt.md) and [Slint](.github/prompts/slint.prompt.md) guidance. Read only the guidance relevant to the requested task; product requirements stay in the handbook.
