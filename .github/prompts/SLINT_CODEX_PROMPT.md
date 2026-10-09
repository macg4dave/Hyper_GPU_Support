# Codex — Slint GUI

Implement only the explicitly requested card in [BACKLOG](../../docs/BACKLOG.md).
Follow [AGENTS](../../AGENTS.md), [ENGINEERING](../../docs/ENGINEERING.md),
[GUI_GUIDE](../../docs/GUI_GUIDE.md) and [SLINT_RULES](../../docs/SLINT_RULES.md).
Preserve unrelated edits. A docs-only request does not authorise code or live tests.

Reuse the [source audit](../../docs/ARCHITECTURE.md#repository-audit--9-october-2026):
sound `gui_model`, backend/workflow/plans, protected runner, Named Pipe, operation
lock and journals already exist. Extend them, without parallel infrastructure.
Win32 presentation is disposable. Use small cohesive Slint components and Rust
adapters, the shared core, layouts/constraints, Fluent controls and central styling;
no Hyper-V logic or privileged commands in UI markup. Verify the chosen Slint APIs.

Follow the agreed design: VM cards, adjustable split view with horizontal scrolling, hybrid details panel, and editable GPU selection plus Min/Optimal/Max for VRAM, compute, encode and decode. Show all VMs. Separate observed state, saved configuration and one pending VM draft; prompt before switching with pending edits. Use only verified provider values and units.

One `hyper-gpu-support.exe` routes no arguments to Slint, explicit commands to CLI
and authenticated restricted worker invocation to per-operation elevation. Per-VM
GUID files under `%ProgramData%\HyperGpuSupport\config\vms\` are written only by
that worker after verified success. Preserve separate protected enrollment/recovery,
detect stale external changes and support save-only recovery without GPU replay.

Review fresh plans and independently recheck scope at the privileged boundary.
Obtain administrator authorisation before approved graceful guest shutdown.
One modifying GPU operation per host, one GUI per session; defer normal active-work
closure. Startup opens the dashboard with a persistent recovery warning and blocked
modifications. No automatic GPU rollback or blind retry. Product consent and agent
test authorisation are distinct: follow AGENTS for selected disposable tests and
the immediate physical-host lifecycle approval boundary.

Keep work off the UI thread; report real progress. Poll lightweight discovery/status,
not full driver validation/planning. Session-only UI diagnostics remain separate
from protected audit/recovery records. Use mock data and Slint preview without real
Hyper-V effects; verify sizing, DPI, keyboard and error states for the selected card.

Make one reviewable change at a time. Run relevant checks and report changes, test results and remaining issues.
