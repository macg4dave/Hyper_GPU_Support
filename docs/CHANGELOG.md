# Changelog

Record meaningful completed changes in one short entry per change, with task
IDs and evidence links. Task statuses remain in BACKLOG; partial session notes
remain in its Resume block. Do not duplicate detailed test output here.

## 2026-09-25

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
