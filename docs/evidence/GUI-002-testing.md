# GUI-002, CFG-001 and OPEN-01 testing — 9 October 2026

Windows 11 Pro 10.0.26300, 64-bit; Rust 1.94.0,
`x86_64-pc-windows-msvc`; Slint 1.18.1 software renderer.
These results qualify implemented slices, not the M3 milestone or protected saving.

## Automated results

- 103 tests passed: 72 library, 21 GUI/dispatch, 4 CLI and 6 configuration-file tests.
  One privileged installed-enrollment integration test was explicitly ignored.
- Formatting, strict all-target Clippy, locked workspace/all-feature build,
  rustdoc with denied warnings and documentation checks passed.
- New configuration coverage exercises missing files, malformed TOML/UTF-8,
  exact 64 KiB acceptance and over-limit rejection, unchanged source bytes,
  omitted allocation versus explicit zero, incomplete/unordered/negative/overflow
  triples and unsupported compute/encode/decode fields. Existing coverage verifies
  GUID binding, canonical filenames, duplicate identity rejection, bundle splitting,
  shared CLI/GUI interpretation and intent/observation/draft separation.
- Fixed formatter drift, moved dispatch tests after production items and collapsed
  the configuration-refresh conditional to satisfy strict Clippy; no behavior change.

Commands: `scripts/testing/check.ps1` ran formatting, Clippy and the tests.
Its initial formatting/Clippy failures were corrected. A Windows executable lock
from concurrent runtime inspection interrupted its build step; runtime then used
a copied executable. The remaining `cargo build --locked --workspace --all-features`,
`cargo doc --locked --workspace --all-features --no-deps` and
`scripts/testing/check-docs.ps1` passed, with the script's strict Rust/rustdoc flags.
Final formatting and strict Clippy also passed with the added tests.

## Runtime results

Development MCP build: `SLINT_EMIT_DEBUG_INFO=1`,
`cargo build --locked --bin hyper-gpu-support --features slint/mcp`.
The MCP feature was enabled only by the development build command, not added to
production dependencies. Tests used `SLINT_BACKEND=headless`, the actual Rust
executable and accessibility interactions/screenshots, not a standalone UI viewer.

`local/scripts/qualify-gui-002-snapshot.ps1` passed candidate/observed separation,
historical System presentation, enabled/disabled shared-plan rehearsal, simulated
stage labels, unapplied draft retention, unchanged input files and external
configuration/snapshot changes blocking review while preserving drafts. Enabled
review requires the existing shutdown rehearsal checkbox; scrolling reaches it.
Filesystem watchers observed no events in the three existing ProgramData/product
installation directories during rehearsal. This is scoped filesystem evidence;
unit tests separately cover credential-source refusal and mutation-adapter refusal.
Synthetic VM/GPU/driver/payload inputs establish rehearsal behavior only.

`local/scripts/qualify-gui-002-live.ps1` passed no-argument real discovery,
no fixture fallback or invented saved intent, System and Refresh through the
existing authenticated runner. Discovery showed the designated disposable VM,
NVIDIA GeForce RTX 5060 and driver 32.0.16.1692. Preparation parity/current guest
health remained unknown, and the existing graphics-check timestamp was explicitly
historical. Mandatory runner read auditing remained enabled. No Apply, credential
entry, guest probe or GPU mutation was requested. Guest build/API/workload results
were not established by this run.

Screenshots, runtime logs, copied development executable and synthetic inputs are
under ignored `local/evidence/gui-002-testing/`. Temporary qualification scripts
remain under ignored `local/scripts/`.

`local/scripts/qualify-gui-002-fixtures.ps1` reused the existing
`qualify-main-slint.ps1` fixture regression and passed pages, theme controls,
draft retention across pages, dirty VM switch confirmation/discard, advanced fields,
technical review, simulated Apply and simulated failure/reconciliation. Fixture
screenshots remain under ignored `local/evidence/gui-promotion/`.

## Remaining acceptance gaps

- Visual defect: enabled/disabled Review dialog body clips long text at its right
  edge at the default 1240 × 860 size. The functional rehearsal passes do not waive
  this presentation issue; preserve the approved layout when correcting wrapping.
- Protected ProgramData publication, ACL/reparse admission, torn writes, durable
  save-only recovery and real GUI Apply remain CFG-001/SEC-001/CORE-028 work.
- Full DPI/keyboard/screen-reader qualification remains GUI-003; headless runtime
  checks do not establish those Windows desktop properties.
- OPEN-01's selected format and implemented reader/serializer are tested; no import
  CLI or protected publisher exists. GUI-002 and CFG-001 remain in progress.
