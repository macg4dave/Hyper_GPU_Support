# Engineering workflow

This page owns task/status conventions, documentation maintenance and [validation boundaries](#validation-and-live-acceptance). [AGENTS.md](../AGENTS.md) owns agent behavior. Use [backlog](backlog.md#next-implementation-action) for work selection within the current user request.

## One task at a time

Start by inspecting current source and user changes. Confirm the requested task's objective and dependencies, then make the smallest coherent change that satisfies its acceptance criteria. Existing implemented paths should be audited and corrected where needed, not rewritten to match an old proposal.

Do not automatically continue into another backlog card. Directly related fixes needed to complete the requested task belong in that task. Scope changes affecting product direction, authority, interface design, or release requirements require discussion and a recorded decision.

Update affected documentation and backlog status as part of the change. Give concise progress updates during longer work. Do not create a new planning file for each conversation or duplicate the same invariant across several prompt files.

## Status and evidence

Use separate implementation and validation fields:

| Implementation status | Meaning |
|---|---|
| Ready | Dependencies satisfied; work may begin when requested |
| In progress | Current requested task is being handled |
| Blocked | A named prerequisite or necessary decision is missing |
| Done | Task deliverables and its stated acceptance criteria are met; validation field still states exactly what ran |
| Deferred | Outside the agreed release or waiting for a new requirement |

Validation is recorded as **Not run**, **Pending milestone gate**, **Passed** with evidence/scope, or **Failed** with the failing stage. Use **Unknown** for capability behavior that cannot be established from existing evidence. Source assessment separately uses Implemented/Incomplete/Unknown; see [baseline](ARCHITECTURE.md).

A code task can be implementation Done with validation Pending milestone gate when its criteria permit deferred checks. It cannot close a milestone whose gate has not passed. A documentation task can complete through static checks without a build. Never label a failure, an existing test function, or a user-facing simulation as a passing runtime result.

Evidence for a gate/run records revision and working-tree context, exact commands/scenarios, environment, timestamps, outcomes, and exclusions. Use ignored `local/evidence/` for raw non-secret output. Only promote concise reviewed summaries into tracked documentation when necessary. Do not fabricate an evidence link before its file exists.

## Documentation ownership

| File | Authority |
|---|---|
| [README](../README.md) | Product overview and navigation |
| [AGENTS.md](../AGENTS.md) | Codex behavior and authorization |
| [Rust guidance](../.github/prompts/rust.prompt.md), [Slint guidance](../.github/prompts/slint.prompt.md) | Focused coding practices, loaded for applicable source work |
| [Roadmap](roadmap.md) | Release requirements, sequence, milestone gates |
| [Backlog](backlog.md) | Task IDs, dependencies, implementation/validation status, next action |
| [UI text guide](UI_TEXT_GUIDE.md) | Catalogue schema, formatting/generation contract and editor handoff |
| [UI text inventory](UI_TEXT_INVENTORY.md) | Presentation consumer/classification audit and extraction coverage |
| [UI text baseline](UI_TEXT_BASELINE.md) | Final extraction mapping, historical output fixtures and semantic regression requirements |
| [Architecture](ARCHITECTURE.md) | Components, state separation, data flow |
| [Configuration](CONFIGURATION.md) | Production intent schemas, storage, compatibility |
| [Agreed workflow](../README.md#agreed-workflow) | Existing user workflow |
| [Security](SECURITY.md) | Authority, trust, recovery semantics |
| [Validation](#validation-and-live-acceptance) | Check effects, gate evidence and live acceptance boundaries |
| `.github/prompts/*.prompt.md` | Focused task/coding guidance routed by AGENTS.md; no duplicated product requirements |

Change the authoritative page, then update links/summaries that need it. Source behavior outranks stale descriptions; current user decisions outrank historical plans. Keep machine-specific values in explicit local configuration/evidence and general policy in the handbook. Old task IDs remain context only; new IDs are stable once assigned.

The root agent file routes Codex to only the applicable specialist/task guidance. Supporting prompt files are read through that explicit routing; their filenames are not an automatic Codex loading mechanism. Keep operational guidance concise and product requirements in their owning pages.

## Tooling and change hygiene

Rust is product code. Narrow PowerShell development scripts may orchestrate Cargo, documentation checks, and explicitly authorized diagnostics. They must not become alternate product logic. Keep the default quality gate non-elevated and non-hardware; do not restore old scripts, CI, agents, or laboratory code wholesale.

Preserve user edits/deletions; do not reset/clean the working tree or stage unrelated changes. No dependency upgrades, format-all rewrites, or generated artifact updates belong in a documentation reset. Current toolchain versions come from manifests, not memory. Reassess removed CI and helpers individually with a concrete purpose and effect review.

## Validation and live acceptance

Run `powershell -NoProfile -File scripts/testing/check.ps1` from the repository root at integration/migration checkpoints. Inspect the script and test effects first. The gate requires a non-elevated shell and records revision, working tree, input hashes, commands and stage results in `local/evidence/`. It runs catalogue failure tests, resource validation, copy scanning, documentation, formatting, non-hardware tests, lint, binary builds and API documentation. Stop on a failing stage; remaining stages are Not run. The explicit resource rebuild proof is `python -B scripts/testing/test-ui-text-rebuild.py`; it temporarily changes prepared resources and restores them in `finally`.

These checks do not authorize a live application launch, native credential prompt, guest access, installed runner/setup, protected writes, VM power transitions or GPU/driver effects. Prepare exact targets, effects, evidence destination and stop conditions before requesting explicit authorization. Use local credential prompts and opt-in storage. Retain uncertainty and recovery evidence; do not blindly replay, clear holds or force power-off.

For UI text acceptance, prepare representative labels, dialogs/consent, progress, accessibility and long content at the supported minimum size, recorded scaling and themes. Record wrapping/clipping, reachable actions, keyboard focus and scrolling, with revision and actual coverage. Rendering does not establish guest or VM behavior. Keep L3/L4 open while their required visual/interaction checks are unrun. Existing source and historical acceptance are separate from new evidence.

## Completion report

State the behavior/files changed, why, exact validation performed or deferred, relevant uncertainty, and next task. Keep the report short, but disclose failures and untested product changes. Update milestone status only from its gate evidence. Publishing, pushing, deployment, and live qualification require their own applicable user instruction.
