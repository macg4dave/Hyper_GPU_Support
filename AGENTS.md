# Agent instructions

Applies to Codex and Claude Code. Complete one requested task; current user instructions take precedence. This file owns agent behavior. Specialist prompts add guidance, not authorization.

## Work

- Inspect working-tree changes and relevant source first; preserve unrelated edits/deletions. Use targeted searches rather than reading the entire repository.
- Follow the requested task. For backlog work, read its card and relevant roadmap gate. An open card alone does not authorize execution.
- QA reports are temporary user-owned inputs. Do not create or maintain QA reports, case files, indexes or report archives unless explicitly requested. Summarise fixes/checks in chat; respect report deletions and repair stale references without recreating reports.
- Do not add ad hoc debugging, QA, test or synthetic recovery files to the installed app or its live data directories. Never use live app state as test fixtures or copy runtime recovery files into the project. Use isolated temporary directories for test inputs.
- For app installs/updates, use a clean reset of app-owned runtime data, enrollment/runner registration, saved profiles/history and user preferences rather than carrying previous app state forward. Confirm the app is closed and no operation is active before resetting; stop on active work. Keep the built app binaries, source and VM/guest state separate from this cleanup. Do not reset data on ordinary app launches.
- Reuse existing components. Keep changes cohesive; no unrelated refactors, dependency upgrades, or restored tooling. Delegate only when explicitly requested.
- Read only applicable guidance below. Requirements belong in project docs, not prompts.

## Load when relevant

| Work | Guidance |
| --- | --- |
| Rust implementation/review | [Rust](.github/prompts/rust.prompt.md) |
| Slint, Rust UI wiring or live GUI testing | [Slint](.github/prompts/slint.prompt.md) |
| Implementation | [Implement](.github/prompts/implement-task.prompt.md) |
| Planning | [Plan](.github/prompts/task-plan.prompt.md) |
| Code review | [Review](.github/prompts/code-review.prompt.md), [Rust review](.github/prompts/rust-review.prompt.md) |
| Privilege, IPC, effects, recovery | [Security review](.github/prompts/security-recovery-review.prompt.md) |
| Documentation | [Docs](.github/prompts/maintain-docs.prompt.md) |

[Backlog](docs/backlog.md) owns tasks; [roadmap](docs/roadmap.md) owns scope/gates. Consult the [agreed user workflow](README.md#agreed-workflow) only as needed.

## Boundaries

- Keep the existing Slint design, shared Rust core, and protected operation architecture. Retired mock/snapshot GUI tooling is not used or maintained.
- Source/docs/tooling changes within scope proceed without repeated permission. Live GUI launches, guest access, VM/GPU effects, privileged setup/enrollment, protected writes, and driver changes require explicit authorization for prepared targets, effects, and evidence.
- Use local credential prompts and explicit vault opt-in; never request/log/store passwords in chat or project files.
- Uncertain effects retain evidence/recovery. Never blindly replay work, clear holds, force power-off, or restart the host.

## Validation and finish

Write meaningful regression and failure tests with behavior changes. Normally run builds/tests at agreed milestone completion or explicit request, not after individual tasks. Check commands exist and inspect test effects first; the [roadmap](docs/roadmap.md) owns the gate.

Update affected documentation and keep implementation and validation status separate. Report changes, checks actually run, untested behavior, and next task. Close gates only from evidence.
