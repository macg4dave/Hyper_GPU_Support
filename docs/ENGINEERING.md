# Rust engineering standards

This is the authoritative engineering policy for all AI-driven coding sessions.
[AGENTS.md](../AGENTS.md) owns session workflow, scope, provenance and the
[development/test authorization](../AGENTS.md#development-and-test-authorization);
[BACKLOG.md](BACKLOG.md) owns scheduled tasks and durable results. Read only the
sections relevant to the active change. Prompts add task-specific guidance, not
competing rules. Documentation records implemented behavior; it is not a preflight
gate for a small change unless a safety, compatibility or public contract depends
on it.

Actual commands are in [README.md](../README.md); results belong on task cards.
Routine repository edits, dependency changes, development commands and targeted
cleanup are normal task execution and do not require separate approval.

## Product and development boundary

**Test-environment automation may support development but must not become production application functionality unless it is explicitly required by the user-facing product. Production code may be used by test tooling; production code must not depend on test tooling.**

`src/` owns the product and reusable product contracts; `tests/` owns Rust tests.
Use existing `scripts/testing/`, `scripts/setup/` and `scripts/diagnostics/` for
contributor infrastructure. The standalone `tools/lab/` package preserves disposable-reset helpers under
`tools/test-harness/`; it is excluded from the default product workspace.
Do not package laboratory binaries or enable its development feature in the product. Product installation is distinct from laboratory
setup; product recovery must preserve the user's existing VM and disk.

## Shell commands and development scripts

Run short, straightforward commands directly. When a procedure needs multiple
steps, substantial PowerShell, a complex pipeline, conditional logic or non-trivial
failure handling, write it to a script and execute the script file. This makes the
exact operation reviewable, reproducible and debuggable instead of leaving it in
terminal history. Prefer a command such as
`./scripts/diagnostics/check-hyperv.ps1` over an equivalent long inline command.

Maintained project tooling belongs in the category described by
[`scripts/README.md`](../scripts/README.md). Give scripts meaningful action-oriented
names and one clear purpose. Parameterize machine-specific paths and target
identities; validate inputs before privileged or destructive effects. Comment
non-obvious operations, handle expected failures, preserve useful native error
details, return meaningful nonzero exit codes and make steps safe to rerun where
practical. Run and report the script path plus parameters so a failure can be
reproduced. Reviewable scripts that remain useful should stay in the repository.

Temporary scripts belong under ignored `local/scripts/` or another clearly marked,
task-scoped temporary location, not in the maintained `scripts/` tree. Creating,
editing and executing project scripts against the designated disposable VM are
normal development work. A script must not restart, shut down or end the physical
host session without the user's explicit permission immediately before that
lifecycle operation.

PowerShell is permitted for repository development, manual diagnostics and
development-environment setup. Ordinary product operation, installation and
recovery belong in Rust. Invoking a cmdlet from Rust does not complete migration;
existing product adapters are tracked in the [migration audit](../scripts/PRODUCT-MIGRATION.md).
Retain working adapters until demonstrated replacements, and do not extend them
as product implementations except for necessary correctness/safety fixes or
bounded behavior comparisons needed for porting. PowerShell must not
become a parallel implementation of application logic, CLI behavior, validation,
diagnostics or GPU/Hyper-V management assigned to Rust. Embedded application-side
shell remains subject to the non-Rust exception process below.

## Windows elevation and UAC

Determine the required Windows privilege before running a command, script, Rust
integration test or utility. Ordinary builds, unit tests, repository checks and
other non-privileged development run with the normal token. Send a known privileged
operation through the approved fixed runner immediately when it exposes that
operation; use another explicitly authorized, bounded elevation path only when the
runner does not provide it. Do not run a known-admin operation unelevated merely to
observe `Access Denied`.

Assume elevation is required for Hyper-V configuration and protected host
management, GPU-PV adapter changes, service/driver/scheduled-task/runner install or
removal, protected filesystem or registry writes, VHD/VHDX mount or servicing, and
Windows feature or system-configuration changes. Before a privileged sequence,
distinguish whether the user belongs to Administrators, whether the current process
has an elevated token and whether the operation itself requires elevation; group
the token check at the start rather than rediscovering it during each step.

Never disable or weaken UAC, bypass consent, silently approve a prompt, or create
an unrestricted elevated command channel. The fixed runner accepts bounded typed
operations, validates the enrolled disposable target and arguments, and records
requests and results. Work already within its authorized scope does not need a new
permission prompt. Administrator privilege is distinct from authorization: the
physical host must never be restarted, shut down, logged out or have its interactive
session terminated without explicit user permission immediately beforehand. Stop
and ask if an otherwise authorized operation reports that a host restart is needed.

Every maintained executable PowerShell script declares one privilege class in its
header and in [`scripts/README.md`](../scripts/README.md): `non-elevated`,
`elevated`, or `either` with the reduced non-elevated behavior stated. An elevated
script checks near startup and fails concisely before other work; prefer PowerShell's
`#Requires -RunAsAdministrator` when applicable. Hardware/native Rust tests that
need elevation remain explicitly selected and separate from the normal test suite,
and must use the runner or fail immediately after a deliberate token check.

For an unexpected access-denied result, first verify the expected privilege class
and execution path, then the runner's effective rights, before treating it as a
different Windows permission defect. Do not blindly retry every permission failure
with elevation; preserve the native error and justify any elevated retry from the
operation being performed.

## Configuration and mutable values

Use the versioned runtime schema for product intent (schema 2 currently; the planned
per-VM transition is owned by [CONFIGURATION.md](CONFIGURATION.md)). Laboratory tooling uses
[`config/project.toml`](../config/project.toml) as its single checked-in source
for non-secret values expected to change with a machine, disposable VM, GPU/driver,
image, tool input or test run. This includes target identities, artifact paths and
hashes, external roots, tunable deadlines/retries and experiment switches. Before
adding a literal to Rust, PowerShell, a prompt or ordinary documentation, decide
whether it describes software behavior or the current environment. Keep true
protocol values, Windows API constants, enums and fixed safety bounds in code.

Prefer reliable Windows discovery for inventory such as the Windows directory,
standard system executables, OS build, installed interfaces, driver facts and the
resolved identity of an already selected target. Configuration expresses operator
intent and safety pins; it must not duplicate discoverable inventory without a
documented verification or security reason. Derive subordinate paths from one
discovered or configured root instead of storing each path independently.

Rust deserializes at a boundary into strongly typed structures, validates the full
document once and passes values through normal interfaces; application logic must
not repeatedly read TOML or environment variables. Maintained scripts use the
shared project-configuration reader and may expose clear parameters as explicit
overrides. Do not introduce another hand-maintained settings format. Generated
security policies/manifests are permitted when their source and regeneration/check
command are explicit and stale output fails validation.

Do not duplicate current machine values in documentation unless recording immutable
evidence from a completed run; refer to configuration keys instead. Repository
configuration must never contain credentials, keys or passwords. Use ignored local
files, environment variables, Windows credential facilities or runtime prompting as
appropriate. See the [configuration ownership and audit](CONFIGURATION.md).

## Rust and native Windows

Driver provisioning dynamically discovers the complete associated payload for the
selected GPU and installed signed driver, expands package trees, calculates guest
destinations and verifies the discovered files. Counts and hashes describe that
run; do not use an old environment's count, static NVIDIA file list or per-file
hash table as product logic. Use small artificial fixtures of different lengths
for ordinary tests; label historical baseline regression data explicitly. A
manifest/receipt count comparison verifies the current contract, not a fixed total.

- Implement application logic, CLI tools, configuration, GPU discovery/management,
  GPU-PV setup, Hyper-V integration, diagnostics and supporting utilities in Rust
  wherever technically possible. Prefer the standard library, existing dependencies,
  maintained Rust crates and native Windows APIs; use
  [`windows`](https://github.com/microsoft/windows-rs) where appropriate.
- Do not add Python, PowerShell, C, C++, C#, JavaScript or another implementation
  language when Rust can do the job. Adapted material retains required attribution
  and notices.
- Before introducing a non-Rust exception, investigate a Rust-native route and
  record in [DECISIONS.md](DECISIONS.md) the technical limitation, alternatives
  investigated, smallest exceptional component, validation, dependencies/notices
  and conditions for revisiting it. Convenience is insufficient.
  An external ABI alone does not require C/C++ source; evaluate
  [Rust FFI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi) first.
- Use existing Windows/Hyper-V facilities; do not reimplement the OS. Invoking an
  installed utility is not itself adopting another implementation language. If an
  operation is available only through a management interface or cmdlet, Rust may
  invoke it through a typed, bounded adapter; document the necessity. Any embedded
  scripts must be minimal interface glue covered by the exception, with application
  decisions and parsing/validation kept in Rust. Never interpolate untrusted scripts.
- Installed vendor tools are external inputs, not application-language choices.
  Record their identity/license and bound their invocation where used. New
  project-owned probes follow this policy; SDK/toolchain constraints use the same
  exception process. Implement the project recipe from our source and architecture,
  without an external-reference comparison prerequisite.

## Code and module design

- Use descriptive Rust naming, strong types and explicit ownership/borrowing.
  Model validated identities, units and state transitions so invalid combinations
  are difficult to construct. Prefer small focused functions and minimal mutable
  state. Avoid unnecessary copies, clones, allocations and implicit global state.
- Keep CLI parsing/output, configuration validation, orchestration, Hyper-V access,
  GPU discovery, GPU-PV configuration and diagnostics separated by responsibility.
  Windows APIs, process execution and privileged effects belong behind narrow
  adapters; ordinary logic should be testable without hardware or elevation.
- Start with ordinary modules in the planned CLI/library package. Split files when
  responsibilities or navigation justify it, without arbitrary line limits, dozens
  of trivial files, circular dependencies, speculative traits or extra crates.
  Introduce an abstraction only for a concrete boundary, repeated need or useful test.
- Keep interfaces explicit and visibility minimal. Inspect callers before changing
  module contracts; update callers, tests and documentation together. Preserve
  promised CLI/config/report compatibility or version and explain the change.
  No premature public SDK or external-project API compatibility is required.
- Handle expected failures with `Result`/`Option`; do not use `unwrap()`, `expect()`
  or `panic!()` where production failures should be handled normally. Test assertions
  and clearly invariant-only cases need judgment, not blanket suppression. Do not
  turn an external input, permission failure or unavailable device into a panic.
- Use `unsafe` only for a demonstrated need. Keep each block small with a local
  `// SAFETY:` justification of its actual preconditions, lifetimes, aliasing and
  thread requirements. Encapsulate it behind a safe interface where possible;
  public unsafe interfaces need a `# Safety` contract. Validate lengths, encodings,
  ABI layout and ownership at FFI boundaries. Do not unwind across a foreign ABI.
- Use RAII for handles, buffers, locks and sessions, respecting native ownership
  and thread/apartment rules. Fallible cleanup needs an explicit path that reports
  errors; `Drop` provides best-effort fallback without panicking. Preserve recovery
  state when cleanup cannot complete. A destructor is not crash recovery.
- Avoid premature optimization. Correct obvious excessive cloning, repeated queries
  or unbounded buffering; measure representative workloads before adding caches,
  concurrency or complex optimizations. Do not trade correctness for a microbenchmark.

## Errors and operation lifetimes

- Use structured error categories where callers need to distinguish configuration,
  permission, environment, driver and implementation failures. Preserve underlying
  error sources and Windows error/HRESULT values, adding operation and target context.
  An unknown or denied result is not absence, success or unsupported capability.
- Never silently discard failures. Explain intentional best-effort actions and
  surface their outcomes. Preserve both the primary error and any recovery failure;
  provide an actionable next step without hiding partial changes.
- Use consistent structured logs with useful levels and fields such as operation,
  stage, outcome and duration; use the existing lightweight logging approach.
  Keep CLI summaries readable. Redact credentials and unnecessary identifying data;
  avoid full environment dumps, secrets in command lines and duplicate error logging.
- Validate all inputs before Hyper-V, GPU or guest-driver changes, revalidate state
  immediately before effects, and verify outcomes. Prefer reversible steps and
  documented recovery, following the [recovery contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Every long-running Windows operation needs a bounded timeout/deadline, cancellation
  behavior and cleanup plan. Use bounded retries only for identified transient errors
  and safe/idempotent steps. Avoid busy waits; bound process output and handle exit
  status, stderr and malformed/partial output. If an API cannot cancel in-flight work,
  report that limitation and reconcile its real state before retry or recovery; a
  caller timing out does not prove that a native mutation stopped.

## Testing

Every function containing meaningful logic needs appropriate behavior coverage,
directly or through a module/integration test. Exercise normal behavior, boundaries,
invalid inputs and expected failure/recovery paths as applicable. No separate test
per trivial accessor, implementation-mirroring assertion or arbitrary coverage
percentage substitutes for this requirement. If testing a function is impractical,
record why and the concrete alternative validation and remaining gap on its task.

- Reuse established results. Run a fresh baseline only to diagnose a regression or
  establish behavior at a changed risky boundary. Add/update focused tests with the
  implementation; do not weaken assertions or add unrelated refactoring for a pass.
- Use unit tests for isolated logic; integration tests for component contracts,
  configuration validation, error propagation and recovery. Test examples with
  Rust doc tests where useful; mark truly non-runnable examples honestly.
- Use small fakes or controlled interfaces for native calls, processes, clocks and
  I/O when they enable meaningful tests. Test real observable behavior and failure
  propagation rather than matching private implementation details. Do not add an
  elaborate mocking framework for a few adapters.
- Keep tests deterministic: fixed fixtures/seeds, controlled time, explicit signals
  and bounded polling for real asynchronous state. Arbitrary sleeps are not readiness
  checks. Isolate each test's files/resources and make parallel runs independent.
- Use RAII and explicit teardown for disposable resources, including failure and
  cancellation paths. Keep fixtures/work directories within the repository; an
  outside path needs existing explicit authorization under AGENTS. Remove only
  resources owned by the test; retain evidence or recovery backups when necessary.

Keep these lanes distinct (workload checks are specified in
[architecture test lanes](ARCHITECTURE.md#test-lanes)):

| Lane | Required behavior |
|---|---|
| Hardware-independent Windows tests | Unit, component integration, configuration, regression and doc tests with no administrator, Hyper-V, GPU or network prerequisite. Codex runs at milestone completion or on explicit request; retain PR checks. |
| Native Windows integration | Test actual API/adapter contracts and OS failure behavior. Run safe unprivileged cases in Windows CI where available; explicitly invoke environment/privilege-dependent cases only on a prepared target. |
| Hyper-V and GPU workloads | Explicitly selected runs on the designated disposable target with exact environment, inputs and checked output. These runs may install/update the runner and mutate/recreate the guest without another approval. Enumeration, loading a DLL or compiling does not prove GPU support. |

The measured normal Hyper-V Code 0, `nvidia-smi`, D3D11/D3D12 and CUDA baseline
is established. Hardware testing now verifies changed Rust behavior, clean-child
reproduction and driver re-stage workflows; it is not a renewed feasibility gate.
Do not add repeated evidence audits or unchanged hardware runs to ordinary tasks.

Default tests and `--all-features` must never implicitly opt into environment-
mutating operations. Use explicit ignored/dedicated suites with runtime prerequisite
and designated-target identity checks for those runs. Report skipped, blocked and
untested cases distinctly from passes.

## Toolchain, dependencies and features

- Keep the exact stable Rust toolchain in
  [`rust-toolchain.toml`](../rust-toolchain.toml), including
  rustfmt/Clippy and the Windows x64 MSVC target. Use one explicit edition consistently
  ([2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) for new code unless a documented
  compatibility constraint requires otherwise). Declare the supported minimum Rust
  version in Cargo metadata and verify it if distinct from the pinned toolchain.
- Commit `Cargo.lock` for this application/CLI; use locked dependency resolution
  for builds/tests/CI. Pin any Git dependency to a reviewed immutable revision and
  CI actions to commit SHAs. Record Windows SDK/MSVC and other native tool inputs
  in the actual build recipe. Toolchain/dependency updates are deliberate scoped
  changes with validation, not incidental cleanup. Do not claim byte-for-byte
  reproducibility without testing it.
- Before adding a dependency, check the standard library, existing crates and
  Windows facilities. Prefer maintained crates with suitable licenses and clear
  reliability/maintenance value; review enabled features, transitive cost, native
  build requirements and security exposure. Avoid a large crate for trivial work.
- Use `cargo audit` and `cargo deny check` for scoped dependency changes or release
  checks where practical.
  Record tool availability, advisory database date/availability and findings; no
  unavailable check is a pass. Add a small project-specific license/advisory policy
  when needed, not a generic compliance framework. Triage findings and document
  narrowly justified exceptions instead of blanket ignores or blind upgrades.
- Keep platform-specific APIs in Windows adapters with intentional `cfg` boundaries.
  Windows 11 x64 is the target; Linux/macOS CI or abstraction layers are not required.
  Features should express real optional capabilities and compose where practical.
  Document supported default/minimal/combined feature sets and test affected sets;
  never use a feature to conceal a required failing path or authorize mutation.
  `--all-features` enables one union, not every combination; follow
  [Cargo feature semantics](https://doc.rust-lang.org/cargo/reference/features.html).

## Required checks and CI

Codex follows [Testing Policy — Codex](../AGENTS.md#testing-policy--codex).
Do not run checks during development or at individual task completion. Run relevant
checks once after all planned milestone tasks are finished, or on explicit request.
Immediate exceptions require an explanation and permission first. CI retains its
existing gates. The following commands define the applicable check set:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Use `--locked` on Cargo builds and checks in CI and reproducible verification.
After a milestone failure, fix the relevant issue and rerun only necessary checks.
Keep doc tests in the test lane:
`cargo test --all-targets` alone
does not run them ([Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)).
If all features cannot validly combine, record the actual
supported commands/matrix in the build documentation and task evidence instead
of invoking an invalid combination or silently reducing coverage.

- Resolve compiler/Clippy findings at their source. Do not add blanket `#[allow]`,
  `#[expect]`, workspace lint exclusions, or remove useful behavior/error handling
  to get green output. A necessary exception must target a specific lint at the
  smallest scope, with a technical justification and review/removal condition.
- Enable compiler warnings as errors for CI build/test/doc-test compilation as
  applicable, in addition to Clippy's `-D warnings`. Test touched runnable examples;
  check generated rustdoc with warnings denied when public docs/interfaces change.
  Do not enable every optional lint family or ban language constructs indiscriminately.
- Keep Windows x64 PR checks focused on formatting, strict Clippy, locked
  build/tests and applicable doc tests. Docs-only work checks touched links and
  consistency without hardware runs or a new crate.
- Privileged/hardware suites stay explicitly selected on the designated dedicated
  target. Never run untrusted PR code with elevated access or repository secrets.
  Hosted Windows CI results are not Hyper-V/GPU workload evidence.
- Report commands actually run, outcome, material failures, missing prerequisites
  and remaining validation. Do not rerun an unchanged check merely to reproduce an
  old evidence record. Fix introduced problems before completion; if technically
  blocked, record the exact next action. Installing tools and mutating the designated
  disposable target are part of normal project validation.

## Documentation

Use `//!` for module purpose/boundaries and `///` for public functions, types and
interfaces. Explain assumptions, invariants, error behavior, resource ownership,
cleanup and non-obvious decisions. Document Windows/Hyper-V semantics, GPU-PV
limitations, feature/platform conditions and unsafe/FFI safety obligations next
to the code, linking relevant authoritative API documentation. Add examples where
they help callers understand correct use.

Comments should explain why and what must remain true, not narrate obvious Rust.
Update comments, examples and affected contracts with behavior changes. Keep
architecture descriptions separate from evidence of implemented/tested behavior.
Documentation follows the implementation in the same change or at milestone
closure; it should not delay an ordinary coding iteration that leaves no public or
safety contract stale. Follow AGENTS for concise results and handovers; do not
rewrite the roadmap, audit unrelated notes or duplicate these standards for a small
code change.
