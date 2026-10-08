# Codex Prompt — Hyper GPU Support Slint GUI

**Status:** Planning / repository audit only  
**Companion specification:** [`GUI_GUIDE.md`](GUI_GUIDE.md) · [`GUI_ROADMAP.md`](GUI_ROADMAP.md)  
**Do not begin implementation until explicitly instructed.**

## Prompt to give Codex in VS Code

You are reviewing an early-development **Rust / Windows Hyper-V GPU-P application**. We have chosen **Slint** for a new, maintainable GUI designed to work well with VS Code and AI-assisted development.

Read `GUI_GUIDE.md` and `GUI_ROADMAP.md` before proceeding. Treat recorded UX and architecture decisions A18–A32 as requirements and remaining `OPEN-*` items as technical questions. Do not make independent product decisions that contradict the guide.

### Important context

- The existing `windows_gui.rs` is an **experiment**. It can be discarded. **Do not plan a Win32 port, feature-parity project, or preservation of its layout/state/event architecture.** It is only optional evidence for how a few backend calls used to be wired up.
- The repository itself is early in development. You may recommend refactoring Rust modules, changing internal contracts, or revising the configuration schema if that produces a cleaner solution. Do not assume old internals are sacred.
- This does **not** authorise destroying real VM configurations, replacing persistent user files, altering Hyper-V settings, or executing privileged operations.
- **One executable:** no args opens Slint GUI; explicit CLI commands run headlessly; a restricted internal mode runs one elevated worker per approved operation. GUI and CLI share one application/domain core.
- **Per-VM configs:** files keyed by Hyper-V GUID under ProgramData; worker owns post-readback writes and save-only retries; GUI drafts remain in memory.
- **IPC/coordination:** private local Named Pipes; one modifying GPU operation host-wide across all entry points; one GUI per Windows session.
- **Recovery:** stop on GPU failure, manual reconciliation, normal dashboard with persistent warning and blocked modifications; no automatic GPU rollback or retry.
- User-facing diagnostic history is session-only. Minimal durable interrupted-operation recovery state is a separate requirement.
- Priority is maintainability, responsive/reasonable layouts, clear debugging, and preventing the AI from producing an enormous GUI source file or scattered fixed-pixel positioning.

### Your task now: READ, REVIEW, PROPOSE — NO CODE

1. **Inspect the actual repository.** Read relevant workspace manifests, current Rust binaries/modules, configuration parser/schema, GPU/VM discovery, runner, workflow/planner, enrollment/permissions, guest credentials, tests, README, ROADMAP, BACKLOG and AGENTS/project instructions where present. Do not assume files exist. Use concrete paths when reporting findings.
2. **Compare real capabilities to `GUI_GUIDE.md`.** Identify what already exists, what is missing, what needs extension, and which assumptions in the guide are technically unsupported or inaccurate.
3. **Propose a compact modular architecture.** Distinguish Slint components, Rust GUI state/controller/presenters, shared Rust application/core, Windows-specific privilege boundary and recovery. Prefer cohesive modules, not one giant file and not dozens of micro-files. Show the proposed file/module tree as a plan only.
4. **Review layout feasibility in current Slint.** Check how to implement a persistent side-by-side split, draggable divider, horizontal scrolling for narrow windows, independent vertical scrolling, virtualised VM cards, theme handling and accessibility. Flag any limitations or required custom components; don't invent an API.
5. **Review GPU-P feasibility on the target Windows host.** Confirm that GPU discovery, selection, all twelve allocation fields, host-wide partition count, driver preparation and privileged enrollment can be supported by existing backend mechanisms or documented Hyper-V APIs. Label each feature as supported, conditional, requires experiment, or unsupported; do not fabricate units or semantics.
6. **Review operation integrity.** Validate per-operation elevation, private Named Pipe trust boundaries, cross-process host-wide lock, fresh preview, guest shutdown approval, manual recovery after partial success, per-VM ProgramData save/write permissions and CLI/GUI conflicts.
7. **Find gaps, unnecessary complexity, and easy wins.** Challenge the guide constructively. Prefer the simplest approach that still meets the user decisions. Explain when a proposed abstraction, log, service, test layer or compatibility mechanism would be unnecessary.
8. **Recommend next steps.** Supply small, independently verifiable milestones and a mock-backed Slint UI proof-of-concept plan. Identify the decisions the user must make before implementing each stage.

### Mandatory output format

Give a concise but substantive review with:

1. **Repository reality check:** existing relevant modules/contracts with concrete file references; missing capabilities and uncertain facts.
2. **Gap matrix:** requirement → current support → risk → proposed approach.
3. **Proposed modules:** Slint/Rust responsibility boundaries, with rationale, and no large monolithic source file.
4. **Slint tooling/setup recommendation:** VS Code extension, official Codex/Slint integration, design/live preview, and test strategy; cite current upstream docs or state what you couldn't verify.
5. **Architecture risks and wins:** ranked by importance, not by the number of files created.
6. **Open technical details:** use the current `OPEN-*` IDs from `GUI_GUIDE.md`, distinguish unresolved implementation from settled policy A18–A32, and propose options without silently selecting one.
7. **Roadmap review:** assess `GUI_ROADMAP.md`, suggest ordered milestone changes and acceptance criteria; do not implement.
8. **Suggested amendments to `GUI_GUIDE.md`:** a short proposed change list or patch preview, clearly distinguished from confirmed user decisions.

If evidence is incomplete, say what is missing and propose how to check it. Prefer verified facts to confident guesses. Do not bury risks under generic explanations.

### Hard restrictions for this initial task

- **NO code generation or source edits.** Do not create or modify `.rs`, `.slint`, Cargo manifests, scripts or configuration files.
- **NO build, install, host reboot, VM power change, live GPU-P alteration, credential access, privileged helper invocation or Windows UAC trigger.** Inspect only unless subsequently authorised.
- Do not delete, migrate or rewrite existing project data.
- Do not add dependencies, start a service or modify external repositories.
- Do not silently resolve an `OPEN-*` item or assume consent for host/VM actions.
- You may identify flaws in the guide; ask for approval before editing this guide or committing decisions.
- Do not add arbitrary compatibility work for the throwaway Win32 GUI.
- No arbitrary file-size quota, no tests-of-tests, and no excessive instrumentation.

### When implementation is separately approved later

These become development requirements, **not an instruction to begin now**:

- Work in small vertical slices with mocks first, confirming every milestone before moving to riskier backend operations.
- Use Slint layout containers and constraints, not hard-coded x/y positioning.
- Keep presentation state distinct from observed Hyper-V state and from one unapplied VM draft.
- Never execute a different operation from the fresh, authorised Review & Apply plan.
- Request on-demand elevation and separate graceful VM shutdown approval before disruptive operations.
- Report real progress events; never infer success from a timeout or blindly retry a possibly completed stage.
- Use the same backend validation, commands and safety rules from GUI and CLI.
- Verify mock UX, Windows DPI/scaling, narrow horizontal scrolling, keyboard access, error states and renderer fallback.
- Require explicit permission for any real VM or host test that changes state.

**Stop after the review.** Your immediate deliverable is an accurate, actionable architectural proposal and a list of questions to resolve together, not a GUI implementation.

---

## How this prompt is intended to be used

1. Add `GUI_GUIDE.md`, `GUI_ROADMAP.md` and `GUI_PROMPT.md` to the repository (location can be decided with Codex).
2. Give Codex the instructions above, or tell it: **“Read `GUI_PROMPT.md` and follow its planning-only task.”**
3. Review Codex's findings together, resolve the open questions, and update the guide.
4. Only then create a separate, explicitly approved implementation task for the first milestone.
