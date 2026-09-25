# Changelog

Record meaningful completed changes in one short entry per change, with task
IDs and evidence links. Task statuses remain in BACKLOG; partial session notes
remain in its Resume block. Do not duplicate detailed test output here.

## 2026-09-25

- [CORE-001](BACKLOG.md#core-001): added versioned read-only Rust inventory with
  exact RTX/GPU-P correlation, typed availability states, bounded/cancellable
  Windows query transport and deterministic failure/selection coverage. Strict
  build/lint/test/doc checks and independent architecture review passed; positive
  VM/GPU-P behavior remains untested. Evidence: [CORE-001](evidence/CORE-001.md).

- [HV-001](BACKLOG.md#hv-001): inventoried Windows 11 Pro 25H2 build
  `26200.9457`, active Hyper-V facilities and the RTX 5060/616.92 driver and
  GPU-P interface. An explicitly approved administrator read-only rerun found
  zero registered Hyper-V VMs and resolved the initial permission blocker without
  changing the host. Evidence: [target inventory](evidence/HV-001.md).

- [REF-004](BACKLOG.md#ref-004): established an official unmodified Windows 11
  ISO baseline after separating AppSandbox's no-prompt ISO rebuild and VHDX
  provisioning from GPU-PV requirements. Added the ignored reproducible `data/`
  artifact tree and documented the native generalized-parent/differencing-child
  workflow, activation/licensing boundary and external-root contract. Ignore,
  tracked-artifact, link/reference, task and diff checks passed; no host/VM change.

- [DOC-010](BACKLOG.md#doc-010): made routine repository development explicitly
  autonomous, retained scoped approval for privileged/destructive/host-wide work,
  and configured future Codex sessions for workspace writes with on-request
  escalation and dependency network access. Consolidated the detailed policy in
  AGENTS and linked engineering/Copilot guidance to it.

- [DOC-008](BACKLOG.md#doc-008): changed development to an immutable Windows 11
  parent plus disposable differencing child, separated presentation from GPU proof,
  and defined a constrained on-demand privileged Rust runner. Moved the thin Rust
  inventory/config/adapter/staging/assignment/probe slice before the final hardware
  gate while preserving task IDs. Independent review closed dependency, identity
  rotation and runner-ordering issues; documentation consistency checks passed.

- [DOC-009](BACKLOG.md#doc-009): configured a project Sol default and an
  Astra/high read-only reviewer with an explicit-model fallback for clients
  without named-agent selection. Added the implementation handoff and verified
  TOML syntax, links and diff whitespace. Runtime model identity was unavailable;
  the first cross-model milestone run remains to be verified.

- [CORE-019](BACKLOG.md#core-019): added a single Rust CLI/library workspace,
  pinned toolchain and lockfile, help/version and explicit errors, ten passing
  tests, Windows CI and clean-machine development instructions. Formatting,
  strict Clippy, locked build/tests, rustdoc and CLI smoke checks passed locally.
  Recorded the incremental-cache workaround in DEC-010. Hardware gates unchanged.

- [DOC-007](BACKLOG.md#doc-007): aligned all 13 AI instruction/prompt files with
  shared [Rust engineering standards](ENGINEERING.md). Documented narrow language
  exceptions, meaningful testing, strict checks, reproducible tooling and native
  operation cleanup; corrected conflicting architecture/decision guidance and
  required initial tests/PR checks with the first Rust code. Independent review
  and Markdown/task consistency checks passed. Documentation only; no application
  implementation, CI execution or hardware validation.

## 2026-09-24

- [DOC-002](BACKLOG.md#doc-002): expanded the plan to v1.0 across M0-M5 and 45
  permanent task cards, preserving existing IDs. Recorded user-selected essential
  D3D11/D3D12/CUDA and single-guest scope; added research gates, transactional
  configuration/recovery, hardening, qualification and release work. Rechecked
  selected upstream/official sources and completed independent plan review plus
  link/dependency/gate checks. Planning only; no application or hardware changes.
- [DOC-001](BACKLOG.md#doc-001): established five documentation files, one task
  register, linked blockers, bounded context loading and compact handover.
  Migrated FORK_PLAN.md into the owned documents and aligned agent prompts.
- [GPU-001](BACKLOG.md#gpu-001): preserved the pinned AppSandbox GPU-PV source
  map and Windows facility research from the earlier planning pass. Corrected
  project direction to a Windows-native Rust project with selective upstream
  adaptation. Research only; no target GPU capability verified.

## Entry template

```markdown
## YYYY-MM-DD
- [TASK-ID](BACKLOG.md#task-id): <meaningful completed outcome>.
  Evidence: <task result or artifact link>; <material validation limit if relevant>.
```
