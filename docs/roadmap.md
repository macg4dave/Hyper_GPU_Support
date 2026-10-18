# Optimization roadmap

## Current scope

The source optimizations are implemented: bounded work counters, batched protected reads and polling, fewer setup transitions, changed-category allocation writes, prepared payload admission, validated guest-helper reuse, completed-record retention and 16 MiB transfer chunks. Windows-version checks are deferred; setup compatibility warnings are advisory.

The remaining task is [OPT-008 candidate acceptance](backlog.md#opt-008---candidate-acceptance). Automated validation and protected installation have recorded passing results in the [checkpoint](ENGINEERING.md#optimization-validation-checkpoint). Installed user journeys, native measurements and VM/GPU/guest acceptance remain pending. Source reductions do not establish measured speedups.

## Outcomes

| Outcome | Required evidence |
| --- | --- |
| Lower polling cost | Compare provider connections/queries, installed-artifact hashes and process starts. Explicit Refresh must still fetch complete display observations. |
| Fewer setup transitions | Confirm actual start/shutdown counts from Off and Running, including attached/reinstall cases and guarded restoration after failure. No identification-only boot is required. |
| Fewer payload passes | Confirm two complete host passes on eligible prepared paths, including independent final authentication. Explicit CLI `plan` adds a separate read-only pass; Disable remains payload-free. |
| Fewer mutations | Retained adapters receive only differing specified categories; recreated adapters receive all specified categories. Check omitted categories and independent final readback. |
| Less guest transfer | Matching, freshly authenticated helpers skip copying. Qualify 16 MiB chunks for throughput, peak memory, exact tails and slow guests. |
| Safe bounded storage | Confirm completed-read audit retention and exact acknowledged guest-bundle cleanup while unresolved, unrelated and retained evidence survives. |
| Reliable foreground work | Poll cancellation waits for exact backend release; stale results cannot replace current selection, drafts, recovery or terminal outcomes. |

## Constraints

Preserve the existing Slint design, shared Rust core and protected operation architecture. [Security](SECURITY.md) owns authentication, admission and recovery; [architecture](ARCHITECTURE.md) owns component/data flow; [configuration](CONFIGURATION.md) owns persistence.

Effects remain serialized and independently checked. Display caches grant no authority. Read cancellation cannot cancel or replay effects. Save-only publishes the exact verified intent; profile deletion changes app records only. Cleanup requires positive durable completion and exact ownership/scope.

Dependency upgrades, UI redesign, broader guest support and transport replacement are outside this work. Broader transfer optimization needs measurements; the selected 16 MiB constant is already part of the candidate.

## Validation and acceptance

1. Identify the exact candidate revision, working tree and binaries. Reuse automated evidence only for its matching inputs; otherwise run the [non-hardware gate](ENGINEERING.md#validation-and-live-acceptance).
2. Prepare comparable before/after VM states, GPU/driver/payload identities, inventory/file counts, cache conditions, workload and diagnostics settings. Record work counts, whole-operation/stage durations, bytes, power transitions and terminal outcomes, accounting for diagnostic-flush overhead.
3. Under scoped authorization, exercise open/poll/Refresh, first setup from Off and Running, ordinary VRAM/toggle changes, Disable, Verify, repair, enrollment continuation, recovery, save-only and profile deletion. Include cancellation, stale state, changed payload/helper, publication failure and interruption where safely prepared.
4. Independently read saved/current state, final power and remaining recovery. Completed progress stages alone are insufficient.
5. Verify protected installation preserves operator/pair authority, configuration, managed history and unresolved recovery. Investigate safety failures and unexplained workflow regressions.
6. Disclose unrun scenarios, unavailable measurements and regressions. Close acceptance only from actual evidence and the user's decision.

The roadmap does not authorize live launches, guest access, protected writes or VM/GPU effects. Prepare targets, effects and stop conditions for that work. Summarize results in chat and task status; create QA reports only when requested.
