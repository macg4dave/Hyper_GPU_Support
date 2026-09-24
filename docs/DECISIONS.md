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
needed by a failing workload. The earlier preference to retain C/C++ shims is
superseded by [DEC-009](#dec-009); the reproduce-before-product gate remains.

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

REF-001 will establish the remaining local upstream Git relationship.
The DOC-002 workspace check found local commit `dd95e58` and an existing origin
at `https://github.com/macg4dave/Hyper_GPU_Support.git`; upstream/ref preservation
remains REF-001 work. Do not reinitialize or replace the existing history/remotes.

Future reviews:

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
- 2026-09-24, DOC-002: selectively rechecked the **same pinned commit**, not a new
  revision range. Confirmed HCS fallback/assignment-status behavior and inspected
  provisioning/copy errors, ACLs, runtime and compute-hook dependencies, project
  build inputs, signing branches and notices. Defer adoption until hardware proof;
  reject default-GPU fallback, masked failures and broad permission changes.
  See [hazards](ARCHITECTURE.md#reference-implementation-hazards). No build/run.

## DEC-005

**Accepted | 2026-09-24 | Five documents with one task register**

The five planning-document responsibilities below remain; [DEC-009](#dec-009)
adds a separate engineering policy without introducing another planning store.

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

FORK_PLAN.md was a migration pointer, never a planning authority. Its later
user deletion is preserved; the pointer need not be restored. Existing optional
workflow prompts remain available, but sessions must not read all of them.
Create evidence files only when a task has real procedures/results to retain.

## DEC-006

**Accepted by user | 2026-09-24 | Essential v1.0 workloads and guest count**

Essential workloads are D3D11 and D3D12 hardware graphics plus CUDA compute on
one Windows 11 x64 guest. Concurrent guests are experimental. Keep other APIs
in the probe matrix but outside the essential release gate; the detailed scope
is owned by the [v1.0 contract](ROADMAP.md#version-10-contract).

This refines DEC-003: reproduction before implementation still applies. The old
blanket gate requiring parity for every optional passing upstream probe is replaced
by essential-workload parity, with optional gaps recorded individually. This follows
the user's explicit choices and request that speculative APIs not block v1.0.
It does not authorize silently dropping CUDA or either Direct3D requirement.

## DEC-007

**Proposed, evidence-triggered | 2026-09-24 | Resolve a failed baseline or backend gap**

No new backend decision is made by this plan. GPU-006 selects the minimum proven
path under DEC-002, and records reasons and remaining limitations.

| Trigger / options | Practical implication / recommendation | Who resolves |
|---|---|---|
| Essential VMMS probe fails: repair provisioning/session/identity; or run minimal HCS comparison | First isolate a reproducible cause. Recommend bounded HCS research only after matching native runtimes fail; do not create two production backends speculatively. | Technical evidence in GPU-005/006; ask user if HCS requires a broader VM-management product. |
| Safe upstream reference cannot be built/run: find a safe pinned artifact; use a source-derived minimal reference; or pause | Recommend finding a safe artifact first. A source-derived reference changes the reproduction contract and needs explicit user acceptance before substituting it. | User after REF-002/GPU-004 evidence. No signing/isolation exception is proposed. |
| Essential capability fails on both paths | Diagnose and retry a specific cause, change target/scope, or stop. Do not hide failure by marking it optional. | User after exact blocker and experiment results. |

These are conditional decisions, not present blockers. No question about choosing
a backend can be answered reliably before target measurements.

## DEC-008

**Proposed, owner input before packaging | 2026-09-24 | Distribution and licensing**

DOC-004 must obtain the owner's project license and intended delivery channel.
Options: source release with reproducible local build; portable CLI archive without
a publisher signature (simpler delivery, less publisher identity assurance); or
signed portable CLI archive (identity and certificate/service cost). An installer
adds maintenance without being needed for existing-VM operation.

Recommendation: a portable archive plus source, checksums, notices and build recipe;
use publisher signing if available. This is **not** selection of a license, purchase
of a certificate, or permission to publish. The owner can resolve this at M4 after
the actual component set and terms are known. No keys enter the repository and no
test-signing/Secure Boot change is an acceptable packaging workaround. Proprietary
driver/OS binaries remain local inputs under their actual terms, not release payload.

## DEC-009

**Accepted by user | 2026-09-24 | Rust-native implementation and shared engineering standards**

Implement all project functionality in Rust wherever technically possible,
including utilities and compatibility components. Native Windows facilities remain
the integration boundary; using an existing utility does not itself add an
implementation language. Supersede DEC-003's C/C++ retention preference and the
former architecture default to PowerShell adapters. Reference artifacts remain
external comparison inputs; ABI or upstream language alone cannot justify a
non-Rust implementation.

[ENGINEERING.md](ENGINEERING.md) owns detailed language-exception, quality,
testing, toolchain, dependency, CI and code-documentation rules. AGENTS remains
the concise mandatory entry point and session/protected-operation authority;
agent-specific instructions and optional prompts link to these sources.
This extends DEC-005's document map without changing its single task register.

Reason: the user's DOC-007 request requires maintainable, idiomatic Rust developed
in small tested changes, with no duplicated or conflicting agent policy. Initial
hardware-independent tests and minimal Windows PR checks start in CORE-001;
CORE-013 extends and verifies them. No application scaffolding or CI is introduced
by this documentation task, and the hardware reproduction gate is unchanged.

Revisit a language exception only with a specific technical limitation and an
investigated Rust-native alternative, recorded before introducing the exception.
See [DOC-007](BACKLOG.md#doc-007) for review and validation evidence.

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
