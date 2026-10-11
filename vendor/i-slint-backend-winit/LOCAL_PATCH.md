# Windows exposure repaint backport

This directory contains the published `i-slint-backend-winit` 1.18.1 crate,
with its original licensing and source retained. Cargo uses it through the
root `[patch.crates-io]`; all Slint versions remain pinned to 1.18.1.

The focused change backports [Slint PR 13568](https://github.com/slint-ui/slint/pull/13568)
(commit `5203fd9`): before drawing on Windows, read `GetUpdateRect` while
`WM_PAINT` is pending and mark that area dirty in the renderer. Windows
exposure damage is independent of Slint property changes and softbuffer's
buffer age. Without this, a redraw may render nothing or only changed UI
regions, leaving newly exposed pixels stale. No timer or backend refresh is added.

Local differences from that patch are a safety comment, extracting the
physical-to-logical region conversion for testing, and two isolated regression
tests in `windows_repaint_tests.rs`. An empty workspace table permits the
separate library-test command below. The test-only Slint dependency provides
an in-memory scene; it does not launch the product or a native window.

At the next authorized automated gate, run the backend library regressions
explicitly (root package tests do not execute dependency unit tests):

```powershell
cargo test --manifest-path vendor/i-slint-backend-winit/Cargo.toml --lib --features renderer-software,raw-window-handle-06 windows_exposure
```

This standalone test command resolves the crate's test dependencies separately;
review its generated lockfile before execution as part of the gate. Product
builds continue to use the root lockfile. The backport and in-memory regressions
alone do not establish live GUI acceptance; the subsequent live check and user
acceptance are recorded below.

## Validation on 11 October 2026

At revision `56b085c`, the non-elevated root gate passed documentation and
formatting, compiled the production backend, and ran library tests: 150 passed
and one failed (`preview_rejects_wrong_identity_generation_allocation_and_journal`,
`src/workflow.rs:1803`). Its allocation scenario still expects a VRAM request
above reported metadata to be rejected, whereas the preceding VRAM change
deliberately permits that request. The gate remains failed; lint, product-binary
build and API-documentation stages were skipped.

The separate GUI controller/startup test command
`cargo test --locked --bin hyper-gpu-support` passed all 56 tests.
The two in-memory repaint regressions passed with `--offline --locked` and
features `renderer-software,raw-window-handle-06`. Their initial compilation
exposed a missing `euclid` import and the feature needed by existing backend
test code; both test-harness issues were corrected before the passing run.
The standalone ignored lockfile was seeded from the root lockfile before
resolving test-only dependencies offline. The root lockfile was unchanged.

Raw logs are under `local/evidence/m1-gate-20261011T200235371Z/` in the root
workspace. That automated run did not launch the product GUI or a native window,
prompt for credentials, exercise an installed worker, or run VM/GPU workloads.

## Windows redraw bugs closed by user acceptance

All Windows redraw bugs are closed by the user's explicit instruction on
11 October 2026 following the successful live taskbar/off-screen check.

The current source compiled with `SLINT_EMIT_DEBUG_INFO=1` and
`cargo build --locked --bin hyper-gpu-support --features slint/mcp`.
A copied executable ran with the native `winit-software` backend and a
loopback Slint MCP connection. The user moved the window beneath the taskbar;
MCP recorded its position as `(312, 960)`, size `1240 x 860`, scale factor `1.0`.
While the window remained partly hidden, MCP switched Dashboard to About and
confirmed the updated element tree without taking a rendered snapshot.
The user brought it fully back into view without clicking inside or pressing
Refresh and confirmed: "Yes, About redraws correctly". The user then explicitly
requested closure of all Windows redraw bugs.

Evidence is retained in `local/evidence/slint-repaint-live-20261011T201439Z/`.
This records the actual live check and user acceptance. Cover/uncover,
minimize/restore, resize and fractional-DPI monitor transitions were not
separately executed; they are not retained as open Windows redraw bugs.
No VM/GPU changes or protected recovery actions were performed for this check.
