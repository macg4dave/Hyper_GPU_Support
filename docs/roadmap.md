# Core simplification roadmap

## Current scope

The GUI is the supported ordinary frontend. Its Configure variants and Verify use the protected prepared-worker exchange. Ordinary CLI dispatch is removed; administrator native-recovery, legacy-installation, import and recorded-outcome commands remain. Their distinct contracts are documented in the [caller review](OPERATION_COMPLEXITY_REVIEW.md#frontend-caller-and-recovery-review).

OPT-009, OPT-010 and OPT-011 source work is complete through the 19 October 2026 repository-wide review. The [architecture review](OPERATION_COMPLEXITY_REVIEW.md) owns findings, decisions, source measurements and verification evidence. OPT-008 installation/live acceptance remains pending. Historical candidate evidence does not qualify changed source or binaries.

Preserve the [agreed workflow](../README.md#agreed-workflow), Slint design, supported features, persistent compatibility and protected boundaries. Source size is descriptive, with no numerical reduction target. Further changes require demonstrated maintenance benefit and behavior evidence; module size and repeated-looking checks alone do not justify replacement.

## Current architecture

| Responsibility | Owner and contract |
| --- | --- |
| Presentation and input | Slint components and `gui/live/`; capture the exact target/draft, deliver progress and return terminal results on the event loop |
| Observation, intent and editing | `gui/live_state/`, `gui_model`, `configuration_store`; current observation, committed settings and editable draft remain different facts |
| Read coordination | `gui/live/polling.rs`, `read_context`, authenticated runner observations; bounded cancellation, stale-delivery suppression and exact backend release |
| Configure decisions and sequence | `workflow/{planning,steps,apply}`; first setup, re-enable/VRAM, Disable and Reinstall consume one worker-owned typed sequence |
| Verify and restoration | `workflow/{verification,guest_check,journal}`; check current files/device/rendering, restore only permitted power and never publish GPU intent |
| Protected execution | `worker/{preparation,dispatch,operations,progress}`; authenticate, lock, prepare, consume private admission, persist receipts, execute, confirm and publish only as allowed |
| Native and guest adapters | `windows_*`, runner backend, guest/payload/transfer/trust and probe binaries; bounded native work, signed files, current guest-file parity and independent rendering |
| Durable recovery | Workflow receipts, native provider-job evidence, pairing and import records; domain-specific recovery never replays Configure |
| Local credentials and diagnostics | `credentials`, reporting/diagnostics and GUI presenters; explicit vault opt-in, cancellation and secret-free operator errors |
| Build and installation | Catalogue generator/build script and `app-install`; fixed artifacts and explicit clean reinstall, separate from ordinary launch and source validation |

### Execution and ordering

```text
captured GUI request
  -> authenticate worker and acquire common lock
  -> read fresh facts and prepare exact scope
  -> acknowledge current target/revision; acquire credentials only if required
  -> consume private admission token
  -> persist receipt before effects
  -> execute guarded steps with independent readback
  -> independently confirm final VM/GPU/payload
  -> publish exact intent only for Configure/save-only
  -> terminal outcome and protected completion observation
  -> merge frontend state and present the domain-specific result
```

Preview and execution share `workflow/steps.rs`. Steps are fixed domain actions, not arbitrary commands, frontend instructions or a persisted workflow language. Restoration follows recorded scope and fresh facts; it does not replay or reverse an interrupted step list. Disable remains credential-free; eligible ordinary changes avoid unnecessary guest setup. Verify cannot restart a guest stopped externally.

Admission, per-effect guards, final confirmation and publication check different boundaries. Retain them. Current authenticated source handles do not replace final package-membership checks; historical preparation/rendering receipts do not establish current guest health.

### Foreground completion and recovery ownership

Configure, save-only, Verify and reconciliation use `gui/live/state_access.rs::complete_operation`. It receives a captured publication target where applicable, an authenticated terminal result and an independent completion bundle. A failed completion read preserves the original operation failure and blocks new execution; successful observation cannot manufacture operation success.

Partial removal reuses `with_completion_state` while resetting a clean editor from saved intent. The other non-publication completions preserve displayed inputs. Enrollment and profile deletion keep domain-specific completion and fresh subsequent observations. A generic callback/controller framework is not justified by these differences.

Recovery receipts have one collection owner in `State`. Selected access is derived by VM GUID; selection reconciles unsaved presentation without cloning the collection or shared recovery records. An unrelated receipt may explain a banner but cannot supply save-only or partial-removal input. Current observation, saved intent, draft, captured request and progress remain separate. Poll generations and foreground admission prevent obsolete delivery.

Status-reader release refreshes the exact Task Scheduler instance returned by launch before reading its state. A cached Running value must not leave admission held after that instance exits. Only a confirmed completed instance or the scheduler's explicit no-running-instance result releases admission; cancellation, unknown state and observation errors do not.

### Consolidation decisions

| Area | Implemented simplification or retained boundary |
| --- | --- |
| Frontend dispatch and request plumbing | Remove ordinary CLI dispatch, obsolete direct-plan paths, unused review/source parameters and one forwarding wrapper |
| Workflow and worker lifecycle | Share Configure steps and Apply/Verify receipt/effect/final-confirmation mechanics |
| Frontend state | Use protected completion bundles directly; remove redundant preparation stores and the cloned selected recovery receipt |
| Records and identity | Use typed equality, one GUID grammar and `security::read_optional`; retain domain decoding, exact revisions and publication rules |
| Guest progress and credentials | Share progress mechanics; let `credentials::prompt_named` own lookup, explanation, prompt and cancellation |
| Native/guest/security | Retain peer/process authentication, provider-job holds, trust, current-file parity, rendering and release checks; no safe deletion established |
| Compatibility | Retain public runner values used by stored audits, exported compatibility hooks and existing locale support; no format migration |
| Tests and dependencies | Adapt fixtures to current state ownership and retain behavior/failure coverage; no dependency removal or test deletion justified in this follow-up |

Current and historical source counts are recorded once in the architecture review. Product Rust, tests, Slint, guest bridge, tools and configuration are separate; generated/vendor/upstream source is excluded. No business logic was relocated to improve measurements.

## Delivery status

1. **OPT-009 - Review the simplification design: source complete.** Repository-wide caller, requirement, dependency and failure-path review is recorded with limitations.
2. **OPT-010 - Simplify operation coordination: source complete.** Supported operations already converge on shared execution. Receipt ownership and unused request plumbing are simplified; distinct record/presentation contracts remain explicit.
3. **OPT-011 - Consolidate demonstrated duplication: source complete.** Shared record reads, comparisons, identity, progress and credentials reuse existing owners. No additional native/guest/security replacement is established by this review.
4. **OPT-008 - Accept the candidate: pending installation/live evidence.** Prepare exact binaries, targets, states, allowed effects and stop conditions before obtaining separate authorization. Passing source checks do not close this gate.

## Validation and acceptance

The old `scripts/testing/check.ps1` gate is removed. Current non-hardware validation uses the commands recorded in the architecture review: formatting, strict workspace Clippy, all-target workspace tests, binary builds, API documentation and the existing documentation validator. Inspect test effects first; tests use in-memory backends, isolated temporary files/local processes and headless UI, without live product or VM actions. Preserve meaningful assertions rather than production layers retained solely for tests.

Retained behavior/failure coverage includes setup from Off/Running, ordinary changes, omitted categories, Disable, Reinstall, Verify/restoration, stale requests, payload/driver drift, cancellation/release, effect/read failures, lost replies, exact publication, verified-unsaved/save-only, independent VM holds and resumable record operations. Native/guest/hardware equivalence still requires separate live evidence; no speedup is inferred from source counts.

For authorized installation, follow the [clean-reinstall contract](../README.md#install): refuse active work, preserve protected evidence in the archive, reset app-owned state through `app-install`, verify artifacts and fresh enrollment, and keep VM/guest/data-root state separate. Do not carry old enrollment/preferences forward as an acceptance requirement. No install or reset occurs during this source task.

Installed acceptance must independently confirm terminal outcomes, saved/current readback, final recovery/power state and record handling for the exact candidate. Transfer qualification uses the configured 8 MiB chunk size and measures counts, timings, bytes, peak memory, exact tails and slow/no-progress failures. Retain interrupted evidence and never retry effects for a better measurement. Summarize evidence in chat; no new QA reports are required.
