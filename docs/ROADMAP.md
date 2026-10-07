# Roadmap: GPU-PV management

Build an understandable Windows application that enables GPU-PV on existing
Hyper-V VMs, then provides GPU on/off and allocation controls through a native GUI.
The NVIDIA RTX 5060 / Windows 11 baseline proves feasibility; it is research
evidence, not an architecture or configuration users must reproduce.
See [architecture](ARCHITECTURE.md) and [backlog](BACKLOG.md#arch-001).

## Product workflow

Host/Hyper-V discovery → VM selection → GPU discovery → GPU-PV configuration →
automatic guest driver preparation → necessary VM settings → verification.

Select multiple existing Generation 2 VMs, one GPU per VM; several VMs may share
the same GPU after qualification. Discover partitionable vendors, implement NVIDIA
preparation first, and add vendors incrementally. GUI example names and operating
systems do not establish support; initial hardware qualification remains Windows 11 x64.

## M1 — Separate product contracts from the laboratory

- Runtime intent contains VM/GPU identities and desired state. Discover driver
  versions, package paths, file counts and hashes at operation time.
- Administrator-protected runner enrollment selects existing VMs without a golden
  image, prescribed disk chain or exact display name.
- Reuse native provider, discovery and rendering primitives. Remove laboratory
  reset and the superseded package-only/CUDA alias path from the production graph.
- Preserve the previous application in the standalone [laboratory](../tools/lab/README.md).
  Production never imports it; its artifacts/installation stay separate.

**Exit:** default builds have no laboratory dependency; runtime discovery and
enrollment work on an existing VM without golden-parent configuration. Independently
review and qualify the rewritten privilege boundary before deployment.

## M2 — Small working GPU-PV core

- Implement discovery, preview, enable/apply, disable, status and health-plus-graphics
  verification through one reusable Rust core and thin CLI.
- Automatically discover, authenticate and prepare the complete current host-driver
  payload. Per-run hashes/receipts are internal integrity data, not operator inputs.
- Use Hyper-V resource defaults first. Preserve CPU/RAM quantities, disks, Secure
  Boot and security devices; change only necessary GPU compatibility settings.
- Gracefully restart guests when needed, restore initial power state and avoid
  restarting a running no-op target. Never force power-off after shutdown failure.
- Reconcile partial preparation and settings through durable state. Disable keeps
  prepared guest files; refresh stale preparation on the next apply.
- Prompt for credentials or explicitly store them in the current user's Windows
  Credential Manager, scoped by selected VM.

**Exit:** one NVIDIA VM completes current preparation, attachment and checked
rendering; reapply and disable pass. Qualify two VMs sharing a GPU before advertising
sharing. Failed default-allocation qualification is a blocker, not permission to
silently substitute the experimental 50% resource profile.

## M3 — Native Windows GUI

- Native controls use the same core for VM listing, GPU selection, per-VM on/off,
  effective status, preparation/verification results and credential prompts.
- Keep management off the UI thread. Display actual failures and pending operations.
- Use the supplied VM-table mockup as a design reference; its OS names, capacities
  and slider ranges are not discovered data or compatibility promises.
- Build the GUI immediately after the working core, before expanding allocation.

**Exit:** GUI/CLI exercise the same qualified operations. Initial controls expose
provider defaults, without a fictional GiB slider. Validate native UI usability and
real management behavior; compilation alone is insufficient.

## M4 — VRAM controls and additional vendors

- Expose supported provider allocation values with effective readback. Translate
  into GiB/percent only when that provider mapping is established for the GPU.
- A requested allocation is not a proven hard VRAM limit or fairness guarantee.
- Add AMD/Intel preparation adapters independently with hardware validation;
  refuse unsupported preparation explicitly.
- Keep encode/decode/compute controls and additional GPUs per VM deferred.

**Exit:** qualified allocation behavior, truthful GUI units and independently
validated preparation for each newly supported vendor. Raw-value validation is
groundwork, not completion of this milestone.

## Development support and deferred work

Golden parents, disposable cloning/reset, test disks, clean-target preparation,
CUDA SDK acquisition and repeated stress/control experiments belong to contributors.
Reuse these tools for affected qualification; do not make laboratory implementation
a product task. Product code must not depend on the harness.

Defer mandatory CUDA, sustained stress, CUDA/D3D interoperability, hard-limit/fairness
guarantees, scheduling, fleet-wide automatic driver updates, VM creation, custom
display/streaming and HCS. GUI and incremental vendor support are planned milestones.

Package the actual product with prerequisites, notices and tested operating/recovery
instructions. Never distribute proprietary drivers, VM disks, OS media or secrets.
Historical hardware results do not qualify rewritten product boundaries.
