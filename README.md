# Hyper GPU Support

Rust GPU-PV management for existing Hyper-V VMs. **The approved completed Slint
prototype is now the main GUI:** run `hyper-gpu-support.exe` without arguments.
Explicit CLI commands remain headless and retain the shared Rust backend.
The GUI currently uses labelled sample inventory and mock operations; it does
not call Hyper-V, collect credentials or save real configuration.

v1.0 prioritizes connecting its existing controls and packaging/validation, without
redesign or extra pages. The protected runner and guest/probe payloads remain
separate binaries; restricted same-executable worker consolidation is still planned.
The main binary currently retains the console subsystem for reliable CLI output;
GUI console packaging is pending APP-001. See [GUI scope](docs/GUI_GUIDE.md),
[roadmap](docs/ROADMAP.md) and [architecture](docs/ARCHITECTURE.md).

## Build and checks

To launch the main GUI sample workspace without Hyper-V or elevation:

```powershell
cargo run --locked
```

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

## Supported CLI and backend operation

In an administrator console, read native inventory before initial enrollment:

```powershell
hyper-gpu-support inventory
hyper-gpu-support install --config my-vms.toml
```

Use [config/product.example.toml](config/product.example.toml), replacing its example
identities with actual discovery. Install requires the runner, guest worker and D3D11
probe beside the CLI. It uses protected Windows installation/state locations and a
fixed one-shot task; caller config never grants new administrator authorization.

After enrollment, CLI commands use the authenticated installed runner:

`plan` returns a shared effect preview: ordered attachment/preparation/verification
actions, compatibility settings before/after, requested raw allocation, credential
need, guest downtime, pending recovery and final power. An unchanged running VM
is checked without a restart. To preview disable, set that target's `enabled = false`
in runtime configuration; `disable` still directly executes disable. GUI Apply
fetches a fresh plan and shows the same summary before credentials and execution.
Apply independently rechecks state and enrollment. Plan makes no VM/guest changes,
but writes runner audit records; enabled plans authenticate the full current payload
and should not be used for dashboard polling. Status does not verify guest graphics.

The Slint dashboard preserves one in-memory sample draft; all its operation results are simulated.

```powershell
hyper-gpu-support plan --config my-vms.toml
hyper-gpu-support apply --config my-vms.toml --vm VM-GUID
hyper-gpu-support status --config my-vms.toml
hyper-gpu-support verify --config my-vms.toml --vm VM-GUID
hyper-gpu-support disable --config my-vms.toml --vm VM-GUID
```

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

## Main Slint application

Sources are in `ui/` and `src/gui/`; the former prototype package is retired.
Dashboard, System, Settings and About retain their completed design. All inventory,
GPU/provider values, Apply/Verify/enrollment, progress/recovery, credentials and
saves remain mocked. Editing, navigation, dialogs, scrolling, themes and draft
prompts work. GPU Memory is a visual mock preference, not a physical GB allocation;
its real provider mapping is unresolved. Advanced fields use illustrative units.

Existing backend functionality will be bound through existing controls only.
Per-VM persistence, same-executable worker, real progress/recovery and provider
allocation gaps are scheduled on existing cards; they are not implemented by
this promotion. No Activity, VM search/filter, extra wizard or diagnostic history.

Preview the moved components with the existing Slint 1.18.1 viewer:

```powershell
slint-viewer --check ui/app.slint
slint-viewer --auto-reload ui/app.slint
```

For development-only runtime inspection, enable `slint/mcp` on the command line
and set `SLINT_EMIT_DEBUG_INFO=1` during build and `SLINT_MCP_PORT` during launch.
Never enable this inspection server in packaged production builds. Slint licensing
and dependency notices must be reviewed before distribution; icon attribution is
preserved in [ui/icons](ui/icons/README.md).

## Contributor laboratory

The previous application survives in [tools/lab](tools/lab/README.md). Golden images,
disposable reset, extended D3D/CUDA probes and `config/project.toml` are contributor
infrastructure. Keep its binaries separate from the product and reuse its guards.

Product host artifacts/state use separate `HyperGpuSupportProduct` known-folder
directories. Interrupted installation disables admission and requires rerunning
administrator install; it never kills an active guest operation. Guest runtime
files retain vetted Windows destination read/execute inheritance. Private worker
bundles and state remain restricted to SYSTEM/Administrators.
