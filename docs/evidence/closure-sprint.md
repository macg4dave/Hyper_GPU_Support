# 10 October 2026 closure sprint

## CORE-012 and APP-001

CORE-012 now shares bounded public diagnostics between CLI and Slint. Individual
VM, GPU-provider and preparation-record failures retain independently available
inventory. Missing, denied, unavailable, unsupported and unknown observations are
distinct. Desired intent, current attachment, historical preparation/graphics
checks and recovery remain separate; failed refresh cannot promote retained data
to current state. Parser/provider/worker errors do not echo raw secret-bearing
input. Public guidance retains refresh, consent and conditional save-only recovery.

APP-001 uses the Windows GUI subsystem. Explicit CLI commands attach to an existing
parent console while preserving redirected handles and exit codes. GUI and internal
worker launches do not attach. The installed same-executable worker performed an
authenticated missing-receipt rejection without admitting an effect. Its protected
configuration directory now resides under the product-owned root rather than the
older user-writable laboratory root; ACL and reparse admission remain enforced.

Windows x64 MSVC / Rust 1.94.0 closure checks passed:

- `cargo test --locked --workspace --all-features`: 89 library, 22 application,
  6 CLI and 6 configuration tests passed (123 total); the opt-in M1 test ignored.
- Four real overlapped Named Pipe tests passed: authenticated exchange, wrong PID,
  bounded timeout/cancellation and oversized header rejection.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
  and `cargo build --locked --workspace --all-features` passed.
- CLI process tests verified redirected output/errors, exit codes, PE GUI subsystem,
  restricted internal dispatch and secret redaction. A real parent-console capture
  verified version stdout and unknown-command stderr, with exit codes 0 and 1.
- Independent architecture/security review found no remaining blocking findings
  for CORE-012 or APP-001 after fixes and inspection of installed-worker evidence.

## GUI-002 scoped correction

Review uses a wider preferred dialog, bounded content width, a scrollbar gutter and
character wrapping for long technical lines. The actual Slint MCP-enabled executable
was inspected at 1240x860 with an enabled historical plan: Review text was readable
without edge clipping. Snapshot runtime checks passed candidate/observed separation,
shared enabled/disabled previews, retained drafts, external-input conflict blocking,
unchanged input bytes and no filesystem events in the three monitored product
directories. This closes the clipping defect, not GUI-002's broader dependency gates
or narrow/DPI/accessibility qualification.

## Failed checks and evidence scope

The first build found an invalid Slint width/min/max combination; it was corrected
and the affected build passed. Strict Clippy found test-harness API/error-style
issues; they were corrected and the check passed. Initial visual automation lacked
Slint debug metadata or used a non-MCP artifact; a correctly built artifact passed.
Initial installation correctly rejected the old laboratory data root and a
junction-crossing source path. The product-root correction and installation from
the non-junction Cargo artifact directory passed without weakening trust checks.
The first hardware wrapper aborted on a normal stderr progress message. Its
terminal failed operation was explicitly reconciled as a disposable contributor
fixture under the exclusive product lock: initial VM/security/disk/settings and
missing configuration were checked, no provider operation remained, the exact
initial journal was atomically restored with preparation still unknown, and only
the matching hold was removed. The signed DLL preimage was retained for the resumed
full preparation; no product recovery semantics were changed.

Local captures are in `local/evidence/closure-sprint/` and
`local/evidence/gui-002-testing/`. Hardware preparation/restoration results are
recorded in the [new Reapply investigation](reapply-investigation-20261010.md):
functional success does not close M2 after the reported host lockup. No sharing,
CUDA, D3D12, driver-upgrade or general host-stability qualification is implied.
