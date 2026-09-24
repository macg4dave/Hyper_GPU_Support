# Roadmap

**Current milestone: M0.** Change this pointer only when the milestone's exit
criteria are met. Task status lives only in the [backlog register](BACKLOG.md#task-register);
there are no copied counts or percentages here.

The objective is reliable Windows GPU-PV on a Windows 11 x64 host and guest
using an NVIDIA RTX 5060 8 GB. Reproduce the reference behavior before building
a minimal Rust implementation, then improve measured capability gaps.
Scope decisions are in [DEC-001](DECISIONS.md#dec-001).

## M0

**Objective:** establish the working documentation, upstream reference and
source-grounded baseline procedure for the actual machine.

- Tasks: DOC-001, GPU-001, HV-001, REF-001, GPU-002, GPU-003.
- Dependencies: none.
- Exit: pinned responsibility map; read-only environment inventory; selected
  driver/runtime manifest and provenance; exact baseline artifact, workloads,
  named targets and recovery procedure.
- Evidence: task result links in the backlog. The source map alone does not
  complete this milestone. Hardware tests are still ahead.

## M1

**Objective:** reproduce AppSandbox's GPU-PV functionality on the target system
using the smallest justified set of native Windows facilities and guest components.

- Tasks: HV-002, GPU-004, GPU-005, GPU-006.
- Dependency: M0.
- Exit: actual accelerated guest workload results, comparable upstream/native
  capability evidence, repeat starts and verified recovery, and a justified
  component/backend choice.
- Every probed capability passing in the upstream baseline must be reproduced.
  Record unresolved gaps as partial reproduction; they keep M1 open.
  Device enumeration or a desktop alone is insufficient.
- A failed reference baseline is useful evidence, not successful reproduction.
  Resolve the failure or explicitly revise the milestone through a recorded
  decision before beginning product implementation.

## M2

**Objective:** encode the proven procedure in a small Rust CLI and reusable core.

- Tasks: CORE-001, CORE-002, CORE-003.
- Dependency: M1.
- Exit: GUI-independent inventory, explicit GPU selection, validated assignment,
  repeatable driver preparation/recovery, useful failure reporting, and the
  same hardware probes passing through the core.
- Implementation and actual build commands are introduced by these tasks;
  no Cargo project, daemon or general VM manager is required by this setup.

## M3

**Objective:** improve a demonstrated limitation in graphics, compute, video,
interop, stability or performance.

- Task: GPU-007; create further permanent IDs only for concrete follow-up work.
- Dependency: M2.
- Exit for each improvement: named workload, before/after evidence, regression
  results for passing capabilities, updated limitations and recovery guidance.
- Candidates after evidence exists: Optical Flow/OptiX and other vendor APIs,
  driver/Windows update handling, VRAM pressure, multiple guests, or display
  improvements. These are not promised capabilities or immediate infrastructure.

## Milestone template

When extending the roadmap, copy only this shape:

```markdown
## M<N>
**Objective:** <outcome>
- Tasks: <permanent IDs>
- Dependency: <milestone or none>
- Exit: <observable completion criteria>
- Evidence: <task result pointers; do not repeat task status>
```
