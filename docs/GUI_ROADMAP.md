# Slint GUI delivery map

**Updated:** 9 October 2026. An isolated exploratory Slint prototype is implemented
in [`tools/gui-prototype`](../tools/gui-prototype/README.md); product integration remains planned.
[ROADMAP](ROADMAP.md) owns product milestone gates, [BACKLOG](BACKLOG.md) owns task
status/dependencies/acceptance, [GUI_GUIDE](GUI_GUIDE.md) owns approved requirements,
and [SLINT_RULES](SLINT_RULES.md) owns implementation practice. This file maps
slices to those authorities; it does not grant implementation or live-test approval.

## Source audit and reuse

PLAN-001's repository/documentation audit is recorded in
[ARCHITECTURE](ARCHITECTURE.md#repository-audit--9-october-2026). No build, runtime or
hardware checks were performed. Preserve existing Rust backend, parser, native
selection/VRAM setter, shared preview, protected runner, authenticated Named Pipe,
exclusive operation lock, journals, credentials and sound `gui_model` state.
Discard Win32 layout/event code; old GUI test results do not qualify Slint.

## Implementation slices

Numbers correspond to M3 slices, not a mandatory serial chain. The backlog register
owns exact prerequisites. Start with mocks; configuration format/conflict work can
precede secure worker integration, but only that worker may commit machine-wide files.

| Slice / owner | Increment beyond existing code |
|---|---|
| M3.0 / [PLAN-001](BACKLOG.md#plan-001) | Source/docs audit complete in its scoped card. Runtime/API questions are assigned below, not a second broad audit. |
| M3.1 / [GUI-001](BACKLOG.md#gui-001) | Modular Slint shell, Fluent styling, VM cards, persistent adjustable split, horizontal narrow-window overflow, independent vertical scrolling, hybrid details and four mock resource groups. No Hyper-V dependency. Verify APIs/tooling against the selected Slint version during implementation. |
| M3.2 / [APP-001](BACKLOG.md#app-001) | One host executable: no-argument GUI, headless CLI and restricted elevated worker. Reuse models/core; investigate Windows console/subsystem and session activation. Ancillary guest/probe payloads remain bounded. |
| M3.3 / [CFG-001](BACKLOG.md#cfg-001) | Extend parser and expected-content save logic to GUID-keyed ProgramData files, worker-only atomic commits, stale-change checks and durable save-only binding. Preserve separate protected enrollment/recovery and per-user preferences. |
| M3.4 / [GPU-010](BACKLOG.md#gpu-010) | Extend native selected-GPU/VRAM support to VRAM/compute/encode/decode Min/Optimal/Max, typed capabilities and enrollment/reassignment planning. Host partition count is read-only. No fabricated units/defaults/enforcement. |
| M3.5 / [SEC-001](BACKLOG.md#sec-001) | Adapt existing runner/pipe/lock to per-operation elevation in the same executable; preserve artifact/enrollment/replay/ACL safeguards, add reviewed-plan/session binding and bounded progress. Independent changed-boundary review before live deployment. |
| M3.6 / [CORE-028](BACKLOG.md#core-028) | Extend workflow/preview/journals for explicit shutdown consent, real stage events, independent readback, worker saving and manual reconciliation holds. Failed save retries only saving. No automatic GPU rollback or blind retry. |
| M3.7 / [GUI-002](BACKLOG.md#gui-002) | Reuse presentation state and credentials; progressively bind lightweight discovery/status, all GPUs/resource fields, one draft, switching confirmation, fresh Review & Apply and persistent recovery banner. Keep full driver validation/planning off dashboard refresh. |
| M3.8 / [GUI-003](BACKLOG.md#gui-003) | Qualify one GUI per session/activate-existing, normal close deferral, crash/pipe-loss uncertainty, saved preferences, DPI/text/keyboard/accessibility and actual renderer fallback. |
| R1 / [CORE-017](BACKLOG.md#core-017), [DOC-003](BACKLOG.md#doc-003), [GPU-014](BACKLOG.md#gpu-014) | Package/test actual consolidated behavior after M2/M3 acceptance. Reuse historical evidence and qualify changed paths only; no sharing claim without separate qualification. |

CORE-012 owns partial discovery and truthful status/errors. CORE-021 owns residual
cross-interface enrollment/configuration guidance and acceptance, without duplicating
CFG-001, SEC-001 or GUI-002 implementation.

## Remaining technical questions

[GUI_GUIDE section 16](GUI_GUIDE.md#16-remaining-open-implementation-questions--resolve-from-repohost-evidence) is the single
OPEN register. Schema/migration and immutable save-only authorization belong to
CFG-001; lock/worker handshake to SEC-001; provider semantics to GPU-010; reconciliation
to CORE-028; console/packaging to APP-001; renderer/accessibility to GUI-003.
Settle questions when needed by the owning slice, without reopening approved A18–A32.

Mock development never invokes real Hyper-V effects. Future designated-disposable
testing follows AGENTS authorization/identity checks; physical-host lifecycle requires
immediate permission. This documentation reconciliation starts no implementation.
