# Hyper GPU Support

A Windows 11 x64 GPU-PV project targeting one Windows 11 guest and an NVIDIA
RTX 5060 8 GB. AppSandbox is the user's known-working HCS reference on this
hardware; the current project executable only provides help and version output
and has not yet reproduced GPU-PV through its own Rust/native path.

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
```

`rust-toolchain.toml` selects Rust 1.94.0, rustfmt, Clippy and the Windows x64
MSVC target; rustup downloads them on first use. The package's minimum Rust
version is the same pin. There are no third-party crates or optional features.
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
$env:RUSTFLAGS = '-Dwarnings'
$env:RUSTDOCFLAGS = '-Dwarnings'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo build --locked --workspace --all-features
cargo doc --locked --workspace --all-features --no-deps
```

Tests include unit tests, executable integration tests and a runnable doc test.
They require no elevation, Hyper-V, GPU, guest or network once the toolchain is
installed. Windows CI runs the same checks on hosted x64 runners; hosted CI is
not hardware qualification. Exit codes are 0 for help/version, 2 for invalid
arguments and 1 for output failure (including unavailable stderr). Errors go to
stderr; normal output to stdout.

## Source map

| Path | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | One package/workspace and pinned build inputs |
| `src/main.rs` | Process arguments, output and exit codes |
| `src/lib.rs`, `src/cli.rs` | Library boundary, CLI parser and unit tests |
| `tests/cli.rs` | Executable behavior tests |
| `.github/workflows/ci.yml` | Windows build, lint, test and rustdoc checks |
| `docs/` | Architecture, roadmap, task evidence and engineering policy |

The [architecture source map](docs/ARCHITECTURE.md#foundation-source-layout)
identifies where future modules belong. Configuration, logging and Windows
adapters are deferred until meaningful behavior requires them.

Keep small machine-local settings in ignored `local/`, build output in `target/`,
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

Normal Rust builds and tests remain unelevated. Planned hardware tests use a
separately approved, administrator-installed runner with fixed operations and
explicit VM/GPU identities. The repository and agent cannot modify its installed
binary or policy. VMConnect, Enhanced Session Mode or RDP may be used for operator
access; no project virtual display driver is planned, and display connectivity is
not evidence of NVIDIA workload execution. See the
[configuration and recovery contract](docs/ARCHITECTURE.md#configuration-and-recovery-contract)
and [display boundary](docs/ARCHITECTURE.md#display-and-presentation-boundary).
