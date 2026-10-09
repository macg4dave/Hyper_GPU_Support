# Hyper GPU Support GUI prototype

An isolated, interactive Rust/Slint **demonstration**, with no dependency on the
production management core. It opens without Hyper-V, administrator privileges,
credentials or a protected runner. Nothing here changes real VMs, GPUs or drivers.

From the repository root:

```powershell
cargo run --locked --manifest-path tools/gui-prototype/Cargo.toml
```

Explore Dashboard, System, Settings and About. Select cards,
drag the divider, scroll each pane, edit the GPU switch/selector and
the mock 1–8 GB GPU Memory slider or expand Advanced Allocation for all twelve
provider-unit fields, expand details, discard drafts, review changes and
confirm a demonstration. Running sample VMs request mock downtime consent.
Settings offers Windows/light/dark appearances and empty, many-VM, recovery,
failure, uncertain-outcome, save-failure and stale-draft scenarios. Quick failure
buttons make the error flows easy to explore. Theme/preferences, configuration
and operation results exist only in memory and reset when the process closes.

The GPU Memory slider is a visual preference, independent of the advanced
allocation values. It does not allocate or guarantee physical VRAM in GB.

All inventory, eligibility, provider values, enrollment, graphics verification,
Apply events, recovery and saving are mocked. Selection,
draft editing/validation, navigation, dialogs, scrolling, resizing and themes
are actual UI behavior. Sample provider units are deliberately not presented
as GiB, percentages or promises of enforcement.

VM cards include state-driven green/red/grey dots and local Windows, Linux or
generic-monitor icons. OS types come from explicit mock metadata, not inferred
Hyper-V guest detection. Icon assets and their provenance are in `ui/icons/`.
The neutral light/dark surfaces retain Fluent controls; technical-details toggles
use a shared, fixed-size Fluent button in both the panel and review dialogs.
VM names are left-aligned and ellipsized to fit the available card width, with
full-name hover tooltips and keyboard-accessible card selection. Search and type
filters are removed from this iteration. Every dialog keeps its technical toggle
below the action buttons; expanded information has its own bounded scroll area.
The VM header has a single heading/Refresh row, and cards use a consistent 12px
layout gap. Dialog diagnostics use read-only, selectable Consolas text on a white
background with black text in either theme. Mouse selection, Shift+arrow selection,
Ctrl+C, Home/End and two-axis scrolling use Slint's built-in text input behavior.

## Live preview

With the official Slint 1.18.1 viewer installed:

```powershell
slint-viewer --auto-reload tools/gui-prototype/ui/app.slint
slint-viewer --check tools/gui-prototype/ui/app.slint
```

The viewer previews layout with initial sample properties; the Rust executable
provides interactive mock callbacks. The prototype pins Slint and slint-build
1.18.1 and compiles Fluent widgets with the software renderer. For development
inspection only, enable Slint's optional MCP feature on the command line:

```powershell
$env:SLINT_EMIT_DEBUG_INFO = '1'
$env:SLINT_MCP_PORT = '9317'
cargo run --locked --manifest-path tools/gui-prototype/Cargo.toml --features slint/mcp
```

## Design limits

Slint has no standard splitter, so `cards.slint` supplies a small bounded drag
divider. At narrow widths the two-pane workspace scrolls horizontally; it does
not collapse. Fluent is a compile-time widget style; its light/dark palette is
switched at runtime. This is restrained Fluent styling, without native Windows
Mica, acrylic or a custom title bar. Dialogs are in-window overlays with disabled
background controls rather than native Windows dialog windows. Full screen-reader,
Windows text-scaling and multi-monitor qualification remain production work.
Fluent TextEdit does not expose colour overrides, so `technical-output.slint`
combines Slint TextInput with standard ScrollView to keep diagnostics black/white
without changing the global palette. Long diagnostic lines scroll horizontally.

The production CLI and Win32 experiment are preserved. This package is explicitly
excluded from the root workspace; it is not shipped product functionality.
Slint licensing options and attribution: <https://slint.dev/license>; dependency
licenses must be reviewed before distributing a packaged application.

## Checks

```powershell
cargo fmt --manifest-path tools/gui-prototype/Cargo.toml -- --check
cargo clippy --locked --manifest-path tools/gui-prototype/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path tools/gui-prototype/Cargo.toml
```
