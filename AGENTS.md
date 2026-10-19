# Agent instructions

Applies to Codex and Claude Code. Complete one requested task; current user instructions take precedence. This file owns agent behavior. Specialist prompts add guidance, not authorization.

## Work

- Inspect working-tree changes and relevant source first; preserve unrelated edits/deletions. Use targeted searches rather than reading the entire repository.
- Follow the requested task. For backlog work, read its card and relevant roadmap gate. An open card alone does not authorize execution.
- QA reports are temporary user-owned inputs. Do not create or maintain QA reports, case files, indexes or report archives unless explicitly requested. Summarise fixes/checks in chat; respect report deletions and repair stale references without recreating reports.
- Do not add ad hoc debugging, QA, test or synthetic recovery files to the installed app or its live data directories. Never use live app state as test fixtures or copy runtime recovery files into the project. Use isolated temporary directories for test inputs.
- For app installs/updates, use a clean reset of app-owned runtime data, enrollment/runner registration, saved profiles/history and user preferences rather than carrying previous app state forward. Confirm the app is closed and no operation is active before resetting; stop on active work. Keep the built app binaries, source and VM/guest state separate from this cleanup. Do not reset data on ordinary app launches. Install only with an official release Inno installer or a locally built Inno installer, including through `cargo run -p app-install`; pass `--yes` to the source launcher only once the user has authorized installation. Do not hand-delete installed files, tasks or state. Official GitHub Releases contain an unsigned binaries ZIP and unsigned Inno installer, prepared and verified by the release workflow. See [Release packaging](docs/releases.md).
- Reuse existing components. Keep changes cohesive; no unrelated refactors, dependency upgrades, or restored tooling. Delegate only when explicitly requested.
- Read only applicable guidance below. Requirements belong in project docs, not prompts.
- Work autonomously on ordinary user-mode work (edits, builds, tests, lints, git status/diff/commit) without asking. Ask before elevation (sudo, UAC/RunAs), machine-wide changes (registry, services, scheduled tasks, Windows features), pushes or history rewrites, and the gated effects under [Boundaries](#boundaries).
- Do not use Python for scripts or tooling unless no reasonable alternative exists; prefer Rust, PowerShell, shell or Node, and say why if Python is unavoidable.

## Honest engineering (no cheating)

- Fix warnings, lints and failing tests at their cause. Do not silence them: no new `#[allow]`/`#[expect]`/`cfg_attr(..., allow)`, lint-level downgrades in `Cargo.toml`/`clippy.toml`, `-A`/`--cap-lints` flags, `#[ignore]`d or deleted tests, weakened assertions, or `todo!`/`unimplemented!` stubs without explicit user approval.
- A suppression is acceptable only when the lint is a genuine false positive or the code shape is deliberate; it must be the narrowest scope, carry `reason = "..."`, and be raised with the user. New `unsafe` needs a `// SAFETY:` comment.
- Do not game checks: no special-casing test inputs, hardcoded expected values, swallowed errors (`let _ =`, `.ok()`, empty `match` arms) to make something compile or pass, or `unwrap`/`expect` on expected runtime failures.
- Before reporting Rust work done, run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` on the result and fix what they report. Report honestly what was run, what failed and what was not verified; never claim a check passed that was not run.
- `.claude/hooks/lint-guard.mjs` enforces part of this by prompting the user; do not edit or bypass it to avoid the prompt.

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
| Preparing or publishing a release | [Release](.github/prompts/release.prompt.md) |

## Boundaries

- Keep the existing Slint design, shared Rust core, and protected operation architecture. Retired mock/snapshot GUI tooling is not used or maintained.
- Source/docs/tooling changes within scope proceed without repeated permission. Live GUI launches, guest access, VM/GPU effects, privileged setup/enrollment, protected writes, and driver changes require explicit authorization for prepared targets, effects, and evidence.
- Use local credential prompts and explicit vault opt-in; never request/log/store passwords in chat or project files.
- Uncertain effects retain evidence/recovery. Never blindly replay work, clear holds, force power-off, or restart the host.
