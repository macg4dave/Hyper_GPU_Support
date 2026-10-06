# Hyper GPU Support

Rust CLI/core for one Windows 11 x64 Generation 2 Hyper-V guest and an NVIDIA
RTX 5060 8 GB. Normal Hyper-V GPU-PV has passed sustained Code 0, `nvidia-smi`,
checked D3D11/D3D12 rendering and CUDA computation. Native Rust discovery matches
the discovered driver/runtime inventory for the measured baseline. Provisioning
derives the complete associated payload and guest destinations from the selected
GPU and installed host driver; no fixed file count defines success.

The complete Rust guest writer passed live full-manifest apply and verified reapply
on a clean child. The fixed Rust runner applies and independently verifies the
validated VM/resource settings. `validate` now orchestrates the fixed guest
readiness and workload checks; combined clean-child live qualification belongs
to GPU-006. The
[architecture](docs/ARCHITECTURE.md) defines our recipe; the
[baseline](docs/evidence/GPU-PV-BASELINE.md) records measured results. Start work at
[AGENTS.md](AGENTS.md) and the relevant [backlog card](docs/BACKLOG.md).
[ROADMAP.md](docs/ROADMAP.md) describes the route to v1.

## Windows development

Install Git for Windows, Rust via rustup, and Microsoft C++ Build Tools with the
x64/x86 MSVC tools and Windows SDK. These provide Rust's native linker/import
libraries. `rust-toolchain.toml` pins Rust 1.94.0, rustfmt, Clippy and
`x86_64-pc-windows-msvc`; `Cargo.lock` pins dependencies.

From PowerShell at the repository root:

```powershell
cargo build --locked --workspace --all-features
cargo run --locked -- --help
cargo run --locked -- inventory
.\scripts\testing\check.ps1
```

The check script runs formatting, strict Clippy, tests, build, rustdoc and generated
configuration checks with warnings denied. Hardware-free tests/CI do not require
elevation, Hyper-V or a GPU and do not establish GPU support. For documentation
changes, run `.\scripts\testing\check-docs.ps1`. See
[scripts/README.md](scripts/README.md) for maintained tooling and required privilege.

The main CLI currently implements help, version, read-only inventory and guest
`validate`. Declared `plan`, `apply`, `status`, `remove`, `recover`, `start`, `shutdown`
and `restart` commands explicitly return implementation exit code 70 until
integrated. Errors go to stderr; normal output goes to stdout. Inventory reports
known/missing/denied/unavailable facts; denied protected facts can appear in an
otherwise successful read-only report.

## Current development executables

These are implemented development entry points, not a completed v1 workflow:

| Entry point | Current capability |
|---|---|
| `hyper-gpu-driver-environment` | Native read-only complete source/destination/length/hash manifest discovery |
| `hyper-gpu-stage` | Discover, inspect and apply the complete driver/runtime manifest; verify every destination on reapply |
| `hyper-gpu-client` | Enrolled fixed-runner inspection, guest lifecycle/reset, exact GPU attach/detach and complete profile application |
| `hyper-gpu-support validate` | Verified runtime transfer, sustained readiness and bounded checked guest workloads |
| D3D11/D3D12/CUDA probe artifacts | Fixed checked workloads used by validation |

Read-only inspectors:

```powershell
cargo run --locked --release --bin hyper-gpu-driver-environment
cargo run --locked --release --bin hyper-gpu-stage
```

The full-environment inspector emits deterministic JSON and does not copy files.
The complete-manifest apply prompts for runtime credentials:

```powershell
cargo run --locked --release --bin hyper-gpu-stage -- apply --interactive
```

Inspection and apply require Hyper-V read access; apply additionally requires the
authorized elevated development path from the outset. Fresh staging requires a
running disposable guest with no GPU adapter attached. The writer uses discovered
destinations and ordinary byte copies, verifies all guest lengths/hashes and records
a full-manifest receipt. Matching reapply rehashes every destination. It does not
apply the VM/resource settings or run GPU workloads. Interrupted or partial staging
retains the staging lock and requires recreation of the disposable child.

With the controlled runner installed and the configured VM off:

```powershell
cargo run --locked --bin hyper-gpu-client -- ensure-gpu
cargo run --locked --bin hyper-gpu-client -- configure-slot
```

`ensure-gpu` verifies an existing exact assignment or attaches the configured GPU.
`configure-slot` requires that exact adapter and an off VM, applies the typed
`[vm_profile]` and all `[resources]` triples, then verifies fresh effective values.
Matching reapply reports `already-applied` after independent readback. Existing
Microsoft Windows Secure Boot and vTPM are required and retained. A partial or
uncertain update keeps the runner reconciliation marker and durable settings preimage;
inspect and reconcile the actual state before another mutation.
Assignment alone does not establish staging/readiness. Additional fixed operations
are defined in the runner protocol; callers cannot supply targets, paths or scripts.

## Configuration and testing target

Run guest validation from an elevated development terminal after staging, settings
and start have completed on the enrolled disposable VM:

```powershell
cargo build --locked --release --bin hyper-gpu-support --bin hyper-gpu-validation-worker `
  --bin d3d11-probe --bin d3d12-probe --bin cuda-identity
cargo run --locked --release -- validate
```

The command first checks local prerequisites, then prompts locally for guest
credentials. It transfers and hashes the fixed probe/sample/FATBIN inputs plus
three app-local Microsoft x64 CRT DLLs from `[validation.crt_directory]`. CUDA
sample preparation/build commands remain in [the probe manifest](probes/MANIFEST.md).
Guest copies and the worker use protected paths; the transport shares the runner's
operation lock. It changes only validation artifacts and runs fixed workloads.

For combined clean-child qualification, run from an elevated interactive terminal:

```powershell
.\scripts\testing\run-clean-child-qualification.ps1
```

This explicitly recreates the enrolled disposable child, invokes the Rust staging,
settings and validation entry points, verifies reapply and graceful shutdown, and
retains a combined report under `[paths.test_output]`. Enter guest credentials only
at the local Rust prompts. A failed run retains its results for reconciliation.

The JSON report distinguishes pass, fail, blocked and untested for input integrity,
sustained Code 0, nvidia-smi, D3D11, D3D12, CUDA selection and vector addition.
Exit 0 requires every essential check to pass. CUDA/DXGI LUID comparison remains
a diagnostic; safe single-device selection is required for computation. Readiness,
sampling and process/worker deadlines are configured under `[validation]`.

[`config/project.toml`](config/project.toml) is the sole hand-edited, non-secret
configuration. It owns mutable VM/GPU/image/runner/tool/test identities, paths,
pins and deadlines. [CONFIGURATION.md](docs/CONFIGURATION.md) explains ownership,
current schema gaps and generated policy/pin updates. Development binaries currently
embed this configuration; operator runtime file selection is planned work.

Normal development runs unelevated. Known privileged testing uses the approved
fixed runner or the authorized development adapter, as described in the
[Windows elevation policy](docs/ENGINEERING.md#windows-elevation-and-uac).
The test target is one fixed VM shell and disposable differencing child of an
immutable clean Windows 11 parent. Immediately verify configured identities before
effects; uncertainty recovers by recreating only that child. Physical-host restart,
shutdown or logout requires explicit user permission immediately beforehand.

Keep local outputs in ignored `local/`, OS/VM/driver artifacts in the configured
[`data/` layout](data/README.md), and build outputs in `target/` unless
`CARGO_TARGET_DIR` overrides it. Never commit credentials, keys, disks, OS images
or proprietary drivers. Project licensing/delivery policy remains an owner decision
before release distribution ([DEC-008](docs/DECISIONS.md#dec-008)).
