# Project decisions

This log owns agreed choices and their consequences. Operational status belongs in [backlog](BACKLOG.md); source assessment belongs in [baseline](BASELINE.md). The following decisions were agreed with the user for the 2026-10-11 documentation reset. New decisions receive stable `D-*` IDs; do not recycle an ID after changing direction.

| ID | Decision | Consequence |
|---|---|---|
| D-001 | Keep the current Rust implementation; rebuild planning/guidance from current source | No wholesale code rewrite or inherited completion claims; inspect existing paths before scheduling implementation |
| D-002 | First deliver a usable GPU-PV GUI for existing Windows 11 guests and NVIDIA hardware | At least two guests must render on hardware concurrently; exact guest versions are decided before qualification |
| D-003 | Keep the existing Slint design | Correct behavior and necessary workflow gaps through the shared core; redesign needs a new agreement |
| D-004 | First-release sharing means checked concurrent graphics, not proven resource isolation | Document requests/readback honestly; fairness, enforced limits, and broader hardware support require separate evidence/requirements |
| D-005 | Codex is the supported coding agent, one requested task at a time | `AGENTS.md` is authoritative; six focused prompts reference it; no automatic backlog sweep or delegation |
| D-006 | Maintain a detailed but focused handbook | Each page owns one subject; prompts link to authorities instead of copying policy; machine values stay in local configuration/evidence |
| D-007 | Normally validate at milestone completion or on explicit request | A task is not a milestone; unrun implementation is labeled; documentation-only static checks are permitted |
| D-008 | Start fresh task IDs and source audit | `V1-*` tasks and current evidence replace old planning statuses; Git history is consulted only for a concrete issue |
| D-009 | Reassess deleted tooling individually | Existing deletions remain; no wholesale restoration of scripts, CI, tests, laboratory tools, or agent configuration |
| D-010 | This reset changes documentation only | No application/configuration/manifest changes, builds, GUI launches, or hardware tests |

## Recording later changes

For a later decision record the date, user-approved choice, reason, consequences, and affected requirements/tasks. Keep unresolved choices in their backlog card rather than inventing a decision. In particular, guest versions, representative workloads, observation duration, current development-config purpose, and optional CI are still open work under V1-002/V1-003/V1-014.
