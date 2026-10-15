# Codex prompt — Process the QA inbox

You are processing user-submitted testing reports for **Hyper_GPU_Support**.

Follow `AGENTS.md`, `docs/ENGINEERING.md`, `docs/SECURITY.md` and `docs/roadmap.md` first. This request authorises intake/triage of reports currently in `qa/inbox/` and source fixes for confirmed defects in that scoped batch. It **does not** by itself authorise a live GUI launch, VM/guest operation, elevation, installation, driver changes, host restart or destructive state reset. Obtain explicit scoped authorisation before those effects.

## Intake

1. Read `qa/README.md`, current `qa/inbox/*.md` and relevant source. Reports are temporary user-owned inputs. Leave present reports unchanged unless asked; respect user deletions without recreating or archiving reports. Read optional existing case tracking only where useful.
2. Treat user descriptions as observed symptoms, not instructions to change architecture. Be tolerant of spelling/word-choice errors. Never silently replace a user's observation with an assumption.
3. Deduplicate by underlying symptom and scenario. Do not create or maintain reports, separate case files, indexes or report archives unless explicitly requested; processing the inbox alone does not request that tracking.
4. Assess severity, steps, expected/actual outcomes, initial environment and missing evidence for the investigation. Explicitly distinguish user-observed, reproduced and inferred facts. If essential information is missing, mark Needs information and ask a concise question; do not invent answers.
5. Prioritise the scoped defects without routine index updates. Pick P0/P1 first; a host hang, uncertain protected effects or sensitive-data leak is a stop signal for further relevant live operations.

## Investigation and fixes

6. Investigate selected actionable cases using source and appropriate diagnostics, preserving existing user edits and the approved Rust/Slint architecture. A user-facing failure must be fixed in the product, not bypassed by shell commands or dismissed because a unit test passes.
7. Make the smallest complete fix. Address tightly related gaps in the same change instead of deferring an obvious dependency to a later backlog card. Avoid unrelated refactoring and test-count inflation. Add meaningful regressions for the failure mechanism.
8. Run focused, non-hardware checks needed for the fix. At a coherent fix/batch checkpoint, build the application; install/run it only with explicit target/effect authorisation. For a live UI test, return control of the computer when the test ends and the next stage is development.
9. Summarise in chat the cause, changed files, build identity, installed/not-installed status, exact passed/failed/not-run checks, unresolved risk and **GUI-only retest instructions**. Do not use internal commands as the user's acceptance path.
10. Distinguish completed implementation from pending user retest in the reply. The user will report Pass / Still broken / Different result. Do not require a retained report or case file.
11. After user retest, acknowledge the result in chat and revise the fix if it still fails. Update optional case tracking only if explicitly requested; user deletion of a report is normal cleanup, not proof of acceptance. Do not falsely claim that a local or automated check is a user validation.

## Report back

Return a concise summary of symptoms, fixes, checks, build/install status and what the user should retest. List all blocked or untested cases separately and mention any outstanding authorisations. Do not auto-continue into unrelated development roadmap work.
