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
cargo test --manifest-path vendor/i-slint-backend-winit/Cargo.toml --lib --features renderer-software windows_exposure
```

This standalone test command resolves the crate's test dependencies separately;
review its generated lockfile before execution as part of the gate. Product
builds continue to use the root lockfile. Tests and compilation are pending. Live acceptance must cover updates while
partially off-screen or covered by the taskbar, return to visibility without
another UI update, minimize/restore, and fractional-DPI monitors. No live
GUI validation is implied by the backport or the in-memory regression tests.
