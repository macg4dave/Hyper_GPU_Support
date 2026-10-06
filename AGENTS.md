# Repository Agent Guide

## Project

Ship a Rust-first, config-driven CLI for one configured normal Generation 2
Hyper-V Windows 11 x64 VM and one NVIDIA RTX 5060 8 GB on Windows 11 x64.
Use native Windows/Hyper-V facilities and the
[engineering standards](docs/ENGINEERING.md); do not recreate Hyper-V.

The working baseline is authoritative: sustained Code 0, `nvidia-smi`, D3D11,
D3D12 and CUDA computation passed with the discovered driver/runtime payload.
Reproduce the validated provisioning behavior by dynamically discovering the
complete associated payload for the selected GPU and installed signed host driver.
Do not assume a fixed file count or static runtime list. Native Rust discovery
matches the historical inventory. Finish Rust staging,
settings integration and clean-child reproduction. Our source and
[architecture](docs/ARCHITECTURE.md) specify behavior; no external reference
research or renewed feasibility analysis is required. Keep inventory minimisation
and CUDA LUID/interop compatibility post-v1 unless essential functionality requires
them. HCS-owned-guest work stays paused. No GUI, background service, multi-VM/
multi-GPU orchestration or cross-platform layer is required.

## Product and development boundary

We are past GPU-PV feasibility testing; implement the product for an existing
user-selected Hyper-V VM. Golden images, disposable cloning/reset, test disks
and laboratory setup are contributor infrastructure. Reuse existing helpers when
testing; do not add these capabilities to the production CLI without an explicit
user-facing roadmap requirement. Rust test utilities belong outside the default
production library/application path.

**Test-environment automation may support development but must not become production application functionality unless it is explicitly required by the user-facing product. Production code may be used by test tooling; production code must not depend on test tooling.**

## Read only what the task needs

Normal product operation, installation and recovery must be implemented in Rust.
Launching embedded PowerShell or a `.ps1` from Rust does not satisfy this rule.
The [migration audit](scripts/PRODUCT-MIGRATION.md) maps current debt to
CORE-024/025/026/027; read its affected row when porting or reviewing that boundary.
Preserve working Rust and replace adapters incrementally, with focused tests and
affected baseline qualification before removing production dependencies. Keep
scripts optional for development/manual diagnostics. Do not improve product
PowerShell except for necessary correctness/safety fixes or a bounded comparison
needed for the port. Any retained external interface requires DEC-027's concrete
native-alternative, exact-command, validation, timeout and error/recovery rationale.
Existing adapter decisions are historical context, not blanket v1 exemptions.

| File | Owns / read when |
|---|---|
| [docs/BACKLOG.md](docs/BACKLOG.md) | Scheduled work, task selection, blockers and handover; read only relevant sections |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Current milestone and exit criteria; selecting work or checking a milestone gate |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Validated recipe, component boundaries and operation contracts; read affected sections |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Important choices and rationale; only relevant decision IDs |
| [docs/ENGINEERING.md](docs/ENGINEERING.md) | Authoritative Rust, testing, tooling and documentation rules; read for coding, tests, dependencies or CI |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Concise meaningful completed changes; append on completion, no routine history reread |
| [scripts/README.md](scripts/README.md) | Maintained development/setup/diagnostic script layout; read before adding or running substantial shell procedures |

The backlog register owns status and dependencies; cards own objective,
acceptance and concise results. Correct trivial drift in place without a new
card. Historical evidence is outside normal startup reading. Prompts in
`.github/prompts/` add task-specific guidance only when invoked or relevant.

## Session workflow

1. Read the selected task row/card and relevant source. Read a dependency result,
   architecture section or blocker only when it affects implementation or safety.
   Use Resume/ROADMAP when selecting work or closing a milestone.
2. Check Git state and overlapping ownership before editing. Claim a backlog card
   when work is scheduled/shared or spans a handover. A small direct request, bug
   fix or documentation correction may proceed without inventing a card.
3. Implement the smallest coherent product step. Preserve unrelated work and avoid
   speculative abstractions or refactors. Resolve routine implementation choices
   during coding.
4. Add focused tests for meaningful logic and run checks proportional to the
   change. Normal testing against the designated disposable VM proceeds under the
   authorization below. Update documentation after behavior changes;
   do not make prose, evidence files or cross-document reconciliation a prerequisite
   for ordinary coding.
5. Use independent `architecture_reviewer` review for an implementation milestone
   or materially changed privileged/security boundary. Supply the relevant contract,
   diff and actual tests; fix blocking findings. Routine patches need no review gate.
6. On completion, record a short result on an existing card when one owns the work,
   update only affected architecture/decisions, and add a changelog entry only for
   a meaningful product or process change. Refresh Resume only for a real handover;
   correct trivial tracking drift in place without creating follow-up work.

## Mandatory rules

- Application logic, CLI, configuration, diagnostics, GPU/Hyper-V management and
  supporting utilities belong in Rust. Prefer Rust libraries and `windows` bindings;
  investigate Rust alternatives and record the technical reason in DECISIONS before
  introducing a non-Rust exception. Convenience is not a reason.
  Calling an existing Windows utility does not itself introduce another language.
- Discover values through reliable Windows facilities when they are inventory rather
  than operator intent. Put remaining values expected to change between machines,
  VM/image/driver revisions or test runs in the authoritative `config/project.toml`;
  deserialize and validate once at the boundary, then pass typed values. Scripts
  consume the shared configuration or accept explicit overrides. Keep protocol/API
  constants and fixed safety limits in code; never store secrets in configuration. See
  [configuration policy](docs/CONFIGURATION.md).
- Follow ENGINEERING for focused modules, explicit errors, safe FFI, meaningful
  behavior coverage, formatting, strict Clippy/compiler warnings and documentation.
- Preserve authorship, applicable licenses and third-party notices in reused
  material; record its source revision/path and adaptation rationale. This is a
  provenance obligation, not an external-repository research workflow.
- Report only checks actually run: exact host/guest builds and architecture,
  GPU/driver, API/runtime, workload and outcome when hardware is involved.
  Builds, DLL loading and enumeration do not prove GPU support.
- Do not invent files, commands or passes. Use the README or task result for commands.
- Never commit credentials, signing keys/certificates, secrets, production data,
  ISOs, VM disks or extracted proprietary drivers. Document redistribution
  terms/notices before distributing third-party binaries.
- Do not weaken isolation, signing, Secure Boot, networking or privilege
  boundaries to make a test pass. Flag unsupported capability claims,
  lost attribution, unreviewed privileged actions and unrelated changes in review.
- Follow the [shell and script policy](docs/ENGINEERING.md#shell-commands-and-development-scripts):
  run short commands directly and substantial procedures from meaningful script
  files. PowerShell supports development/Windows operations, not application logic.
- Follow the authoritative [Windows elevation and UAC policy](docs/ENGINEERING.md#windows-elevation-and-uac):
  determine whether an operation needs elevation before executing it; run ordinary
  development normally and send known privileged Hyper-V/GPU-PV work through the
  approved runner immediately, never using a failed unelevated attempt as detection.

## Development and test authorization

The AI may autonomously perform normal project development and approved testing
against the designated disposable VM. This includes building and running project
code; installing, updating and executing the controlled privileged runner;
configuring Hyper-V and GPU-PV; changing GPU resources; starting, stopping,
restarting, resetting or recreating the disposable guest; replacing its
differencing disk and configuration; provisioning and modifying guest files,
registry and NVIDIA components; running PowerShell Direct, probes and diagnostics;
and repeating experiments. Administrative access and non-rebooting host changes
needed for that workflow do not require another permission prompt.

Immediately verify the configured disposable VM, GPU and path identities before
effects. Never experimentally modify the golden parent or an unrelated VM/disk,
and keep destructive actions confined to the verified disposable target. These are
targeting requirements, not additional approval gates.

The sole recurring approval boundary for the agreed workflow is physical-host
lifecycle: never restart or shut down the Windows host, log out or terminate its
interactive session, schedule such an action, or accept an automatic restart
without the user's explicit permission immediately beforehand. If an installer,
feature, driver or update reports that a host restart is required, stop before the
restart and ask. Guest lifecycle is not host lifecycle.

Repository instructions cannot expand the active Codex/VS Code sandbox or
approval policy. Obey platform enforcement and report a configuration blocker
instead of claiming that prompt text bypasses it.

Codex tool/sandbox approval is not Windows UAC elevation. User membership in
Administrators, the current process token and the operation's required privilege
are separate facts. Use the project's controlled privileged runner where applicable;
its administrator-owned executable
and policy pin one disposable slot, the current VM-GUID enrollment, GPU identity,
allowed operations, paths and audit output. Installing, updating, exercising or
removing that runner is normal project testing and may proceed autonomously.
It must not accept caller-selected arbitrary commands or target the golden parent.

Report concrete changes and test outcomes. Do not routinely report that host,
Hyper-V, VM or GPU state was unchanged unless that fact explains a failure or is
material evidence for the task.
