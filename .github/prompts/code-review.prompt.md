---
name: code-review
description: Review a scoped change for correctness and product-contract drift.
---

Read [AGENTS.md](../../AGENTS.md), the relevant [backlog](../../docs/BACKLOG.md) card, [architecture](../../docs/ARCHITECTURE.md), and [engineering](../../docs/ENGINEERING.md). Apply [configuration](../../docs/CONFIGURATION.md) and [GUI guide](../../docs/GUI_GUIDE.md) where relevant.

- Inspect the actual diff and related source. Preserve user changes; do not assume old descriptions or completion claims are current truth.
- Look for concrete correctness problems, stale target/plan approval, lost drafts, intent/observation confusion, unsafe retries, broken compatibility, and misleading outcomes.
- Check meaningful decision/failure coverage and whether claimed validation actually ran. Respect milestone timing in [testing](../../docs/TESTING.md).
- Report actionable findings by severity with source references, triggering conditions, and consequences. Separate confirmed defects from open questions.
- If no findings are established, state that plainly with material validation limits. Static review does not prove live support.

Do not edit files, run tests/builds, launch the UI, or perform live operations unless the current request separately authorizes that work.
