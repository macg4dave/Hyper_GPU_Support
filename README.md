# Hyper GPU Support

Rust-based GPU-PV tooling for existing Windows Hyper-V Generation 2 virtual machines.

This project is focused on one core problem: making it practical to prepare and manage GPU access for a selected VM without turning the host environment into a fragile one-off setup. The toolchain centers on a shared Rust backend, a protected Windows runner, and a GUI that reflects the product workflow while keeping the underlying architecture explicit and reviewable.

## Why this project exists

Modern virtualization workflows often need more than just "install a driver". GPU passthrough and related VM preparation require careful handling of:

- Hyper-V VM inventory and identity
- GPU discovery and compatibility checks
- Guest preparation and driver staging
- Secure, audited admin operations
- Recovery, status checks, and safe rollback boundaries

Hyper GPU Support is designed around that model: it treats GPU enablement as an operational workflow rather than a loose collection of scripts.

## Product status

This repository is an active engineering project with a clear product direction:

- The approved Slint GUI launches in live mode without arguments. `--mock-gui` explicitly selects simulated rehearsal.
- The CLI uses the shared Rust backend; binding the existing GUI controls to that backend remains scheduled.
- The CLI is still available for headless workflows and lower-level validation.
- Live startup reads actual VM/GPU inventory; unconnected actions remain unavailable. Mock mode currently uses fixtures and simulated outcomes, with real-data read-only rehearsal planned as integration progresses.
- Protected runner, guest worker, and transport boundaries remain intentionally separated as part of a staged, safe implementation approach.

Restricted same-executable worker consolidation and Windows GUI console packaging
remain planned. The current host binary retains the console subsystem so CLI
stdout/stderr and exit codes remain reliable.

## What is included

### Core capabilities

- Windows-focused VM and GPU inventory discovery
- Protected administrator-side execution flow
- Prepare/apply/verify/disable operation model
- Shared Rust core retained for CLI operation and planned GUI binding
- Reviewable, auditable operation logic for privileged tasks

### Current product shape

- GUI in the Slint app layer
- Rust core logic in the main project sources
- Separate host/guest artifacts and runner boundaries for security and safety
- Contributor infrastructure kept distinctly outside the product path

## Architecture at a glance

The project is organized around a small set of operating principles:

- Rust owns the application logic and workflow coordination.
- Hyper-V and Windows inventory are discovered from native interfaces rather than hardcoded assumptions.
- Security-sensitive operations are routed through the protected runner model.
- The GUI and CLI share the same execution concepts, even when the UI is currently mocked for presentation and validation work.

For deeper technical detail, see:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/ROADMAP.md](docs/ROADMAP.md)
- [docs/GUI_GUIDE.md](docs/GUI_GUIDE.md)
- [docs/ENGINEERING.md](docs/ENGINEERING.md)

## Quick start

To run the main GUI against actual inventory:

```powershell
cargo run --locked
```

To run the mock GUI without persistent writes or real operations:

```powershell
cargo run --locked -- --mock-gui
```

Plain mock mode uses fixtures. To inspect an existing JSON capture from the
`inventory` command without calling the backend:

```powershell
hyper-gpu-support --mock-gui --snapshot inventory.json
```

Snapshot input uses the real-data cards/System presentation and labels every
observation historical. Recorded enrolled pairs support draft editing and shared
plan rehearsal. An optional `--config GUID.toml` loads one candidate per-VM file
through the production parser, separately from enrollment and observation.
Enabled previews require a `plans` array containing shared CLI plan JSON with a
unique recorded payload digest for the selected VM/GPU/driver; missing managed
records remain unknown. Disabled previews do not require a payload digest.
Review can display simulated stages without executing them or saving the draft.
Refresh rereads the inputs; external changes block an existing draft until it is
discarded and refreshed. Credentials and execution remain blocked.
This route passed scoped automated and headless runtime
[testing](docs/evidence/GUI-002-testing.md); desktop and protected-effect acceptance remain open;
capturing inventory is a separate operation and may write mandatory runner audits.
See the
[mode contract](docs/GUI_GUIDE.md#mock-mode-development-direction).
Normal mode never substitutes simulated success for an unconnected operation.
Ordinary-token live discovery retains the existing runner's required audit records;
it does not automatically install or elevate a runner.

To build the project and check the CLI surface:

```powershell
cargo build --locked
cargo run --locked -- --help
```

The repository also includes validation scripts for project checks:

```powershell
.\scripts\testing\check.ps1
.\scripts\testing\check-docs.ps1
```

## CLI usage

The project retains headless CLI commands for operational flows. A typical admin workflow looks like this:

```powershell
hyper-gpu-support inventory
hyper-gpu-support install --config my-vms.toml
hyper-gpu-support plan --config my-vms.toml
hyper-gpu-support apply --config my-vms.toml --vm VM-GUID
hyper-gpu-support status --config my-vms.toml
hyper-gpu-support verify --config my-vms.toml --vm VM-GUID
hyper-gpu-support disable --config my-vms.toml --vm VM-GUID
```

Use the example configuration as a starting point:

- [config/product.example.toml](config/product.example.toml)

The configuration model keeps machine-specific values out of the code and supports the project’s Windows-native operation flow.

## GUI preview

The Slint UI is a first-class product surface and is the main GUI entry point for the project. It can be checked and previewed with the existing Slint tooling:

```powershell
slint-viewer --check src/gui/ui/app.slint
slint-viewer --auto-reload src/gui/ui/app.slint
```

The approved design is preserved in both modes. Component preview uses fixture
properties; normal executable startup uses actual discovery and `--mock-gui`
selects the simulated callbacks.

## Safety and boundaries

This project is intentionally careful about privileged operations:

- GPU-PV and Hyper-V operations are treated as high-risk workflows.
- Protected runner and guest-side boundaries are separate by design.
- Product operations avoid broad, unsafe assumptions about host state.
- Contributor tooling and lab automation remain separate from the runtime product path.

The product manages existing user-selected VMs. Development hardware testing uses
verified disposable targets; that laboratory setup is not an operator requirement.

Slint and dependency licensing/notices must be reviewed before distribution.
Local icon attribution is preserved in [src/gui/ui/icons](src/gui/ui/icons/README.md).

## Repository layout

- [src](src) — Rust runtime, workflow logic, Windows integration, and model types
- [src/gui/ui](src/gui/ui) — Slint UI sources and presentation layer
- [config](config) — configuration examples and product policy inputs
- [docs](docs) — architecture, roadmap, engineering, and design artifacts
- [scripts](scripts) — helper scripts and validation-oriented tooling
- [tools/lab](tools/lab) — contributor lab automation and research-only assets

## Documentation and next steps

The roadmap and architecture docs are the authoritative source for product direction and boundaries:

- [docs/ROADMAP.md](docs/ROADMAP.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/GUI_GUIDE.md](docs/GUI_GUIDE.md)
- [docs/ENGINEERING.md](docs/ENGINEERING.md)
- [docs/DECISIONS.md](docs/DECISIONS.md)

This project is best understood as a serious Windows GPU virtualization workflow tool in active development: real enough to validate the architecture, clear enough to extend, and disciplined enough to keep privileged operations safe.
