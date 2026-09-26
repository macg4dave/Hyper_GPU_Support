# Rust engineering standards

This is the authoritative engineering policy for all AI-driven coding sessions.
[AGENTS.md](../AGENTS.md) owns session workflow, scope, provenance and the
three-category [permission boundary](../AGENTS.md#permission-boundary);
[BACKLOG.md](BACKLOG.md) owns tasks and evidence. Read the sections
relevant to the active change. Prompts add task-specific guidance, not competing
rules. Changes to these standards require explicit task scope.

CORE-019 establishes the Cargo project, tests and basic Windows PR checks.
CORE-001 extends the foundation with inventory; CORE-013 extends test coverage.
Actual commands are in [README.md](../README.md); results belong on task cards.
Routine repository edits, dependency changes, development commands and targeted
cleanup are normal task execution and do not require separate approval.

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
task-scoped temporary location, not in the maintained `scripts/` tree. Creating or
editing any script is routine repository work; executing it follows the
[permission boundary](../AGENTS.md#permission-boundary) for its actual effects.
In particular, a script must not restart, shut down or end a host session without
the user's explicit permission immediately before that lifecycle operation.

PowerShell is permitted for repository development, diagnostics, Windows
environment setup and invoking existing Hyper-V/Windows facilities. It must not
become a parallel implementation of application logic, CLI behavior, validation,
diagnostics or GPU/Hyper-V management assigned to Rust. Embedded application-side
shell remains subject to the non-Rust exception process below.

## Rust and native Windows

- Implement application logic, CLI tools, configuration, GPU discovery/management,
  GPU-PV setup, Hyper-V integration, diagnostics and supporting utilities in Rust
  wherever technically possible. Prefer the standard library, existing dependencies,
  maintained Rust crates and native Windows APIs; use
  [`windows`](https://github.com/microsoft/windows-rs) where appropriate.
- Do not add Python, PowerShell, C, C++, C#, JavaScript or another implementation
  language when Rust can do the job. AppSandbox is a behavioral reference for
  selective Rust reimplementation, not a language or architecture template.
  Translations and adaptations retain required attribution and notices.
- Before introducing a non-Rust exception, investigate a Rust-native route and
  record in [DECISIONS.md](DECISIONS.md) the technical limitation, alternatives
  investigated, smallest exceptional component, validation, dependencies/notices
  and conditions for revisiting it. Convenience or upstream language is insufficient.
  An external ABI alone does not require C/C++ source; evaluate
  [Rust FFI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi) first.
- Use existing Windows/Hyper-V facilities; do not reimplement the OS. Invoking an
  installed utility is not itself adopting another implementation language. If an
  operation is available only through a management interface or cmdlet, Rust may
  invoke it through a typed, bounded adapter; document the necessity. Any embedded
  scripts must be minimal interface glue covered by the exception, with application
  decisions and parsing/validation kept in Rust. Never interpolate untrusted scripts.
- Unmodified reference artifacts and vendor tools used for comparison are external
  inputs, not application-language choices. Inspect and pin their provenance,
  dependencies, license and side effects. New project-owned probe/utility code follows
  this policy; genuine SDK/toolchain constraints use the same exception process.

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
  No AppSandbox API compatibility is required; no premature public SDK is implied.
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
  stage, outcome and duration; choose one lightweight approach when code exists.
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

- Establish the affected test baseline before edits; record existing failures.
  Add/update tests with implementation and include a regression test for each bug
  fix wherever feasible, showing the old failure when practical. Avoid unrelated
  refactoring or weakening assertions to obtain a pass.
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

Keep these lanes distinct (task ownership and hardware evidence requirements are
in [architecture test lanes](ARCHITECTURE.md#test-lanes)):

| Lane | Required behavior |
|---|---|
| Hardware-independent Windows tests | Unit, component integration, configuration, regression and doc tests with no administrator, Hyper-V, GPU or network prerequisite. Run routinely and on PRs. |
| Native Windows integration | Test actual API/adapter contracts and OS failure behavior. Run safe unprivileged cases in Windows CI where available; explicitly invoke environment/privilege-dependent cases only on a prepared target. |
| Hyper-V and GPU workloads | Opt-in runs with exact environment, permissions, inputs and checked output. Require prior scoped authorization for protected changes. Enumeration, loading a DLL or compiling does not prove GPU support. |

Default tests and `--all-features` must never implicitly opt into protected
operations. Use explicit ignored/dedicated suites with runtime prerequisite and
authorization checks for those runs; a feature flag or `--ignored` alone is not
authorization. Report skipped, blocked and untested cases distinctly from passes.

## Toolchain, dependencies and features

- At CORE-019, select and record an exact stable Rust toolchain in
  [`rust-toolchain.toml`](https://rust-lang.github.io/rustup/overrides.html), including
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
- Use `cargo audit` and `cargo deny check` where practical once dependencies exist.
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

Once a Cargo workspace exists, the normal development checks are:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Use `--locked` on Cargo build, Clippy and test commands in CI and reproducible
verification. Run these checks after meaningful Rust changes, plus focused tests
while iterating. Keep doc tests in the test lane: `cargo test --all-targets` alone
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
- CORE-019 adds minimal Windows x64 PR checks with the first code: formatting,
  strict Clippy, locked build/tests and applicable doc tests. CORE-013 extends
  feature coverage and failure/artifact checks. Every PR runs the applicable gates;
  docs-only work checks links/consistency without requiring hardware or a new crate.
- Privileged/hardware suites stay explicitly selected on the authorized dedicated
  target. Never run untrusted PR code with elevated access or repository secrets.
  Hosted Windows CI results are not Hyper-V/GPU workload evidence.
- Report commands actually run, outcome, pre-existing versus introduced failures,
  missing prerequisites and remaining validation. Fix introduced problems before
  completion; if blocked, record the exact next action. Do not install a tool or
  mutate a protected target beyond existing authorization merely to run a check.

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
Follow AGENTS for the concise task result, changelog and handover; do not rewrite
the roadmap or duplicate these standards for a small code change.
