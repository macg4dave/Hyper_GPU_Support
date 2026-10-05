# Hyper GPU Support

A Windows 11 x64 GPU-PV project targeting one Windows 11 guest and an NVIDIA
RTX 5060 8 GB, using a normal Generation 2 Hyper-V VM and Rust CLI/core.
Easy-GPU-PV is the primary reference for VM configuration, GPU assignment and
driver staging; AppSandbox is secondary GPU-PV reference material. HCS-owned-guest
work is paused. The project implements inventory, fixed disposable lifecycle,
GPU attachment and manifest staging, but has not yet demonstrated successful
guest D3D11/D3D12/CUDA workloads. See the
[Easy-GPU-PV comparison](docs/evidence/GPU-009-easy-gpu-pv-comparison.md).

## Windows development

Install Git for Windows, [Rust via rustup](https://www.rust-lang.org/tools/install),
and [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
Select **Desktop development with C++**, including x64/x86 MSVC tools and a
Windows SDK. The Visual Studio IDE is not required. These supply Rust's native
linker and Windows import libraries; no project-owned C++ code is used.

Clone this repository and open PowerShell in its root. Use the MSVC host:

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup show
cargo build --locked --workspace --all-features
cargo run --locked -- --help
cargo run --locked -- --version
cargo run --locked -- inventory
```

`rust-toolchain.toml` selects Rust 1.94.0, rustfmt, Clippy and the Windows x64
MSVC target; rustup downloads them on first use. The package's minimum Rust
version is the same pin. Third-party Rust dependencies are locked in `Cargo.lock`;
the package currently defines no optional Cargo features.
`.cargo/config.toml` defaults builds to that target and repository-local `target/`.
If your shell sets `CARGO_TARGET_DIR`, it overrides that location; for local
verification set `$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'`.
`Cargo.lock` is retained for reproducible dependency resolution. Default rustfmt
and Clippy settings are sufficient; no separate configuration is necessary.

The baseline was built with MSVC 14.51.36231 and Windows SDK 10.0.26100.0 on
Windows build 26200.9457 (25H2), x64. Other native tool versions are not yet
qualified; this does not claim identical binary output across machines.

## Checks

```powershell
.\scripts\testing\check.ps1
```

The script runs formatting, strict Clippy, tests, build and rustdoc with locked
dependencies and compiler/doc warnings denied. It stops on the first failure,
returns a nonzero exit code and restores the caller's Rust flag environment.
Documentation-only changes can also run
`.\scripts\testing\check-docs.ps1` for local link targets, prompt frontmatter and
diff-whitespace validation.

Tests include unit tests, executable integration tests and a runnable doc test.
They require no elevation, Hyper-V, GPU, guest or network once the toolchain is
installed. Windows CI runs the same checks on hosted x64 runners; hosted CI is
not hardware qualification. Exit codes are 0 for successful commands, 1 for the
current inventory adapter/output boundary, 2 for invalid arguments, 3 for
configuration/plan errors, 4 for permission failures, 5 for environment failures,
6 for driver/runtime failures and 70 for implementation/protocol failures. Errors
go to stderr; normal output to stdout. Until their owning M1 tasks implement them,
the declared `plan`, `apply`, `status`, `validate`, `remove`, `recover`, `start`,
`shutdown` and `restart` commands return 70 explicitly.

`inventory` executes fixed, read-only Windows/Hyper-V queries and emits a stable
line-oriented report beginning with `inventory.schema=1`. Each fact is
`key=status[:value]`, where status is `known`, `missing`, `denied` or
`unavailable`. Newlines, carriage returns and percent signs in values are percent
escaped. A successful report may contain denied or missing facts; those are
observations, not process failures. Run it without elevation for normal diagnosis:

```text
inventory.schema=1
gpu.model=known:NVIDIA GeForce RTX 5060
gpup.interface=denied
vm.selection=denied
```

The command accepts no target or script argument and performs no mutation.
Its child query process has a 15-second deadline and 64 KiB limits for both stdout
and stderr; Rust terminates and reaps it on timeout. Rust also validates the
protocol, correlates the GPU-P interface to the exact RTX 5060 PCI identity and
selects a VM only when exactly one is registered. Missing cmdlets, denied access,
provider failures, ambiguity and adapter launch/timeout/exit failures remain
distinct. The narrow query-process decision is recorded in
[DEC-013](docs/DECISIONS.md#dec-013).
Administrator execution is not required by the CLI contract and must follow the
repository permission boundary when explicitly needed for protected facts.

## Source map

| Path | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | One package/workspace and pinned build inputs |
| `src/main.rs` | Process arguments, output and exit codes |
| `src/lib.rs`, `src/cli.rs` | Library boundary, CLI parser and unit tests |
| `src/config.rs`, `config/project.toml` | Typed project/environment configuration plus plan/report contracts |
| `src/guest.rs`, `src/staging.rs`, `src/windows_guest.rs` | Verified guest-transfer/manifest contracts and fixed PowerShell Direct adapters |
| `src/inventory.rs` | Typed fact/report model, validated adapter protocol and tests |
| `src/windows_inventory.rs` | Fixed read-only Windows/Hyper-V process adapter |
| `tests/cli.rs` | Executable behavior tests |
| `.github/workflows/ci.yml` | Windows build, lint, test and rustdoc checks |
| `scripts/` | Maintained development, setup, diagnostic, Hyper-V and test scripts |
| `docs/` | Architecture, roadmap, task evidence and engineering policy |

The [architecture source map](docs/ARCHITECTURE.md#foundation-source-layout)
identifies where future modules belong. Logging and additional Windows adapters
are deferred until meaningful behavior requires them.

The authoritative non-secret environment is [`config/project.toml`](config/project.toml).
Rust deserializes and validates it into typed structures; maintained scripts use the
shared configuration reader. It owns mutable VM/GPU/image/runner/tool/test identities,
paths, input hashes and deadlines. Generated `config/runner-policy-v1.json` and
`config/artifact-pins.toml` retain the exact privileged policy and release hashes
without becoming hand-edited settings. See the
[configuration ownership and audit](docs/CONFIGURATION.md), including the pin refresh
command and the boundary between mutable settings and implementation constants.

The desired-state subset still requires one canonical VM GUID, explicit GPU-P
interface and immutable manifest identity/hash. Each resource is either
`provider-default` or an exact `minimum,maximum,optimal` triple in opaque
provider-defined units. `driver_manifest.sha256` pins the deterministic encoded
file manifest and must match the package inspector before apply validation.

CORE-009's inspector validates the configured DriverStore source,
complete file/byte extent, INF version, catalog hash, every file hash and the
canonical tree/encoded-manifest digests without copying proprietary files:

```powershell
cargo run --locked --bin hyper-gpu-stage
```

The bounded apply mode prompts for the guest password with masked `*` feedback,
revalidates the required
Authenticode signer, copies the exact manifest through one PowerShell Direct session,
atomically finalizes the `HostDriverStore` package, creates the verified CUDA alias,
and writes an applied receipt. A matching reapply rehashes the package and is a no-op:

```powershell
cargo run --locked --bin hyper-gpu-stage -- apply <guest-user>
cargo run --locked --bin hyper-gpu-stage -- apply --interactive
```

Apply requires Hyper-V administrator access and is not a fixed-runner operation;
invoke it through the explicitly authorized elevated development path from the
start when the normal token lacks that access. A pre-mutation failure reports only its fixed phase name;
native error text and credentials are not emitted. Post-mutation uncertainty still
requires disposable-child recreation.

The configured host build is a qualification baseline, not an equality gate. Apply
reports the qualified, measured-host and measured-guest builds. It emits
`warning=host-build-outside-qualified-baseline` or
`warning=host-guest-build-drift` as applicable. The exact GPU driver/package
identity, target identity and signatures remain mandatory. Active servicing or a
pending rename/replacement remains a hard failure. Only exact reviewed delete-only
sources pinned in configuration are reported as `warning=host-pending-delete-cleanup`;
an unknown deletion remains a hard review gate.

Any partial marker, mismatched destination, timeout or interrupted apply makes the
guest state uncertain; recreate the disposable child instead of retrying in place.

The fixed privileged runner is implemented in `src/runner.rs` and
`src/bin/hyper-gpu-runner.rs`. Its administrator-owned scheduled task accepts only
compiled, policy-listed operations for the exact disposable VM, GPU and parent/
child identities; it is not a general CLI or shell boundary. Installation
identity, test evidence and remaining protected CORE-005 validation are recorded in
[CORE-005 evidence](docs/evidence/CORE-005.md).

From the enrolled normal account, ensure the configured GPU is attached while the
disposable VM is off:

```powershell
cargo run --locked --bin hyper-gpu-client -- ensure-gpu
```

The command strictly validates the runner's published inspection and assignment
records. An exact existing assignment is a verified no-op; a foreign adapter,
different target, running VM, failed provider operation or malformed result fails.
`staging_readiness=not-checked` is intentional: attachment does not imply that the
guest driver/runtime receipt remains ready, which is validated separately.

CORE-008's development harness copies one hash-pinned file to a flat filename in the
configured protected guest staging root. It prompts on the console so the password
never appears in the command line or repository:

```powershell
cargo run --locked --bin hyper-gpu-guest-copy -- `
  <guest-user> <absolute-source> <destination-filename> <lowercase-sha256>
```

Use `--interactive` in place of `<guest-user>` to prompt for both the username and
the hidden password, keeping all guest credentials out of command-line arguments.

The VM must be running locally and the caller must have Hyper-V access. Any timeout
or interrupted copy makes guest staging state uncertain; recreate the disposable
child rather than retrying it in place.

Keep secrets and uncommitted overrides in ignored `local/`, build output in `target/`,
and OS/VM/driver/test artifacts in the reproducible ignored
[`data/` layout](data/README.md). The relative tree is the default; hardware work
records an explicit external data root when large artifacts live elsewhere. Never
add secrets, VM disks, OS images or proprietary drivers. Ignore rules are only a
convenience; review staged content. Source and configuration templates belong in
Git. Project licensing/distribution remains an owner decision before packaging
(DEC-008); the package is not publishable to a registry.

AppSandbox remains an external, pinned [reference map](docs/ARCHITECTURE.md#upstream-reference-map).
No upstream implementation or binary has been copied into this scaffold.
Start future sessions at [AGENTS.md](AGENTS.md) and the
[backlog](docs/BACKLOG.md).

## Development target workflow

The hardware path uses one immutable clean Windows 11 Generation 2 parent VHDX
and one disposable test VM backed by a differencing VHDX. GPU-PV assignment,
NVIDIA runtime staging and probes occur only in the disposable child. If its state
is damaged or uncertain, discard and recreate the child instead of repairing it;
never boot or modify the parent for an experiment.

Normal Rust builds and tests remain unelevated. Privileged hardware tests use the
approved administrator-installed runner with fixed operations and
explicit VM/GPU identities. The repository and agent cannot modify its installed
binary or policy. VMConnect, Enhanced Session Mode or RDP may be used for operator
access; no project virtual display driver is planned, and display connectivity is
not evidence of NVIDIA workload execution. See the
[Windows elevation policy](docs/ENGINEERING.md#windows-elevation-and-uac),
[configuration and recovery contract](docs/ARCHITECTURE.md#configuration-and-recovery-contract)
and [display boundary](docs/ARCHITECTURE.md#display-and-presentation-boundary).
