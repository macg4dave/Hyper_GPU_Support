# Codex instructions

This is an AI-led coding repository. The user sets product direction and authorizes work; Codex implements one requested task at a time. These are the authoritative repository instructions for Codex. Supporting prompts must reference them, not introduce competing rules.

## Start here

1. Read [README](README.md), [roadmap](docs/ROADMAP.md), and [next implementation action](docs/BACKLOG.md#next-implementation-action).
2. Inspect the working tree and the relevant source before proposing edits. Preserve existing user changes and deletions.
3. Read the handbook pages relevant to the task. Use the [source baseline](docs/BASELINE.md) as an index, then verify its claims against current code.
4. If the user names a task, follow that task. Otherwise select the next ready task from the backlog within the user's requested scope. A backlog entry alone does not authorize execution.

## Product boundaries

- Keep the existing Slint design. Complete behavior through the shared Rust core; do not create a replacement GUI, alternate configuration schema, or parallel effect implementation without an agreed change in direction.
- The first release targets existing Generation 2 Windows 11 VMs and an NVIDIA GPU, with at least two guests rendering on hardware concurrently. Exact host/guest versions are selected before qualification. Resource isolation is not a release promise.
- Preserve the separation between observed state, saved intent, drafts, enrollment authority, verification, and recovery. See [architecture](docs/ARCHITECTURE.md) and [configuration](docs/CONFIGURATION.md).
- Product logic belongs in Rust. The existing PowerShell Direct transport is an explicit bridge, not evidence that the product is entirely native. Narrow development helpers may orchestrate tools; do not put a second product backend in scripts.
- Respect the protected runner, fixed elevated worker/setup modes, authenticated IPC, artifact checks, ACL checks, fresh plan validation, operation locking, and durable recovery. See [security and recovery](docs/SECURITY.md).
- Mock and snapshot routes must remain explicit and no-write. Simulated success does not establish live support.

## Authorization and uncertainty

- Repository source, documentation, and local development tooling changes within the requested task can proceed without repeated permission questions.
- Obtain explicit authorization before live VM/GPU operations, guest access, privileged setup/enrollment, protected-store writes, driver changes, or hardware qualification. Before requesting authorization, prepare a concrete operation sequence and identify targets, effects, and evidence. Authorization persists for its stated scope.
- A GUI launch can initiate elevated first-install discovery. Do not treat launching the live application as a harmless visual check. Use an explicitly authorized mock preview for appearance checks.
- Do not request passwords in chat or put them in files, logs, configuration, or persistent recovery. Use the product's local credential prompts and explicit vault opt-in.
- On timeout, disconnect, missing terminal result, or uncertain mutation, retain evidence and recovery. Never automatically replay GPU work, rerun hardware qualification, restart the host, or clear a hold to make progress.
- Do not restore deleted files simply because old guidance refers to them. Restore or replace individual tools only within a task that justifies doing so.
- Use sub-agents only when the user explicitly requests delegation or parallel agent work.

## Task and validation workflow

- Make a coherent change for one requested task; do not consume the rest of the backlog automatically. Fix directly related issues needed to complete that task.
- Follow [engineering](docs/ENGINEERING.md) for statuses and completion reporting. Record implementation and validation separately in the backlog.
- Normally run builds and automated tests at an agreed milestone's completion, or when the user explicitly requests them sooner. A task or subtask is not a milestone. Do not run checks after every small edit or redefine a milestone to justify doing so.
- Follow [testing](docs/TESTING.md) for the intended gate and hardware scenarios. Check that a command exists and has the expected effects before using it. Never report a deleted script or an unexecuted command as a passing check.
- Documentation-only work may use link, dependency, consistency, and diff checks without compiling or launching the product.
- Add tests for meaningful behavior and failure boundaries where needed; reuse useful existing coverage. Defer execution according to the validation policy and label untested changes plainly.
- Finish by reporting what changed, what checks actually ran, remaining uncertainty, and the next task. Update the backlog and affected handbook pages; update roadmap gates only when their criteria are met.

## Instruction precedence and documentation ownership

Current user instructions take precedence over repository guidance. `AGENTS.md` owns agent behavior, [roadmap](docs/ROADMAP.md) owns release requirements and milestone gates, [backlog](docs/BACKLOG.md) owns task order/status, and each handbook page owns its subject. See the [documentation ownership table](docs/ENGINEERING.md#documentation-ownership). Historical notes and Git history are context, not current task authority or proof of current behavior.
