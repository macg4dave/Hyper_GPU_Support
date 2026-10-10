# Codex — Implement the approved Slint application

Follow [AGENTS](../../AGENTS.md), relevant [ENGINEERING](../../docs/ENGINEERING.md)
and [SLINT_RULES](../../docs/SLINT_RULES.md). [ROADMAP](../../docs/ROADMAP.md) and
[BACKLOG](../../docs/BACKLOG.md#next-implementation-action) determine priorities.
[GUI_GUIDE](../../docs/GUI_GUIDE.md) freezes the existing v1.0 interface.

- Inspect `src/gui/`, `src/gui/ui/` and the affected Rust entry points first.
  Implement functional existing controls; preserve layout, navigation, styling,
  dialogs and CLI behavior. No new pages, wizard, Activity or search/filter.
- Reuse discovery, `gui_model`, configuration/parser, planner, credentials,
  protected runner/worker, authenticated IPC, locks and journals. No second backend.
- Keep committed intent, observed state, protected enrollment and draft separate.
  Preserve raw invalid edits and selection. Fresh reviewed plans and independent
  worker revalidation precede effects; retain UAC and separate downtime consent.
- Validate real provider units/bounds. Unsupported fields stay visibly blocked;
  the illustrative memory slider grants no physical-GB allocation promise.
- Use actual progress and readback; uncertain outcomes require reconciliation.
  Save-only retry cannot replay effects. Preserve artifact trust, ACL/reparse
  guards, exact identity, audit and host-wide recovery holds.
- Live data never falls back to fixtures. Explicit mock/snapshot rehearsal is
  no-effect/no-write, including credentials/config/enrollment/journals/audits and
  guest probes. Never suppress the real runner's mandatory audit.
- Author focused tests for meaningful new logic; execute checks only under AGENTS
  milestone/explicit-request policy. No automatic UI launch or live testing.
  Host lifecycle always needs immediate explicit permission.
- Implement directly. Record a concrete blocker once, take another safe GUI slice,
  and report implemented behavior and deferred validation concisely.
