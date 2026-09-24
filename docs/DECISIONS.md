# Decisions

Record rationale here; current component descriptions belong in
[ARCHITECTURE.md](ARCHITECTURE.md). IDs never change or get reused.
Append a replacement decision and mark the old one superseded when direction
changes. Do not turn routine implementation choices into decision records.

## DEC-001

**Accepted | 2026-09-24 | Purpose-built Windows GPU-PV scope**

Windows 11 x64 host and guest; NVIDIA RTX 5060 8 GB; Rust-first;
native Windows/Hyper-V integration; CLI/core independent of any GUI.
AppSandbox supplies technical reference and selective improvements.

Reason: focus effort on graphics, compute, video and supported vendor
capabilities for one actual system. Linux/macOS, ARM, other GPU vendors,
AppSandbox architecture/API compatibility and its full VM feature set are
outside the initial scope. A later GUI should primarily edit configuration.
32-bit Windows application support is a separate workload decision.

Revisit only for an explicit requirement, not to accommodate upstream structure.

## DEC-002

**Accepted investigation direction | 2026-09-24 | Native management first**

Prefer a dedicated persistent Generation 2 VM managed by Hyper-V/VMMS.
Measure the AppSandbox HCS reference against native Hyper-V operations using
matching Windows builds, NVIDIA runtime files and workloads.

Reason: Windows already owns VM lifecycle/storage/integration and GPU-PV.
The backend is not finally selected: edition, available cmdlets and vendor
extension behavior need target evidence. HCS becomes an implementation choice
only for a demonstrated gap, recorded by GPU-006.
See [native boundaries](ARCHITECTURE.md#native-windows-boundaries).

## DEC-003

**Accepted | 2026-09-24 | Reproduce before implementing**

Complete the hardware reproduction milestone before a Rust product skeleton.
Start comparisons with matching unmodified vendor runtimes; add only shims
needed by a failing workload. Keep necessary C/C++ ABI components small and
attributed instead of rewriting them solely to change language.

Reason: source inspection, DLL loading and physical GPU specifications cannot
establish guest behavior. Native management, driver staging, compatibility
hooks and desktop presentation must be evaluated independently.
See the [validation contract](ARCHITECTURE.md#validation-contract).

## DEC-004

**Accepted | 2026-09-24 | Independent history and selective upstream review**

Use independent project history: `origin` is our repository when one is known;
`upstream` is `https://github.com/jamesstringer90/appsandbox.git`.
GitHub fork-network membership is optional. Keep upstream history/authorship
available without importing its working tree or routinely merging it into ours.

REF-001 will establish the local Git relationship. Future reviews:

1. Fetch without merging. Compare last-reviewed and new commits in upstream's
   own history, not a merge-base with our unrelated project history.
2. Inspect the [source map](ARCHITECTURE.md#upstream-reference-map), its callers,
   new/renamed GPU code, build/provisioning dependencies and notices.
3. Add a short review entry below with range, relevant commits, capability,
   adopt/defer/reject result and rationale.
4. Adapt a focused change to our architecture; link its source commit/path.
   Cherry-pick only self-contained compatible code with authorship preserved.
5. Retain a durable ref for reviewed commits and validate affected workloads.

Preserve MIT copyright/license text for adapted AppSandbox material, including
translations to Rust, and applicable third-party notices for included components.
Retain original notices in reference checkouts. Local driver use does not imply
redistribution permission. Do not import proprietary binaries into this project.
[Upstream license][license], [component notices][notices].

### Upstream review log

- 2026-09-24: inspected
  [`6f3adb6aafd4fc819d7715bdfacf52ac87df26a6`][reference]
  (0.1.9 version-bump commit), with no earlier reviewed range.
  Recorded GPU assignment, staging, shim and display responsibilities.
  Adopt the responsibility boundaries; defer code adoption to hardware evidence.
  Full findings: [source map](ARCHITECTURE.md#upstream-reference-map).
  No code imported or target capability verified.

## DEC-005

**Accepted | 2026-09-24 | Five documents with one task register**

Use ROADMAP, BACKLOG, ARCHITECTURE, DECISIONS and CHANGELOG under `docs/`.
Keep blockers, task cards and one overwrite-in-place resume note in BACKLOG,
where they can be read together. A separate BLOCKERS file would duplicate
task relationships and require another context read.

The backlog register owns task status, priority, milestone and dependencies.
Task cards own scope, acceptance and result evidence. Roadmap owns the current
milestone and its exit criteria. Architecture owns current/proposed components
and the validation contract; decisions own rationale; changelog owns concise
completed-change history. Link instead of copying.

Use the compact Markdown register instead of a YAML/JSON/TOML manifest.
It provides an index readable with ordinary search and has no second status
store to synchronize. No tracker service, scripts or generated dashboards.
Revisit if measured lookup cost justifies automation; any generated index must
derive from this single task source.

Keep FORK_PLAN.md only as a migration pointer. Existing optional workflow
prompts remain available, but sessions must not read all of them.
Create evidence files only when a task has real procedures/results to retain.

## Decision template

```markdown
## DEC-<next number>
**Accepted / Proposed / Superseded by DEC-... | YYYY-MM-DD | Title**
Decision: <choice>
Reason: <relevant evidence and tradeoff>
Revisit when: <concrete trigger>
Links: <task, source or affected architecture section>
```

[reference]: https://github.com/jamesstringer90/appsandbox/commit/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6
[license]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/LICENSE
[notices]: https://github.com/jamesstringer90/appsandbox/blob/6f3adb6aafd4fc819d7715bdfacf52ac87df26a6/THIRD-PARTY-NOTICES.md
