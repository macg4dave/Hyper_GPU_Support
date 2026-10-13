# Codex instructions

Codex only. Complete one requested task; current user instructions take precedence. This file owns agent behavior. Specialist prompts add guidance, not authorization.

## Work

- Inspect working-tree changes and relevant source first; preserve unrelated edits/deletions. Use targeted searches rather than reading the entire repository.
- Follow the requested task. For backlog work, read its card and relevant roadmap gate. An open card alone does not authorize execution.
- Reuse existing components. Keep changes cohesive; no unrelated refactors, dependency upgrades, or restored tooling. Delegate only when explicitly requested.
- Read only applicable guidance below. Requirements belong in the handbook, not prompts.

## Load when relevant

| Work | Guidance |
|---|---|
| Rust implementation/review | [Rust](.github/prompts/rust.prompt.md) |
| Slint or Rust UI wiring | [Slint](.github/prompts/slint.prompt.md) |
| Implementation | [Implement](.github/prompts/implement-task.prompt.md) |
| Planning | [Plan](.github/prompts/task-plan.prompt.md) |
| Code review | [Review](.github/prompts/code-review.prompt.md) |
| Privilege, IPC, effects, recovery | [Security review](.github/prompts/security-recovery-review.prompt.md) |
| Documentation | [Docs](.github/prompts/maintain-docs.prompt.md) |
| Milestone validation | [Validate](.github/prompts/validate-milestone.prompt.md) |

[Backlog](docs/backlog.md) owns tasks; [roadmap](docs/roadmap.md) owns scope/gates. Consult [architecture](docs/ARCHITECTURE.md), [configuration](docs/CONFIGURATION.md), the [agreed user workflow](README.md#agreed-workflow), and [security](docs/SECURITY.md) only as needed.

## Boundaries

- Keep the existing Slint design, shared Rust core, and protected operation architecture. Retired mock/snapshot GUI tooling is not used or maintained.
- Source/docs/tooling changes within scope proceed without repeated permission. Live GUI launches, guest access, VM/GPU effects, privileged setup/enrollment, protected writes, and driver changes require explicit authorization for prepared targets, effects, and evidence.
- Use local credential prompts and explicit vault opt-in; never request/log/store passwords in chat or project files.
- Uncertain effects retain evidence/recovery. Never blindly replay work, clear holds, force power-off, or restart the host.

## Validation and finish

Write meaningful regression and failure tests with behavior changes. Normally run builds/tests at agreed milestone completion or explicit request, not after individual tasks. Check commands exist and inspect test effects first; [testing](docs/ENGINEERING.md#validation-and-live-acceptance) owns the gate.

Update affected documentation and separate implementation/validation status using [engineering](docs/ENGINEERING.md). Report changes, checks actually run, untested behavior, and next task. Close gates only from evidence.
