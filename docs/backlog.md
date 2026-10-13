# QA backlog

Follow the [QA roadmap](QA_ROADMAP.md). These are planned test tasks, not completed results or authorisation to run live operations. Defects use separate BUG IDs in [qa/INDEX.md](../qa/INDEX.md).

| ID | Stage | Task / expected result | Implementation | Validation |
|---|---|---|---|---|
| QA-001 | Prepare | Record installed build, host/GPU versions, prepared VMs, reset method and approved effects | Ready | Not run |
| QA-002 | Core | Open/close app; discover and select the correct VM/GPU; refresh without losing selection | Blocked: QA-001 | Not run |
| QA-003 | Core | Complete initial setup through review, enrolment, Apply and Verify; confirm guest health and agreed final power state | Blocked: QA-002 | Not run |
| QA-004 | Core | Change VRAM within 1 GB–selected-card capacity; disable/re-enable GPU; confirm saved and observed state without repeating setup | Blocked: QA-003 | Not run |
| QA-005 | Failure/UI | Check Verify, repair and profile removal against missing drivers, stale state and missing VM/GPU; show truthful results and supported next steps | Blocked: QA-004 | Not run |
| QA-006 | Failure/UI | Cancel review/credentials and exercise authorised interrupted-work recovery; retain unresolved warnings without silent replay | Blocked: QA-004 | Not run |
| QA-007 | Failure/UI | Check keyboard focus, scrolling, long text, minimum window size, DPI/theme and wording; keep actions readable and reachable | Blocked: QA-002 | Not run |
| QA-008 | Failure/UI | Observe periodic refresh and operation progress; check responsiveness, repeated actions and useful diagnostics | Blocked: QA-002 | Not run |
| QA-009 | Acceptance | Retest fixed bugs on the installed candidate and run the existing non-hardware gate; record exact coverage and failures | Blocked: QA-005–QA-008 | Not run |
| QA-010 | Acceptance | Review P0/P1 blockers, deferred issues and untested scenarios; record the user's acceptance decision | Blocked: QA-009 | Not run |

## Next action

Start with **QA-001** when requested. After each session, submit observations to `qa/inbox/`; update task validation with the actual outcome and evidence. Source fixes and automated passes remain separate from user GUI acceptance.
