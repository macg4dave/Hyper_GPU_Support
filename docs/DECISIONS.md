# Project decisions

These choices were confirmed in the product Q&A on 11 October 2026. They replace the previous planning assumptions. [Roadmap](ROADMAP.md) owns release requirements; [backlog](BACKLOG.md) owns implementation status.

| ID | Decision | Consequence |
|---|---|---|
| D-001 | Simple GPU support GUI for existing Hyper-V VMs | NVIDIA hardware and existing Generation 2 Windows 11 guests are the initial target; multiple VMs may share one GPU |
| D-002 | Hyper-V owns VM creation and general management | This app controls GPU support and GPU VRAM; it does not manage guest system RAM |
| D-003 | Keep the existing Slint design and shared Rust core | Complete behavior through existing architecture; no replacement GUI or parallel backend |
| D-004 | Retire legacy mock/snapshot GUI machinery | Remove through APP-001; do not use it or maintain its scenarios; backend unit test doubles are distinct |
| D-005 | VRAM range is 1 GB to selected card capacity | Range changes with hardware; only the toggle disables support; no balancing or full-capacity warnings now |
| D-006 | Review before effects | Edits stay pending; first Apply shows target, changes, timing warnings and power transitions; second Apply confirms |
| D-007 | App manages required VM power transitions | Explain starts, shutdowns, restarts and expected final state in review |
| D-008 | Initial setup verifies GPU operation | Preparation and verification report pass/fail for the selected VM; completed setup ends with the VM off |
| D-009 | Later changes are simple | Toggle/VRAM changes use review and Apply without repeating full setup or initial rendering verification |
| D-010 | Friendly explanation before local credentials | Name the VM and explain guest Windows sign-in versus host credentials; remembering is optional |
| D-011 | Preserve protected operations and truthful recovery | Simplifying workflow does not remove authorization, fresh plan checks, trust, IPC, locks, readback or durable records |
| D-012 | Documentation rewrite first | No source change, product build/test, launch, or live operation is authorized by this Q&A |

## Open details

The precise final power policy for ordinary changes is still to be settled in APP-006. The final-off decision applies to completed initial setup. Exact qualification environment and live provider behavior are established through APP-009.

VRAM display/request is not proof of physical reservation or isolation. Do not introduce extra policy or assume provider semantics.

## Work policy

Follow [AGENTS.md](../AGENTS.md): one requested task at a time, no automatic backlog sweep, and builds/tests normally at milestone completion or on explicit request. Live actions need a concrete sequence and separate authorization. No passwords in chat or persistent evidence.
