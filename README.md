# Hyper GPU Support

Hyper GPU Support is a Windows application for managing GPU partitioning on existing Hyper-V virtual machines. It has a Slint GUI and a CLI over a shared Rust backend, with protected enrollment, reviewed operations, and durable recovery.

The first release goal is a usable GUI managing one NVIDIA GPU shared by **at least two Windows 11 guests running hardware-rendered workloads concurrently**. Existing Generation 2 VMs are the target; creating VMs or operating a disposable laboratory is not the product's everyday workflow. Exact Windows builds and the initial qualification environment remain to be selected.

## Current state

The source contains live discovery, first-install setup, additive VM/GPU enrollment, reviewed Apply, separate Verify, configuration publication, and reconciliation paths. These are source findings, not a new runtime validation result. Concurrent sharing, host stability, packaging, and the complete GUI workflow have not been established by this documentation reset. Read the [source baseline](docs/BASELINE.md) for evidence and known gaps.

GPU replacement and Forget pairing are unavailable in the live GUI. CLI `forget` removes a stored credential; it does not forget enrollment. Resource requests and physical VRAM display must not be presented as proven per-VM isolation or fair sharing.

## Entrypoints

These describe the current dispatcher; they are not instructions to perform live operations now.

| Entrypoint | Purpose |
|---|---|
| `hyper-gpu-support.exe` with no arguments | Live GUI; startup can request elevated discovery when installation is missing |
| `--mock-gui` | Explicit fixture demonstration with no backend effects or persistent writes |
| `--mock-gui --snapshot FILE [--config GUID.toml]` | Historical inventory and optional per-VM intent rehearsal, with no backend effects or writes |
| `inventory`, `plan`, `status` | Discovery or selected-target inspection; live reads can require the installed protected runner |
| `install`, `apply`, `enable`, `disable`, `verify` | Protected setup or operations; authorization and lifecycle review are required |
| `credentials`, `forget` | Local credential storage/removal, separate from pairing authority |
| `import`, `import-reconcile` | Explicit configuration import and interrupted import reconciliation |
| `save-only`, `reconcile` | Bound configuration publication retry or uncertain-operation reconciliation |

See [GUI workflows](docs/GUI_GUIDE.md), [configuration](docs/CONFIGURATION.md), and [security/recovery](docs/SECURITY.md) before using live operations. Help text is known to omit some implemented commands; the baseline records this discrepancy.

## Development

The manifests pin Rust 1.94.0 and Slint 1.18.1 and select `x86_64-pc-windows-msvc`. Windows build tools are required. The intended local gate is described in [testing](docs/TESTING.md); its previous script and CI workflow were deleted and have not been restored. Existing inline Rust tests remain. Cached executables in `target/` are not evidence that this checkout builds or works.

`config/product.example.toml` and [sample configurations](config/samples/README.md) use the production intent schema. `config/project.toml`, artifact pins, and the old runner policy contain development context; they are not product runtime configuration or a support matrix. Keep passwords, VM disks, driver payloads, and machine-specific evidence out of Git.

## Project and AI guidance

| Document | Owns |
|---|---|
| [Roadmap](docs/ROADMAP.md) | First-release requirements, milestone sequence, and completion gates |
| [Backlog](docs/BACKLOG.md) | Actionable tasks, dependencies, status, and next task |
| [Baseline](docs/BASELINE.md) | Current source assessment, discrepancies, and validation limits |
| [Architecture](docs/ARCHITECTURE.md) | Components, data flow, and boundaries |
| [Configuration](docs/CONFIGURATION.md) | Intent schemas, storage, and compatibility |
| [GUI guide](docs/GUI_GUIDE.md) | Existing design and workflow behavior |
| [Security and recovery](docs/SECURITY.md) | Authority, trust, uncertainty, and recovery contracts |
| [Engineering](docs/ENGINEERING.md) | Task/status conventions and documentation ownership |
| [Testing](docs/TESTING.md) | Validation policy, planned gate, and qualification scenarios |
| [Decisions](docs/DECISIONS.md) | Agreed project choices and their consequences |
| [AGENTS.md](AGENTS.md) | Authoritative Codex instructions |

Focused Codex prompts live in `.github/prompts/`: [plan a task](.github/prompts/task-plan.prompt.md), [implement a task](.github/prompts/implement-task.prompt.md), [review code](.github/prompts/code-review.prompt.md), [review security/recovery](.github/prompts/security-recovery-review.prompt.md), [maintain documentation](.github/prompts/maintain-docs.prompt.md), and [validate a milestone](.github/prompts/validate-milestone.prompt.md). Their location is a repository convention; Codex is the supported agent for this reset.
