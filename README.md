# Hyper GPU Support

Rust GPU-PV management for existing Hyper-V VMs. The approved product uses one
`hyper-gpu-support.exe`: no arguments launch Slint; explicit commands run headlessly;
a restricted internal mode runs an elevated worker per approved operation. These
consolidated modes and Slint are **planned**, not yet implemented. The current build
has separate CLI, Win32 prototype and protected runner binaries using the same Rust
core. Select multiple Generation 2 VMs, one GPU each. NVIDIA preparation
is implemented first; discovery can list other vendors without claiming support.
The historical RTX 5060/Windows 11 baseline proves feasibility, but the rewritten
product still needs changed-boundary hardware qualification. See the
[roadmap](docs/ROADMAP.md), [architecture](docs/ARCHITECTURE.md) and
[current acceptance card](docs/BACKLOG.md#arch-001).

## Build and checks

Use the pinned Rust/MSVC/Windows SDK toolchain from `rust-toolchain.toml`.
The Windows x64 Cargo configuration statically links the C runtime so the guest
worker and graphics probe do not require a separately installed VC runtime.
If overriding `RUSTFLAGS`, retain `-C target-feature=+crt-static`; the quality
script and CI include it with warnings denied. See [Rust linkage](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes).

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

## Current runtime operation (before Slint consolidation)

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

`plan` returns a shared effect preview: ordered attachment/preparation/verification
actions, compatibility settings before/after, requested raw allocation, credential
need, guest downtime, pending recovery and final power. An unchanged running VM
is checked without a restart. To preview disable, set that target's `enabled = false`
in runtime configuration; `disable` still directly executes disable. GUI Apply
fetches a fresh plan and shows the same summary before credentials and execution.
Apply independently rechecks state and enrollment. Plan makes no VM/guest changes,
but writes runner audit records; enabled plans authenticate the full current payload
and should not be used for dashboard polling. Status does not verify guest graphics.

The experimental Win32 dashboard retains one VM's staged change across selection and refresh.
Use **Discard Changes** to clear it, or **Reapply / Update** to stage current driver
preparation for an already enabled VM. Settings provides runner/enrollment guidance
and explicit credential-vault actions. A failed refresh leaves historical rows with
effects disabled. If a VM operation succeeds but saving configuration fails, use
Settings to retry the save; the GUI does not repeat the VM operation for that error.

```powershell
hyper-gpu-support plan --config my-vms.toml
hyper-gpu-support apply --config my-vms.toml --vm VM-GUID
hyper-gpu-support status --config my-vms.toml
hyper-gpu-support verify --config my-vms.toml --vm VM-GUID
hyper-gpu-support disable --config my-vms.toml --vm VM-GUID
```

The optional current prototype is `hyper-gpu-gui --config my-vms.toml`; it is
disposable presentation code, not the product's implementation direction.

Guest administrator credentials are prompted. `credentials --config FILE --vm GUID`
explicitly stores an entry in the current user's Windows vault; `forget` deletes it.
No credentials belong in TOML, command arguments or logs. Enable/prepare may gracefully
restart guests automatically and restores initial power state. Disable keeps driver
files. Interrupted preparation retains state; the current workflow can reconcile
it on a subsequent Apply. The approved architecture instead requires explicit
manual reconciliation before further GPU modifications; that policy is not yet
implemented. No forced power-off or host lifecycle action is exposed.

Current allocation support is provider defaults or an optional raw VRAM triple.
Compute, encode and decode fields are not implemented. Raw values do not promise
GiB allocations or hard enforcement.

## Approved Slint product

Initial release includes physical GPU selection and Min/Optimal/Max for VRAM,
compute, encode and decode, with unsupported/unknown capabilities shown truthfully.
The modular UI uses VM cards, a persistent adjustable split view, one staged VM
draft and fresh Review & Apply through the shared backend. Dashboard discovery/status
stays separate from expensive driver validation/planning.

Per-VM files keyed by Hyper-V GUID will live under
`%ProgramData%\HyperGpuSupport\config\vms\`. Only the restricted elevated worker
writes them after verified success; failed persistence offers save-only recovery.
Enrollment and recovery records remain protected separately. Stale external changes
block Apply/save. One modifying GPU operation runs per host, one GUI per Windows
session; active work defers normal closure. Interrupted startup opens the dashboard
with a persistent warning and blocked modifications, without automatic GPU rollback
or blind retry. See [GUI requirements](docs/GUI_GUIDE.md),
[configuration](docs/CONFIGURATION.md) and [audited gaps](docs/ARCHITECTURE.md#repository-audit--9-october-2026).

## Contributor laboratory

The previous application survives in [tools/lab](tools/lab/README.md). Golden images,
disposable reset, extended D3D/CUDA probes and `config/project.toml` are contributor
infrastructure. Keep its binaries separate from the product and reuse its guards.

Product host artifacts/state use separate `HyperGpuSupportProduct` known-folder
directories. Interrupted installation disables admission and requires rerunning
administrator install; it never kills an active guest operation. Guest runtime
files retain vetted Windows destination read/execute inheritance. Private worker
bundles and state remain restricted to SYSTEM/Administrators.
