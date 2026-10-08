# Decisions

Current rationale belongs here; implementation contracts are in
[ARCHITECTURE.md](ARCHITECTURE.md) and work/status in [BACKLOG.md](BACKLOG.md).
Decision IDs remain stable and are not reused. Superseded investigation is condensed;
routine implementation choices do not need a decision record.

## DEC-001

**Accepted, narrowed by DEC-026 | 2026-09-24 | Product scope**

Rust-first CLI/core, TOML configuration, Windows 11 x64 host/guest, normal Generation
2 Hyper-V and one configured RTX 5060 8 GB target. No GUI, multi-VM scheduler,
multi-GPU orchestration or background GPU service is required. Windows owns VM
and GPU-PV facilities. Expand scope only for an explicit product requirement.

## DEC-002

**Superseded by DEC-024/025/026 | 2026-09-24 | Backend exploration retired**

Normal Hyper-V / VMMS is proven and chosen. HCS backend selection and broad
feasibility comparisons have no remaining delivery dependency.

## DEC-003

**Superseded by DEC-011/025/026 | 2026-09-24 | Reproduction-before-code gate retired**

The working baseline settles feasibility. Implement and test the remaining Rust
product slices directly; fresh reproduction qualifies automation rather than
reopening whether GPU-PV works.

## DEC-004

**Reference-review workflow retired by DEC-026 | 2026-09-24**

The initial recipe was developed during reference research, then measured on this
project's target system. Future implementation uses our source, recipe and
validation results. No external repository lookup, comparison or periodic review
is required. Preserve existing authorship/license notices for any reused material;
historical origin does not create an active research obligation.

## DEC-005

**Accepted, simplified by DEC-026 | 2026-09-24 | One task register**

ROADMAP owns milestones/exit criteria; BACKLOG owns task status, dependencies,
cards, actual blockers and the latest handover; ARCHITECTURE owns current behavior;
DECISIONS owns rationale; CHANGELOG owns concise meaningful completed changes.
ENGINEERING owns shared engineering rules. Link instead of duplicating stores.
Update affected documentation after ordinary implementation/testing, without
creating reconciliation tasks or evidence gates for trivial corrections.

## DEC-006

**Accepted | 2026-09-24 | Essential workloads**

v1 requires D3D11 and D3D12 checked hardware rendering plus checked CUDA computation
on the configured guest. Those workloads already passed experimentally. Optional
APIs, video codecs, vendor features and interoperability do not block release
unless an essential function demonstrably needs them. Extra claims need their
own measured workload.

## DEC-007

**Superseded by DEC-025/026 | 2026-09-24 | Feasibility contingencies retired**

Diagnose concrete failures against our complete validated recipe. Do not restore
backend/reference investigations because an integration test fails.

## DEC-008

**Owner decision before distribution | 2026-09-24 | Licensing and delivery**

The owner selects the project license and distribution channel when preparing
release delivery. Source delivery, portable binaries and publisher signing have
different prerequisites; no current choice is implied. Local candidate packaging
can proceed without public distribution. An installer is optional.

Review actual dependency/probe/runtime terms and preserve applicable copyright,
license and third-party notices, including source translations. Local provisioning
does not grant redistribution rights. Drivers, OS images, disks, credentials and
signing keys are excluded from release payloads. Retain Secure Boot/signing
boundaries; unresolved essential rights block distribution, not unrelated development.

## DEC-009

**Accepted | 2026-09-24 | Rust and native Windows**

Application logic, CLI, configuration, diagnostics and supporting utilities belong
in Rust wherever technically possible. Existing Windows utilities can provide
bounded native interfaces; ABI language or convenience does not justify non-Rust
application logic. Investigate Rust alternatives and record a technical exception
before introducing another language. [ENGINEERING.md](ENGINEERING.md) owns the
detailed language, quality, test and FFI rules.

## DEC-010

**Implemented foundation; sequencing superseded by DEC-026 | 2026-09-25**

Keep one root package/workspace, edition 2024, resolver 3 and the pinned Windows
x64 MSVC toolchain. Current inputs/commands belong in repository configuration and
README. Development incremental compilation remains disabled after observed
Windows cache-finalization failures; revisit only if build cost warrants it.
Do not add scaffolding abstractions without behavior needing them.

## DEC-011

**Accepted, refined by DEC-025/026 | 2026-09-25 | Disposable target and elevation**

Use one immutable clean parent and disposable differencing child. Recover uncertain
guest state by recreating the child, not a general guest-file rollback engine.
Native lifecycle/assignment and bounded privileged operations remain project
testing. GPU computation and operator desktop presentation are separate; a custom
display device/transport is outside v1 unless essential workload evidence requires it.

## DEC-012

**Accepted | 2026-09-25 | Effect-based development authorization**

[AGENTS.md](../AGENTS.md#development-and-test-authorization) owns authorization;
[ENGINEERING.md](ENGINEERING.md#windows-elevation-and-uac) owns elevation paths.
Normal development and approved disposable-target testing proceed autonomously
after identity checks. Physical-host lifecycle requires explicit permission
immediately beforehand. Repository text never overrides platform enforcement,
and Codex tool approval never supplies a Windows elevated token.

## DEC-013

**Implemented | 2026-09-25 | Bounded inventory query process**

The initial inventory adapter uses a fixed parameter-free Windows query process.
Rust owns deadlines, bounded output, termination/reaping, protocol validation,
target selection and reporting. Synchronous in-process provider calls did not
provide that cancellation boundary without additional FFI. This records the
initial transport, not a v1 PowerShell exemption. CORE-024 replaced the inventory
script on 2026-10-06 with a fixed native Rust worker, retaining a process deadline,
suspended launch/job containment and an independent worker watchdog for synchronous
COM/provider calls. Native registry/system APIs supply host facts and the existing
WMI bindings supply GPU/VM facts. Read-only parity passed; no external inventory
interface exception remains. Native full-driver discovery is already implemented.

## DEC-014

**Accepted | 2026-09-26 | Reviewable development procedures**

Short commands run directly; substantial procedures live in meaningful scripts.
Maintained tooling uses `scripts/`; temporary task procedures use ignored
`local/scripts/`. Application behavior remains Rust-owned. Host restart,
shutdown, logout or session termination is never implicit in a script or test.

## DEC-015

**Accepted | 2026-09-26 | Fixed VM identity**

Reset only the child of the configured persistent VM shell, retaining VM GUID,
firmware/vTPM, MAC and guest identity. The completed un-generalized parent is used
only to restore that same virtual machine, never to create a separately registered
clone. Another VM identity, domain-joined clone or portable image requires separately
preparing/generalizing a parent. Target identities belong in TOML and enrollment,
not duplicated decision prose.

## DEC-016

**Implemented | 2026-09-27 | Fixed native-management runner**

The administrator-installed Rust runner exposes compiled policy-listed operations
for the enrolled slot. Fixed Windows cmdlet bodies provide measured VMMS management
and supervised cancellation without recreating Hyper-V. Rust owns authentication,
exact policy verification, locking, request validation and audit/results. Callers
cannot supply commands, scripts, targets or paths. Wider command/target surfaces
require security review; normal installed-runner updates/testing are authorized.
CORE-025 replaces the cmdlet bodies incrementally while retaining this fixed
Rust trust boundary. Measured behavior and cancellation needs do not establish
that Hyper-V lacks a practical native management interface.

## DEC-017

**Implemented | 2026-09-28 | Exact-SID account-right changes**

Runner setup/recovery uses native LSA add/remove calls for only the fixed batch/
deny-logon right delta on its enrolled SID, verifying unrelated rights remain.
Recovery deletes the persisted SID rather than trusting an account name.
Replacing entire local/domain privilege membership lists would risk unrelated
state and is not the contract.

## DEC-018

**Implemented | 2026-09-28 | Mutual named-pipe identity**

The client checks the pipe's enrolled owner SID before sending its bounded request.
The runner reads the frame, identifies the client through pipe impersonation,
reverts and checks the enrolled client SID before dispatch. Client rights exclude
pipe-instance creation. Cross-account process-token access and additional
impersonation privilege are unnecessary; remote clients and pre-authentication
effects are rejected.

## DEC-019

**Implemented, operator cleanup remains CORE-021 | 2026-09-30 | Typed configuration**

Use `config/project.toml` as the sole hand-edited mutable, non-secret configuration.
Validate at the boundary and pass typed values. Discover reliable Windows inventory.
Generated policy and artifact hashes are integrity artifacts, not independent
settings; retain installed privileged enrollment as a distinct trust boundary.
[CONFIGURATION.md](CONFIGURATION.md) records current embedding and schema gaps.

## DEC-020

**Implemented | 2026-10-02 | Bounded PowerShell Direct transport**

Existing PowerShell Direct provides supported registered-VM sessions/file transfer;
reimplementing its remoting internals would enlarge the security surface. This is
the historical implementation rationale, not a blanket v1 exception. CORE-026
moves substantive guest file/security/hash/receipt behavior into Rust and evaluates
the smallest supported bootstrap/transfer/launch interface. Any retained command
needs the specific limitation/alternatives/validation/error record below. Rust owns
target/source/path/hash validation, structured requests and process supervision.
Load/verify protected system modules before accepting credentials, keep credentials
off command lines/reports, and zero password buffers. Killing the host process
does not establish guest-copy cancellation; interruption makes guest state uncertain.

CORE-003 reuses this transport only to verify/copy the fixed validation inputs and
launch one Rust worker in the enrolled guest. Native Rust WMI, clocks, bounded
processes, kill jobs and parsers own readiness and workload decisions. The remote
launch watchdog is minimal transport glue; a worker-owned hard deadline independently
terminates blocked calls after remoting loss. Existing/copy destination ACLs are
checked before elevated execution. Workload processes start suspended and enter
their kill job before any instructions can create descendants. No general guest
command API or additional implementation language is introduced.

## DEC-027

**Accepted | 2026-10-06 | Native Rust production path**

The proven baseline is the implementation specification. Functionality required
for normal CLI operation, installation or recovery belongs in Rust, including
Windows management and guest writer behavior. Embedded PowerShell is production
debt just as an external `.ps1` dependency is. Reuse working Rust logic; replace
adapters incrementally, test equivalent behavior and qualify changed boundaries
before removing them. Development/manual diagnostic scripts remain optional.

CORE-024/025/026/027 own the remaining native replacements; CORE-006 and the existing
operation/configuration cards integrate one CLI. The
[maintained-script/backend audit](../scripts/PRODUCT-MIGRATION.md) records ownership.
DEC-013/016/020 retain historical rationale without exempting those adapters from v1.

Before accepting any remaining external interface, record the missing capability,
investigated Rust/Win32/COM/WMI alternatives and why they are impractical, exact
executable/arguments, typed input/result checks, bounds, error propagation and
uncertain-state recovery. Convenience or a process cancellation boundary alone
is insufficient: a bounded Rust native worker is an alternative. No such new
exception is established by this audit. Preserve enrollment, isolation, parent
protection, credentials and physical-host lifecycle policy throughout migration.

## DEC-021

**Package implementation retained; full recipe superseded by DEC-025 | 2026-10-03**

The current writer verifies/atomically publishes its package, CUDA alias and receipt.
It is a bounded staging implementation, not the complete provisioning contract.
Keep integrity, partial-state and verified-no-op principles while extending the
writer to every discovered environment destination. Do not retain package-only
assumptions as full-driver requirements.

## DEC-022

**Accepted | 2026-10-04 | Windows build drift is qualification evidence**

Record configured qualification, measured host and measured guest builds. A build
difference is a visible warning, not automatic refusal. The working baseline used
different host/guest builds. Driver/package identity, signatures, stable servicing
and checked capabilities remain mandatory; changed combinations need affected
workload validation before a new compatibility claim.

## DEC-023

**Accepted | 2026-10-04 | Reviewed cleanup versus active servicing**

Only exact configured raw pending-delete source/empty-destination records may be
reported as reviewed cleanup warnings. Active servicing, unknown deletions,
malformed records and rename/replacement destinations fail before mutation.
Delete-only does not mean harmless; never clear Windows servicing state to pass.

## DEC-024

**Product boundary retained; reference policy superseded by DEC-026 | 2026-10-05**

The product uses a normal Generation 2 Windows 11 Hyper-V VM. HCS ownership and
vendor-extension investigations are outside the delivery plan. Changing this
boundary requires an explicit user scope change.

## DEC-025

**Accepted | 2026-10-05 | Complete measured recipe is authoritative**

Sustained Code 0, `nvidia-smi`, checked D3D11/D3D12 and CUDA computation passed.
Use the complete ordinary-copy environment and measured VM/resource settings in
[ARCHITECTURE.md](ARCHITECTURE.md#working-recipe). Native Rust discovery matches
the measured source/destination/length/hash records; the older primary-package
and manually selected alias subset was incomplete. Reproduce the discovery and
placement behavior dynamically for the currently selected signed host driver,
including associated external files. No historical count or static list defines
the payload; rediscover and regenerate manifest identity after driver changes.

Files and settings changed together. Reproduce the supported complete recipe
without claiming isolated necessity or minimizing it before v1. CUDA computation
passed despite the identity companion's host/guest LUID mismatch. Correlation/
interop is separate post-v1 work; essential target selection and correctness remain
release requirements. [Baseline evidence](evidence/GPU-PV-BASELINE.md).

## DEC-026

**Accepted by user | 2026-10-05 | Engineering delivery replaces investigation**

Use our implementation and measured behavior as the specification. Remove active
external-reference lookup, comparison and feasibility tasks. Preserve useful
historical evidence outside the normal task-reading path, and applicable reuse
notices without imposing a continuing research workflow.

The delivery milestones are Rust clean-child reproduction, usable daily operation
and maintenance, then release. Complete bounded guest writing, recipe settings and
probe integration as independent useful slices; compose them into reproduction.
Then integrate CLI/config, lifecycle/recovery, diagnostics and full-manifest
restaging, followed by bounded repeatability and candidate release rehearsal.

Do not make minimum-file/resource optimization, optional APIs, CUDA/D3D interop,
actual host driver upgrades, physical-host reboots or new platform abstractions
release prerequisites. Keep identity, destructive-target, privilege, signature and
host-lifecycle boundaries. A materially widened privileged boundary needs focused
review; routine changes use implement → test → update the existing card.

## New decisions

Add a stable new ID only for a changed architectural/public/security contract,
with choice, reason and concrete revisit condition. A task, code comment or concise
correction is enough for ordinary implementation details.

## DEC-028

**Accepted | 2026-10-07 | Runtime management core and narrow guest transport**

The user explicitly approved the architectural rebase: existing VM/GPU runtime
selection, multiple managed VMs, incremental vendors, native GUI after the core,
provider-default resources first and truthful later VRAM controls. Laboratory
identities, fixed driver pins, reset and extended probes leave the product graph.
Prior one-slot/no-GUI sequencing and adapter-preservation rules are superseded for
this redesign; historical results remain evidence, not completion of new gates.

The user chose Rust guest logic with a narrow PowerShell Direct bridge rather than
requiring a new native transport before automatic preparation. Existing local
WMI/COM controls Hyper-V but supplies no established equivalent of New-PSSession
VMId / Copy-Item ToSession. Hyper-V sockets require a guest listener/setup protocol;
reimplementing PSRP or introducing that guest-agent platform expands this core.
Retain inbox Windows PowerShell solely for registered-VM session creation, transfer,
protected bootstrap integrity and fixed worker launch. This is the concrete exception
to DEC-027, not permission for application decisions in scripts.

Exact invocation: discovered inbox powershell.exe with -NoLogo -NoProfile
-NonInteractive -Command and the fixed `src/guest_transport.ps1` text. Pin module
loading to the protected absolute inbox Hyper-V module directory; the native loader
resolves installed versioned manifests. Pass typed input/credentials over
stdin, never command arguments. Bootstrap protects paths/ownership and verifies
fixed worker hashes before launch; Rust owns driver membership, destination mapping,
copy/reapply logic, receipt validation and hardware checking. Supervised host process
and guest-worker deadlines bound transport/worker lifetimes; failures retain pending
state and never recreate disks. No source executable or arbitrary script is accepted
from a caller. Artifact hashes are installed integrity data, not driver intent.

Revisit this bridge if a supported native registered-VM session/transfer API becomes
available. Do not introduce a guest-agent/remoting framework merely to remove glue.
The installed runner is a fixed one-shot task, not a resident service. Its changed
security boundary must pass independent review before deployment. Current hardware
qualification, provider defaults and two-VM sharing remain pending.

Product host artifacts/state use separate `HyperGpuSupportProduct` known-folder
directories. Interrupted installation disables admission and requires rerunning
administrator install; it never kills an active guest operation. Guest runtime
files retain vetted Windows destination read/execute inheritance. Private worker
bundles and state remain restricted to SYSTEM/Administrators.

The selected installed driver includes Windows-generated `.PNF` caches which are
not signed catalog members. Accept only protected native Windows DriverStore cache
sources with a same-stem INF authenticated in the operation manifest. Their bytes
rely on installed-Windows provenance and per-operation integrity. This exception
does not extend to unrelated unsigned files or executable/runtime payloads.
