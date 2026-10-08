# Hyper GPU Support

Rust GPU-PV management for existing Hyper-V VMs, with a native Windows GUI using
the same core. Select multiple Generation 2 VMs, one GPU each. NVIDIA preparation
is implemented first; discovery can list other vendors without claiming support.
The historical RTX 5060/Windows 11 baseline proves feasibility, but the rewritten
product still needs changed-boundary hardware qualification. See the
[roadmap](docs/ROADMAP.md), [architecture](docs/ARCHITECTURE.md) and
[current acceptance card](docs/BACKLOG.md#arch-001).

## Build and checks

Use the pinned Rust/MSVC/Windows SDK toolchain from `rust-toolchain.toml`.

```powershell
cargo build --locked
cargo run --locked -- --help
.\scripts\testing\check.ps1
.\scripts\testing\check-docs.ps1
```

Hardware-free builds/tests do not establish GPU support. Do not install the new
privileged artifacts until their independent review and affected checks pass.

The explicit M1 installation/enrollment test is separate from ordinary checks.
After independent boundary review, run the following in an administrator console,
using the built product artifact directory:

```powershell
.\scripts\testing\qualify-product-enrollment.ps1 -ArtifactDirectory PRODUCT-BUILD-DIRECTORY -OutputDirectory "$PWD\local\m1-enrollment"
```

The harness checks contributor-configured disposable identities before effects and
writes a schema-2 target with no laboratory inputs. It exercises installation,
interrupted-install refusal/recovery and discovery/status/preview without guest
mutations. Then run the installed-runner contract test with the ordinary user token:

```powershell
$env:HYPER_GPU_M1_CONFIG = "$PWD\local\m1-enrollment\target.toml"
cargo test --locked --test m1_enrollment -- --ignored --test-threads=1
```

This tests authentication, replay and unenrolled-target refusal, durable audit
outcomes and protected write denial. It does not qualify guest rendering or sharing.

## Runtime operation

In an administrator console, read native inventory before initial enrollment:

```powershell
hyper-gpu-support inventory
hyper-gpu-support install --config my-vms.toml
```

Use [config/product.example.toml](config/product.example.toml), replacing its example
identities with actual discovery. Install requires the runner, guest worker and D3D11
probe beside the CLI. It uses protected Windows installation/state locations and a
fixed one-shot task; caller config never grants new administrator authorization.

After enrollment, ordinary CLI/GUI commands use the authenticated installed runner:

```powershell
hyper-gpu-support plan --config my-vms.toml
hyper-gpu-support apply --config my-vms.toml --vm VM-GUID
hyper-gpu-support status --config my-vms.toml
hyper-gpu-support verify --config my-vms.toml --vm VM-GUID
hyper-gpu-support disable --config my-vms.toml --vm VM-GUID
hyper-gpu-gui --config my-vms.toml
```

Guest administrator credentials are prompted. `credentials --config FILE --vm GUID`
explicitly stores an entry in the current user's Windows vault; `forget` deletes it.
No credentials belong in TOML, command arguments or logs. Enable/prepare may gracefully
restart guests automatically and restores initial power state. Disable keeps driver
files. Interrupted preparation retains state; retry reconciles rather than recreating
a VM or replacing disks. No forced power-off or host lifecycle action is exposed.

VRAM defaults are used unless raw provider values are explicitly requested. Neither
raw values nor UI mockup sliders promise GiB allocations or hard enforcement.

## Contributor laboratory

The previous application survives in [tools/lab](tools/lab/README.md). Golden images,
disposable reset, extended D3D/CUDA probes and `config/project.toml` are contributor
infrastructure. Keep its binaries separate from the product and reuse its guards.

Product host artifacts/state use separate `HyperGpuSupportProduct` known-folder
directories. Interrupted installation disables admission and requires rerunning
administrator install; it never kills an active guest operation. Guest runtime
files retain vetted Windows destination read/execute inheritance. Private worker
bundles and state remain restricted to SYSTEM/Administrators.
