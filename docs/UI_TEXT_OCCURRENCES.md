# UI text occurrence register

Companion to [the reviewed inventory](UI_TEXT_INVENTORY.md). Source snapshot: 13 October 2026. The worker, live-controller and live-state links follow the current module layout; their displayed line numbers and source fingerprints retain the historical snapshot. For moved worker and controller producers, links point to the owning module rather than historical line anchors; the former `run` producer now spans lifecycle, editor, action, credential and recovery modules. These are source literals, including formatting fragments and immutable inputs, not evaluated outputs or a ready-to-use catalogue. Tests/comments are excluded. Repeated occurrences in one producer are grouped, with every line retained. The main inventory supplies consumer, branch conditions, parameter bindings, disposition rules and resolution paths.

Source spellings retain quotes, raw-literal delimiters, escapes and whitespace inside literals. HTML character references in this table protect Markdown rendering only. `{}` fields are positional arguments in source order; `{:?}` and numeric format specifications remain code-owned until LANG-002/003 resolve the typed contract. Slint concatenations are fragments of the enclosing expression, not independent whole messages. Keys are provisional semantic families with variant numbers; LANG-002/003 must assign final meaning-specific variant names and consolidate only equivalent contexts.

Classes: C = editable copy; M = mixed copy/control requiring separation; P = immutable protocol/classifier/schema input; F = code-owned formatting/empty sentinel; X = justified non-copy or unconnected helper. Batches and exception reasons are defined in the main inventory.

## src/gui/actions.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [17](../src/gui/actions.rs#L17) | <code>reason</code> | <code>"verify"</code> | — | — | B2 | P |
| [18](../src/gui/actions.rs#L18) | <code>reason</code> | <code>"repair"</code> | — | — | B2 | P |
| [19](../src/gui/actions.rs#L19) | <code>reason</code> | <code>"remove-gpu"</code> | — | — | B2 | P |
| [20](../src/gui/actions.rs#L20) | <code>reason</code> | <code>"forget"</code> | — | — | B2 | P |
| [35](../src/gui/actions.rs#L35) | <code>selected_actions</code> | <code>"Select a VM first."</code> | — | <code>operations.selected_actions.variant_001</code> | B2 | M |
| [37](../src/gui/actions.rs#L37) | <code>selected_actions</code> | <code>"Use Refresh first."</code> | — | <code>operations.selected_actions.variant_002</code> | B2 | M |
| [40](../src/gui/actions.rs#L40) | <code>selected_actions</code> | <code>"This VM/GPU pair is not authorized. Close this panel and review first-time setup."</code> | — | <code>operations.selected_actions.variant_003</code> | B2 | C |
| [45](../src/gui/actions.rs#L45) | <code>selected_actions</code> | <code>"Review and Apply, or discard the pending edits before Verify."</code> | — | <code>operations.selected_actions.variant_004</code> | B2 | C |
| [51](../src/gui/actions.rs#L51) | <code>selected_actions</code> | <code>"VM unavailable; use Refresh."</code> | — | <code>operations.selected_actions.variant_005</code> | B2 | M |
| [89](../src/gui/actions.rs#L89) | <code>action_prerequisite</code> | <code>"Select a VM first."</code> | — | <code>operations.action_prerequisite.variant_001</code> | B2 | M |
| [91](../src/gui/actions.rs#L91) | <code>action_prerequisite</code> | <code>"An unfinished operation blocks new actions. Choose Review recovery; your edits are kept."</code> | — | <code>operations.action_prerequisite.variant_002</code> | B2 | M |
| [95](../src/gui/actions.rs#L95) | <code>action_prerequisite</code> | <code>"Current VM information is unavailable. Use Refresh; your edits are kept."</code> | — | <code>operations.action_prerequisite.variant_003</code> | B2 | C |
| [99](../src/gui/actions.rs#L99) | <code>action_prerequisite</code> | <code>"The operation outcome is not saved. Use Refresh and inspect recovery before another action."</code> | — | <code>operations.action_prerequisite.variant_004</code> | B2 | M |
| [108](../src/gui/actions.rs#L108) | <code>action_prerequisite</code> | <code>"Selected VM observations are incomplete. Correct access and use Refresh; your edits are kept."</code> | — | <code>operations.action_prerequisite.variant_005</code> | B2 | M |
| [113](../src/gui/actions.rs#L113) | <code>action_prerequisite</code> | <code>"The selected VM is unavailable. Use Refresh or discard its draft."</code> | — | <code>operations.action_prerequisite.variant_006</code> | B2 | M |
| [118](../src/gui/actions.rs#L118) | <code>action_prerequisite</code> | <code>"Managed operation history is unavailable. Use Refresh before reviewing new actions."</code> | — | <code>operations.action_prerequisite.variant_007</code> | B2 | M |
| [121](../src/gui/actions.rs#L121) | <code>action_prerequisite</code> | <code>"GPU support requires a Generation 2 VM. Close this panel and select a supported VM."</code> | — | <code>operations.action_prerequisite.variant_008</code> | B2 | M |
| [124](../src/gui/actions.rs#L124) | <code>action_prerequisite</code> | <code>"The VM is changing power state or is not running/off. Wait for a stable state, then Refresh."</code> | — | <code>operations.action_prerequisite.variant_009</code> | B2 | M |
## src/gui/diagnostics.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [6](../src/gui/diagnostics.rs#L6) | <code>normalized</code> | <code>" "</code> | — | — | B4 | F |
| [12](../src/gui/diagnostics.rs#L12), [13](../src/gui/diagnostics.rs#L13) | <code>completion_failure</code> | <code>"Original operation"</code> | — | <code>diagnostics.completion_failure.variant_001</code> | B4 | C |
| [16](../src/gui/diagnostics.rs#L16) | <code>completion_failure</code> | <code>"{summary}\n\n{original}\nCompletion records could not be confirmed. Use Refresh and inspect recovery before another operation."</code> | <code>summary, original</code> | <code>diagnostics.completion_failure.variant_002</code> | B4 | C |
| [22](../src/gui/diagnostics.rs#L22) | <code>completion_failure</code> | <code>"\n\n"</code> | — | — | B4 | F |
| [44](../src/gui/diagnostics.rs#L44) | <code>failure_details</code> | <code>"Native error codes: "</code> | — | <code>diagnostics.failure_details.variant_001</code> | B4 | M |
| [45](../src/gui/diagnostics.rs#L45) | <code>failure_details</code> | <code>"not available"</code> | — | <code>diagnostics.failure_details.variant_002</code> | B4 | M |
| [47](../src/gui/diagnostics.rs#L47) | <code>failure_details</code> | <code>"Component message: "</code> | — | <code>diagnostics.failure_details.variant_003</code> | B4 | M |
| [48](../src/gui/diagnostics.rs#L48) | <code>failure_details</code> | <code>"not available as a safe structured message"</code> | — | <code>diagnostics.failure_details.variant_004</code> | B4 | M |
| [55](../src/gui/diagnostics.rs#L55) | <code>failure_details</code> | <code>"password"</code> | — | — | B4 | P |
| [55](../src/gui/diagnostics.rs#L55) | <code>failure_details</code> | <code>"token"</code> | — | — | B4 | P |
| [55](../src/gui/diagnostics.rs#L55) | <code>failure_details</code> | <code>"secret"</code> | — | — | B4 | P |
| [55](../src/gui/diagnostics.rs#L55) | <code>failure_details</code> | <code>"credential"</code> | — | — | B4 | P |
| [55](../src/gui/diagnostics.rs#L55) | <code>failure_details</code> | <code>"bearer"</code> | — | — | B4 | P |
| [64](../src/gui/diagnostics.rs#L64) | <code>failure_details</code> | <code>"Source: this reported failure and its last reported stages; current VM state may require Refresh.\n"</code> | — | <code>diagnostics.failure_details.variant_005</code> | B4 | C |
| [69](../src/gui/diagnostics.rs#L69) | <code>failure_details</code> | <code>"\nLast reported stages (not proof of final state):\n"</code> | — | <code>diagnostics.failure_details.variant_006</code> | B4 | C |
| [70](../src/gui/diagnostics.rs#L70) | <code>failure_details</code> | <code>"\n"</code> | — | — | B4 | F |
| [108](../src/gui/diagnostics.rs#L108) | <code>selected_vm_details</code> | <code>"vm_id"</code> | — | — | B4 | P |
| [109](../src/gui/diagnostics.rs#L109) | <code>selected_vm_details</code> | <code>"observed_host_state"</code> | — | — | B4 | P |
| [110](../src/gui/diagnostics.rs#L110) | <code>selected_vm_details</code> | <code>"saved_intent_revision"</code> | — | — | B4 | P |
| [111](../src/gui/diagnostics.rs#L111) | <code>selected_vm_details</code> | <code>"saved_intent_read_error"</code> | — | — | B4 | P |
| [112](../src/gui/diagnostics.rs#L112) | <code>selected_vm_details</code> | <code>"pending_edits"</code> | — | — | B4 | P |
| [112](../src/gui/diagnostics.rs#L112) | <code>selected_vm_details</code> | <code>"Unapplied draft; see editor for current raw values"</code> | — | <code>diagnostics.selected_vm_details.variant_001</code> | B4 | C |
| [112](../src/gui/diagnostics.rs#L112) | <code>selected_vm_details</code> | <code>"No draft recorded"</code> | — | <code>diagnostics.selected_vm_details.variant_002</code> | B4 | C |
| [113](../src/gui/diagnostics.rs#L113) | <code>selected_vm_details</code> | <code>"saved_intent"</code> | — | — | B4 | P |
| [114](../src/gui/diagnostics.rs#L114) | <code>selected_vm_details</code> | <code>"authorized_pair"</code> | — | — | B4 | P |
| [115](../src/gui/diagnostics.rs#L115) | <code>selected_vm_details</code> | <code>"selected_gpu_interface"</code> | — | — | B4 | P |
| [116](../src/gui/diagnostics.rs#L116) | <code>selected_vm_details</code> | <code>"observed_gpu"</code> | — | — | B4 | P |
| [117](../src/gui/diagnostics.rs#L117) | <code>selected_vm_details</code> | <code>"inventory_diagnostics"</code> | — | — | B4 | P |
| [118](../src/gui/diagnostics.rs#L118) | <code>selected_vm_details</code> | <code>"recorded_managed_history"</code> | — | — | B4 | P |
| [119](../src/gui/diagnostics.rs#L119) | <code>selected_vm_details</code> | <code>"recorded_history_status"</code> | — | — | B4 | P |
| [122](../src/gui/diagnostics.rs#L122) | <code>selected_vm_details</code> | <code>"Current observation unavailable. Retained host observations/history below are prior reads, not current health or execution authority; use Refresh."</code> | — | <code>diagnostics.selected_vm_details.variant_003</code> | B4 | C |
| [124](../src/gui/diagnostics.rs#L124) | <code>selected_vm_details</code> | <code>"Source: latest host inventory and saved app records. No fresh guest check; setup/rendering history is historical, not current guest health."</code> | — | <code>diagnostics.selected_vm_details.variant_004</code> | B4 | C |
| [127](../src/gui/diagnostics.rs#L127) | <code>selected_vm_details</code> | <code>"Diagnostic formatting unavailable"</code> | — | <code>diagnostics.selected_vm_details.variant_005</code> | B4 | C |
| [129](../src/gui/diagnostics.rs#L129) | <code>selected_vm_details</code> | <code>"{source}\n{facts}\n\nHost comparison:\n{}\n\nApplied operation:\n{}\n\nVRAM values are exact byte requests/readback; provider values do not establish reserved memory or enforced isolation. Missing records and null fields mean unavailable/absent facts, not proof of success."</code> | <code>source, facts, arg3, arg4</code> | <code>diagnostics.selected_vm_details.variant_006</code> | B4 | C |
## src/gui/host_configuration.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [7](../src/gui/host_configuration.rs#L7) | <code>comparison</code> | <code>"Host configuration: Unknown; {reason}."</code> | <code>reason</code> | <code>operations.comparison.variant_001</code> | B2 | C |
| [9](../src/gui/host_configuration.rs#L9) | <code>comparison</code> | <code>"saved intent is unreadable"</code> | — | <code>operations.comparison.variant_002</code> | B2 | C |
| [12](../src/gui/host_configuration.rs#L12) | <code>comparison</code> | <code>"no saved intent"</code> | — | <code>operations.comparison.variant_003</code> | B2 | C |
| [15](../src/gui/host_configuration.rs#L15) | <code>comparison</code> | <code>"retained host observations are historical; Refresh to compare"</code> | — | <code>operations.comparison.variant_004</code> | B2 | C |
| [18](../src/gui/host_configuration.rs#L18) | <code>comparison</code> | <code>"host inventory is unavailable"</code> | — | <code>operations.comparison.variant_005</code> | B2 | C |
| [21](../src/gui/host_configuration.rs#L21) | <code>comparison</code> | <code>"VM was not returned by the latest host read"</code> | — | <code>operations.comparison.variant_006</code> | B2 | C |
| [24](../src/gui/host_configuration.rs#L24) | <code>comparison</code> | <code>"Preparation record"</code> | — | <code>operations.comparison.variant_007</code> | B2 | M |
| [27](../src/gui/host_configuration.rs#L27) | <code>comparison</code> | <code>"latest host observations are incomplete; inspect diagnostics"</code> | — | <code>operations.comparison.variant_008</code> | B2 | C |
| [40](../src/gui/host_configuration.rs#L40) | <code>comparison</code> | <code>"none"</code> | — | <code>operations.comparison.variant_009</code> | B2 | C |
| [42](../src/gui/host_configuration.rs#L42) | <code>comparison</code> | <code>", "</code> | — | — | B2 | F |
| [46](../src/gui/host_configuration.rs#L46) | <code>comparison</code> | <code>"GPU attachment: expected {}; observed {}"</code> | <code>arg1, arg2</code> | <code>operations.comparison.variant_011</code> | B2 | C |
| [57](../src/gui/host_configuration.rs#L57) | <code>comparison</code> | <code>"{} has no explicit saved request"</code> | <code>arg1</code> | <code>operations.comparison.variant_012</code> | B2 | C |
| [60](../src/gui/host_configuration.rs#L60) | <code>comparison</code> | <code>"{}: expected {}; observed Unknown"</code> | <code>arg1, arg2</code> | <code>operations.comparison.variant_013</code> | B2 | C |
| [62](../src/gui/host_configuration.rs#L62), [68](../src/gui/host_configuration.rs#L68), [69](../src/gui/host_configuration.rs#L69) | <code>comparison</code> | <code>"Unknown"</code> | — | <code>operations.comparison.variant_014</code> | B2 | C |
| [66](../src/gui/host_configuration.rs#L66) | <code>comparison</code> | <code>"{}: expected {}; observed {}"</code> | <code>arg1, arg2, arg3</code> | <code>operations.comparison.variant_015</code> | B2 | C |
| [92](../src/gui/host_configuration.rs#L92) | <code>comparison</code> | <code>"Low MMIO (MiB)"</code> | — | <code>operations.comparison.variant_016</code> | B2 | C |
| [97](../src/gui/host_configuration.rs#L97) | <code>comparison</code> | <code>"High MMIO (MiB)"</code> | — | <code>operations.comparison.variant_017</code> | B2 | C |
| [102](../src/gui/host_configuration.rs#L102) | <code>comparison</code> | <code>"Guest cache types"</code> | — | <code>operations.comparison.variant_018</code> | B2 | C |
| [107](../src/gui/host_configuration.rs#L107) | <code>comparison</code> | <code>"Automatic checkpoints"</code> | — | <code>operations.comparison.variant_019</code> | B2 | C |
| [114](../src/gui/host_configuration.rs#L114) | <code>comparison</code> | <code>"{name} (managed host baseline): expected {expected}; observed {observed}"</code> | <code>name, expected, observed</code> | <code>operations.comparison.variant_020</code> | B2 | C |
| [119](../src/gui/host_configuration.rs#L119) | <code>comparison</code> | <code>"managed compatibility baseline is unavailable or not applicable"</code> | — | <code>operations.comparison.variant_021</code> | B2 | C |
| [123](../src/gui/host_configuration.rs#L123) | <code>comparison</code> | <code>"Host configuration: compared values match the saved intent."</code> | — | <code>operations.comparison.variant_022</code> | B2 | C |
| [126](../src/gui/host_configuration.rs#L126) | <code>comparison</code> | <code>"Host configuration differences (saved intent / latest Hyper-V read): {}. Review changes before repair or GPU removal."</code> | <code>arg1</code> | <code>operations.comparison.variant_023</code> | B2 | C |
| [127](../src/gui/host_configuration.rs#L127), [131](../src/gui/host_configuration.rs#L131) | <code>comparison</code> | <code>"; "</code> | — | — | B2 | F |
| [131](../src/gui/host_configuration.rs#L131) | <code>comparison</code> | <code>" Unknown: {}."</code> | <code>arg1</code> | <code>operations.comparison.variant_025</code> | B2 | C |
## src/gui/live/presentation.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [35](../src/gui/live/presentation.rs#L35) | <code>vm_row</code> | <code>"No GPU attached"</code> | — | <code>operations.vm_row.variant_001</code> | B2 | C |
| [45](../src/gui/live/presentation.rs#L45) | <code>vm_row</code> | <code>"Attached GPU (identity unavailable)"</code> | — | <code>operations.vm_row.variant_002</code> | B2 | C |
| [50](../src/gui/live/presentation.rs#L50) | <code>vm_row</code> | <code>", "</code> | — | — | B2 | F |
| [56](../src/gui/live/presentation.rs#L56) | <code>vm_row</code> | <code>"Running"</code> | — | <code>operations.vm_row.variant_004</code> | B2 | M |
| [57](../src/gui/live/presentation.rs#L57) | <code>vm_row</code> | <code>"Stopped"</code> | — | <code>operations.vm_row.variant_005</code> | B2 | M |
| [58](../src/gui/live/presentation.rs#L58), [62](../src/gui/live/presentation.rs#L62) | <code>vm_row</code> | <code>"Unknown"</code> | — | <code>operations.vm_row.variant_006</code> | B2 | C |
| [64](../src/gui/live/presentation.rs#L64) | <code>vm_row</code> | <code>"Unsupported"</code> | — | <code>operations.vm_row.variant_007</code> | B2 | M |
| [68](../src/gui/live/presentation.rs#L68) | <code>vm_row</code> | <code>"unknown"</code> | — | — | B2 | P |
| [76](../src/gui/live/presentation.rs#L76) | <code>observed_row</code> | <code>"Recovery required"</code> | — | <code>operations.observed_row.variant_001</code> | B2 | C |
| [78](../src/gui/live/presentation.rs#L78) | <code>observed_row</code> | <code>"Authorized"</code> | — | <code>operations.observed_row.variant_002</code> | B2 | C |
| [83](../src/gui/live/presentation.rs#L83), [85](../src/gui/live/presentation.rs#L85) | <code>observed_row</code> | <code>"Status unavailable"</code> | — | <code>operations.observed_row.variant_003</code> | B2 | C |
| [84](../src/gui/live/presentation.rs#L84) | <code>observed_row</code> | <code>"Refresh needed"</code> | — | <code>operations.observed_row.variant_004</code> | B2 | C |
| [129](../src/gui/live/presentation.rs#L129) | <code>present_actions</code> | <code>"Partial GPU removal requires Windows."</code> | — | <code>operations.present_actions.variant_001</code> | B2 | C |
| [136](../src/gui/live/presentation.rs#L136) | <code>pending_summary</code> | <code>"VRAM fields unavailable"</code> | — | <code>operations.pending_summary.variant_001</code> | B2 | M |
| [140](../src/gui/live/presentation.rs#L140) | <code>pending_summary</code> | <code>"Pending: {} GPU support. VRAM request: {}. Review before applying."</code> | <code>arg1, arg2</code> | <code>operations.pending_summary.variant_002</code> | B2 | C |
| [141](../src/gui/live/presentation.rs#L141) | <code>pending_summary</code> | <code>"enable"</code> | — | <code>operations.pending_summary.variant_003</code> | B2 | C |
| [141](../src/gui/live/presentation.rs#L141) | <code>pending_summary</code> | <code>"disable"</code> | — | <code>operations.pending_summary.variant_004</code> | B2 | C |
| [143](../src/gui/live/presentation.rs#L143) | <code>pending_summary</code> | <code>"provider default"</code> | — | <code>operations.pending_summary.variant_005</code> | B2 | C |
| [147](../src/gui/live/presentation.rs#L147) | <code>pending_summary</code> | <code>"Pending edits retained. Correct the VRAM fields before reviewing."</code> | — | <code>operations.pending_summary.variant_006</code> | B2 | M |
| [153](../src/gui/live/presentation.rs#L153), [170](../src/gui/live/presentation.rs#L170), [223](../src/gui/live/presentation.rs#L223) | <code>present_inner</code> | <code>""</code> | — | — | B2 | F |
| [154](../src/gui/live/presentation.rs#L154), [157](../src/gui/live/presentation.rs#L157), [158](../src/gui/live/presentation.rs#L158), [159](../src/gui/live/presentation.rs#L159) | <code>present_inner</code> | <code>"No VM selected"</code> | — | <code>operations.present_inner.variant_002</code> | B2 | C |
| [155](../src/gui/live/presentation.rs#L155), [156](../src/gui/live/presentation.rs#L156), [177](../src/gui/live/presentation.rs#L177) | <code>present_inner</code> | <code>"Unknown"</code> | — | <code>operations.present_inner.variant_003</code> | B2 | C |
| [160](../src/gui/live/presentation.rs#L160), [188](../src/gui/live/presentation.rs#L188), [260](../src/gui/live/presentation.rs#L260) | <code>present_inner</code> | <code>"No unapplied edits"</code> | — | <code>operations.present_inner.variant_004</code> | B2 | C |
| [161](../src/gui/live/presentation.rs#L161) | <code>present_inner</code> | <code>"Unknown; no VM selected"</code> | — | <code>operations.present_inner.variant_005</code> | B2 | C |
| [162](../src/gui/live/presentation.rs#L162) | <code>present_inner</code> | <code>"Select a VM to read its state."</code> | — | <code>operations.present_inner.variant_006</code> | B2 | C |
| [176](../src/gui/live/presentation.rs#L176) | <code>present_inner</code> | <code>"Selected VM unavailable"</code> | — | <code>operations.present_inner.variant_007</code> | B2 | C |
| [178](../src/gui/live/presentation.rs#L178) | <code>present_inner</code> | <code>"Unavailable"</code> | — | <code>operations.present_inner.variant_008</code> | B2 | C |
| [180](../src/gui/live/presentation.rs#L180) | <code>present_inner</code> | <code>"Selected VM was not returned by the latest inventory read"</code> | — | <code>operations.present_inner.variant_009</code> | B2 | C |
| [181](../src/gui/live/presentation.rs#L181) | <code>present_inner</code> | <code>"Unknown; the selected VM is unavailable"</code> | — | <code>operations.present_inner.variant_010</code> | B2 | C |
| [186](../src/gui/live/presentation.rs#L186) | <code>present_inner</code> | <code>"Unapplied edits retained for the unavailable VM."</code> | — | <code>operations.present_inner.variant_011</code> | B2 | C |
| [198](../src/gui/live/presentation.rs#L198) | <code>present_inner</code> | <code>"The draft VM is no longer present. Discard the draft and Refresh."</code> | — | <code>operations.present_inner.variant_012</code> | B2 | C |
| [219](../src/gui/live/presentation.rs#L219) | <code>present_inner</code> | <code>"{}{}; {}; {}"</code> | <code>arg1, arg2, arg3, arg4</code> | — | B2 | F |
| [221](../src/gui/live/presentation.rs#L221) | <code>present_inner</code> | <code>"Historical: "</code> | — | <code>operations.present_inner.variant_014</code> | B2 | C |
| [230](../src/gui/live/presentation.rs#L230) | <code>present_inner</code> | <code>"{}: unobserved or provider default"</code> | <code>arg1</code> | <code>operations.present_inner.variant_015</code> | B2 | C |
| [233](../src/gui/live/presentation.rs#L233) | <code>present_inner</code> | <code>"{} readback: {} / {} / {} (raw)"</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.present_inner.variant_016</code> | B2 | C |
| [243](../src/gui/live/presentation.rs#L243) | <code>present_inner</code> | <code>"; "</code> | — | — | B2 | F |
| [266](../src/gui/live/presentation.rs#L266) | <code>present_inner</code> | <code>"VM status unavailable - retry the status update. Saved settings and check history are in Technical Details."</code> | — | <code>operations.present_inner.variant_018</code> | B2 | C |
| [268](../src/gui/live/presentation.rs#L268) | <code>present_inner</code> | <code>"Setup interrupted - review recovery before changing GPU support."</code> | — | <code>operations.present_inner.variant_019</code> | B2 | C |
| [270](../src/gui/live/presentation.rs#L270) | <code>present_inner</code> | <code>"Saved settings unavailable - Refresh before reviewing changes."</code> | — | <code>operations.present_inner.variant_020</code> | B2 | C |
| [272](../src/gui/live/presentation.rs#L272) | <code>present_inner</code> | <code>"No GPU attached. Enable support in the draft, then review Apply."</code> | — | <code>operations.present_inner.variant_021</code> | B2 | C |
| [274](../src/gui/live/presentation.rs#L274) | <code>present_inner</code> | <code>"GPU attached. Use Verify GPU setup to check current guest files and rendering. Earlier results remain history."</code> | — | <code>operations.present_inner.variant_022</code> | B2 | C |
| [306](../src/gui/live/presentation.rs#L306) | <code>present_inner</code> | <code>"Configuration validation"</code> | — | <code>operations.present_inner.variant_023</code> | B2 | C |
| [330](../src/gui/live/presentation.rs#L330) | <code>resource_controls</code> | <code>"Select an available physical GPU; capability unknown"</code> | — | <code>operations.resource_controls.variant_001</code> | B2 | C |
| [337](../src/gui/live/presentation.rs#L337) | <code>resource_controls</code> | <code>"; physical dedicated memory: {}"</code> | <code>arg1</code> | <code>operations.resource_controls.variant_002</code> | B2 | C |
| [380](../src/gui/live/presentation.rs#L380) | <code>resource_controls</code> | <code>" GB"</code> | — | <code>operations.resource_controls.variant_003</code> | B2 | M |
| [393](../src/gui/live/presentation.rs#L393) | <code>resource_controls</code> | <code>"\n{note}"</code> | <code>note</code> | <code>operations.resource_controls.variant_004</code> | B2 | C |
| [394](../src/gui/live/presentation.rs#L394) | <code>resource_controls</code> | <code>"Choose a request from 1 GB to the card's detected capacity. Hyper-V validates the requested bytes; actual allocation behavior awaits live qualification.{note}"</code> | <code>note</code> | <code>operations.resource_controls.variant_005</code> | B2 | C |
| [398](../src/gui/live/presentation.rs#L398) | <code>resource_controls</code> | <code>""</code> | — | — | B2 | F |
| [402](../src/gui/live/presentation.rs#L402) | <code>resource_controls</code> | <code>"Unavailable"</code> | — | <code>operations.resource_controls.variant_007</code> | B2 | C |
| [403](../src/gui/live/presentation.rs#L403), [404](../src/gui/live/presentation.rs#L404) | <code>resource_controls</code> | <code>"—"</code> | — | — | B2 | F |
## src/gui/live/profile_removal.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [10](../src/gui/live/profile_removal.rs#L10) | <code>review_forget_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.review_forget_live.variant_001</code> | B2 | M |
| [18](../src/gui/live/profile_removal.rs#L18) | <code>review_forget_live</code> | <code>"Review app profile deletion"</code> | — | <code>operations.review_forget_live.variant_002</code> | B2 | C |
| [33](../src/gui/live/profile_removal.rs#L33), [55](../src/gui/live/profile_removal.rs#L55) | <code>review_forget_live</code> | <code>"error"</code> | — | — | B2 | P |
| [33](../src/gui/live/profile_removal.rs#L33) | <code>review_forget_live</code> | <code>"Review Delete profile again"</code> | — | <code>operations.review_forget_live.variant_004</code> | B2 | C |
| [45](../src/gui/live/profile_removal.rs#L45) | <code>review_forget_live</code> | <code>"forget-review"</code> | — | — | B2 | P |
| [46](../src/gui/live/profile_removal.rs#L46) | <code>review_forget_live</code> | <code>"Delete this app profile?"</code> | — | <code>operations.review_forget_live.variant_006</code> | B2 | C |
| [48](../src/gui/live/profile_removal.rs#L48) | <code>review_forget_live</code> | <code>"Virtual machine: {name}\nVM ID: {}\nGPU: {}\n\nDelete only this VM's app profile: its saved GPU settings, authorized VM/GPU pairing and managed setup/verification history. Keep all other app profiles, the Hyper-V VM, disks, checkpoints, guest driver files and shared application artifacts.\n\nThis action does not remove GPU support, change VM power, uninstall drivers or request guest credentials. The existing protected deletion path requires GPU support already inactive and the VM off.\n\nDelete profile confirms this review; Windows requests approval to remove the protected app records. Cancel keeps all records. The VM stays discoverable and a new reviewed setup is required to enable GPU support through this app again."</code> | <code>name, arg2, arg3</code> | <code>operations.review_forget_live.variant_007</code> | B2 | C |
| [56](../src/gui/live/profile_removal.rs#L56) | <code>review_forget_live</code> | <code>"Profile deletion unavailable"</code> | — | <code>operations.review_forget_live.variant_008</code> | B2 | C |
| [57](../src/gui/live/profile_removal.rs#L57) | <code>review_forget_live</code> | <code>"Profile deletion review"</code> | — | <code>operations.review_forget_live.variant_009</code> | B2 | C |
| [71](../src/gui/live/profile_removal.rs#L71) | <code>forget_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.forget_live.variant_001</code> | B2 | M |
| [81](../src/gui/live/profile_removal.rs#L81) | <code>forget_live</code> | <code>"Delete app profile"</code> | — | <code>operations.forget_live.variant_002</code> | B2 | C |
| [96](../src/gui/live/profile_removal.rs#L96) | <code>forget_live</code> | <code>"profile-deleted"</code> | — | — | B2 | P |
| [97](../src/gui/live/profile_removal.rs#L97) | <code>forget_live</code> | <code>"App profile deleted"</code> | — | <code>operations.forget_live.variant_004</code> | B2 | C |
| [99](../src/gui/live/profile_removal.rs#L99) | <code>forget_live</code> | <code>"{vm_name}: deletion of this app's saved settings, authorization and managed history was verified. The Hyper-V VM, disks, checkpoints, guest files and other app profiles remain. No GPU or power changes were performed. Current status is being updated. Continue to a new permission/setup review when ready."</code> | <code>vm_name</code> | <code>operations.forget_live.variant_005</code> | B2 | C |
| [104](../src/gui/live/profile_removal.rs#L104) | <code>forget_live</code> | <code>"error"</code> | — | — | B2 | P |
| [105](../src/gui/live/profile_removal.rs#L105) | <code>forget_live</code> | <code>"Profile deletion did not finish"</code> | — | <code>operations.forget_live.variant_007</code> | B2 | C |
| [107](../src/gui/live/profile_removal.rs#L107) | <code>forget_live</code> | <code>"{vm_name}: {}\nSome app profile records may have been removed; which records remain is unknown until read back. Use Refresh and review profile recovery before continuing; do not repeat deletion with a new request. This operation does not change the VM, GPU assignment or guest files."</code> | <code>vm_name, arg2</code> | <code>operations.forget_live.variant_008</code> | B2 | C |
| [108](../src/gui/live/profile_removal.rs#L108) | <code>forget_live</code> | <code>"Profile deletion"</code> | — | <code>operations.forget_live.variant_009</code> | B2 | C |
## src/gui/live/progress.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [16](../src/gui/live/progress.rs#L16), [45](../src/gui/live/progress.rs#L45) | <code>progress_to_ui</code> | <code>"Persist operation recovery"</code> | — | <code>operations.progress_to_ui.variant_001</code> | B3 | M |
| [16](../src/gui/live/progress.rs#L16) | <code>progress_to_ui</code> | <code>"Saving progress"</code> | — | <code>operations.progress_to_ui.variant_002</code> | B3 | C |
| [17](../src/gui/live/progress.rs#L17) | <code>progress_to_ui</code> | <code>"Prepare current signed payload"</code> | — | <code>operations.progress_to_ui.variant_003</code> | B3 | M |
| [17](../src/gui/live/progress.rs#L17) | <code>progress_to_ui</code> | <code>"Preparing GPU driver files"</code> | — | <code>operations.progress_to_ui.variant_004</code> | B3 | C |
| [18](../src/gui/live/progress.rs#L18) | <code>progress_to_ui</code> | <code>"Compatibility settings"</code> | — | <code>operations.progress_to_ui.variant_005</code> | B3 | M |
| [18](../src/gui/live/progress.rs#L18) | <code>progress_to_ui</code> | <code>"Updating GPU compatibility settings"</code> | — | <code>operations.progress_to_ui.variant_006</code> | B3 | C |
| [19](../src/gui/live/progress.rs#L19) | <code>progress_to_ui</code> | <code>"Provider allocation"</code> | — | <code>operations.progress_to_ui.variant_007</code> | B3 | M |
| [19](../src/gui/live/progress.rs#L19) | <code>progress_to_ui</code> | <code>"Applying your GPU resource settings"</code> | — | <code>operations.progress_to_ui.variant_008</code> | B3 | C |
| [20](../src/gui/live/progress.rs#L20) | <code>progress_to_ui</code> | <code>"Graceful guest shutdown"</code> | — | <code>operations.progress_to_ui.variant_009</code> | B3 | M |
| [20](../src/gui/live/progress.rs#L20) | <code>progress_to_ui</code> | <code>"Shutting down the VM safely"</code> | — | <code>operations.progress_to_ui.variant_010</code> | B3 | C |
| [21](../src/gui/live/progress.rs#L21) | <code>progress_to_ui</code> | <code>"Start guest"</code> | — | <code>operations.progress_to_ui.variant_011</code> | B3 | M |
| [21](../src/gui/live/progress.rs#L21) | <code>progress_to_ui</code> | <code>"Starting the VM"</code> | — | <code>operations.progress_to_ui.variant_012</code> | B3 | C |
| [22](../src/gui/live/progress.rs#L22) | <code>progress_to_ui</code> | <code>"Publish checked VM configuration"</code> | — | <code>operations.progress_to_ui.variant_013</code> | B3 | M |
| [22](../src/gui/live/progress.rs#L22) | <code>progress_to_ui</code> | <code>"Saving the checked VM settings"</code> | — | <code>operations.progress_to_ui.variant_014</code> | B3 | C |
| [23](../src/gui/live/progress.rs#L23) | <code>progress_to_ui</code> | <code>"Save verified configuration only"</code> | — | <code>operations.progress_to_ui.variant_015</code> | B3 | M |
| [23](../src/gui/live/progress.rs#L23) | <code>progress_to_ui</code> | <code>"Retrying the bound configuration save"</code> | — | <code>operations.progress_to_ui.variant_016</code> | B3 | C |
| [27](../src/gui/live/progress.rs#L27) | <code>progress_to_ui</code> | <code>"In progress"</code> | — | <code>operations.progress_to_ui.variant_017</code> | B3 | M |
| [28](../src/gui/live/progress.rs#L28) | <code>progress_to_ui</code> | <code>"Complete"</code> | — | <code>operations.progress_to_ui.variant_018</code> | B3 | C |
| [29](../src/gui/live/progress.rs#L29) | <code>progress_to_ui</code> | <code>"Failed"</code> | — | <code>operations.progress_to_ui.variant_019</code> | B3 | C |
| [31](../src/gui/live/progress.rs#L31) | <code>progress_to_ui</code> | <code>"{stage} · {status}"</code> | <code>stage, status</code> | <code>operations.progress_to_ui.variant_020</code> | B3 | C |
| [35](../src/gui/live/progress.rs#L35) | <code>progress_to_ui</code> | <code>"Reported events (not proof of final state):\n{}"</code> | <code>arg1</code> | <code>operations.progress_to_ui.variant_021</code> | B3 | C |
| [40](../src/gui/live/progress.rs#L40) | <code>progress_to_ui</code> | <code>"\n"</code> | — | — | B3 | F |
| [49](../src/gui/live/progress.rs#L49) | <code>progress_to_ui</code> | <code>"File copying complete; waiting for guest checks"</code> | — | <code>operations.progress_to_ui.variant_023</code> | B3 | M |
| [50](../src/gui/live/progress.rs#L50) | <code>progress_to_ui</code> | <code>"Copying files · Complete"</code> | — | <code>operations.progress_to_ui.variant_024</code> | B3 | C |
| [67](../src/gui/live/progress.rs#L67) | <code>transfer_fraction</code> | <code>"%)"</code> | — | — | B3 | F |
| [81](../src/gui/live/progress.rs#L81) | <code>update_steps</code> | <code>"Copying files"</code> | — | <code>operations.update_steps.variant_001</code> | B3 | M |
| [83](../src/gui/live/progress.rs#L83) | <code>update_steps</code> | <code>" · "</code> | — | — | B3 | F |
| [135](../src/gui/live/progress.rs#L135) | <code>transfer_update</code> | <code>"In progress"</code> | — | <code>operations.transfer_update.variant_001</code> | B3 | M |
## src/gui/live/source.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [86](../src/gui/live/source.rs#L86) | <code>operation_readback</code> | <code>"Saved configuration requires Windows"</code> | — | <code>operations.operation_readback.variant_001</code> | B2 | M |
| [97](../src/gui/live/source.rs#L97) | <code>discover</code> | <code>"First-install inventory read through Windows elevation. Select a VM/GPU and Apply to review package setup and exact enrollment. GPU changes require a separate subsequent review."</code> | — | <code>operations.discover.variant_001</code> | B2 | C |
| [102](../src/gui/live/source.rs#L102) | <code>discover</code> | <code>"Invalid protected inventory reply: {e}"</code> | <code>e</code> | <code>operations.discover.variant_002</code> | B2 | M |
| [112](../src/gui/live/source.rs#L112) | <code>discover</code> | <code>"{}. Native inventory only; editing is blocked."</code> | <code>arg1</code> | <code>operations.discover.variant_003</code> | B2 | C |
| [113](../src/gui/live/source.rs#L113) | <code>discover</code> | <code>"Protected enrollment"</code> | — | <code>operations.discover.variant_004</code> | B2 | C |
| [122](../src/gui/live/source.rs#L122) | <code>discover</code> | <code>"Hyper-V discovery requires Windows x64"</code> | — | <code>operations.discover.variant_005</code> | B2 | M |
| [130](../src/gui/live/source.rs#L130) | <code>discover_status</code> | <code>"Setup requires attention. Use Refresh to continue."</code> | — | <code>operations.discover_status.variant_001</code> | B2 | M |
| [135](../src/gui/live/source.rs#L135) | <code>discover_status</code> | <code>"Invalid protected inventory reply: {e}"</code> | <code>e</code> | <code>operations.discover_status.variant_002</code> | B2 | M |
| [140](../src/gui/live/source.rs#L140) | <code>discover_status</code> | <code>"Hyper-V discovery requires Windows x64"</code> | — | <code>operations.discover_status.variant_003</code> | B2 | M |
| [152](../src/gui/live/source.rs#L152) | <code>plan</code> | <code>"Invalid plan reply: {e}"</code> | <code>e</code> | <code>operations.plan.variant_001</code> | B2 | M |
| [156](../src/gui/live/source.rs#L156) | <code>plan</code> | <code>"Planning requires Windows x64"</code> | — | <code>operations.plan.variant_002</code> | B2 | M |
| [172](../src/gui/live/source.rs#L172) | <code>credential_dialog</code> | <code>"Guest credentials require Windows"</code> | — | <code>operations.credential_dialog.variant_001</code> | B2 | M |
## src/gui/live/ (formerly src/gui/live.rs)

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [40](../src/gui/live/credentials.rs) | <code>credential_request_cancelled</code> | <code>"info"</code> | — | — | B2 | P |
| [41](../src/gui/live/credentials.rs) | <code>credential_request_cancelled</code> | <code>"Guest credential request cancelled"</code> | — | <code>operations.credential_request_cancelled.variant_002</code> | B2 | C |
| [58](../src/gui/live/state_access.rs) | <code>with_state</code> | <code>"Live presentation state is unavailable; restart the application."</code> | — | <code>operations.with_state.variant_001</code> | B2 | M |
| [67](../src/gui/live/state_access.rs) | <code>with_state</code> | <code>"error"</code> | — | — | B2 | P |
| [67](../src/gui/live/state_access.rs) | <code>with_state</code> | <code>"Action unavailable"</code> | — | <code>operations.with_state.variant_003</code> | B2 | C |
| [119](../src/gui/live/background.rs) | <code>background_refresh</code> | <code>"gui-status-poll"</code> | — | <code>operations.background_refresh.variant_001</code> | B2 | C |
| [132](../src/gui/live/background.rs) | <code>background_refresh</code> | <code>"Cannot start status poll: {error}"</code> | <code>error</code> | <code>operations.background_refresh.variant_002</code> | B2 | M |
| [138](../src/gui/live/background.rs) | <code>run_background_task</code> | <code>"The backend task ended without a terminal outcome. Refresh and inspect protected recovery before retrying."</code> | — | <code>operations.run_background_task.variant_001</code> | B2 | M |
| [145](../src/gui/live/background.rs) | <code>run_background_task</code> | <code>"Keep operation state, progress context, task and completion explicit at this orchestration boundary"</code> | — | <code>operations.run_background_task.variant_002</code> | B2 | C |
| [188](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Another GUI operation is still running."</code> | — | <code>operations.background_for_vm.variant_001</code> | B2 | M |
| [191](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Updating VM status"</code> | — | <code>operations.background_for_vm.variant_002</code> | B2 | M |
| [196](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"the selected VM"</code> | — | <code>operations.background_for_vm.variant_003</code> | B2 | C |
| [199](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"progress"</code> | — | — | B2 | P |
| [203](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Apply approved changes"</code> | — | <code>operations.background_for_vm.variant_005</code> | B2 | M |
| [204](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Applying the reviewed GPU changes and checking host readback. The VM will be left off after successful completion. No guest credentials, preparation or graphics check are required."</code> | — | <code>operations.background_for_vm.variant_006</code> | B2 | C |
| [205](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Check recorded outcome"</code> | — | <code>operations.background_for_vm.variant_007</code> | B2 | M |
| [206](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Checking the already-achieved recorded outcome without repeating GPU work or accessing the guest. Unresolved state retains recovery."</code> | — | <code>operations.background_for_vm.variant_008</code> | B2 | C |
| [207](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Building fresh plan"</code> | — | <code>operations.background_for_vm.variant_009</code> | B2 | M |
| [208](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Preparing your changes may take some time. Checking the selected VM, GPU and installed driver files before showing the changes for approval. Nothing is being applied yet."</code> | — | <code>operations.background_for_vm.variant_010</code> | B2 | C |
| [209](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Retry configuration save only"</code> | — | <code>operations.background_for_vm.variant_011</code> | B2 | M |
| [210](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Requesting Windows elevation to save the verified configuration. No GPU operation will be repeated."</code> | — | <code>operations.background_for_vm.variant_012</code> | B2 | C |
| [211](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Remove inactive pairing and profile"</code> | — | <code>operations.background_for_vm.variant_013</code> | B2 | M |
| [212](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Requesting Windows approval to remove the reviewed inactive pairing, saved settings and managed history. Other profiles, the VM, guest files and shared package artifacts remain. Interrupted work retains profile recovery."</code> | — | <code>operations.background_for_vm.variant_014</code> | B2 | C |
| [214](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Reading the information needed for this action…"</code> | — | <code>operations.background_for_vm.variant_015</code> | B2 | C |
| [221](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"gui-backend-read"</code> | — | <code>operations.background_for_vm.variant_016</code> | B2 | C |
| [231](../src/gui/live/background.rs), [245](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>""</code> | — | — | B2 | F |
| [247](../src/gui/live/background.rs) | <code>background_for_vm</code> | <code>"Cannot start backend task: {error}"</code> | <code>error</code> | <code>operations.background_for_vm.variant_018</code> | B2 | M |
| [262](../src/gui/live/refresh.rs) | <code>refresh</code> | <code>"status-error"</code> | — | — | B2 | P |
| [262](../src/gui/live/refresh.rs) | <code>refresh</code> | <code>"recovery-blocked"</code> | — | — | B2 | P |
| [264](../src/gui/live/refresh.rs) | <code>refresh</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.refresh.variant_003</code> | B2 | M |
| [268](../src/gui/live/refresh.rs) | <code>refresh</code> | <code>"The result belongs to another VM. Close it before refreshing the selected VM."</code> | — | <code>operations.refresh.variant_004</code> | B2 | C |
| [307](../src/gui/live/refresh.rs) | <code>refresh_after_result</code> | <code>"status-error"</code> | — | — | B2 | P |
| [310](../src/gui/live/refresh.rs) | <code>refresh_after_result</code> | <code>"{}\n\nVM status update could not start: {error}\nRetry the status update before another review."</code> | <code>arg1, error</code> | <code>operations.refresh_after_result.variant_002</code> | B2 | C |
| [327](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.refresh_with_result.variant_001</code> | B2 | M |
| [343](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Updating VM status..."</code> | — | <code>operations.refresh_with_result.variant_002</code> | B2 | C |
| [352](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Updating VM status"</code> | — | <code>operations.refresh_with_result.variant_003</code> | B2 | M |
| [354](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Refreshing inventory"</code> | — | <code>operations.refresh_with_result.variant_004</code> | B2 | C |
| [406](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Interface: {}\nDriver: {}\n{}\nPreparation adapter: {}\nAttachment does not prove guest graphics health; sharing is not qualified."</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.refresh_with_result.variant_005</code> | B2 | C |
| [406](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"\n"</code> | — | — | B2 | F |
| [406](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"available"</code> | — | — | B2 | P |
| [406](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"not implemented"</code> | — | <code>operations.refresh_with_result.variant_008</code> | B2 | C |
| [445](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Hyper-V virtual machines and available GPUs. Graphics results reflect previous checks."</code> | — | <code>operations.refresh_with_result.variant_009</code> | B2 | C |
| [450](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"inventory unavailable"</code> | — | <code>operations.refresh_with_result.variant_010</code> | B2 | M |
| [455](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"VM status updated. Graphics results reflect previous checks."</code> | — | <code>operations.refresh_with_result.variant_011</code> | B2 | C |
| [460](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Partial inventory: {}. Other successful observations remain available."</code> | <code>arg1</code> | <code>operations.refresh_with_result.variant_012</code> | B2 | C |
| [465](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>" "</code> | — | — | B2 | F |
| [469](../src/gui/live/refresh.rs), [472](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>" {warning}"</code> | <code>warning</code> | <code>operations.refresh_with_result.variant_014</code> | B2 | C |
| [475](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>" Some committed configuration could not be read or validated. Affected intent remains unknown; other readable VM files are available."</code> | — | <code>operations.refresh_with_result.variant_015</code> | B2 | C |
| [483](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Some VM or GPU information could not be read. See Technical Details."</code> | — | <code>operations.refresh_with_result.variant_016</code> | B2 | C |
| [504](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Inventory read"</code> | — | <code>operations.refresh_with_result.variant_017</code> | B2 | C |
| [518](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Your edits are kept. Refresh VM information before Apply."</code> | — | <code>operations.refresh_with_result.variant_018</code> | B2 | C |
| [521](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Could not refresh VM status. You can keep editing; use Refresh before Apply."</code> | — | <code>operations.refresh_with_result.variant_019</code> | B2 | C |
| [523](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"Live discovery unavailable: {error}"</code> | <code>error</code> | <code>operations.refresh_with_result.variant_020</code> | B2 | C |
| [529](../src/gui/live/refresh.rs) | <code>refresh_with_result</code> | <code>"status-error"</code> | — | — | B2 | P |
| [568](../src/gui/live/review.rs), [584](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Live presentation state unavailable"</code> | — | <code>operations.review_operation.variant_001</code> | B2 | M |
| [577](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Repair / Reinstall requires an authorized VM/GPU pair. Review enrollment first."</code> | — | <code>operations.review_operation.variant_002</code> | B2 | M |
| [599](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Building fresh plan"</code> | — | <code>operations.review_operation.variant_003</code> | B2 | M |
| [604](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Committed configuration changed externally. Draft preserved; discard and Refresh."</code> | — | <code>operations.review_operation.variant_004</code> | B2 | M |
| [635](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.review_operation.variant_005</code> | B2 | M |
| [641](../src/gui/live/review.rs), [663](../src/gui/live/review.rs), [669](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"error"</code> | — | — | B2 | P |
| [641](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Review again"</code> | — | <code>operations.review_operation.variant_007</code> | B2 | C |
| [648](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"review"</code> | — | — | B2 | P |
| [650](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Review Repair / Reinstall"</code> | — | <code>operations.review_operation.variant_009</code> | B2 | C |
| [652](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Review changes before Apply"</code> | — | <code>operations.review_operation.variant_010</code> | B2 | C |
| [655](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"{}\nPhysical GPU: {gpu_name}\n{memory_summary}\n\nThese changes are still pending. Apply confirms this review; Cancel keeps your edits. Windows will request approval before changes. Settings are saved only after the required outcome checks pass."</code> | <code>arg1, gpu_name, memory_summary</code> | <code>operations.review_operation.variant_011</code> | B2 | C |
| [664](../src/gui/live/review.rs), [670](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"Plan unavailable"</code> | — | <code>operations.review_operation.variant_012</code> | B2 | C |
| [665](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"The backend plan does not match the selected draft. Refresh before retrying."</code> | — | <code>operations.review_operation.variant_013</code> | B2 | C |
| [672](../src/gui/live/review.rs) | <code>review_operation</code> | <code>"{error}\nYour draft is preserved. No VM or guest changes were performed."</code> | <code>error</code> | <code>operations.review_operation.variant_014</code> | B2 | C |
| [690](../src/gui/live/enrollment.rs), [753](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"Review permission before GPU setup"</code> | — | <code>operations.review_enrollment.variant_001</code> | B2 | C |
| [708](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"Interrupted first-install setup belongs to another original pair/package. Select its original VM/GPU and reviewed package; setup recovery cannot replace authority."</code> | — | <code>operations.review_enrollment.variant_002</code> | B2 | M |
| [717](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"Virtual machine: {}\nPhysical GPU: {}\n\nStep 1: allow this app to manage this VM's selected GPU. This saves permission only; GPU support and VM power will not change.\n\nStep 2: after this permission is confirmed and refreshed, use Apply to review the actual GPU setup and power changes, then confirm that separate review. Your edits are kept. Other VMs' permissions are preserved."</code> | <code>arg1, arg2</code> | <code>operations.review_enrollment.variant_003</code> | B2 | C |
| [724](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"\n\nThe application also needs to install or finish setting up its management components on this Windows host. Windows will ask for administrator approval. If setup was interrupted, the original VM, GPU and application package must still match."</code> | — | <code>operations.review_enrollment.variant_004</code> | B2 | C |
| [726](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"\n\nWindows approval is required to save this permission."</code> | — | <code>operations.review_enrollment.variant_005</code> | B2 | C |
| [729](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"VM ID: {}\nGPU interface: {}\nOperator: {}\nPolicy revision: {:?}\n\nAuthorize this exact VM/GPU pair through Windows elevation, preserving all other enrolled VMs. Enrollment does not enable GPU support, change guest power, prepare drivers or save desired configuration. Your draft is retained. After fresh authority readback, Apply obtains a separate GPU operation plan."</code> | <code>arg1, arg2, arg3, :?</code> | <code>operations.review_enrollment.variant_006</code> | B2 | C |
| [736](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"\n\nFIRST INSTALL / INTERRUPTED SETUP: install the fixed product package into Program Files, create protected ProgramData state and register the fixed SYSTEM runner task for this operator. Interrupted setup retains its original pair and package pins. If authority is already published, recovery only validates it and completes task/audit registration; authority and artifact files are never overwritten. No VM disk, CPU/RAM or Secure Boot change. Reviewed source artifact SHA-256 pins:\n"</code> | — | <code>operations.review_enrollment.variant_007</code> | B2 | C |
| [738](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"{name}: {hash}\n"</code> | <code>name, hash</code> | <code>operations.review_enrollment.variant_008</code> | B2 | C |
| [752](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"enroll-review"</code> | — | — | B2 | P |
| [758](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"error"</code> | — | — | B2 | P |
| [758](../src/gui/live/enrollment.rs) | <code>review_enrollment</code> | <code>"Permission review unavailable"</code> | — | <code>operations.review_enrollment.variant_011</code> | B2 | M |
| [771](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.enroll_live.variant_001</code> | B2 | M |
| [775](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"Review exact enrollment first"</code> | — | <code>operations.enroll_live.variant_002</code> | B2 | M |
| [778](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"Pair changed after enrollment review; review again"</code> | — | <code>operations.enroll_live.variant_003</code> | B2 | M |
| [789](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"Enroll reviewed VM/GPU pair"</code> | — | <code>operations.enroll_live.variant_004</code> | B2 | C |
| [801](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"permission-saved"</code> | — | — | B2 | P |
| [802](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"GPU management permission saved"</code> | — | <code>operations.enroll_live.variant_006</code> | B2 | C |
| [803](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"Permission for the reviewed VM/GPU pair was saved. Your draft is retained. Continue to a separate setup review; GPU support and guest readiness have not been confirmed."</code> | — | <code>operations.enroll_live.variant_007</code> | B2 | C |
| [819](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"error"</code> | — | — | B2 | P |
| [820](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"App permission was not confirmed"</code> | — | <code>operations.enroll_live.variant_009</code> | B2 | C |
| [822](../src/gui/live/enrollment.rs) | <code>enroll_live</code> | <code>"{error}\nYour edits are kept. Use Refresh and Review recovery before retrying this permission step. GPU setup has not been confirmed."</code> | <code>error</code> | <code>operations.enroll_live.variant_010</code> | B2 | C |
| [838](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.reconcile_enrollment_live.variant_001</code> | B2 | M |
| [842](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Refresh enrollment recovery first"</code> | — | <code>operations.reconcile_enrollment_live.variant_002</code> | B2 | M |
| [848](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Review authority readback first"</code> | — | <code>operations.reconcile_enrollment_live.variant_003</code> | B2 | M |
| [858](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Reconcile enrollment authority"</code> | — | <code>operations.reconcile_enrollment_live.variant_004</code> | B2 | C |
| [863](../src/gui/live/enrollment.rs), [870](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"error"</code> | — | — | B2 | P |
| [863](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Permission status unavailable"</code> | — | <code>operations.reconcile_enrollment_live.variant_006</code> | B2 | C |
| [871](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"Enrollment recovery retained"</code> | — | <code>operations.reconcile_enrollment_live.variant_007</code> | B2 | C |
| [872](../src/gui/live/enrollment.rs) | <code>reconcile_enrollment_live</code> | <code>"{error}\nRefresh before reviewing authority again."</code> | <code>error</code> | <code>operations.reconcile_enrollment_live.variant_008</code> | B2 | C |
| [882](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"The reviewed protected worker is not available."</code> | — | <code>operations.apply_live.variant_001</code> | B2 | M |
| [885](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.apply_live.variant_002</code> | B2 | M |
| [902](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"Apply approved changes"</code> | — | <code>operations.apply_live.variant_003</code> | B2 | M |
| [945](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"info"</code> | — | — | B2 | P |
| [947](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU setup/update passed"</code> | — | <code>operations.apply_live.variant_005</code> | B2 | C |
| [949](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU change confirmed and configuration saved"</code> | — | <code>operations.apply_live.variant_006</code> | B2 | C |
| [953](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{result_vm_name}: GPU support {}. VRAM request: {}. Hardware rendering passed, graceful final shutdown was confirmed, and your settings were saved. The VM is off; start it through Hyper-V when ready."</code> | <code>result_vm_name, arg2, arg3</code> | <code>operations.apply_live.variant_007</code> | B2 | C |
| [955](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"enabled"</code> | — | <code>operations.apply_live.variant_008</code> | B2 | C |
| [957](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"disabled"</code> | — | <code>operations.apply_live.variant_009</code> | B2 | C |
| [960](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"provider default"</code> | — | <code>operations.apply_live.variant_010</code> | B2 | C |
| [966](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{result_vm_name}: GPU assignment, requested settings and final-off state were confirmed; your settings were saved. No guest graphics check was run. Start the VM through Hyper-V when ready."</code> | <code>result_vm_name</code> | <code>operations.apply_live.variant_011</code> | B2 | C |
| [973](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"save"</code> | — | — | B2 | P |
| [975](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU setup/update passed; settings not saved"</code> | — | <code>operations.apply_live.variant_013</code> | B2 | C |
| [977](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU change confirmed; configuration not saved"</code> | — | <code>operations.apply_live.variant_014</code> | B2 | C |
| [980](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{}{}\nThe verified intent is retained separately from committed configuration. Retry saving only; do not repeat Apply."</code> | <code>arg1, arg2</code> | <code>operations.apply_live.variant_015</code> | B2 | C |
| [983](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{result_vm_name}: hardware rendering passed and final shutdown was confirmed. The VM is off.\n"</code> | <code>result_vm_name</code> | <code>operations.apply_live.variant_016</code> | B2 | C |
| [987](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{result_vm_name}: GPU changes and final-off state were confirmed. No guest graphics check was run. The VM is off.\n"</code> | <code>result_vm_name</code> | <code>operations.apply_live.variant_017</code> | B2 | C |
| [1022](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"error"</code> | — | — | B2 | P |
| [1024](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU setup/update did not complete"</code> | — | <code>operations.apply_live.variant_019</code> | B2 | C |
| [1026](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"Operation did not complete"</code> | — | <code>operations.apply_live.variant_020</code> | B2 | C |
| [1031](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"Applying changes"</code> | — | <code>operations.apply_live.variant_021</code> | B2 | C |
| [1033](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"{reason}\n{}\nYour draft is retained. Changes may have started; the final VM state has not been confirmed. Use Refresh and inspect recovery before trying again. Do not repeat Apply while recovery is required."</code> | <code>reason, arg2</code> | <code>operations.apply_live.variant_022</code> | B2 | C |
| [1034](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"No execution stage was reported."</code> | — | <code>operations.apply_live.variant_023</code> | B2 | C |
| [1034](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"Last reported step: {stage}"</code> | <code>stage</code> | <code>operations.apply_live.variant_024</code> | B2 | C |
| [1048](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.save_only_live.variant_001</code> | B2 | M |
| [1057](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Retry configuration save only"</code> | — | <code>operations.save_only_live.variant_002</code> | B2 | M |
| [1068](../src/gui/live/apply.rs), [1085](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Completion readback was already consumed"</code> | — | <code>operations.save_only_live.variant_003</code> | B2 | M |
| [1095](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"info"</code> | — | — | B2 | P |
| [1096](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Configuration saved"</code> | — | <code>operations.save_only_live.variant_005</code> | B2 | C |
| [1097](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Protected publication passed readback. No GPU operation was repeated."</code> | — | <code>operations.save_only_live.variant_006</code> | B2 | C |
| [1102](../src/gui/live/apply.rs), [1108](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"error"</code> | — | — | B2 | P |
| [1103](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Configuration still unsaved"</code> | — | <code>operations.save_only_live.variant_008</code> | B2 | C |
| [1104](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"The protected receipt remains available for inspection. No GPU operation was repeated."</code> | — | <code>operations.save_only_live.variant_009</code> | B2 | C |
| [1109](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Save-only retry blocked"</code> | — | <code>operations.save_only_live.variant_010</code> | B2 | C |
| [1111](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"{error}\nThe protected recovery record remains; no GPU operation was repeated."</code> | <code>error</code> | <code>operations.save_only_live.variant_011</code> | B2 | C |
| [1125](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.review_verification.variant_001</code> | B2 | M |
| [1128](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Discard or Apply the draft before reviewing a separate graphics check."</code> | — | <code>operations.review_verification.variant_002</code> | B2 | C |
| [1138](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Reviewing Verify GPU setup"</code> | — | <code>operations.review_verification.variant_003</code> | B2 | C |
| [1162](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"verify-review"</code> | — | — | B2 | P |
| [1162](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Review Verify GPU setup"</code> | — | <code>operations.review_verification.variant_005</code> | B2 | C |
| [1165](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"error"</code> | — | — | B2 | P |
| [1165](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Verify GPU setup unavailable"</code> | — | <code>operations.review_verification.variant_007</code> | B2 | M |
| [1173](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.verify_live.variant_001</code> | B2 | M |
| [1176](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Review verification first"</code> | — | <code>operations.verify_live.variant_002</code> | B2 | M |
| [1186](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Verify GPU setup"</code> | — | <code>operations.verify_live.variant_003</code> | B2 | M |
| [1225](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"info"</code> | — | — | B2 | P |
| [1226](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Verify GPU setup passed"</code> | — | <code>operations.verify_live.variant_005</code> | B2 | C |
| [1228](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"{result_vm_name}: current guest driver files and setup record matched the approved driver package; device health and hardware rendering passed. The reviewed final power state was confirmed. Refresh to read the latest check result; it is a record of this check, not a guarantee of future health."</code> | <code>result_vm_name</code> | <code>operations.verify_live.variant_006</code> | B2 | C |
| [1233](../src/gui/live/verification.rs), [1241](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"error"</code> | — | — | B2 | P |
| [1234](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Verify GPU setup outcome unknown"</code> | — | <code>operations.verify_live.variant_008</code> | B2 | C |
| [1236](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"{result_vm_name}: no completed check outcome was established. Use Refresh and Review recovery before another operation."</code> | <code>result_vm_name</code> | <code>operations.verify_live.variant_009</code> | B2 | C |
| [1242](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Verify GPU setup did not complete"</code> | — | <code>operations.verify_live.variant_010</code> | B2 | C |
| [1244](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"{result_vm_name}: {error}\nThe check did not complete. Use Refresh and Review recovery before another operation; prior successful checks do not establish current health."</code> | <code>result_vm_name, error</code> | <code>operations.verify_live.variant_011</code> | B2 | C |
| [1259](../src/gui/live/recovery.rs), [1268](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.reconcile_live.variant_001</code> | B2 | M |
| [1262](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"No reviewed operation recovery is loaded"</code> | — | <code>operations.reconcile_live.variant_002</code> | B2 | M |
| [1272](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Recovery VM unavailable; Refresh first"</code> | — | <code>operations.reconcile_live.variant_003</code> | B2 | M |
| [1311](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Check recorded outcome"</code> | — | <code>operations.reconcile_live.variant_004</code> | B2 | M |
| [1363](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"recovery-ended"</code> | — | — | B2 | P |
| [1364](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Interrupted attempt ended"</code> | — | <code>operations.reconcile_live.variant_006</code> | B2 | C |
| [1365](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Fresh checks established the recorded host state and safely ended the attempt without repeating GPU work or saving requested settings. Any failed or unknown guest check remains recorded; ending recovery does not certify guest setup or rendering. Review repair or another supported operation."</code> | — | <code>operations.reconcile_live.variant_007</code> | B2 | C |
| [1369](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"info"</code> | — | — | B2 | P |
| [1370](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Recorded outcome confirmed"</code> | — | <code>operations.reconcile_live.variant_009</code> | B2 | C |
| [1371](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Fresh checks established the reviewed outcome and any required configuration publication. No GPU assignment, allocation or preparation was repeated. VM status is updated before another review."</code> | — | <code>operations.reconcile_live.variant_010</code> | B2 | C |
| [1375](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"save"</code> | — | — | B2 | P |
| [1376](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Outcome verified; configuration unsaved"</code> | — | <code>operations.reconcile_live.variant_012</code> | B2 | C |
| [1378](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"{}\nRetry configuration saving only."</code> | <code>arg1</code> | <code>operations.reconcile_live.variant_013</code> | B2 | C |
| [1383](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"error"</code> | — | — | B2 | P |
| [1383](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Recovery remains blocked"</code> | — | <code>operations.reconcile_live.variant_015</code> | B2 | C |
| [1384](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"recovery-blocked"</code> | — | — | B2 | P |
| [1386](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"{}\nReview partial removal here. For an interrupted enable on a stopped VM, Remove partial GPU setup can restore its recorded starting host configuration. If that review cannot establish safe removal, recovery stays blocked and its explanation names the unavailable facts."</code> | <code>arg1</code> | <code>operations.reconcile_live.variant_017</code> | B2 | C |
| [1387](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Reconciliation"</code> | — | <code>operations.reconcile_live.variant_018</code> | B2 | C |
| [1407](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Current VM status is unavailable. Retry the status update before reviewing removal."</code> | — | <code>operations.partial_removal_candidate.variant_001</code> | B2 | C |
| [1411](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Refresh recovery first"</code> | — | <code>operations.partial_removal_candidate.variant_002</code> | B2 | M |
| [1416](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Refresh inventory first"</code> | — | <code>operations.partial_removal_candidate.variant_003</code> | B2 | M |
| [1420](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Recovery VM missing; Refresh"</code> | — | <code>operations.partial_removal_candidate.variant_004</code> | B2 | M |
| [1431](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Recovery GPU missing; Refresh"</code> | — | <code>operations.partial_removal_candidate.variant_005</code> | B2 | M |
| [1437](../src/gui/live/partial_removal.rs) | <code>partial_removal_candidate</code> | <code>"Recovery journal missing; Refresh"</code> | — | <code>operations.partial_removal_candidate.variant_006</code> | B2 | M |
| [1450](../src/gui/live/partial_removal.rs) | <code>review_reset</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.review_reset.variant_001</code> | B2 | M |
| [1453](../src/gui/live/partial_removal.rs) | <code>review_reset</code> | <code>"Refresh recovery first"</code> | — | <code>operations.review_reset.variant_002</code> | B2 | M |
| [1455](../src/gui/live/partial_removal.rs) | <code>review_reset</code> | <code>"Virtual machine: {}\n\nRemove the partial {} attachment and its allocations. Restore the compatibility settings recorded before the interrupted change. The VM stays off; no guest connection or driver installation is performed.\n\nKeep the saved settings, guest files and archived failure evidence. Invalidate old setup eligibility so a later setup performs fresh checks. After removal, review the next step here. A fresh enrollment may require deleting the inactive app profile first; deletion has its own review and confirmation.\n\nStarting compatibility settings to restore: low MMIO {} MB, high MMIO {} MB, cache types {}, automatic checkpoints {}. This does not complete the interrupted allocation or save its requested settings."</code> | <code>arg1, arg2, arg3, arg4, arg5, arg6</code> | <code>operations.review_reset.variant_003</code> | B2 | C |
| [1465](../src/gui/live/partial_removal.rs) | <code>review_reset</code> | <code>"reset-review"</code> | — | — | B2 | P |
| [1465](../src/gui/live/partial_removal.rs) | <code>review_reset</code> | <code>"Review partial GPU removal"</code> | — | <code>operations.review_reset.variant_005</code> | B2 | C |
| [1472](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.reset_live.variant_001</code> | B2 | M |
| [1473](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Refresh recovery first"</code> | — | <code>operations.reset_live.variant_002</code> | B2 | M |
| [1477](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Review partial removal first"</code> | — | <code>operations.reset_live.variant_003</code> | B2 | M |
| [1483](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Recovery changed; review partial removal again"</code> | — | <code>operations.reset_live.variant_004</code> | B2 | M |
| [1494](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Remove partial GPU setup"</code> | — | <code>operations.reset_live.variant_005</code> | B2 | C |
| [1509](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"reset-complete"</code> | — | — | B2 | P |
| [1510](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Partial GPU setup removed"</code> | — | <code>operations.reset_live.variant_007</code> | B2 | C |
| [1511](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"The partial attachment was removed and the recorded starting compatibility settings were confirmed. The VM remains off. Old setup eligibility is invalid; guest files and failure evidence are retained. Review Delete profile here if a fresh enrollment is needed. It removes saved settings, app permission and managed history, and requires its own confirmation. Otherwise close this result and review Repair / Reinstall."</code> | — | <code>operations.reset_live.variant_008</code> | B2 | C |
| [1513](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"error"</code> | — | — | B2 | P |
| [1513](../src/gui/live/partial_removal.rs) | <code>reset_live</code> | <code>"Partial GPU removal did not complete"</code> | — | <code>operations.reset_live.variant_010</code> | B2 | M |
| [1520](../src/gui/live/verification.rs) | <code>review_verification</code> | <code>"Verify GPU setup requires Windows"</code> | — | <code>operations.review_verification.variant_008</code> | B2 | M |
| [1524](../src/gui/live/verification.rs) | <code>verify_live</code> | <code>"Verify GPU setup requires Windows"</code> | — | <code>operations.verify_live.variant_012</code> | B2 | M |
| [1528](../src/gui/live/recovery.rs) | <code>reconcile_live</code> | <code>"Reconciliation requires Windows"</code> | — | <code>operations.reconcile_live.variant_019</code> | B2 | M |
| [1532](../src/gui/live/apply.rs) | <code>apply_live</code> | <code>"GPU operations require Windows"</code> | — | <code>operations.apply_live.variant_025</code> | B2 | M |
| [1536](../src/gui/live/apply.rs) | <code>save_only_live</code> | <code>"Protected publication requires Windows"</code> | — | <code>operations.save_only_live.variant_012</code> | B2 | M |
| [1541](../src/gui/live/actions.rs), [1544](../src/gui/live/actions.rs), [1620](../src/gui/live/actions.rs), [1829](../src/gui/live/actions.rs), [1839](../src/gui/live/actions.rs) | <code>run</code> | <code>""</code> | — | — | B2 | F |
| [1546](../src/gui/live/actions.rs) | <code>run</code> | <code>"Virtual machine information has not been read yet."</code> | — | <code>operations.run.variant_002</code> | B2 | C |
| [1556](../src/gui/live/actions.rs), [1835](../src/gui/live/actions.rs), [1838](../src/gui/live/actions.rs) | <code>run</code> | <code>"switch"</code> | — | — | B2 | P |
| [1556](../src/gui/live/actions.rs) | <code>run</code> | <code>"Keep your changes?"</code> | — | <code>operations.run.variant_004</code> | B2 | C |
| [1556](../src/gui/live/actions.rs) | <code>run</code> | <code>"There is one unapplied VM draft. Stay here, review it, or discard it before switching. Nothing has been applied."</code> | — | <code>operations.run.variant_005</code> | B2 | C |
| [1573](../src/gui/live/actions.rs) | <code>run</code> | <code>"The recorded GPU pairing is fixed. GPU replacement is unavailable; your edits are kept."</code> | — | <code>operations.run.variant_006</code> | B2 | C |
| [1607](../src/gui/live/actions.rs) | <code>run</code> | <code>"No unapplied edits"</code> | — | <code>operations.run.variant_007</code> | B2 | C |
| [1629](../src/gui/live/actions.rs) | <code>run</code> | <code>"Configuration validation"</code> | — | <code>operations.run.variant_008</code> | B2 | C |
| [1656](../src/gui/live/actions.rs), [1701](../src/gui/live/actions.rs), [1815](../src/gui/live/actions.rs) | <code>run</code> | <code>"Presentation state unavailable"</code> | — | <code>operations.run.variant_009</code> | B2 | M |
| [1728](../src/gui/live/actions.rs), [1772](../src/gui/live/actions.rs) | <code>run</code> | <code>"refresh"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1787](../src/gui/live/actions.rs), [1792](../src/gui/live/actions.rs) | <code>run</code> | <code>"review"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1802](../src/gui/live/actions.rs) | <code>run</code> | <code>"reapply"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1733](../src/gui/live/actions.rs), [1808](../src/gui/live/actions.rs) | <code>run</code> | <code>"repair"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1733](../src/gui/live/actions.rs), [1814](../src/gui/live/actions.rs) | <code>run</code> | <code>"remove-gpu"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1733](../src/gui/live/actions.rs), [1784](../src/gui/live/actions.rs) | <code>run</code> | <code>"forget"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1786](../src/gui/live/actions.rs) | <code>run</code> | <code>"confirm-forget"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1733](../src/gui/live/actions.rs), [1795](../src/gui/live/actions.rs) | <code>run</code> | <code>"verify"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1792](../src/gui/live/actions.rs), [1793](../src/gui/live/actions.rs) | <code>run</code> | <code>"confirm"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1789](../src/gui/live/actions.rs) | <code>run</code> | <code>"confirm-enroll"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1796](../src/gui/live/actions.rs) | <code>run</code> | <code>"confirm-verify"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1797](../src/gui/live/actions.rs) | <code>run</code> | <code>"reconcile"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1799](../src/gui/live/actions.rs) | <code>run</code> | <code>"review-reset"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1801](../src/gui/live/actions.rs) | <code>run</code> | <code>"confirm-reset"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1791](../src/gui/live/actions.rs) | <code>run</code> | <code>"reconcile-enrollment"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1794](../src/gui/live/actions.rs) | <code>run</code> | <code>"save-only"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1848](../src/gui/live/actions.rs) | <code>run</code> | <code>"credentials"</code> | — | — | B2 | P |
| [1728](../src/gui/live/actions.rs), [1774](../src/gui/live/actions.rs) | <code>run</code> | <code>"discard-review-forget"</code> | — | — | B2 | P |
| [1730](../src/gui/live/actions.rs) | <code>run</code> | <code>"VM information is updating. You can keep editing; try this action when the update finishes."</code> | — | <code>operations.run.variant_028</code> | B2 | C |
| [1735](../src/gui/live/actions.rs), [1753](../src/gui/live/actions.rs), [1853](../src/gui/live/actions.rs) | <code>run</code> | <code>"Live presentation state unavailable"</code> | — | <code>operations.run.variant_029</code> | B2 | M |
| [1739](../src/gui/live/actions.rs) | <code>run</code> | <code>"recovery-needed"</code> | — | — | B2 | P |
| [1739](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review the unfinished operation first"</code> | — | <code>operations.run.variant_031</code> | B2 | C |
| [1743](../src/gui/live/actions.rs), [1765](../src/gui/live/actions.rs), [1859](../src/gui/live/actions.rs), [1930](../src/gui/live/actions.rs) | <code>run</code> | <code>"error"</code> | — | — | B2 | P |
| [1743](../src/gui/live/actions.rs) | <code>run</code> | <code>"Cannot read operation state"</code> | — | <code>operations.run.variant_033</code> | B2 | C |
| [1749](../src/gui/live/actions.rs), [1761](../src/gui/live/actions.rs), [1855](../src/gui/live/actions.rs), [1856](../src/gui/live/actions.rs), [1889](../src/gui/live/actions.rs), [1920](../src/gui/live/actions.rs), [1926](../src/gui/live/actions.rs) | <code>run</code> | <code>"info"</code> | — | — | B2 | P |
| [1749](../src/gui/live/actions.rs), [1761](../src/gui/live/actions.rs) | <code>run</code> | <code>"Action unavailable"</code> | — | <code>operations.run.variant_035</code> | B2 | C |
| [1749](../src/gui/live/actions.rs) | <code>run</code> | <code>"Use Refresh to check application setup and worker availability. Your edits are kept."</code> | — | <code>operations.run.variant_036</code> | B2 | C |
| [1765](../src/gui/live/actions.rs) | <code>run</code> | <code>"Cannot read action availability"</code> | — | <code>operations.run.variant_037</code> | B2 | C |
| [1774](../src/gui/live/actions.rs) | <code>run</code> | <code>"reset-complete"</code> | — | — | B2 | P |
| [1776](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review unfinished recovery first."</code> | — | <code>operations.run.variant_039</code> | B2 | M |
| [1786](../src/gui/live/actions.rs) | <code>run</code> | <code>"forget-review"</code> | — | — | B2 | P |
| [1793](../src/gui/live/actions.rs) | <code>run</code> | <code>"Open a fresh review before Apply."</code> | — | <code>operations.run.variant_041</code> | B2 | M |
| [1801](../src/gui/live/actions.rs) | <code>run</code> | <code>"reset-review"</code> | — | — | B2 | P |
| [1826](../src/gui/live/actions.rs) | <code>run</code> | <code>"close"</code> | — | — | B2 | P |
| [1834](../src/gui/live/actions.rs), [1936](../src/gui/live/actions.rs) | <code>run</code> | <code>"exit"</code> | — | — | B2 | P |
| [1835](../src/gui/live/actions.rs) | <code>run</code> | <code>"discard"</code> | — | — | B2 | P |
| [1840](../src/gui/live/actions.rs) | <code>run</code> | <code>"Draft discarded. No configuration was saved or VM state changed."</code> | — | <code>operations.run.variant_046</code> | B2 | C |
| [1843](../src/gui/live/actions.rs), [1846](../src/gui/live/actions.rs) | <code>run</code> | <code>"more"</code> | — | — | B2 | P |
| [1845](../src/gui/live/actions.rs) | <code>run</code> | <code>"Virtual machine: {}\n\nChoose an action to open its review. Unavailable actions explain what to do first. Your edits are kept until you explicitly discard or apply them."</code> | <code>arg1</code> | <code>operations.run.variant_048</code> | B2 | C |
| [1846](../src/gui/live/actions.rs) | <code>run</code> | <code>"VM actions"</code> | — | <code>operations.run.variant_049</code> | B2 | C |
| [1854](../src/gui/live/actions.rs) | <code>run</code> | <code>"Guest credentials"</code> | — | <code>operations.run.variant_050</code> | B2 | M |
| [1855](../src/gui/live/actions.rs) | <code>run</code> | <code>"Stored guest credentials available"</code> | — | <code>operations.run.variant_051</code> | B2 | C |
| [1855](../src/gui/live/actions.rs) | <code>run</code> | <code>"A stored credential was confirmed in this user's Windows Credential Manager for the selected VM. No guest connection or graphics check was performed."</code> | — | <code>operations.run.variant_052</code> | B2 | C |
| [1856](../src/gui/live/actions.rs) | <code>run</code> | <code>"Guest credentials not retained"</code> | — | <code>operations.run.variant_053</code> | B2 | C |
| [1856](../src/gui/live/actions.rs) | <code>run</code> | <code>"No stored credential is available. Apply or Verify will prompt locally when credentials are needed. Select Remember to store them in Windows Credential Manager."</code> | — | <code>operations.run.variant_054</code> | B2 | C |
| [1859](../src/gui/live/actions.rs) | <code>run</code> | <code>"Guest credentials unavailable"</code> | — | <code>operations.run.variant_055</code> | B2 | C |
| [1864](../src/gui/live/actions.rs) | <code>run</code> | <code>"diagnostics"</code> | — | — | B2 | P |
| [1864](../src/gui/live/actions.rs), [1867](../src/gui/live/actions.rs), [1888](../src/gui/live/actions.rs), [1892](../src/gui/live/actions.rs), [1902](../src/gui/live/actions.rs), [1908](../src/gui/live/actions.rs), [1910](../src/gui/live/actions.rs) | <code>run</code> | <code>"recovery"</code> | — | — | B2 | P |
| [1871](../src/gui/live/actions.rs) | <code>run</code> | <code>"Authority does not match either recorded revision. Recovery remains held for administrator inspection."</code> | — | <code>operations.run.variant_058</code> | B2 | M |
| [1873](../src/gui/live/actions.rs) | <code>run</code> | <code>"Profile removal was interrupted. Confirm finishes only remaining reviewed record deletions and authority removal after fresh inactive-VM/revision checks. Completed deletions must remain absent; changed or unrelated records stay blocked. Other VMs, guest files and shared artifacts are retained. No guest checks, credentials or VM/GPU effects run."</code> | — | <code>operations.run.variant_059</code> | B2 | C |
| [1875](../src/gui/live/actions.rs) | <code>run</code> | <code>"The complete new enrollment policy is present. Confirm independently checks this reviewed authority and ends the hold; no enrollment write or GPU work is replayed."</code> | — | <code>operations.run.variant_060</code> | B2 | C |
| [1877](../src/gui/live/actions.rs) | <code>run</code> | <code>"The original policy is present; the pair was not added. Confirm independently checks this reviewed authority and ends the hold; no enrollment write or GPU work is replayed."</code> | — | <code>operations.run.variant_061</code> | B2 | C |
| [1879](../src/gui/live/actions.rs) | <code>run</code> | <code>"Operation: {}\nVM ID: {}\nGPU: {}\n\n{outcome}"</code> | <code>arg1, arg2, arg3, outcome</code> | <code>operations.run.variant_062</code> | B2 | C |
| [1880](../src/gui/live/actions.rs) | <code>run</code> | <code>"VM name unavailable; see Technical Details"</code> | — | <code>operations.run.variant_063</code> | B2 | C |
| [1881](../src/gui/live/actions.rs) | <code>run</code> | <code>"Virtual machine: {name}\n\n{outcome}\n\nReviewing this record makes no changes. Confirmation is required for the recovery action shown."</code> | <code>name, outcome</code> | <code>operations.run.variant_064</code> | B2 | C |
| [1884](../src/gui/live/actions.rs) | <code>run</code> | <code>"enrollment-recovery"</code> | — | — | B2 | P |
| [1884](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review interrupted profile deletion"</code> | — | <code>operations.run.variant_066</code> | B2 | C |
| [1884](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review interrupted app permission"</code> | — | <code>operations.run.variant_067</code> | B2 | C |
| [1889](../src/gui/live/actions.rs) | <code>run</code> | <code>"Configuration import requires reconciliation"</code> | — | <code>operations.run.variant_068</code> | B2 | C |
| [1889](../src/gui/live/actions.rs) | <code>run</code> | <code>"Import operation: {}\n{} destinations, {} recorded as published. Resolve this explicit CLI import with import-reconcile --operation {}. Changed files block continuation; import never changes enrollment or GPU state."</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.run.variant_069</code> | B2 | C |
| [1899](../src/gui/live/actions.rs) | <code>run</code> | <code>"save"</code> | — | — | B2 | P |
| [1899](../src/gui/live/actions.rs) | <code>run</code> | <code>"Verified operation awaits configuration publication"</code> | — | <code>operations.run.variant_071</code> | B2 | C |
| [1899](../src/gui/live/actions.rs) | <code>run</code> | <code>"GPU effects already passed independent readback. Retry saves only the bound configuration; it will not repeat GPU modification. Changed configuration, driver, payload or effective state blocks this retry."</code> | — | <code>operations.run.variant_072</code> | B2 | C |
| [1900](../src/gui/live/actions.rs) | <code>run</code> | <code>"Operation: {}\nVM ID: {}\nGPU: {}\nReviewed configuration revision: {:?}\nDriver: {}\nExpected final power: {:?}"</code> | <code>arg1, arg2, arg3, :?, arg5, :?</code> | <code>operations.run.variant_073</code> | B2 | C |
| [1902](../src/gui/live/actions.rs) | <code>run</code> | <code>"Reconcile interrupted graphics check"</code> | — | <code>operations.run.variant_074</code> | B2 | C |
| [1902](../src/gui/live/actions.rs) | <code>run</code> | <code>"Check the recorded pair and fresh graphics outcome; restore only the originally approved guest power if safe. No GPU assignment, allocation, preparation or configuration publication is performed."</code> | — | <code>operations.run.variant_075</code> | B2 | C |
| [1908](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review interrupted GPU change"</code> | — | <code>operations.run.variant_076</code> | B2 | C |
| [1908](../src/gui/live/actions.rs) | <code>run</code> | <code>"Virtual machine: {name}\n\nCheck the already-achieved GPU assignment, settings and final-off state without repeating changes or accessing the guest. Partial or changed state retains recovery. No guest boot, credential prompt or graphics check is required."</code> | <code>name</code> | <code>operations.run.variant_077</code> | B2 | C |
| [1910](../src/gui/live/actions.rs) | <code>run</code> | <code>"Review interrupted operation"</code> | — | <code>operations.run.variant_078</code> | B2 | C |
| [1910](../src/gui/live/actions.rs) | <code>run</code> | <code>"Virtual machine: {name}\n\nCheck the interrupted GPU operation without repeating it. A terminal failed guest check can end recovery after fresh reads confirm its original host lifecycle was restored; its failure remains recorded. A setup/reinstall attempt that never entered preparation can end after fresh reads confirm unchanged starting host state. These exits require no new guest check or credentials and do not certify setup. If fresh checks prove that this first setup attempt is back at its original stopped, unattached host configuration, end that attempt without saving its requested changes. Guest files may remain and will not count as verified preparation.\n\nOtherwise, check the already-achieved approved outcome. This may temporarily start the VM and require its guest administrator username and password. Stop it safely afterward only with your approval. Partial or changed state stays blocked; no GPU setup work is replayed."</code> | <code>name</code> | <code>operations.run.variant_079</code> | B2 | C |
| [1912](../src/gui/live/actions.rs) | <code>run</code> | <code>"Operation: {}\nVM ID: {}\nGPU: {}\nOriginal guest power: {:?}\nReviewed final guest power: {:?}"</code> | <code>arg1, arg2, arg3, :?, :?</code> | <code>operations.run.variant_080</code> | B2 | C |
| [1917](../src/gui/live/actions.rs) | <code>run</code> | <code>"Refresh inventory first."</code> | — | <code>operations.run.variant_081</code> | B2 | M |
| [1919](../src/gui/live/actions.rs) | <code>run</code> | <code>"Last protected inventory read; no fresh guest verification. Refresh obtains current host state. Pending journals are retained and are not cleared by inspection."</code> | — | <code>operations.run.variant_082</code> | B2 | C |
| [1920](../src/gui/live/actions.rs) | <code>run</code> | <code>"Observed state and recovery records"</code> | — | <code>operations.run.variant_083</code> | B2 | C |
| [1920](../src/gui/live/actions.rs) | <code>run</code> | <code>"{provenance}\n\nTechnical Details contains the host observation, recorded history and authorization from that inventory. Missing or old records do not establish current guest health. Opening these details performs no new checks or changes."</code> | <code>provenance</code> | <code>operations.run.variant_084</code> | B2 | C |
| [1924](../src/gui/live/actions.rs) | <code>run</code> | <code>"shortcuts"</code> | — | — | B2 | P |
| [1925](../src/gui/live/actions.rs) | <code>run</code> | <code>"Select a real VM and physical GPU, then enable GPU support in the draft and choose a VRAM request from 1 GB to the card's detected capacity. VRAM requests use bytes without a metadata-derived 1 GB ceiling; hardware behavior awaits live validation. Use the toggle or More actions &gt; Remove GPU support to disable support. Advanced Allocation retains exact VRAM triples; saved Compute/Encode/Decode requests are preserved but are not edited here. Apply first reviews exact enrollment and initial package setup when needed. After enrollment, Apply separately reviews GPU changes; confirmation requests elevation and any guest downtime consent. New publication uses schema 3; schema-2 input remains readable. Failed saving permits save-only retry; uncertain operations require explicit reconciliation. Settings opens the native credential dialog. GPU replacement remains unavailable. Delete profile removes only app records, requires an inactive profile and separate confirmation, and never changes VM power or GPU support."</code> | — | <code>operations.run.variant_086</code> | B2 | C |
| [1926](../src/gui/live/actions.rs) | <code>run</code> | <code>"Application help"</code> | — | <code>operations.run.variant_087</code> | B2 | C |
| [1928](../src/gui/live/actions.rs) | <code>run</code> | <code>"This action is not connected yet. Use the supported CLI; no operation was performed."</code> | — | <code>operations.run.variant_088</code> | B2 | M |
| [1930](../src/gui/live/actions.rs) | <code>run</code> | <code>"Action unavailable"</code> | — | <code>operations.run.variant_089</code> | B2 | M |
| [1936](../src/gui/live/actions.rs) | <code>run</code> | <code>"Unapplied draft"</code> | — | <code>operations.run.variant_090</code> | B2 | C |
| [1936](../src/gui/live/actions.rs) | <code>run</code> | <code>"Keep editing or discard the in-memory draft and close. Nothing has been applied."</code> | — | <code>operations.run.variant_091</code> | B2 | C |
| [1955](../src/gui/live/actions.rs) | <code>run</code> | <code>"VM status refresh unavailable: {error}. Use Refresh to retry."</code> | <code>error</code> | <code>operations.run.variant_092</code> | B2 | C |
## src/gui/live_state.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [101](../src/gui/live_state/recovery.rs) | <code>finish_apply</code> | <code>"The protected recovery receipt changed after completion. Inspect the loaded operation before any retry."</code> | — | <code>operations.finish_apply.variant_001</code> | B2 | M |
| [104](../src/gui/live_state/recovery.rs) | <code>finish_apply</code> | <code>"Applied changes were confirmed, but their bound save-only receipt is unavailable. Refresh and inspect recovery; do not repeat Apply."</code> | — | <code>operations.finish_apply.variant_002</code> | B2 | M |
| [114](../src/gui/live_state/recovery.rs) | <code>finish_apply</code> | <code>"Publication completed, but current saved intent changed or is unreadable. Refresh before editing; do not repeat GPU work."</code> | — | <code>operations.finish_apply.variant_003</code> | B2 | M |
| [143](../src/gui/live_state/recovery.rs) | <code>save_only_record</code> | <code>"No bound save-only receipt is loaded; Refresh and inspect recovery."</code> | — | <code>operations.save_only_record.variant_001</code> | B2 | M |
| [151](../src/gui/live_state/recovery.rs) | <code>save_only_record</code> | <code>"This operation requires reconciliation, not a configuration save retry."</code> | — | <code>operations.save_only_record.variant_002</code> | B2 | C |
| [161](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"Operation outcome uncertain; inspect protected recovery."</code> | — | <code>operations.applied_text.variant_001</code> | B2 | C |
| [162](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"Applied and checked; configuration publication is pending. Use the bound save-only retry."</code> | — | <code>operations.applied_text.variant_002</code> | B2 | C |
| [163](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"Applied and published; protected recovery finalization is pending."</code> | — | <code>operations.applied_text.variant_003</code> | B2 | C |
| [164](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"Graphics check completed; protected recovery finalization is pending. No configuration publication is authorized."</code> | — | <code>operations.applied_text.variant_004</code> | B2 | C |
| [173](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"Applied changes were checked; saving is not confirmed. Refresh for the bound receipt."</code> | — | <code>operations.applied_text.variant_005</code> | B2 | C |
| [176](../src/gui/live_state/recovery.rs) | <code>applied_text</code> | <code>"No unpublished operation is recorded; see current observations and saved intent separately."</code> | — | <code>operations.applied_text.variant_006</code> | B2 | C |
| [205](../src/gui/live_state/review.rs) | <code>accept_review</code> | <code>"The target, draft or saved settings changed while planning. Review again."</code> | — | <code>operations.accept_review.variant_001</code> | B2 | C |
| [218](../src/gui/live_state/review.rs) | <code>reviewed_plan</code> | <code>"Review the current draft before Apply"</code> | — | <code>operations.reviewed_plan.variant_001</code> | B2 | M |
| [224](../src/gui/live_state/review.rs) | <code>reviewed_plan</code> | <code>"The target, draft or saved settings changed after review. Review again."</code> | — | <code>operations.reviewed_plan.variant_002</code> | B2 | C |
| [256](../src/gui/live_state/draft.rs) | <code>memory_slider</code> | <code>"Selected GPU is unavailable. Use Refresh to retry."</code> | — | <code>operations.memory_slider.variant_001</code> | B2 | M |
| [259](../src/gui/live_state/draft.rs) | <code>memory_slider</code> | <code>"VRAM bounds are unavailable. Use Refresh to retry."</code> | — | <code>operations.memory_slider.variant_002</code> | B2 | M |
| [261](../src/gui/live_state/draft.rs) | <code>memory_slider</code> | <code>"VRAM fields are unavailable."</code> | — | <code>operations.memory_slider.variant_003</code> | B2 | M |
| [263](../src/gui/live_state/draft.rs) | <code>memory_slider</code> | <code>"Physical VRAM capacity could not be matched to the selected GPU. Use raw Advanced Allocation within the provider range."</code> | — | <code>operations.memory_slider.variant_004</code> | B2 | M |
| [277](../src/gui/live_state/draft.rs) | <code>memory_values</code> | <code>"Expected four complete allocation triples"</code> | — | <code>operations.memory_values.variant_001</code> | B2 | M |
| [297](../src/gui/live_state/draft.rs) | <code>memory_text_values</code> | <code>"Expected four complete allocation triples"</code> | — | <code>operations.memory_text_values.variant_001</code> | B2 | M |
| [358](../src/gui/live_state/recovery.rs) | <code>sync_recovery</code> | <code>"Recovery record is absent, but verified intent is not confirmed in readable committed configuration. Refresh and inspect before another operation."</code> | — | <code>operations.sync_recovery.variant_001</code> | B2 | M |
| [362](../src/gui/live_state/recovery.rs) | <code>sync_recovery</code> | <code>"Unsaved outcome has no verified target. Refresh and inspect before another operation."</code> | — | <code>operations.sync_recovery.variant_002</code> | B2 | M |
| [431](../src/gui/live_state/inventory.rs) | <code>target</code> | <code>"Select an available VM and physical GPU, then Refresh."</code> | — | <code>operations.target.variant_001</code> | B2 | M |
| [450](../src/gui/live_state/draft.rs) | <code>proposal_eligibility</code> | <code>"This selected GPU is not enrolled. Changing an enrolled GPU requires the separate replacement transition, which is not connected yet."</code> | — | <code>operations.proposal_eligibility.variant_001</code> | B2 | M |
| [480](../src/gui/live_state/recovery.rs) | <code>action_recovery_message</code> | <code>"A configuration import did not finish."</code> | — | <code>operations.action_recovery_message.variant_001</code> | B2 | C |
| [488](../src/gui/live_state/recovery.rs) | <code>action_recovery_message</code> | <code>"An earlier operation for {name} did not finish."</code> | <code>name</code> | <code>operations.action_recovery_message.variant_002</code> | B2 | C |
| [492](../src/gui/live_state/recovery.rs) | <code>action_recovery_message</code> | <code>"{operation}\n\nBefore verifying, repairing, removing GPU support or deleting an app profile, review what happened. Choose Review recovery to see the recorded operation and the available next step. Your edits are kept. Reviewing recovery does not make changes; any available recovery action needs separate confirmation."</code> | <code>operation</code> | <code>operations.action_recovery_message.variant_003</code> | B2 | C |
| [498](../src/gui/live_state/draft.rs) | <code>editor_eligibility</code> | <code>"Protected enrollment changed. Draft preserved; discard and Refresh before a new enrollment review."</code> | — | <code>operations.editor_eligibility.variant_001</code> | B2 | M |
| [501](../src/gui/live_state/draft.rs) | <code>editor_eligibility</code> | <code>"Committed configuration changed or became unavailable. Draft preserved; discard it to reload the latest committed intent."</code> | — | <code>operations.editor_eligibility.variant_002</code> | B2 | M |
| [508](../src/gui/live_state/draft.rs) | <code>editor_eligibility</code> | <code>"A host-wide operation requires manual reconciliation or save-only retry."</code> | — | <code>operations.editor_eligibility.variant_003</code> | B2 | C |
| [517](../src/gui/live_state/draft.rs) | <code>editor_eligibility</code> | <code>"The selected VM has pending or inconsistent managed recovery. Inspect and reconcile it before editing or reviewing new changes."</code> | — | <code>operations.editor_eligibility.variant_004</code> | B2 | M |
| [567](../src/gui/live_state/configuration.rs) | <code>saved_desired_text</code> | <code>"Committed configuration unavailable; saved intent is unknown. Editing is blocked until the file can be read and validated."</code> | — | <code>operations.saved_desired_text.variant_001</code> | B2 | M |
| [588](../src/gui/live_state/draft.rs) | <code>draft</code> | <code>"Refresh inventory first."</code> | — | <code>operations.draft.variant_001</code> | B2 | M |
| [592](../src/gui/live_state/draft.rs) | <code>draft</code> | <code>"Select an available physical GPU."</code> | — | <code>operations.draft.variant_002</code> | B2 | M |
| [596](../src/gui/live_state/draft.rs) | <code>draft</code> | <code>"Expected four complete allocation triples"</code> | — | <code>operations.draft.variant_003</code> | B2 | M |
| [603](../src/gui/live_state/draft.rs) | <code>draft</code> | <code>"Enter all three {} values as unsigned provider integers, or leave all three empty."</code> | <code>arg1</code> | <code>operations.draft.variant_004</code> | B2 | M |
| [625](../src/gui/live_state/draft.rs) | <code>removal_draft</code> | <code>"The selected VM changed; review removal again."</code> | — | <code>operations.removal_draft.variant_001</code> | B2 | M |
| [629](../src/gui/live_state/draft.rs) | <code>removal_draft</code> | <code>"GPU removal requires this exact authorized VM/GPU pair; no enrollment or guest work was performed."</code> | — | <code>operations.removal_draft.variant_002</code> | B2 | M |
| [636](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Select a VM first."</code> | — | <code>operations.forget_target.variant_001</code> | B2 | M |
| [639](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Discard or complete pending edits and Refresh before Delete profile."</code> | — | <code>operations.forget_target.variant_002</code> | B2 | C |
| [650](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Resolve recovery or conflicting/unreadable records before Delete profile."</code> | — | <code>operations.forget_target.variant_003</code> | B2 | C |
| [653](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Refresh first."</code> | — | <code>operations.forget_target.variant_004</code> | B2 | M |
| [659](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"This VM has no authorized pairing; Refresh for current state."</code> | — | <code>operations.forget_target.variant_005</code> | B2 | M |
| [660](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"VM unavailable; Refresh first."</code> | — | <code>operations.forget_target.variant_006</code> | B2 | M |
| [663](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Complete GPU removal and Refresh its managed history before Delete profile."</code> | — | <code>operations.forget_target.variant_007</code> | B2 | C |
| [674](../src/gui/live_state/profile.rs) | <code>forget_target</code> | <code>"Delete profile changes app records only and cannot remove GPU support or change VM power. {error}"</code> | <code>error</code> | <code>operations.forget_target.variant_008</code> | B2 | M |
| [716](../src/gui/live_state/profile.rs) | <code>check_forget_review</code> | <code>"Selection, saved profiles or pairing changed; review Delete profile again."</code> | — | <code>operations.check_forget_review.variant_001</code> | B2 | C |
| [736](../src/gui/live_state/profile.rs) | <code>reviewed_forget</code> | <code>"Open a fresh Delete profile review first."</code> | — | <code>operations.reviewed_forget.variant_001</code> | B2 | M |
| [758](../src/gui/live_state/review_text.rs) | <code>memory_review</code> | <code>"GPU memory (bytes: minimum / optimal / maximum): {} → {}\n"</code> | <code>arg1, arg2</code> | <code>operations.memory_review.variant_001</code> | B2 | C |
| [759](../src/gui/live_state/review_text.rs), [760](../src/gui/live_state/review_text.rs), [765](../src/gui/live_state/review_text.rs) | <code>memory_review</code> | <code>"Unknown"</code> | — | <code>operations.memory_review.variant_002</code> | B2 | C |
| [764](../src/gui/live_state/review_text.rs) | <code>memory_review</code> | <code>"GPU memory request (bytes: minimum / optimal / maximum): {}. No VRAM write is planned.\n"</code> | <code>arg1</code> | <code>operations.memory_review.variant_003</code> | B2 | C |
| [768](../src/gui/live_state/review_text.rs) | <code>memory_review</code> | <code>"GPU memory: no explicit request.\n"</code> | — | <code>operations.memory_review.variant_004</code> | B2 | C |
| [775](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"Unspecified; no explicit VRAM request"</code> | — | <code>operations.review_text.variant_001</code> | B2 | C |
| [779](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"No allocation write planned; existing/provider values are not explicitly reset"</code> | — | <code>operations.review_text.variant_002</code> | B2 | C |
| [789](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"{} draft (Minimum / Optimal / Maximum): {}\nPlanned {} write: {}"</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.review_text.variant_003</code> | B2 | C |
| [793](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"Unspecified; no explicit request"</code> | — | <code>operations.review_text.variant_004</code> | B2 | C |
| [798](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"No explicit category write; new/recreated attachments use provider defaults"</code> | — | <code>operations.review_text.variant_005</code> | B2 | C |
| [803](../src/gui/live_state/review_text.rs), [814](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"\n"</code> | — | — | B2 | F |
| [805](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"VM: {}\nVM ID: {}\nGPU interface: {}\nDesired GPU support: {}\nVRAM draft (Minimum / Optimal / Maximum): {requested}\nPlanned allocation write: {planned}\n{extended}\n\n{}\n\nAllocation units: {}\nConfiguration publication: schema 3, retaining schema-2 input compatibility.\n\nPreview from the shared CLI planner. No VM or guest changes have been performed during review."</code> | <code>arg1, arg2, arg3, arg4, requested, planned, extended, arg8, arg9</code> | <code>operations.review_text.variant_007</code> | B2 | C |
| [810](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"Enabled"</code> | — | <code>operations.review_text.variant_008</code> | B2 | C |
| [812](../src/gui/live_state/review_text.rs) | <code>review_text</code> | <code>"Disabled"</code> | — | <code>operations.review_text.variant_009</code> | B2 | C |
| [823](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"VM: {}"</code> | <code>arg1</code> | <code>operations.review_summary.variant_001</code> | B2 | C |
| [825](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"GPU support: {} → {}"</code> | <code>arg1, arg2</code> | <code>operations.review_summary.variant_002</code> | B2 | C |
| [827](../src/gui/live_state/review_text.rs), [834](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Disabled"</code> | — | <code>operations.review_summary.variant_003</code> | B2 | C |
| [829](../src/gui/live_state/review_text.rs), [832](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Enabled"</code> | — | <code>operations.review_summary.variant_004</code> | B2 | C |
| [839](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Repair / Reinstall: run complete signed driver preparation even with matching history, verify current payload and selected-hardware rendering, then leave the VM off."</code> | — | <code>operations.review_summary.variant_005</code> | B2 | C |
| [842](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Remove GPU support: detach the exact managed GPU if present, restore still-owned compatibility values and leave the VM off. Keep guest driver files, authorization and setup history. No guest credentials or graphics check."</code> | — | <code>operations.review_summary.variant_006</code> | B2 | C |
| [845](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Initial GPU setup: check hardware rendering, then gracefully leave the VM off. Start it through Hyper-V after setup passes."</code> | — | <code>operations.review_summary.variant_007</code> | B2 | C |
| [852](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"GPU setup/update required: preparation changed or is invalid. Review driver work, hardware rendering and final shutdown before confirming."</code> | — | <code>operations.review_summary.variant_008</code> | B2 | C |
| [854](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Verify GPU setup required: check current guest driver files, setup record and hardware rendering without reinstalling drivers. Apply follows the final power state shown below."</code> | — | <code>operations.review_summary.variant_009</code> | B2 | C |
| [856](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Ordinary GPU change: no guest access or graphics check. Leave the VM off after success; start it through Hyper-V when ready."</code> | — | <code>operations.review_summary.variant_010</code> | B2 | C |
| [866](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"{} (Minimum / Optimal / Maximum, raw provider values): {} → {}"</code> | <code>arg1, arg2, arg3</code> | <code>operations.review_summary.variant_011</code> | B2 | C |
| [868](../src/gui/live_state/review_text.rs), [869](../src/gui/live_state/review_text.rs), [931](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Unknown"</code> | — | <code>operations.review_summary.variant_012</code> | B2 | C |
| [876](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Low MMIO (MiB)"</code> | — | <code>operations.review_summary.variant_013</code> | B2 | C |
| [881](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"High MMIO (MiB)"</code> | — | <code>operations.review_summary.variant_014</code> | B2 | C |
| [886](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Guest cache types"</code> | — | <code>operations.review_summary.variant_015</code> | B2 | C |
| [891](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Automatic checkpoints"</code> | — | <code>operations.review_summary.variant_016</code> | B2 | C |
| [897](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"{name}: {before} → {after}"</code> | <code>name, before, after</code> | <code>operations.review_summary.variant_017</code> | B2 | C |
| [901](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"\nPlanned steps, in order:"</code> | — | <code>operations.review_summary.variant_018</code> | B2 | C |
| [903](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"No VM or guest changes are needed. Save the reviewed intent."</code> | — | <code>operations.review_summary.variant_019</code> | B2 | C |
| [912](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Gracefully stop the VM."</code> | — | <code>operations.review_summary.variant_020</code> | B2 | C |
| [913](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Start the VM."</code> | — | <code>operations.review_summary.variant_021</code> | B2 | C |
| [914](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Remove the GPU attachment."</code> | — | <code>operations.review_summary.variant_022</code> | B2 | C |
| [916](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Prepare the guest with the current signed GPU driver files."</code> | — | <code>operations.review_summary.variant_023</code> | B2 | C |
| [918](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Update or restore GPU compatibility settings."</code> | — | <code>operations.review_summary.variant_024</code> | B2 | C |
| [919](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Attach the selected physical GPU."</code> | — | <code>operations.review_summary.variant_025</code> | B2 | C |
| [920](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Set the GPU resources shown above."</code> | — | <code>operations.review_summary.variant_026</code> | B2 | C |
| [921](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Verify GPU setup: check driver files, setup record, device health and hardware rendering."</code> | — | <code>operations.review_summary.variant_027</code> | B2 | C |
| [923](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"{}. {step}"</code> | <code>arg1, step</code> | <code>operations.review_summary.variant_028</code> | B2 | C |
| [927](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"\nExpected final VM power after successful completion: {}."</code> | <code>arg1</code> | <code>operations.review_summary.variant_029</code> | B2 | C |
| [929](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Running"</code> | — | <code>operations.review_summary.variant_030</code> | B2 | M |
| [930](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Stopped"</code> | — | <code>operations.review_summary.variant_031</code> | B2 | M |
| [935](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"The running VM will be interrupted. Save work inside it before Apply. Approval below is required."</code> | — | <code>operations.review_summary.variant_032</code> | B2 | C |
| [938](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Driver preparation and file transfer can take several minutes."</code> | — | <code>operations.review_summary.variant_033</code> | B2 | C |
| [943](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Allow time for Windows to start or shut down gracefully. Shutdown refusal stops further changes."</code> | — | <code>operations.review_summary.variant_034</code> | B2 | C |
| [946](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"The graphics check may take additional time and requires Windows credentials inside this VM."</code> | — | <code>operations.review_summary.variant_035</code> | B2 | C |
| [949](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Completion time varies; no exact duration is promised. Keep the application open until it reports an outcome."</code> | — | <code>operations.review_summary.variant_036</code> | B2 | C |
| [952](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Externally changed compatibility settings will be preserved."</code> | — | <code>operations.review_summary.variant_037</code> | B2 | C |
| [955](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"Interrupted work requires recovery before further changes."</code> | — | <code>operations.review_summary.variant_038</code> | B2 | C |
| [957](../src/gui/live_state/review_text.rs) | <code>review_summary</code> | <code>"\n"</code> | — | — | B2 | F |
| [963](../src/gui/live_state/review_text.rs) | <code>allocation_description</code> | <code>"{} / {} / {}"</code> | <code>arg1, arg2, arg3</code> | — | B2 | F |
## src/gui/memory.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [26](../src/gui/memory.rs#L26) | <code>format_gb</code> | <code>"{whole} GB"</code> | <code>whole</code> | — | B2 | F |
| [28](../src/gui/memory.rs#L28) | <code>format_gb</code> | <code>"{fraction:09}"</code> | <code>fraction:09</code> | — | B2 | F |
| [29](../src/gui/memory.rs#L29) | <code>format_gb</code> | <code>"{whole}.{} GB"</code> | <code>whole, arg2</code> | — | B2 | F |
| [36](../src/gui/memory.rs#L36) | <code>rounded_capacity</code> | <code>"{whole} GB"</code> | <code>whole</code> | — | B2 | F |
| [41](../src/gui/memory.rs#L41) | <code>format_gib</code> | <code>"{:.2} GiB"</code> | <code>:.2</code> | — | B2 | F |
| [47](../src/gui/memory.rs#L47) | <code>request</code> | <code>"VRAM fields are unavailable."</code> | — | <code>operations.request.variant_001</code> | B2 | M |
| [56](../src/gui/memory.rs#L56) | <code>request</code> | <code>"Complete or clear all three VRAM fields to use the slider."</code> | — | <code>operations.request.variant_002</code> | B2 | M |
| [73](../src/gui/memory.rs#L73) | <code>new</code> | <code>"Detected physical VRAM is below the slider's 1 GB minimum. GPU memory selection is unavailable."</code> | — | <code>operations.new.variant_001</code> | B2 | M |
| [78](../src/gui/memory.rs#L78) | <code>new</code> | <code>"Correct the VRAM range in Advanced Allocation to use the slider."</code> | — | <code>operations.new.variant_002</code> | B2 | M |
| [84](../src/gui/memory.rs#L84) | <code>new</code> | <code>"The current request exceeds detected physical VRAM. Use Advanced Allocation; it has not been rewritten."</code> | — | <code>operations.new.variant_003</code> | B2 | M |
| [88](../src/gui/memory.rs#L88) | <code>new</code> | <code>"VRAM bounds exceed exact slider precision. Use Advanced Allocation."</code> | — | <code>operations.new.variant_004</code> | B2 | C |
| [100](../src/gui/memory.rs#L100) | <code>new</code> | <code>"No explicit request"</code> | — | <code>operations.new.variant_005</code> | B2 | C |
| [114](../src/gui/memory.rs#L114) | <code>new</code> | <code>"The maximum endpoint label is rounded to the nearest whole GB. Review and Technical Details retain the exact byte values."</code> | — | <code>operations.new.variant_006</code> | B2 | C |
| [117](../src/gui/memory.rs#L117) | <code>new</code> | <code>"The existing request is below 1 GB and has been retained exactly. The slider starts at 1 GB; edit it or use Advanced Allocation to change the request."</code> | — | <code>operations.new.variant_007</code> | B2 | C |
| [119](../src/gui/memory.rs#L119) | <code>new</code> | <code>"\n"</code> | — | — | B2 | F |
| [128](../src/gui/memory.rs#L128) | <code>select</code> | <code>"The selected memory value is outside the current VRAM range."</code> | — | <code>operations.select.variant_001</code> | B2 | M |
| [131](../src/gui/memory.rs#L131) | <code>select</code> | <code>"Invalid memory value"</code> | — | <code>operations.select.variant_002</code> | B2 | M |
| [133](../src/gui/memory.rs#L133) | <code>select</code> | <code>"Memory conversion overflow"</code> | — | <code>operations.select.variant_003</code> | B2 | M |
| [141](../src/gui/memory.rs#L141) | <code>select_gb</code> | <code>""</code> | — | — | B2 | F |
| [147](../src/gui/memory.rs#L147) | <code>select_gb</code> | <code>"Enter a decimal GB value with at most nine decimal places. The previous request is kept."</code> | — | <code>operations.select_gb.variant_002</code> | B2 | M |
| [151](../src/gui/memory.rs#L151), [158](../src/gui/memory.rs#L158) | <code>select_gb</code> | <code>"Memory value is too large"</code> | — | <code>operations.select_gb.variant_003</code> | B2 | M |
| [152](../src/gui/memory.rs#L152) | <code>select_gb</code> | <code>"{fraction:0&lt;9}"</code> | <code>fraction:0&lt;9</code> | <code>operations.select_gb.variant_004</code> | B2 | C |
| [154](../src/gui/memory.rs#L154) | <code>select_gb</code> | <code>"Invalid decimal memory value"</code> | — | <code>operations.select_gb.variant_005</code> | B2 | M |
| [161](../src/gui/memory.rs#L161) | <code>select_gb</code> | <code>"Enter a request from 1 GB to {}. The previous request is kept."</code> | <code>arg1</code> | <code>operations.select_gb.variant_006</code> | B2 | C |
## src/gui/mod.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [25](../src/gui/mod.rs#L25), [30](../src/gui/mod.rs#L30) | <code>dialog</code> | <code>""</code> | — | — | B2 | F |
| [27](../src/gui/mod.rs#L27), [34](../src/gui/mod.rs#L34) | <code>dialog</code> | <code>"error"</code> | — | — | B2 | P |
| [46](../src/gui/mod.rs#L46) | <code>set_details</code> | <code>""</code> | — | — | B2 | F |
| [47](../src/gui/mod.rs#L47) | <code>set_details</code> | <code>"Source: facts captured for this review/record. Fresh protected checks are required before any effects.\n\n{details}"</code> | <code>details</code> | <code>operations.set_details.variant_002</code> | B2 | C |
## src/gui/result_notice.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [17](../src/gui/result_notice.rs#L17) | <code>unavailable_body</code> | <code>"{}\n\nCurrent VM status is unavailable: {error}\nRetry the status update before another review."</code> | <code>arg1, error</code> | <code>operations.unavailable_body.variant_001</code> | B2 | C |
## src/gui/wording.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [9](../src/gui/wording.rs#L9) | <code>power_name</code> | <code>"Off"</code> | — | <code>operations.power_name.variant_001</code> | B2 | M |
| [10](../src/gui/wording.rs#L10) | <code>power_name</code> | <code>"Running"</code> | — | <code>operations.power_name.variant_002</code> | B2 | M |
| [11](../src/gui/wording.rs#L11) | <code>power_name</code> | <code>"Unknown"</code> | — | <code>operations.power_name.variant_003</code> | B2 | C |
| [17](../src/gui/wording.rs#L17) | <code>apply_consent</code> | <code>"Approve reviewed starts/shutdowns; final state after successful completion: {}."</code> | <code>arg1</code> | <code>operations.apply_consent.variant_001</code> | B2 | C |
| [24](../src/gui/wording.rs#L24) | <code>recovery_consent</code> | <code>"Approve reviewed power changes; expected final state if confirmed: {}."</code> | <code>arg1</code> | <code>operations.recovery_consent.variant_001</code> | B2 | C |
| [32](../src/gui/wording.rs#L32) | <code>verify_review</code> | <code>"Temporarily start this VM for the check, then gracefully shut it down to Off."</code> | — | <code>operations.verify_review.variant_001</code> | B2 | C |
| [34](../src/gui/wording.rs#L34) | <code>verify_review</code> | <code>"This VM is already running and is expected to remain Running."</code> | — | <code>operations.verify_review.variant_002</code> | B2 | C |
| [36](../src/gui/wording.rs#L36) | <code>verify_review</code> | <code>"The VM's power state is unknown or changing. Refresh before a new review."</code> | — | <code>operations.verify_review.variant_003</code> | B2 | C |
| [40](../src/gui/wording.rs#L40) | <code>verify_review</code> | <code>"Virtual machine: {}\nPhysical GPU: {}\n\nVerify GPU setup: compare the current guest driver files and setup record with the approved driver package, then check the GPU's device health and hardware rendering. This checks current setup; a past successful check alone does not establish current health.\n\n{lifecycle}\nExpected final power after successful completion: {}.\n\nUse the Windows administrator sign-in for this VM if prompted. File checks, starting Windows and rendering can take time; no exact duration is promised. This refreshes the check tools and diagnostic information inside the VM, but does not reinstall drivers, change GPU assignment or save settings.\n\nVerify GPU setup confirms this review. Cancel keeps your settings and starts no check."</code> | <code>arg1, arg2, lifecycle, arg4</code> | <code>operations.verify_review.variant_004</code> | B2 | C |
| [49](../src/gui/wording.rs#L49) | <code>verify_details</code> | <code>"VM ID: {}\nGPU interface: {}\nDriver version: {}\nAuthenticated payload digest: {}\nObserved power: {:?}\n\nCompares current guest destinations and the preparation receipt with the authenticated host manifest before device health/rendering. Protected guest helpers, manifest metadata and optional diagnostics may be refreshed. Fresh protected admission independently checks the reviewed state."</code> | <code>arg1, arg2, arg3, arg4, :?</code> | <code>operations.verify_details.variant_001</code> | B2 | C |
| [61](../src/gui/wording.rs#L61) | <code>verify_consent</code> | <code>"Approve temporarily starting this VM, then gracefully shutting it down to Off."</code> | — | <code>operations.verify_consent.variant_001</code> | B2 | C |
| [63](../src/gui/wording.rs#L63) | <code>verify_consent</code> | <code>"This VM is expected to remain Running during Verify GPU setup."</code> | — | <code>operations.verify_consent.variant_002</code> | B2 | C |
| [65](../src/gui/wording.rs#L65) | <code>verify_consent</code> | <code>"Power state is unknown. Refresh and review again before continuing."</code> | — | <code>operations.verify_consent.variant_003</code> | B2 | C |
| [72](../src/gui/wording.rs#L72) | <code>guest_progress</code> | <code>"Apply approved changes"</code> | — | <code>operations.guest_progress.variant_001</code> | B2 | M |
| [73](../src/gui/wording.rs#L73) | <code>guest_progress</code> | <code>"Applying the changes you reviewed, including any required driver preparation and GPU setup checks. Successful completion leaves the VM off."</code> | — | <code>operations.guest_progress.variant_002</code> | B2 | C |
| [75](../src/gui/wording.rs#L75) | <code>guest_progress</code> | <code>"Verify GPU setup"</code> | — | <code>operations.guest_progress.variant_003</code> | B2 | M |
| [76](../src/gui/wording.rs#L76) | <code>guest_progress</code> | <code>"Checking the current driver files, setup record, device health and hardware rendering. Drivers are not being reinstalled. VM power follows the reviewed sequence."</code> | — | <code>operations.guest_progress.variant_004</code> | B2 | C |
| [78](../src/gui/wording.rs#L78) | <code>guest_progress</code> | <code>"Check recorded outcome"</code> | — | <code>operations.guest_progress.variant_005</code> | B2 | M |
| [79](../src/gui/wording.rs#L79) | <code>guest_progress</code> | <code>"Checking the interrupted operation and any reviewed guest/power steps. GPU assignment and driver preparation are not repeated. Uncertain results keep recovery blocked."</code> | — | <code>operations.guest_progress.variant_006</code> | B2 | C |
| [81](../src/gui/wording.rs#L81) | <code>guest_progress</code> | <code>"Guest credentials"</code> | — | <code>operations.guest_progress.variant_007</code> | B2 | M |
| [82](../src/gui/wording.rs#L82) | <code>guest_progress</code> | <code>"Managing this VM's optional saved sign-in details. This does not connect to the VM, prepare drivers or check graphics."</code> | — | <code>operations.guest_progress.variant_008</code> | B2 | C |
| [85](../src/gui/wording.rs#L85) | <code>guest_progress</code> | <code>"Performing the guest steps shown in your review. Wait for the actual result before treating the operation as complete."</code> | — | <code>operations.guest_progress.variant_009</code> | B2 | C |
| [89](../src/gui/wording.rs#L89) | <code>guest_progress</code> | <code>"Virtual machine: {name}\n\n{purpose}\n\nIf prompted, use the Windows administrator account inside this VM. Remembering it is optional. Progress appears below."</code> | <code>name, purpose</code> | <code>operations.guest_progress.variant_010</code> | B2 | C |
| [95](../src/gui/wording.rs#L95) | <code>credential_cancellation</code> | <code>"No recovery action was started. The unfinished operation and its recovery hold remain. Your pending edits are kept. Review recovery again when ready."</code> | — | <code>operations.credential_cancellation.variant_001</code> | B2 | C |
| [97](../src/gui/wording.rs#L97) | <code>credential_cancellation</code> | <code>"No guest operation was started. Your pending edits are kept; this does not mean setup or verification completed. Review the action again when ready."</code> | — | <code>operations.credential_cancellation.variant_002</code> | B2 | C |
## src/gui/ui/app.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/app.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/app.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [3](../src/gui/ui/app.slint#L3) | <code>defaults</code> | <code>"cards.slint"</code> | — | — | B1 | X |
| [4](../src/gui/ui/app.slint#L4) | <code>defaults</code> | <code>"details.slint"</code> | — | — | B1 | X |
| [5](../src/gui/ui/app.slint#L5) | <code>defaults</code> | <code>"pages.slint"</code> | — | — | B1 | X |
| [6](../src/gui/ui/app.slint#L6) | <code>defaults</code> | <code>"dialogs.slint"</code> | — | — | B1 | X |
| [9](../src/gui/ui/app.slint#L9), [95](../src/gui/ui/app.slint#L95) | <code>AppWindow</code> | <code>"Hyper GPU Support"</code> | — | <code>dashboard.AppWindow.variant_001</code> | B1 | C |
| [13](../src/gui/ui/app.slint#L13) | <code>AppWindow</code> | <code>"Segoe UI"</code> | — | — | B1 | X |
| [21](../src/gui/ui/app.slint#L21), [23](../src/gui/ui/app.slint#L23), [37](../src/gui/ui/app.slint#L37), [38](../src/gui/ui/app.slint#L38), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [63](../src/gui/ui/app.slint#L63), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [66](../src/gui/ui/app.slint#L66), [81](../src/gui/ui/app.slint#L81), [152](../src/gui/ui/app.slint#L152), [157](../src/gui/ui/app.slint#L157), [177](../src/gui/ui/app.slint#L177), [187](../src/gui/ui/app.slint#L187), [187](../src/gui/ui/app.slint#L187), [190](../src/gui/ui/app.slint#L190) | <code>AppWindow</code> | <code>""</code> | — | — | B1 | F |
| [26](../src/gui/ui/app.slint#L26), [27](../src/gui/ui/app.slint#L27), [28](../src/gui/ui/app.slint#L28), [30](../src/gui/ui/app.slint#L30) | <code>AppWindow</code> | <code>"Select a VM first."</code> | — | <code>dashboard.AppWindow.variant_002</code> | B1 | C |
| [39](../src/gui/ui/app.slint#L39), [40](../src/gui/ui/app.slint#L40) | <code>AppWindow</code> | <code>"Unknown"</code> | — | <code>dashboard.AppWindow.variant_003</code> | B1 | C |
| [41](../src/gui/ui/app.slint#L41) | <code>AppWindow</code> | <code>"Not read"</code> | — | <code>dashboard.AppWindow.variant_004</code> | B1 | C |
| [42](../src/gui/ui/app.slint#L42) | <code>AppWindow</code> | <code>"No saved configuration"</code> | — | <code>dashboard.AppWindow.variant_005</code> | B1 | C |
| [43](../src/gui/ui/app.slint#L43) | <code>AppWindow</code> | <code>"No unapplied edits"</code> | — | <code>dashboard.AppWindow.variant_006</code> | B1 | C |
| [44](../src/gui/ui/app.slint#L44) | <code>AppWindow</code> | <code>"No unpublished operation recorded"</code> | — | <code>dashboard.AppWindow.variant_007</code> | B1 | C |
| [45](../src/gui/ui/app.slint#L45) | <code>AppWindow</code> | <code>"Not verified"</code> | — | <code>dashboard.AppWindow.variant_008</code> | B1 | C |
| [59](../src/gui/ui/app.slint#L59) | <code>AppWindow</code> | <code>"Unavailable"</code> | — | <code>dashboard.AppWindow.variant_009</code> | B1 | C |
| [65](../src/gui/ui/app.slint#L65), [65](../src/gui/ui/app.slint#L65), [65](../src/gui/ui/app.slint#L65), [65](../src/gui/ui/app.slint#L65) | <code>AppWindow</code> | <code>"Capability unknown"</code> | — | <code>dashboard.AppWindow.variant_010</code> | B1 | C |
| [69](../src/gui/ui/app.slint#L69) | <code>AppWindow</code> | <code>"Reading virtual machine information."</code> | — | <code>dashboard.AppWindow.variant_011</code> | B1 | C |
| [94](../src/gui/ui/app.slint#L94) | <code>AppWindow</code> | <code>"H"</code> | — | <code>dashboard.AppWindow.variant_012</code> | B1 | C |
| [104](../src/gui/ui/app.slint#L104) | <code>AppWindow</code> | <code>"Dashboard"</code> | — | <code>dashboard.AppWindow.variant_013</code> | B1 | C |
| [105](../src/gui/ui/app.slint#L105) | <code>AppWindow</code> | <code>"System"</code> | — | <code>dashboard.AppWindow.variant_014</code> | B1 | C |
| [106](../src/gui/ui/app.slint#L106) | <code>AppWindow</code> | <code>"Settings"</code> | — | <code>dashboard.AppWindow.variant_015</code> | B1 | C |
| [107](../src/gui/ui/app.slint#L107) | <code>AppWindow</code> | <code>"About"</code> | — | <code>dashboard.AppWindow.variant_016</code> | B1 | C |
| [116](../src/gui/ui/app.slint#L116) | <code>AppWindow</code> | <code>"Pending protected recovery record. New GPU changes are blocked; inspection does not clear recovery."</code> | — | <code>dashboard.AppWindow.variant_017</code> | B1 | C |
| [119](../src/gui/ui/app.slint#L119), [120](../src/gui/ui/app.slint#L120) | <code>AppWindow</code> | <code>"recovery"</code> | — | — | B1 | P |
| [120](../src/gui/ui/app.slint#L120) | <code>AppWindow</code> | <code>"Inspect recovery"</code> | — | <code>dashboard.AppWindow.variant_018</code> | B1 | C |
| [132](../src/gui/ui/app.slint#L132) | <code>AppWindow</code> | <code>"Virtual Machines"</code> | — | <code>dashboard.AppWindow.variant_019</code> | B1 | C |
| [134](../src/gui/ui/app.slint#L134), [135](../src/gui/ui/app.slint#L135) | <code>AppWindow</code> | <code>"refresh"</code> | — | — | B1 | P |
| [135](../src/gui/ui/app.slint#L135) | <code>AppWindow</code> | <code>"Refresh"</code> | — | <code>dashboard.AppWindow.variant_020</code> | B1 | C |
| [137](../src/gui/ui/app.slint#L137) | <code>AppWindow</code> | <code>" VMs shown"</code> | — | <code>dashboard.AppWindow.variant_021</code> | B1 | C |
| [138](../src/gui/ui/app.slint#L138) | <code>AppWindow</code> | <code>"No live VM rows available. Check the discovery message or Refresh."</code> | — | <code>dashboard.AppWindow.variant_022</code> | B1 | C |
| [157](../src/gui/ui/app.slint#L157) | <code>AppWindow</code> | <code>"Updating VM status"</code> | — | <code>dashboard.AppWindow.variant_023</code> | B1 | M |
| [160](../src/gui/ui/app.slint#L160) | <code>AppWindow</code> | <code>"Updating VM status..."</code> | — | <code>dashboard.AppWindow.variant_024</code> | B1 | C |
| [179](../src/gui/ui/app.slint#L179) | <code>AppWindow</code> | <code>"Your VM workspace starts here"</code> | — | <code>dashboard.AppWindow.variant_025</code> | B1 | C |
| [179](../src/gui/ui/app.slint#L179) | <code>AppWindow</code> | <code>"Choose a discovered VM on the left. Discovery never enrolls or modifies it."</code> | — | <code>dashboard.AppWindow.variant_026</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Review recovery before new changes"</code> | — | <code>dashboard.AppWindow.variant_027</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Updating VM information"</code> | — | <code>dashboard.AppWindow.variant_028</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Select a VM"</code> | — | <code>dashboard.AppWindow.variant_029</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Check the selected VM's status; setup or Refresh may be required"</code> | — | <code>dashboard.AppWindow.variant_030</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Pending edits; review required before changes"</code> | — | <code>dashboard.AppWindow.variant_031</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"No pending edits"</code> | — | <code>dashboard.AppWindow.variant_032</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"Operation in progress"</code> | — | <code>dashboard.AppWindow.variant_033</code> | B1 | C |
| [187](../src/gui/ui/app.slint#L187) | <code>AppWindow</code> | <code>"No operation running"</code> | — | <code>dashboard.AppWindow.variant_034</code> | B1 | C |
## src/gui/ui/cards.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/cards.slint#L1) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [10](../src/gui/ui/cards.slint#L10), [13](../src/gui/ui/cards.slint#L13) | <code>GuestOsIcon</code> | <code>"windows"</code> | — | — | B1 | P |
| [10](../src/gui/ui/cards.slint#L10) | <code>GuestOsIcon</code> | <code>"Windows guest, sample metadata"</code> | — | <code>dashboard.GuestOsIcon.variant_001</code> | B1 | C |
| [10](../src/gui/ui/cards.slint#L10), [11](../src/gui/ui/cards.slint#L11), [12](../src/gui/ui/cards.slint#L12) | <code>GuestOsIcon</code> | <code>"linux"</code> | — | — | B1 | P |
| [10](../src/gui/ui/cards.slint#L10) | <code>GuestOsIcon</code> | <code>"Linux guest, sample metadata"</code> | — | <code>dashboard.GuestOsIcon.variant_002</code> | B1 | C |
| [10](../src/gui/ui/cards.slint#L10) | <code>GuestOsIcon</code> | <code>"Unknown guest OS"</code> | — | <code>dashboard.GuestOsIcon.variant_003</code> | B1 | C |
| [11](../src/gui/ui/cards.slint#L11) | <code>GuestOsIcon</code> | <code>"icons/linux.svg"</code> | — | — | B1 | X |
| [13](../src/gui/ui/cards.slint#L13) | <code>GuestOsIcon</code> | <code>"icons/windows.svg"</code> | — | — | B1 | X |
| [13](../src/gui/ui/cards.slint#L13) | <code>GuestOsIcon</code> | <code>"icons/monitor.svg"</code> | — | — | B1 | X |
| [36](../src/gui/ui/cards.slint#L36) | <code>VmCard</code> | <code>" "</code> | — | — | B1 | F |
| [55](../src/gui/ui/cards.slint#L55) | <code>VmCard</code> | <code>"Running"</code> | — | <code>dashboard.VmCard.variant_001</code> | B1 | M |
| [55](../src/gui/ui/cards.slint#L55) | <code>VmCard</code> | <code>"Stopped"</code> | — | <code>dashboard.VmCard.variant_002</code> | B1 | M |
| [55](../src/gui/ui/cards.slint#L55) | <code>VmCard</code> | <code>"Off"</code> | — | <code>dashboard.VmCard.variant_003</code> | B1 | M |
| [58](../src/gui/ui/cards.slint#L58) | <code>VmCard</code> | <code>"Updating VM status"</code> | — | <code>dashboard.VmCard.variant_004</code> | B1 | M |
| [59](../src/gui/ui/cards.slint#L59), [61](../src/gui/ui/cards.slint#L61) | <code>VmCard</code> | <code>""</code> | — | — | B1 | F |
## src/gui/ui/details.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/details.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/details.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [3](../src/gui/ui/details.slint#L3) | <code>defaults</code> | <code>"resources.slint"</code> | — | — | B1 | X |
| [4](../src/gui/ui/details.slint#L4) | <code>defaults</code> | <code>"technical-output.slint"</code> | — | — | B1 | X |
| [10](../src/gui/ui/details.slint#L10) | <code>VmDetails</code> | <code>"refresh"</code> | — | — | B1 | P |
| [10](../src/gui/ui/details.slint#L10), [105](../src/gui/ui/details.slint#L105) | <code>VmDetails</code> | <code>"recovery"</code> | — | — | B1 | P |
| [16](../src/gui/ui/details.slint#L16), [103](../src/gui/ui/details.slint#L103) | <code>VmDetails</code> | <code>"verify"</code> | — | — | B1 | P |
| [16](../src/gui/ui/details.slint#L16), [59](../src/gui/ui/details.slint#L59), [69](../src/gui/ui/details.slint#L69), [70](../src/gui/ui/details.slint#L70), [70](../src/gui/ui/details.slint#L70), [70](../src/gui/ui/details.slint#L70), [99](../src/gui/ui/details.slint#L99), [174](../src/gui/ui/details.slint#L174), [174](../src/gui/ui/details.slint#L174), [174](../src/gui/ui/details.slint#L174), [176](../src/gui/ui/details.slint#L176), [177](../src/gui/ui/details.slint#L177), [181](../src/gui/ui/details.slint#L181), [185](../src/gui/ui/details.slint#L185) | <code>VmDetails</code> | <code>""</code> | — | — | B1 | F |
| [17](../src/gui/ui/details.slint#L17), [185](../src/gui/ui/details.slint#L185) | <code>VmDetails</code> | <code>"review"</code> | — | — | B1 | P |
| [81](../src/gui/ui/details.slint#L81) | <code>VmDetails</code> | <code>" · "</code> | — | — | B1 | F |
| [82](../src/gui/ui/details.slint#L82) | <code>VmDetails</code> | <code>"● Draft changed"</code> | — | <code>dashboard.VmDetails.variant_001</code> | B1 | C |
| [96](../src/gui/ui/details.slint#L96) | <code>VmDetails</code> | <code>"GPU support"</code> | — | <code>dashboard.VmDetails.variant_002</code> | B1 | C |
| [100](../src/gui/ui/details.slint#L100) | <code>VmDetails</code> | <code>"Verify GPU setup"</code> | — | <code>dashboard.VmDetails.variant_003</code> | B1 | M |
| [105](../src/gui/ui/details.slint#L105) | <code>VmDetails</code> | <code>"Review recovery"</code> | — | <code>dashboard.VmDetails.variant_004</code> | B1 | C |
| [106](../src/gui/ui/details.slint#L106) | <code>VmDetails</code> | <code>"Enable GPU support in this draft"</code> | — | <code>dashboard.VmDetails.variant_005</code> | B1 | C |
| [107](../src/gui/ui/details.slint#L107) | <code>VmDetails</code> | <code>"Physical GPU"</code> | — | <code>dashboard.VmDetails.variant_006</code> | B1 | C |
| [109](../src/gui/ui/details.slint#L109) | <code>VmDetails</code> | <code>"This VM is authorized for its recorded GPU. Changing that GPU is unavailable; Refresh if the recorded GPU is missing. Your saved memory requests are kept."</code> | — | <code>dashboard.VmDetails.variant_007</code> | B1 | C |
| [111](../src/gui/ui/details.slint#L111) | <code>VmDetails</code> | <code>"GPU Memory request"</code> | — | <code>dashboard.VmDetails.variant_008</code> | B1 | C |
| [117](../src/gui/ui/details.slint#L117) | <code>VmDetails</code> | <code>"GPU memory request in decimal GB"</code> | — | <code>dashboard.VmDetails.variant_009</code> | B1 | C |
| [118](../src/gui/ui/details.slint#L118) | <code>VmDetails</code> | <code>"GB"</code> | — | <code>dashboard.VmDetails.variant_010</code> | B1 | C |
| [124](../src/gui/ui/details.slint#L124) | <code>VmDetails</code> | <code>"Set request"</code> | — | <code>dashboard.VmDetails.variant_011</code> | B1 | C |
| [129](../src/gui/ui/details.slint#L129) | <code>VmDetails</code> | <code>"Enter GB and press Enter or Set request. Use the slider's arrow keys for 0.1 GB steps."</code> | — | <code>dashboard.VmDetails.variant_012</code> | B1 | C |
| [132](../src/gui/ui/details.slint#L132) | <code>VmDetails</code> | <code>"Requested GPU memory in GB"</code> | — | <code>dashboard.VmDetails.variant_013</code> | B1 | C |
| [142](../src/gui/ui/details.slint#L142) | <code>VmDetails</code> | <code>"Use "</code> | — | <code>dashboard.VmDetails.variant_014</code> | B1 | C |
| [149](../src/gui/ui/details.slint#L149) | <code>VmDetails</code> | <code>"Changes take effect only after review and approval. GPU sharing is not qualified."</code> | — | <code>dashboard.VmDetails.variant_015</code> | B1 | C |
| [152](../src/gui/ui/details.slint#L152) | <code>VmDetails</code> | <code>"Setup required"</code> | — | <code>dashboard.VmDetails.variant_016</code> | B1 | M |
| [155](../src/gui/ui/details.slint#L155) | <code>VmDetails</code> | <code>"First-time setup"</code> | — | <code>dashboard.VmDetails.variant_017</code> | B1 | C |
| [156](../src/gui/ui/details.slint#L156) | <code>VmDetails</code> | <code>"This Generation 2 VM has no saved GPU configuration. Review the pairing and enrollment before preparing the guest."</code> | — | <code>dashboard.VmDetails.variant_018</code> | B1 | C |
| [157](../src/gui/ui/details.slint#L157) | <code>VmDetails</code> | <code>"Review enrollment &amp; NVIDIA setup"</code> | — | <code>dashboard.VmDetails.variant_019</code> | B1 | C |
| [157](../src/gui/ui/details.slint#L157) | <code>VmDetails</code> | <code>"reapply"</code> | — | — | B1 | P |
| [162](../src/gui/ui/details.slint#L162), [163](../src/gui/ui/details.slint#L163) | <code>VmDetails</code> | <code>"Unsupported"</code> | — | <code>dashboard.VmDetails.variant_020</code> | B1 | M |
| [162](../src/gui/ui/details.slint#L162) | <code>VmDetails</code> | <code>"This VM is not eligible"</code> | — | <code>dashboard.VmDetails.variant_021</code> | B1 | C |
| [162](../src/gui/ui/details.slint#L162) | <code>VmDetails</code> | <code>"Configuration is unavailable"</code> | — | <code>dashboard.VmDetails.variant_022</code> | B1 | C |
| [163](../src/gui/ui/details.slint#L163) | <code>VmDetails</code> | <code>"Non-Generation 2 guests are outside the GPU-P workflow."</code> | — | <code>dashboard.VmDetails.variant_023</code> | B1 | C |
| [164](../src/gui/ui/details.slint#L164) | <code>VmDetails</code> | <code>"Inspect diagnostics"</code> | — | <code>dashboard.VmDetails.variant_024</code> | B1 | C |
| [164](../src/gui/ui/details.slint#L164) | <code>VmDetails</code> | <code>"diagnostics"</code> | — | — | B1 | P |
| [173](../src/gui/ui/details.slint#L173) | <code>VmDetails</code> | <code>"Exact VRAM request for compatibility with saved settings. Minimum ≤ Optimal ≤ Maximum, in native bytes. Grey defaults are suggestions. Editing an empty group fills its other defaults. Clear all three fields for no explicit write. Use the GPU support toggle or Remove GPU support to disable support."</code> | — | <code>dashboard.VmDetails.variant_025</code> | B1 | C |
| [174](../src/gui/ui/details.slint#L174) | <code>VmDetails</code> | <code>"VRAM"</code> | — | <code>dashboard.VmDetails.variant_026</code> | B1 | C |
| [185](../src/gui/ui/details.slint#L185) | <code>VmDetails</code> | <code>"Apply"</code> | — | <code>dashboard.VmDetails.variant_027</code> | B1 | C |
| [186](../src/gui/ui/details.slint#L186) | <code>VmDetails</code> | <code>"Discard draft"</code> | — | <code>dashboard.VmDetails.variant_028</code> | B1 | C |
| [186](../src/gui/ui/details.slint#L186) | <code>VmDetails</code> | <code>"discard"</code> | — | — | B1 | P |
| [187](../src/gui/ui/details.slint#L187) | <code>VmDetails</code> | <code>"More actions…"</code> | — | <code>dashboard.VmDetails.variant_029</code> | B1 | C |
| [187](../src/gui/ui/details.slint#L187) | <code>VmDetails</code> | <code>"more"</code> | — | — | B1 | P |
## src/gui/ui/dialogs.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/dialogs.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/dialogs.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [3](../src/gui/ui/dialogs.slint#L3) | <code>defaults</code> | <code>"technical-output.slint"</code> | — | — | B1 | X |
| [14](../src/gui/ui/dialogs.slint#L14), [15](../src/gui/ui/dialogs.slint#L15) | <code>ActionChoice</code> | <code>""</code> | — | — | B1 | F |
| [15](../src/gui/ui/dialogs.slint#L15) | <code>ActionChoice</code> | <code>"VM information is updating. Wait for the update to finish."</code> | — | <code>dialogs.ActionChoice.variant_001</code> | B1 | C |
| [15](../src/gui/ui/dialogs.slint#L15) | <code>ActionChoice</code> | <code>"Use Refresh to check application setup and worker availability."</code> | — | <code>dialogs.ActionChoice.variant_002</code> | B1 | C |
| [15](../src/gui/ui/dialogs.slint#L15) | <code>ActionChoice</code> | <code>"Opens a fresh review. Changes require separate confirmation."</code> | — | <code>dialogs.ActionChoice.variant_003</code> | B1 | C |
| [59](../src/gui/ui/dialogs.slint#L59), [148](../src/gui/ui/dialogs.slint#L148) | <code>OperationDialog</code> | <code>"close"</code> | — | — | B1 | P |
| [72](../src/gui/ui/dialogs.slint#L72) | <code>OperationDialog</code> | <code>"Hyper GPU Support"</code> | — | <code>dialogs.OperationDialog.variant_001</code> | B1 | C |
| [84](../src/gui/ui/dialogs.slint#L84), [135](../src/gui/ui/dialogs.slint#L135), [141](../src/gui/ui/dialogs.slint#L141), [146](../src/gui/ui/dialogs.slint#L146), [151](../src/gui/ui/dialogs.slint#L151), [156](../src/gui/ui/dialogs.slint#L156), [175](../src/gui/ui/dialogs.slint#L175), [199](../src/gui/ui/dialogs.slint#L199) | <code>OperationDialog</code> | <code>"review"</code> | — | — | B1 | P |
| [84](../src/gui/ui/dialogs.slint#L84), [135](../src/gui/ui/dialogs.slint#L135), [146](../src/gui/ui/dialogs.slint#L146), [157](../src/gui/ui/dialogs.slint#L157) | <code>OperationDialog</code> | <code>"verify-review"</code> | — | — | B1 | P |
| [84](../src/gui/ui/dialogs.slint#L84), [135](../src/gui/ui/dialogs.slint#L135), [159](../src/gui/ui/dialogs.slint#L159), [160](../src/gui/ui/dialogs.slint#L160), [162](../src/gui/ui/dialogs.slint#L162), [164](../src/gui/ui/dialogs.slint#L164), [184](../src/gui/ui/dialogs.slint#L184), [198](../src/gui/ui/dialogs.slint#L198) | <code>OperationDialog</code> | <code>"recovery"</code> | — | — | B1 | P |
| [88](../src/gui/ui/dialogs.slint#L88), [195](../src/gui/ui/dialogs.slint#L195) | <code>OperationDialog</code> | <code>"more"</code> | — | — | B1 | P |
| [90](../src/gui/ui/dialogs.slint#L90), [157](../src/gui/ui/dialogs.slint#L157) | <code>OperationDialog</code> | <code>"Verify GPU setup"</code> | — | <code>dialogs.OperationDialog.variant_002</code> | B1 | M |
| [90](../src/gui/ui/dialogs.slint#L90) | <code>OperationDialog</code> | <code>"verify"</code> | — | — | B1 | P |
| [91](../src/gui/ui/dialogs.slint#L91) | <code>OperationDialog</code> | <code>"Repair / Reinstall"</code> | — | <code>dialogs.OperationDialog.variant_003</code> | B1 | C |
| [91](../src/gui/ui/dialogs.slint#L91), [180](../src/gui/ui/dialogs.slint#L180) | <code>OperationDialog</code> | <code>"repair"</code> | — | — | B1 | P |
| [92](../src/gui/ui/dialogs.slint#L92) | <code>OperationDialog</code> | <code>"Remove GPU support"</code> | — | <code>dialogs.OperationDialog.variant_004</code> | B1 | C |
| [92](../src/gui/ui/dialogs.slint#L92) | <code>OperationDialog</code> | <code>"remove-gpu"</code> | — | — | B1 | P |
| [93](../src/gui/ui/dialogs.slint#L93), [155](../src/gui/ui/dialogs.slint#L155) | <code>OperationDialog</code> | <code>"Delete profile"</code> | — | <code>dialogs.OperationDialog.variant_005</code> | B1 | C |
| [93](../src/gui/ui/dialogs.slint#L93), [193](../src/gui/ui/dialogs.slint#L193) | <code>OperationDialog</code> | <code>"forget"</code> | — | — | B1 | P |
| [97](../src/gui/ui/dialogs.slint#L97), [131](../src/gui/ui/dialogs.slint#L131), [160](../src/gui/ui/dialogs.slint#L160), [165](../src/gui/ui/dialogs.slint#L165), [179](../src/gui/ui/dialogs.slint#L179), [187](../src/gui/ui/dialogs.slint#L187), [192](../src/gui/ui/dialogs.slint#L192), [203](../src/gui/ui/dialogs.slint#L203) | <code>OperationDialog</code> | <code>""</code> | — | — | B1 | F |
| [136](../src/gui/ui/dialogs.slint#L136) | <code>OperationDialog</code> | <code>"I agree to the power changes described above."</code> | — | <code>dialogs.OperationDialog.variant_006</code> | B1 | C |
| [141](../src/gui/ui/dialogs.slint#L141) | <code>OperationDialog</code> | <code>"Apply is unavailable. Close this review and use Refresh to check setup or recovery."</code> | — | <code>dialogs.OperationDialog.variant_007</code> | B1 | C |
| [146](../src/gui/ui/dialogs.slint#L146) | <code>OperationDialog</code> | <code>"Operation in progress"</code> | — | <code>dialogs.OperationDialog.variant_008</code> | B1 | C |
| [146](../src/gui/ui/dialogs.slint#L146), [150](../src/gui/ui/dialogs.slint#L150), [150](../src/gui/ui/dialogs.slint#L150), [151](../src/gui/ui/dialogs.slint#L151) | <code>OperationDialog</code> | <code>"switch"</code> | — | — | B1 | P |
| [146](../src/gui/ui/dialogs.slint#L146) | <code>OperationDialog</code> | <code>"Stay on VM"</code> | — | <code>dialogs.OperationDialog.variant_009</code> | B1 | C |
| [146](../src/gui/ui/dialogs.slint#L146), [152](../src/gui/ui/dialogs.slint#L152), [152](../src/gui/ui/dialogs.slint#L152) | <code>OperationDialog</code> | <code>"exit"</code> | — | — | B1 | P |
| [146](../src/gui/ui/dialogs.slint#L146), [153](../src/gui/ui/dialogs.slint#L153) | <code>OperationDialog</code> | <code>"enroll-review"</code> | — | — | B1 | P |
| [146](../src/gui/ui/dialogs.slint#L146) | <code>OperationDialog</code> | <code>"Keep editing"</code> | — | <code>dialogs.OperationDialog.variant_010</code> | B1 | C |
| [146](../src/gui/ui/dialogs.slint#L146), [155](../src/gui/ui/dialogs.slint#L155) | <code>OperationDialog</code> | <code>"forget-review"</code> | — | — | B1 | P |
| [146](../src/gui/ui/dialogs.slint#L146) | <code>OperationDialog</code> | <code>"Cancel"</code> | — | <code>dialogs.OperationDialog.variant_011</code> | B1 | C |
| [146](../src/gui/ui/dialogs.slint#L146) | <code>OperationDialog</code> | <code>"Close"</code> | — | <code>dialogs.OperationDialog.variant_012</code> | B1 | C |
| [150](../src/gui/ui/dialogs.slint#L150) | <code>OperationDialog</code> | <code>"Discard &amp; Switch"</code> | — | <code>dialogs.OperationDialog.variant_013</code> | B1 | C |
| [151](../src/gui/ui/dialogs.slint#L151) | <code>OperationDialog</code> | <code>"Review Changes"</code> | — | <code>dialogs.OperationDialog.variant_014</code> | B1 | C |
| [152](../src/gui/ui/dialogs.slint#L152) | <code>OperationDialog</code> | <code>"Discard &amp; Close"</code> | — | <code>dialogs.OperationDialog.variant_015</code> | B1 | C |
| [153](../src/gui/ui/dialogs.slint#L153) | <code>OperationDialog</code> | <code>"Allow GPU management"</code> | — | <code>dialogs.OperationDialog.variant_016</code> | B1 | C |
| [153](../src/gui/ui/dialogs.slint#L153) | <code>OperationDialog</code> | <code>"confirm-enroll"</code> | — | — | B1 | P |
| [154](../src/gui/ui/dialogs.slint#L154) | <code>OperationDialog</code> | <code>"enrollment-recovery"</code> | — | — | B1 | P |
| [154](../src/gui/ui/dialogs.slint#L154) | <code>OperationDialog</code> | <code>"Reconcile enrollment authority"</code> | — | <code>dialogs.OperationDialog.variant_017</code> | B1 | C |
| [154](../src/gui/ui/dialogs.slint#L154) | <code>OperationDialog</code> | <code>"reconcile-enrollment"</code> | — | — | B1 | P |
| [155](../src/gui/ui/dialogs.slint#L155) | <code>OperationDialog</code> | <code>"confirm-forget"</code> | — | — | B1 | P |
| [156](../src/gui/ui/dialogs.slint#L156) | <code>OperationDialog</code> | <code>"Apply"</code> | — | <code>dialogs.OperationDialog.variant_018</code> | B1 | C |
| [156](../src/gui/ui/dialogs.slint#L156) | <code>OperationDialog</code> | <code>"Apply unavailable"</code> | — | <code>dialogs.OperationDialog.variant_019</code> | B1 | C |
| [156](../src/gui/ui/dialogs.slint#L156) | <code>OperationDialog</code> | <code>"confirm"</code> | — | — | B1 | P |
| [157](../src/gui/ui/dialogs.slint#L157) | <code>OperationDialog</code> | <code>"confirm-verify"</code> | — | — | B1 | P |
| [158](../src/gui/ui/dialogs.slint#L158) | <code>OperationDialog</code> | <code>"save"</code> | — | — | B1 | P |
| [158](../src/gui/ui/dialogs.slint#L158) | <code>OperationDialog</code> | <code>"Retry configuration save only"</code> | — | <code>dialogs.OperationDialog.variant_020</code> | B1 | M |
| [158](../src/gui/ui/dialogs.slint#L158) | <code>OperationDialog</code> | <code>"save-only"</code> | — | — | B1 | P |
| [159](../src/gui/ui/dialogs.slint#L159) | <code>OperationDialog</code> | <code>"Check recorded outcome"</code> | — | <code>dialogs.OperationDialog.variant_021</code> | B1 | M |
| [159](../src/gui/ui/dialogs.slint#L159) | <code>OperationDialog</code> | <code>"reconcile"</code> | — | — | B1 | P |
| [160](../src/gui/ui/dialogs.slint#L160), [164](../src/gui/ui/dialogs.slint#L164), [168](../src/gui/ui/dialogs.slint#L168) | <code>OperationDialog</code> | <code>"recovery-blocked"</code> | — | — | B1 | P |
| [160](../src/gui/ui/dialogs.slint#L160) | <code>OperationDialog</code> | <code>"Remove partial GPU setup…"</code> | — | <code>dialogs.OperationDialog.variant_022</code> | B1 | C |
| [160](../src/gui/ui/dialogs.slint#L160) | <code>OperationDialog</code> | <code>"review-reset"</code> | — | — | B1 | P |
| [161](../src/gui/ui/dialogs.slint#L161) | <code>OperationDialog</code> | <code>"reset-review"</code> | — | — | B1 | P |
| [161](../src/gui/ui/dialogs.slint#L161) | <code>OperationDialog</code> | <code>"Remove partial GPU setup"</code> | — | <code>dialogs.OperationDialog.variant_023</code> | B1 | C |
| [161](../src/gui/ui/dialogs.slint#L161) | <code>OperationDialog</code> | <code>"confirm-reset"</code> | — | — | B1 | P |
| [162](../src/gui/ui/dialogs.slint#L162) | <code>OperationDialog</code> | <code>"recovery-needed"</code> | — | — | B1 | P |
| [162](../src/gui/ui/dialogs.slint#L162), [183](../src/gui/ui/dialogs.slint#L183), [198](../src/gui/ui/dialogs.slint#L198) | <code>OperationDialog</code> | <code>"Review recovery"</code> | — | <code>dialogs.OperationDialog.variant_024</code> | B1 | C |
| [165](../src/gui/ui/dialogs.slint#L165) | <code>OperationDialog</code> | <code>"Checking whether partial removal is safe..."</code> | — | <code>dialogs.OperationDialog.variant_025</code> | B1 | C |
| [165](../src/gui/ui/dialogs.slint#L165) | <code>OperationDialog</code> | <code>"Partial removal can be reviewed. Nothing changes until you confirm that review."</code> | — | <code>dialogs.OperationDialog.variant_026</code> | B1 | C |
| [168](../src/gui/ui/dialogs.slint#L168) | <code>OperationDialog</code> | <code>"status-error"</code> | — | — | B1 | P |
| [169](../src/gui/ui/dialogs.slint#L169) | <code>OperationDialog</code> | <code>"Retry status update"</code> | — | <code>dialogs.OperationDialog.variant_027</code> | B1 | C |
| [170](../src/gui/ui/dialogs.slint#L170), [201](../src/gui/ui/dialogs.slint#L201) | <code>OperationDialog</code> | <code>"refresh"</code> | — | — | B1 | P |
| [172](../src/gui/ui/dialogs.slint#L172), [173](../src/gui/ui/dialogs.slint#L173) | <code>OperationDialog</code> | <code>"permission-saved"</code> | — | — | B1 | P |
| [172](../src/gui/ui/dialogs.slint#L172), [174](../src/gui/ui/dialogs.slint#L174), [175](../src/gui/ui/dialogs.slint#L175) | <code>OperationDialog</code> | <code>"profile-deleted"</code> | — | — | B1 | P |
| [173](../src/gui/ui/dialogs.slint#L173) | <code>OperationDialog</code> | <code>"Continue to setup review"</code> | — | <code>dialogs.OperationDialog.variant_028</code> | B1 | C |
| [173](../src/gui/ui/dialogs.slint#L173) | <code>OperationDialog</code> | <code>"Review new GPU setup"</code> | — | <code>dialogs.OperationDialog.variant_029</code> | B1 | C |
| [175](../src/gui/ui/dialogs.slint#L175) | <code>OperationDialog</code> | <code>"reapply"</code> | — | — | B1 | P |
| [177](../src/gui/ui/dialogs.slint#L177), [186](../src/gui/ui/dialogs.slint#L186), [190](../src/gui/ui/dialogs.slint#L190) | <code>OperationDialog</code> | <code>"reset-complete"</code> | — | — | B1 | P |
| [177](../src/gui/ui/dialogs.slint#L177) | <code>OperationDialog</code> | <code>"recovery-ended"</code> | — | — | B1 | P |
| [178](../src/gui/ui/dialogs.slint#L178) | <code>OperationDialog</code> | <code>"Review Repair / Reinstall"</code> | — | <code>dialogs.OperationDialog.variant_030</code> | B1 | C |
| [182](../src/gui/ui/dialogs.slint#L182) | <code>OperationDialog</code> | <code>"error"</code> | — | — | B1 | P |
| [187](../src/gui/ui/dialogs.slint#L187) | <code>OperationDialog</code> | <code>"Updating VM status..."</code> | — | <code>dialogs.OperationDialog.variant_031</code> | B1 | C |
| [187](../src/gui/ui/dialogs.slint#L187) | <code>OperationDialog</code> | <code>"Fresh enrollment requires a separate profile deletion review. Repair / Reinstall can reuse the existing permission."</code> | — | <code>dialogs.OperationDialog.variant_032</code> | B1 | C |
| [191](../src/gui/ui/dialogs.slint#L191) | <code>OperationDialog</code> | <code>"Discard draft and review Delete profile"</code> | — | <code>dialogs.OperationDialog.variant_033</code> | B1 | C |
| [191](../src/gui/ui/dialogs.slint#L191) | <code>OperationDialog</code> | <code>"Review Delete profile"</code> | — | <code>dialogs.OperationDialog.variant_034</code> | B1 | C |
| [193](../src/gui/ui/dialogs.slint#L193) | <code>OperationDialog</code> | <code>"discard-review-forget"</code> | — | — | B1 | P |
| [199](../src/gui/ui/dialogs.slint#L199) | <code>OperationDialog</code> | <code>"Review changes"</code> | — | <code>dialogs.OperationDialog.variant_035</code> | B1 | C |
| [200](../src/gui/ui/dialogs.slint#L200) | <code>OperationDialog</code> | <code>"Discard draft"</code> | — | <code>dialogs.OperationDialog.variant_036</code> | B1 | C |
| [200](../src/gui/ui/dialogs.slint#L200) | <code>OperationDialog</code> | <code>"discard"</code> | — | — | B1 | P |
| [201](../src/gui/ui/dialogs.slint#L201) | <code>OperationDialog</code> | <code>"Refresh"</code> | — | <code>dialogs.OperationDialog.variant_037</code> | B1 | C |
## src/gui/ui/pages.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/pages.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/pages.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [17](../src/gui/ui/pages.slint#L17) | <code>Pages</code> | <code>"Settings"</code> | — | <code>dashboard.Pages.variant_001</code> | B1 | C |
| [17](../src/gui/ui/pages.slint#L17) | <code>Pages</code> | <code>"System information"</code> | — | <code>dashboard.Pages.variant_002</code> | B1 | C |
| [17](../src/gui/ui/pages.slint#L17) | <code>Pages</code> | <code>"About Hyper GPU Support"</code> | — | <code>dashboard.Pages.variant_003</code> | B1 | C |
| [18](../src/gui/ui/pages.slint#L18) | <code>Pages</code> | <code>"Make the workspace yours. Preferences reset on launch."</code> | — | <code>dashboard.Pages.variant_004</code> | B1 | C |
| [18](../src/gui/ui/pages.slint#L18) | <code>Pages</code> | <code>"Virtual machines and GPUs discovered on this computer."</code> | — | <code>dashboard.Pages.variant_005</code> | B1 | C |
| [18](../src/gui/ui/pages.slint#L18) | <code>Pages</code> | <code>"Manage GPU support for existing Hyper-V virtual machines."</code> | — | <code>dashboard.Pages.variant_006</code> | B1 | C |
| [21](../src/gui/ui/pages.slint#L21) | <code>Pages</code> | <code>"This computer"</code> | — | <code>dashboard.Pages.variant_007</code> | B1 | C |
| [28](../src/gui/ui/pages.slint#L28) | <code>Pages</code> | <code>"Appearance"</code> | — | <code>dashboard.Pages.variant_008</code> | B1 | C |
| [29](../src/gui/ui/pages.slint#L29) | <code>Pages</code> | <code>"Theme"</code> | — | <code>dashboard.Pages.variant_009</code> | B1 | C |
| [31](../src/gui/ui/pages.slint#L31) | <code>Pages</code> | <code>"Follow Windows"</code> | — | <code>dashboard.Pages.variant_010</code> | B1 | C |
| [32](../src/gui/ui/pages.slint#L32) | <code>Pages</code> | <code>"Light"</code> | — | <code>dashboard.Pages.variant_011</code> | B1 | C |
| [33](../src/gui/ui/pages.slint#L33) | <code>Pages</code> | <code>"Dark"</code> | — | <code>dashboard.Pages.variant_012</code> | B1 | C |
| [35](../src/gui/ui/pages.slint#L35) | <code>Pages</code> | <code>"Show contextual help"</code> | — | <code>dashboard.Pages.variant_013</code> | B1 | C |
| [36](../src/gui/ui/pages.slint#L36) | <code>Pages</code> | <code>"Fluent controls follow the selected palette immediately."</code> | — | <code>dashboard.Pages.variant_014</code> | B1 | C |
| [40](../src/gui/ui/pages.slint#L40) | <code>Pages</code> | <code>"Credentials &amp; diagnostics"</code> | — | <code>dashboard.Pages.variant_015</code> | B1 | C |
| [41](../src/gui/ui/pages.slint#L41) | <code>Pages</code> | <code>"Guest credentials use the native Windows dialog for the selected enrolled VM. Remember is an explicit opt-in to this user's Credential Manager."</code> | — | <code>dashboard.Pages.variant_016</code> | B1 | C |
| [42](../src/gui/ui/pages.slint#L42) | <code>Pages</code> | <code>"Guest credentials for selected VM"</code> | — | <code>dashboard.Pages.variant_017</code> | B1 | C |
| [42](../src/gui/ui/pages.slint#L42) | <code>Pages</code> | <code>"credentials"</code> | — | — | B1 | P |
| [48](../src/gui/ui/pages.slint#L48) | <code>Pages</code> | <code>"Hyper GPU Support"</code> | — | <code>dashboard.Pages.variant_018</code> | B1 | C |
| [49](../src/gui/ui/pages.slint#L49) | <code>Pages</code> | <code>"Hyper GPU Support · 0.1.0"</code> | — | <code>dashboard.Pages.variant_019</code> | B1 | C |
| [50](../src/gui/ui/pages.slint#L50) | <code>Pages</code> | <code>"Review and approve GPU support changes for your virtual machines. Available operations depend on setup, supported GPU capabilities and recovery status."</code> | — | <code>dashboard.Pages.variant_020</code> | B1 | C |
| [51](../src/gui/ui/pages.slint#L51) | <code>Pages</code> | <code>"Explicit CLI commands retain their existing capabilities. Unconnected GUI operations never produce simulated success."</code> | — | <code>dashboard.Pages.variant_021</code> | B1 | C |
| [52](../src/gui/ui/pages.slint#L52) | <code>Pages</code> | <code>"Designed for existing VMs"</code> | — | <code>dashboard.Pages.variant_022</code> | B1 | C |
| [53](../src/gui/ui/pages.slint#L53) | <code>Pages</code> | <code>"NVIDIA first. Further vendor support and simultaneous GPU sharing require independent qualification."</code> | — | <code>dashboard.Pages.variant_023</code> | B1 | C |
| [54](../src/gui/ui/pages.slint#L54) | <code>Pages</code> | <code>"View application help"</code> | — | <code>dashboard.Pages.variant_024</code> | B1 | C |
| [54](../src/gui/ui/pages.slint#L54) | <code>Pages</code> | <code>"shortcuts"</code> | — | — | B1 | P |
## src/gui/ui/resources.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/resources.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/resources.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [8](../src/gui/ui/resources.slint#L8) | <code>AllocationDisclosure</code> | <code>"Advanced Allocation, expanded"</code> | — | <code>dashboard.AllocationDisclosure.variant_001</code> | B1 | C |
| [8](../src/gui/ui/resources.slint#L8) | <code>AllocationDisclosure</code> | <code>"Advanced Allocation, collapsed"</code> | — | <code>dashboard.AllocationDisclosure.variant_002</code> | B1 | C |
| [23](../src/gui/ui/resources.slint#L23) | <code>AllocationDisclosure</code> | <code>" "</code> | — | — | B1 | F |
| [30](../src/gui/ui/resources.slint#L30) | <code>AllocationDisclosure</code> | <code>"icons/chevron-down.svg"</code> | — | — | B1 | X |
| [30](../src/gui/ui/resources.slint#L30) | <code>AllocationDisclosure</code> | <code>"icons/chevron-right.svg"</code> | — | — | B1 | X |
| [33](../src/gui/ui/resources.slint#L33) | <code>AllocationDisclosure</code> | <code>"Advanced Allocation"</code> | — | <code>dashboard.AllocationDisclosure.variant_003</code> | B1 | C |
| [53](../src/gui/ui/resources.slint#L53) | <code>AllocationField</code> | <code>" allocation"</code> | — | <code>dashboard.AllocationField.variant_001</code> | B1 | C |
| [56](../src/gui/ui/resources.slint#L56) | <code>AllocationField</code> | <code>""</code> | — | — | B1 | F |
| [56](../src/gui/ui/resources.slint#L56) | <code>AllocationField</code> | <code>"Not specified"</code> | — | <code>dashboard.AllocationField.variant_002</code> | B1 | C |
| [56](../src/gui/ui/resources.slint#L56) | <code>AllocationField</code> | <code>" (default)"</code> | — | <code>dashboard.AllocationField.variant_003</code> | B1 | C |
| [64](../src/gui/ui/resources.slint#L64) | <code>ResourceEditor</code> | <code>"Raw provider capability unknown; editing unavailable"</code> | — | <code>dashboard.ResourceEditor.variant_001</code> | B1 | C |
| [78](../src/gui/ui/resources.slint#L78) | <code>ResourceEditor</code> | <code>"Collapse"</code> | — | <code>dashboard.ResourceEditor.variant_002</code> | B1 | C |
| [78](../src/gui/ui/resources.slint#L78) | <code>ResourceEditor</code> | <code>"Expand"</code> | — | <code>dashboard.ResourceEditor.variant_003</code> | B1 | C |
| [85](../src/gui/ui/resources.slint#L85) | <code>ResourceEditor</code> | <code>"Minimum"</code> | — | <code>dashboard.ResourceEditor.variant_004</code> | B1 | C |
| [86](../src/gui/ui/resources.slint#L86) | <code>ResourceEditor</code> | <code>"Optimal"</code> | — | <code>dashboard.ResourceEditor.variant_005</code> | B1 | C |
| [87](../src/gui/ui/resources.slint#L87) | <code>ResourceEditor</code> | <code>"Maximum"</code> | — | <code>dashboard.ResourceEditor.variant_006</code> | B1 | C |
## src/gui/ui/technical-output.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/technical-output.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [2](../src/gui/ui/technical-output.slint#L2) | <code>defaults</code> | <code>"theme.slint"</code> | — | — | B1 | X |
| [24](../src/gui/ui/technical-output.slint#L24) | <code>TechnicalOutput</code> | <code>"Technical details, read only"</code> | — | <code>dashboard.TechnicalOutput.variant_001</code> | B1 | C |
| [32](../src/gui/ui/technical-output.slint#L32) | <code>TechnicalOutput</code> | <code>"Consolas"</code> | — | <code>dashboard.TechnicalOutput.variant_002</code> | B1 | C |
## src/gui/ui/theme.slint

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [1](../src/gui/ui/theme.slint#L1) | <code>defaults</code> | <code>"std-widgets.slint"</code> | — | — | B1 | X |
| [32](../src/gui/ui/theme.slint#L32) | <code>TechnicalDetailsButton</code> | <code>"Hide Technical Details"</code> | — | <code>dashboard.TechnicalDetailsButton.variant_001</code> | B1 | C |
| [32](../src/gui/ui/theme.slint#L32) | <code>TechnicalDetailsButton</code> | <code>"Show Technical Details"</code> | — | <code>dashboard.TechnicalDetailsButton.variant_002</code> | B1 | C |
## src/gui_model.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [38](../src/gui_model.rs#L38) | <code>recorded_graphics_summary</code> | <code>"recovery required; current health unknown"</code> | — | — | B2 | X |
| [41](../src/gui_model.rs#L41) | <code>recorded_graphics_summary</code> | <code>"unknown; no matching check record"</code> | — | — | B2 | X |
| [44](../src/gui_model.rs#L44) | <code>recorded_graphics_summary</code> | <code>"latest check incomplete; current health unknown"</code> | — | — | B2 | X |
| [45](../src/gui_model.rs#L45) | <code>recorded_graphics_summary</code> | <code>"latest check found a driver-file mismatch"</code> | — | — | B2 | X |
| [46](../src/gui_model.rs#L46) | <code>recorded_graphics_summary</code> | <code>"latest check failed; current health unknown"</code> | — | — | B2 | X |
| [47](../src/gui_model.rs#L47) | <code>recorded_graphics_summary</code> | <code>"latest check unavailable; current health unknown"</code> | — | — | B2 | X |
| [48](../src/gui_model.rs#L48) | <code>recorded_graphics_summary</code> | <code>"previously verified; current health unknown"</code> | — | — | B2 | X |
| [49](../src/gui_model.rs#L49) | <code>recorded_graphics_summary</code> | <code>"not verified"</code> | — | — | B2 | X |
| [58](../src/gui_model.rs#L58) | <code>recorded_state</code> | <code>"Unknown; no matching graphics verification record was read"</code> | — | <code>operations.recorded_state.variant_001</code> | B2 | C |
| [62](../src/gui_model.rs#L62) | <code>recorded_state</code> | <code>"Preparation record"</code> | — | <code>operations.recorded_state.variant_002</code> | B2 | M |
| [67](../src/gui_model.rs#L67) | <code>recorded_state</code> | <code>"Preparation record unavailable"</code> | — | <code>operations.recorded_state.variant_003</code> | B2 | C |
| [69](../src/gui_model.rs#L69) | <code>recorded_state</code> | <code>"No preparation record"</code> | — | <code>operations.recorded_state.variant_004</code> | B2 | C |
| [71](../src/gui_model.rs#L71), [104](../src/gui_model.rs#L104) | <code>recorded_state</code> | <code>"No recorded graphics verification; attachment does not prove guest health"</code> | — | <code>operations.recorded_state.variant_005</code> | B2 | C |
| [83](../src/gui_model.rs#L83) | <code>recorded_state</code> | <code>"Preparation record does not match protected enrollment; inspect recovery"</code> | — | <code>operations.recorded_state.variant_006</code> | B2 | C |
| [89](../src/gui_model.rs#L89) | <code>recorded_state</code> | <code>"Recovery pending; preparation outcome requires inspection"</code> | — | <code>operations.recorded_state.variant_007</code> | B2 | C |
| [91](../src/gui_model.rs#L91) | <code>recorded_state</code> | <code>"Previously prepared; current guest files and driver parity are unknown. A checkpoint rollback can undo guest setup. Use Verify to check current graphics health."</code> | — | <code>operations.recorded_state.variant_008</code> | B2 | C |
| [93](../src/gui_model.rs#L93) | <code>recorded_state</code> | <code>"No completed preparation recorded"</code> | — | <code>operations.recorded_state.variant_009</code> | B2 | C |
| [96](../src/gui_model.rs#L96) | <code>recorded_state</code> | <code>"{}{}"</code> | <code>arg1, arg2</code> | — | B2 | F |
| [97](../src/gui_model.rs#L97) | <code>recorded_state</code> | <code>""</code> | — | — | B2 | F |
| [98](../src/gui_model.rs#L98) | <code>recorded_state</code> | <code>"Latest check interrupted or incomplete; current guest state unknown. "</code> | — | <code>operations.recorded_state.variant_012</code> | B2 | C |
| [99](../src/gui_model.rs#L99) | <code>recorded_state</code> | <code>"Latest guest payload and graphics check passed; recovery/lifecycle status applies separately. "</code> | — | <code>operations.recorded_state.variant_013</code> | B2 | C |
| [100](../src/gui_model.rs#L100) | <code>recorded_state</code> | <code>"Latest check found a guest payload mismatch; reviewed preparation is required. "</code> | — | <code>operations.recorded_state.variant_014</code> | B2 | C |
| [101](../src/gui_model.rs#L101) | <code>recorded_state</code> | <code>"Latest guest check failed; current setup is not certified. "</code> | — | <code>operations.recorded_state.variant_015</code> | B2 | C |
| [102](../src/gui_model.rs#L102) | <code>recorded_state</code> | <code>"Latest guest check unavailable; current setup is unknown. "</code> | — | <code>operations.recorded_state.variant_016</code> | B2 | C |
| [105](../src/gui_model.rs#L105) | <code>recorded_state</code> | <code>"Last successful checked rendering: Unix UTC {time} (historical; current guest health unknown)"</code> | <code>time</code> | <code>operations.recorded_state.variant_017</code> | B2 | C |
| [148](../src/gui_model.rs#L148) | <code>saved_desired_text</code> | <code>"No saved desired configuration loaded"</code> | — | <code>operations.saved_desired_text.variant_002</code> | B2 | C |
| [152](../src/gui_model.rs#L152) | <code>saved_desired_text</code> | <code>"No explicit {} request; existing/provider values are not reset"</code> | <code>arg1</code> | <code>operations.saved_desired_text.variant_003</code> | B2 | C |
| [153](../src/gui_model.rs#L153) | <code>saved_desired_text</code> | <code>"{} provider integers: {} / {} / {}"</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.saved_desired_text.variant_004</code> | B2 | C |
| [155](../src/gui_model.rs#L155) | <code>saved_desired_text</code> | <code>"; "</code> | — | — | B2 | F |
| [157](../src/gui_model.rs#L157) | <code>saved_desired_text</code> | <code>"GPU {}; {}; {allocation}"</code> | <code>arg1, arg2, allocation</code> | <code>operations.saved_desired_text.variant_006</code> | B2 | C |
| [159](../src/gui_model.rs#L159) | <code>saved_desired_text</code> | <code>"enabled"</code> | — | <code>operations.saved_desired_text.variant_007</code> | B2 | C |
| [161](../src/gui_model.rs#L161) | <code>saved_desired_text</code> | <code>"disabled"</code> | — | <code>operations.saved_desired_text.variant_008</code> | B2 | C |
| [208](../src/gui_model.rs#L208) | <code>eligibility</code> | <code>"Refresh inventory first."</code> | — | <code>operations.eligibility.variant_001</code> | B2 | M |
| [213](../src/gui_model.rs#L213) | <code>eligibility</code> | <code>"This VM/GPU pair is not enrolled; separate enrollment review is required before Apply."</code> | — | <code>operations.eligibility.variant_002</code> | B2 | M |
| [220](../src/gui_model.rs#L220) | <code>proposal_eligibility</code> | <code>"Refresh VM information before applying changes."</code> | — | <code>operations.proposal_eligibility.variant_002</code> | B2 | M |
| [226](../src/gui_model.rs#L226) | <code>draft_eligibility</code> | <code>"Refresh inventory first."</code> | — | <code>operations.draft_eligibility.variant_001</code> | B2 | M |
| [233](../src/gui_model.rs#L233) | <code>draft_eligibility</code> | <code>"Selected VM observations are incomplete; correct access and refresh before applying."</code> | — | <code>operations.draft_eligibility.variant_002</code> | B2 | M |
| [237](../src/gui_model.rs#L237) | <code>draft_eligibility</code> | <code>"The selected VM is no longer present; refresh or discard."</code> | — | <code>operations.draft_eligibility.variant_003</code> | B2 | M |
| [239](../src/gui_model.rs#L239) | <code>draft_eligibility</code> | <code>"A stable Generation 2 VM is required."</code> | — | <code>operations.draft_eligibility.variant_004</code> | B2 | M |
| [242](../src/gui_model.rs#L242) | <code>draft_eligibility</code> | <code>"Another GPU assignment is present; reconcile it before applying."</code> | — | <code>operations.draft_eligibility.variant_005</code> | B2 | M |
| [249](../src/gui_model.rs#L249) | <code>draft_eligibility</code> | <code>"The enrolled GPU is unavailable."</code> | — | <code>operations.draft_eligibility.variant_006</code> | B2 | M |
| [259](../src/gui_model.rs#L259) | <code>draft_eligibility</code> | <code>"Managed state belongs to a different pair; reconcile enrollment first."</code> | — | <code>operations.draft_eligibility.variant_007</code> | B2 | C |
| [263](../src/gui_model.rs#L263) | <code>draft_eligibility</code> | <code>"Guest preparation is not supported for this GPU vendor."</code> | — | <code>operations.draft_eligibility.variant_008</code> | B2 | M |
| [274](../src/gui_model.rs#L274) | <code>stage</code> | <code>"Refresh after the uncertain operation before editing."</code> | — | — | B2 | X |
| [278](../src/gui_model.rs#L278) | <code>stage</code> | <code>"Save the completed configuration before staging another operation."</code> | — | — | B2 | X |
| [282](../src/gui_model.rs#L282) | <code>stage</code> | <code>"Apply or discard the other VM's pending change first."</code> | — | — | B2 | X |
| [290](../src/gui_model.rs#L290) | <code>stage</code> | <code>"Add this VM to configuration and enroll it first."</code> | — | — | B2 | X |
| [293](../src/gui_model.rs#L293) | <code>stage</code> | <code>"The VM is unavailable."</code> | — | — | B2 | X |
| [316](../src/gui_model.rs#L316) | <code>reapply</code> | <code>"Unconfigured VM"</code> | — | — | B2 | X |
| [319](../src/gui_model.rs#L319) | <code>reapply</code> | <code>"just staged"</code> | — | — | B2 | X |
| [327](../src/gui_model.rs#L327) | <code>verification</code> | <code>"Refresh first"</code> | — | <code>operations.verification.variant_001</code> | B2 | M |
| [330](../src/gui_model.rs#L330) | <code>verification</code> | <code>"VM unavailable"</code> | — | <code>operations.verification.variant_002</code> | B2 | M |
| [340](../src/gui_model.rs#L340) | <code>apply_target</code> | <code>"Refresh/read back the uncertain operation before retrying."</code> | — | — | B2 | X |
| [344](../src/gui_model.rs#L344) | <code>apply_target</code> | <code>"The operation succeeded; save its configuration instead of replaying apply."</code> | — | — | B2 | X |
| [348](../src/gui_model.rs#L348) | <code>apply_target</code> | <code>"Stage a change first."</code> | — | — | B2 | X |
| [375](../src/gui_model.rs#L375) | <code>published</code> | <code>"publication does not match verified-unsaved intent"</code> | — | <code>operations.published.variant_001</code> | B2 | M |
| [402](../src/gui_model.rs#L402) | <code>support_text</code> | <code>"Disabled"</code> | — | — | B2 | X |
| [404](../src/gui_model.rs#L404) | <code>support_text</code> | <code>"Enabled"</code> | — | — | B2 | X |
| [406](../src/gui_model.rs#L406) | <code>support_text</code> | <code>"Other assignment"</code> | — | — | B2 | X |
| [410](../src/gui_model.rs#L410) | <code>support_text</code> | <code>"{effective} — Pending {}"</code> | <code>effective, arg2</code> | — | B2 | X |
| [411](../src/gui_model.rs#L411) | <code>support_text</code> | <code>"enable"</code> | — | — | B2 | X |
| [411](../src/gui_model.rs#L411) | <code>support_text</code> | <code>"disable"</code> | — | — | B2 | X |
| [420](../src/gui_model.rs#L420) | <code>allocation_text</code> | <code>"Not assigned"</code> | — | — | B2 | X |
| [422](../src/gui_model.rs#L422) | <code>allocation_text</code> | <code>"Provider values"</code> | — | — | B2 | X |
| [424](../src/gui_model.rs#L424) | <code>allocation_text</code> | <code>"Provider default"</code> | — | — | B2 | X |
| [436](../src/gui_model.rs#L436) | <code>save_configuration</code> | <code>"Read configuration before saving: {e}"</code> | <code>e</code> | — | B2 | X |
| [439](../src/gui_model.rs#L439) | <code>save_configuration</code> | <code>"Configuration changed outside the GUI. Resolve the file before saving; do not replay the successful VM operation."</code> | — | — | B2 | X |
| [444](../src/gui_model.rs#L444) | <code>save_configuration</code> | <code>".hyper-gpu-save-{}-{}.tmp"</code> | <code>arg1, arg2</code> | — | B2 | X |
| [462](../src/gui_model.rs#L462) | <code>save_configuration</code> | <code>"Configuration changed during save; resolve the file before retrying save."</code> | — | — | B2 | X |
| [473](../src/gui_model.rs#L473) | <code>save_configuration</code> | <code>"{}; temporary save cleanup failed: {cleanup}"</code> | <code>arg1, cleanup</code> | — | B2 | X |
| [474](../src/gui_model.rs#L474) | <code>save_configuration</code> | <code>"Save completed"</code> | — | — | B2 | X |
## src/reporting.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [5](../src/reporting.rs#L5) | <code>defaults</code> | <code>"Copying files: "</code> | — | <code>diagnostics.defaults.variant_001</code> | B4 | M |
| [10](../src/reporting.rs#L10) | <code>transfer_progress</code> | <code>"{TRANSFER_PROGRESS_PREFIX}progress unavailable; verification pending"</code> | <code>TRANSFER_PROGRESS_PREFIX</code> | <code>diagnostics.transfer_progress.variant_001</code> | B4 | C |
| [13](../src/reporting.rs#L13) | <code>transfer_progress</code> | <code>"0 bytes"</code> | — | <code>diagnostics.transfer_progress.variant_002</code> | B4 | C |
| [19](../src/reporting.rs#L19) | <code>transfer_progress</code> | <code>"{} of {} ({percent}%)"</code> | <code>arg1, arg2, percent</code> | <code>diagnostics.transfer_progress.variant_003</code> | B4 | C |
| [24](../src/reporting.rs#L24) | <code>transfer_progress</code> | <code>"{TRANSFER_PROGRESS_PREFIX}{sizes}; {files} of {count} files; verification pending"</code> | <code>TRANSFER_PROGRESS_PREFIX, sizes, files, count</code> | <code>diagnostics.transfer_progress.variant_004</code> | B4 | C |
| [29](../src/reporting.rs#L29) | <code>transfer_size</code> | <code>"{bytes} {}"</code> | <code>bytes, arg2</code> | — | B4 | F |
| [29](../src/reporting.rs#L29) | <code>transfer_size</code> | <code>"byte"</code> | — | — | B4 | F |
| [29](../src/reporting.rs#L29), [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"bytes"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"KB"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"MB"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"GB"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"TB"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"PB"</code> | — | — | B4 | F |
| [31](../src/reporting.rs#L31) | <code>transfer_size</code> | <code>"EB"</code> | — | — | B4 | F |
| [41](../src/reporting.rs#L41) | <code>transfer_size</code> | <code>"{}.{:01} {}"</code> | <code>arg1, :01, arg3</code> | — | B4 | F |
| [140](../src/reporting.rs#L140) | <code>into_error</code> | <code>"HGFAIL1:{}"</code> | <code>arg1</code> | — | B4 | P |
| [141](../src/reporting.rs#L141) | <code>into_error</code> | <code>"fixed failure serialization"</code> | — | — | B4 | P |
| [153](../src/reporting.rs#L153) | <code>decode</code> | <code>"HGFAIL1:"</code> | — | — | B4 | P |
| [171](../src/reporting.rs#L171) | <code>text</code> | <code>"HRESULT 0x{code:08X}"</code> | <code>code:08X</code> | <code>diagnostics.text.variant_001</code> | B4 | C |
| [172](../src/reporting.rs#L172) | <code>text</code> | <code>"provider code {code}"</code> | <code>code</code> | <code>diagnostics.text.variant_002</code> | B4 | C |
| [173](../src/reporting.rs#L173) | <code>text</code> | <code>"process exit {code}"</code> | <code>code</code> | <code>diagnostics.text.variant_003</code> | B4 | C |
| [174](../src/reporting.rs#L174) | <code>text</code> | <code>"no terminal numeric code available"</code> | — | <code>diagnostics.text.variant_004</code> | B4 | C |
| [177](../src/reporting.rs#L177) | <code>text</code> | <code>"Stage: {:?}\nComponent: {:?}\nCause: {:?}\nCode: {code}"</code> | <code>:?, :?, :?, code</code> | <code>diagnostics.text.variant_005</code> | B4 | C |
| [189](../src/reporting.rs#L189) | <code>failure_at_stage</code> | <code>"Persist operation recovery"</code> | — | — | B4 | P |
| [190](../src/reporting.rs#L190) | <code>failure_at_stage</code> | <code>"Graceful guest shutdown"</code> | — | — | B4 | P |
| [190](../src/reporting.rs#L190) | <code>failure_at_stage</code> | <code>"Start guest"</code> | — | — | B4 | P |
| [191](../src/reporting.rs#L191) | <code>failure_at_stage</code> | <code>"Attach selected GPU"</code> | — | — | B4 | P |
| [191](../src/reporting.rs#L191) | <code>failure_at_stage</code> | <code>"Detach selected GPU"</code> | — | — | B4 | P |
| [192](../src/reporting.rs#L192) | <code>failure_at_stage</code> | <code>"Compatibility settings"</code> | — | — | B4 | P |
| [193](../src/reporting.rs#L193) | <code>failure_at_stage</code> | <code>"Provider allocation"</code> | — | — | B4 | P |
| [194](../src/reporting.rs#L194) | <code>failure_at_stage</code> | <code>"Prepare current signed payload"</code> | — | — | B4 | P |
| [195](../src/reporting.rs#L195) | <code>failure_at_stage</code> | <code>"Check guest health and hardware rendering"</code> | — | — | B4 | P |
| [232](../src/reporting.rs#L232) | <code>observation</code> | <code>"0x{code:08x}"</code> | <code>code:08x</code> | — | B4 | P |
| [235](../src/reporting.rs#L235) | <code>observation</code> | <code>"access denied"</code> | — | — | B4 | P |
| [236](../src/reporting.rs#L236) | <code>observation</code> | <code>"access is denied"</code> | — | — | B4 | P |
| [237](../src/reporting.rs#L237) | <code>observation</code> | <code>"0x80070005"</code> | — | — | B4 | P |
| [238](../src/reporting.rs#L238) | <code>observation</code> | <code>"0x80041003"</code> | — | — | B4 | P |
| [239](../src/reporting.rs#L239) | <code>observation</code> | <code>"permission denied"</code> | — | — | B4 | P |
| [242](../src/reporting.rs#L242) | <code>observation</code> | <code>"not found"</code> | — | — | B4 | P |
| [243](../src/reporting.rs#L243) | <code>observation</code> | <code>"0x80070002"</code> | — | — | B4 | P |
| [244](../src/reporting.rs#L244) | <code>observation</code> | <code>"0x80041002"</code> | — | — | B4 | P |
| [247](../src/reporting.rs#L247) | <code>observation</code> | <code>"unsupported"</code> | — | — | B4 | P |
| [248](../src/reporting.rs#L248) | <code>observation</code> | <code>"not supported"</code> | — | — | B4 | P |
| [249](../src/reporting.rs#L249) | <code>observation</code> | <code>"0x80041010"</code> | — | — | B4 | P |
| [252](../src/reporting.rs#L252) | <code>observation</code> | <code>"unavailable"</code> | — | — | B4 | P |
| [253](../src/reporting.rs#L253) | <code>observation</code> | <code>"timeout"</code> | — | — | B4 | P |
| [254](../src/reporting.rs#L254) | <code>observation</code> | <code>"timed out"</code> | — | — | B4 | P |
| [255](../src/reporting.rs#L255) | <code>observation</code> | <code>"0x800706ba"</code> | — | — | B4 | P |
| [264](../src/reporting.rs#L264) | <code>observation</code> | <code>"Check operator enrollment and provider permissions; refresh after correcting access."</code> | — | <code>diagnostics.observation.variant_001</code> | B4 | C |
| [265](../src/reporting.rs#L265) | <code>observation</code> | <code>"Refresh and check the selected identity; do not infer successful removal."</code> | — | <code>diagnostics.observation.variant_002</code> | B4 | C |
| [266](../src/reporting.rs#L266) | <code>observation</code> | <code>"Use a supported provider/capability; no capability result was established."</code> | — | <code>diagnostics.observation.variant_003</code> | B4 | C |
| [267](../src/reporting.rs#L267) | <code>observation</code> | <code>"Refresh read-only state and inspect recovery before retrying an uncertain operation."</code> | — | <code>diagnostics.observation.variant_004</code> | B4 | C |
| [274](../src/reporting.rs#L274) | <code>text</code> | <code>"{}: {:?}. {}"</code> | <code>arg1, :?, arg3</code> | — | B4 | F |
| [283](../src/reporting.rs#L283) | <code>operator_error</code> | <code>"{} The restoration proof could not be saved or independently confirmed; recovery remains required."</code> | <code>arg1</code> | <code>diagnostics.operator_error.variant_001</code> | B4 | C |
| [290](../src/reporting.rs#L290) | <code>operator_error</code> | <code>"Required guest driver files are missing. Current preparation does not match the host payload."</code> | — | <code>diagnostics.operator_error.variant_002</code> | B4 | C |
| [293](../src/reporting.rs#L293) | <code>operator_error</code> | <code>"Guest driver files differ from the current authenticated host payload."</code> | — | <code>diagnostics.operator_error.variant_003</code> | B4 | C |
| [296](../src/reporting.rs#L296) | <code>operator_error</code> | <code>"The guest preparation receipt is absent or does not match the current payload and guest identity."</code> | — | <code>diagnostics.operator_error.variant_004</code> | B4 | C |
| [299](../src/reporting.rs#L299) | <code>operator_error</code> | <code>"Current guest payload verification was denied; payload health is unknown."</code> | — | <code>diagnostics.operator_error.variant_005</code> | B4 | C |
| [302](../src/reporting.rs#L302) | <code>operator_error</code> | <code>"Current guest payload verification timed out; payload health is unknown."</code> | — | <code>diagnostics.operator_error.variant_006</code> | B4 | C |
| [307](../src/reporting.rs#L307) | <code>operator_error</code> | <code>"Current guest payload facts are unavailable; payload health is unknown."</code> | — | <code>diagnostics.operator_error.variant_007</code> | B4 | C |
| [312](../src/reporting.rs#L312) | <code>operator_error</code> | <code>"{stage}: {description} Refresh and inspect recovery before any reviewed repair; do not repeat Apply automatically."</code> | <code>stage, description</code> | <code>diagnostics.operator_error.variant_008</code> | B4 | C |
| [316](../src/reporting.rs#L316) | <code>operator_error</code> | <code>"{stage}: {:?} reported {:?}. Refresh and inspect recovery before retrying; reconcile uncertain effects rather than repeating Apply."</code> | <code>stage, :?, :?</code> | <code>diagnostics.operator_error.variant_009</code> | B4 | C |
| [324](../src/reporting.rs#L324) | <code>operator_error</code> | <code>"selected gpu is not attached"</code> | — | — | B4 | P |
| [324](../src/reporting.rs#L324) | <code>operator_error</code> | <code>"The selected GPU is not attached on the host. Review GPU support changes or removal before Verify; no guest verification ran."</code> | — | <code>diagnostics.operator_error.variant_010</code> | B4 | C |
| [325](../src/reporting.rs#L325) | <code>operator_error</code> | <code>"write-capable"</code> | — | — | B4 | P |
| [325](../src/reporting.rs#L325) | <code>operator_error</code> | <code>"The protected installation or state has unsafe write permissions. Restore trusted administrator/SYSTEM ownership and restricted ACLs before installing or operating."</code> | — | <code>diagnostics.operator_error.variant_011</code> | B4 | C |
| [326](../src/reporting.rs#L326) | <code>operator_error</code> | <code>"untrusted"</code> | — | — | B4 | P |
| [326](../src/reporting.rs#L326) | <code>operator_error</code> | <code>"Protected ownership or integrity validation failed. Inspect the installed artifacts and state; do not bypass trust checks."</code> | — | <code>diagnostics.operator_error.variant_012</code> | B4 | C |
| [327](../src/reporting.rs#L327) | <code>operator_error</code> | <code>"--config file is required"</code> | — | — | B4 | P |
| [327](../src/reporting.rs#L327) | <code>operator_error</code> | <code>"Supply --config FILE with validated runtime configuration."</code> | — | <code>diagnostics.operator_error.variant_013</code> | B4 | C |
| [328](../src/reporting.rs#L328) | <code>operator_error</code> | <code>"unknown command"</code> | — | — | B4 | P |
| [328](../src/reporting.rs#L328) | <code>operator_error</code> | <code>"Unknown command; use --help for supported commands."</code> | — | <code>diagnostics.operator_error.variant_014</code> | B4 | C |
| [329](../src/reporting.rs#L329) | <code>operator_error</code> | <code>"approved plan is stale"</code> | — | — | B4 | P |
| [329](../src/reporting.rs#L329) | <code>operator_error</code> | <code>"The approved plan is stale. Refresh and review it again before effects."</code> | — | <code>diagnostics.operator_error.variant_015</code> | B4 | C |
| [330](../src/reporting.rs#L330) | <code>operator_error</code> | <code>"approved plan changed"</code> | — | — | B4 | P |
| [330](../src/reporting.rs#L330) | <code>operator_error</code> | <code>"The approved plan changed. Refresh and review it again before effects."</code> | — | <code>diagnostics.operator_error.variant_016</code> | B4 | C |
| [331](../src/reporting.rs#L331) | <code>operator_error</code> | <code>"changed after review"</code> | — | — | B4 | P |
| [331](../src/reporting.rs#L331) | <code>operator_error</code> | <code>"The draft changed after review. Review the current draft again."</code> | — | <code>diagnostics.operator_error.variant_017</code> | B4 | C |
| [332](../src/reporting.rs#L332) | <code>operator_error</code> | <code>"--approve-shutdown"</code> | — | — | B4 | P |
| [332](../src/reporting.rs#L332) | <code>operator_error</code> | <code>"Guest lifecycle consent is required. Review the plan, then supply --approve-shutdown to approve graceful shutdown and restoration."</code> | — | <code>diagnostics.operator_error.variant_018</code> | B4 | C |
| [333](../src/reporting.rs#L333) | <code>operator_error</code> | <code>"shutdown was not separately approved"</code> | — | — | B4 | P |
| [333](../src/reporting.rs#L333) | <code>operator_error</code> | <code>"Guest lifecycle consent is required. Review and approve graceful guest shutdown and restoration before effects."</code> | — | <code>diagnostics.operator_error.variant_019</code> | B4 | C |
| [334](../src/reporting.rs#L334) | <code>operator_error</code> | <code>"shutdown were not approved"</code> | — | — | B4 | P |
| [334](../src/reporting.rs#L334) | <code>operator_error</code> | <code>"Guest lifecycle consent is required. Review and approve temporary guest startup, graceful shutdown and restoration before effects."</code> | — | <code>diagnostics.operator_error.variant_020</code> | B4 | C |
| [335](../src/reporting.rs#L335) | <code>operator_error</code> | <code>"changed externally"</code> | — | — | B4 | P |
| [335](../src/reporting.rs#L335) | <code>operator_error</code> | <code>"Configuration changed externally. Keep the draft; resolve the conflict, discard if appropriate, and refresh."</code> | — | <code>diagnostics.operator_error.variant_021</code> | B4 | C |
| [336](../src/reporting.rs#L336) | <code>operator_error</code> | <code>"save-only"</code> | — | — | B4 | P |
| [336](../src/reporting.rs#L336) | <code>operator_error</code> | <code>"Inspect the recorded recovery phase. Use save-only only for verified effects; reconcile uncertain effects before another operation."</code> | — | <code>diagnostics.operator_error.variant_022</code> | B4 | C |
| [337](../src/reporting.rs#L337) | <code>operator_error</code> | <code>"publication incomplete"</code> | — | — | B4 | P |
| [337](../src/reporting.rs#L337) | <code>operator_error</code> | <code>"Publication is incomplete. Retain the receipt and retry saving only; do not repeat GPU effects."</code> | — | <code>diagnostics.operator_error.variant_023</code> | B4 | C |
| [338](../src/reporting.rs#L338) | <code>operator_error</code> | <code>"worker elevation refused"</code> | — | — | B4 | P |
| [338](../src/reporting.rs#L338) | <code>operator_error</code> | <code>"Windows elevation was refused or unavailable. Check UAC consent and the protected installation; inspect recovery before retrying."</code> | — | <code>diagnostics.operator_error.variant_024</code> | B4 | C |
| [339](../src/reporting.rs#L339) | <code>operator_error</code> | <code>"0x800704c7"</code> | — | — | B4 | P |
| [339](../src/reporting.rs#L339) | <code>operator_error</code> | <code>"Windows elevation was cancelled. Approve the UAC prompt when ready; inspect recovery before retrying."</code> | — | <code>diagnostics.operator_error.variant_025</code> | B4 | C |
| [340](../src/reporting.rs#L340) | <code>operator_error</code> | <code>"restricted worker"</code> | — | — | B4 | P |
| [340](../src/reporting.rs#L340) | <code>operator_error</code> | <code>"The restricted worker is unavailable or the invocation is invalid. Use the supported frontend and update the protected product installation if necessary."</code> | — | <code>diagnostics.operator_error.variant_026</code> | B4 | C |
| [341](../src/reporting.rs#L341) | <code>operator_error</code> | <code>"requires an elevated"</code> | — | — | B4 | P |
| [341](../src/reporting.rs#L341) | <code>operator_error</code> | <code>"Use an elevated administrator console for this installation operation."</code> | — | <code>diagnostics.operator_error.variant_027</code> | B4 | C |
| [342](../src/reporting.rs#L342) | <code>operator_error</code> | <code>"not enrolled"</code> | — | — | B4 | P |
| [342](../src/reporting.rs#L342) | <code>operator_error</code> | <code>"The selected pair is not enrolled. Review configuration and enroll it through administrator install."</code> | — | <code>diagnostics.operator_error.variant_028</code> | B4 | C |
| [343](../src/reporting.rs#L343) | <code>operator_error</code> | <code>"no longer present"</code> | — | — | B4 | P |
| [343](../src/reporting.rs#L343) | <code>operator_error</code> | <code>"The selected VM is no longer present. Refresh and check its identity; keep or discard the draft deliberately."</code> | — | <code>diagnostics.operator_error.variant_029</code> | B4 | C |
| [344](../src/reporting.rs#L344) | <code>operator_error</code> | <code>"inventory is historical"</code> | — | — | B4 | P |
| [344](../src/reporting.rs#L344) | <code>operator_error</code> | <code>"Inventory is historical. Refresh successfully before further actions."</code> | — | <code>diagnostics.operator_error.variant_030</code> | B4 | C |
| [345](../src/reporting.rs#L345) | <code>operator_error</code> | <code>"pending recovery"</code> | — | — | B4 | P |
| [345](../src/reporting.rs#L345) | <code>operator_error</code> | <code>"Recovery is pending. Inspect and reconcile the recorded operation; do not replay Apply."</code> | — | <code>diagnostics.operator_error.variant_031</code> | B4 | C |
| [346](../src/reporting.rs#L346) | <code>operator_error</code> | <code>"recorded outcome is not independently established"</code> | — | — | B4 | P |
| [346](../src/reporting.rs#L346) | <code>operator_error</code> | <code>"The VM does not match the interrupted operation's approved outcome. An administrator must reconcile the partial state before new changes can be applied."</code> | — | <code>diagnostics.operator_error.variant_032</code> | B4 | C |
| [347](../src/reporting.rs#L347) | <code>operator_error</code> | <code>"unfinished"</code> | — | — | B4 | P |
| [347](../src/reporting.rs#L347) | <code>operator_error</code> | <code>"An unfinished operation requires manual reconciliation. Inspect its durable record before another operation."</code> | — | <code>diagnostics.operator_error.variant_033</code> | B4 | C |
| [348](../src/reporting.rs#L348) | <code>operator_error</code> | <code>"another gui operation"</code> | — | — | B4 | P |
| [348](../src/reporting.rs#L348) | <code>operator_error</code> | <code>"Another GUI operation is still running. Wait for its result before continuing."</code> | — | <code>diagnostics.operator_error.variant_034</code> | B4 | C |
| [351](../src/reporting.rs#L351) | <code>operator_error</code> | <code>"{stage}: {guidance}"</code> | <code>stage, guidance</code> | <code>diagnostics.operator_error.variant_035</code> | B4 | C |
| [361](../src/reporting.rs#L361) | <code>technical_error</code> | <code>"{}\n\nRestoration-proof persistence failed; recovery remains required.\n{}"</code> | <code>arg1, arg2</code> | <code>diagnostics.technical_error.variant_001</code> | B4 | C |
| [363](../src/reporting.rs#L363) | <code>technical_error</code> | <code>"Restoration proof"</code> | — | <code>diagnostics.technical_error.variant_002</code> | B4 | C |
| [368](../src/reporting.rs#L368) | <code>technical_error</code> | <code>"Operation: {stage}\n{}\n{}"</code> | <code>stage, arg2, arg3</code> | <code>diagnostics.technical_error.variant_003</code> | B4 | C |
| [375](../src/reporting.rs#L375) | <code>technical_error</code> | <code>"password"</code> | — | — | B4 | P |
| [375](../src/reporting.rs#L375) | <code>technical_error</code> | <code>"token"</code> | — | — | B4 | P |
| [375](../src/reporting.rs#L375) | <code>technical_error</code> | <code>"secret"</code> | — | — | B4 | P |
| [375](../src/reporting.rs#L375) | <code>technical_error</code> | <code>"credential"</code> | — | — | B4 | P |
| [375](../src/reporting.rs#L375) | <code>technical_error</code> | <code>"bearer"</code> | — | — | B4 | P |
| [381](../src/reporting.rs#L381) | <code>technical_error</code> | <code>"0x"</code> | — | — | B4 | P |
| [393](../src/reporting.rs#L393) | <code>technical_error</code> | <code>"recorded outcome is not independently established; partial state requires administrator reconciliation, never Apply replay"</code> | — | <code>diagnostics.technical_error.variant_004</code> | B4 | M |
| [394](../src/reporting.rs#L394) | <code>technical_error</code> | <code>"current payload differs from the recorded operation"</code> | — | <code>diagnostics.technical_error.variant_005</code> | B4 | M |
| [395](../src/reporting.rs#L395) | <code>technical_error</code> | <code>"recorded pair is no longer enrolled"</code> | — | <code>diagnostics.technical_error.variant_006</code> | B4 | M |
| [396](../src/reporting.rs#L396) | <code>technical_error</code> | <code>"state changed during reconciliation"</code> | — | <code>diagnostics.technical_error.variant_007</code> | B4 | M |
| [397](../src/reporting.rs#L397) | <code>technical_error</code> | <code>"operation journal does not prove the reviewed preparation and power intent"</code> | — | <code>diagnostics.technical_error.variant_008</code> | B4 | M |
| [398](../src/reporting.rs#L398) | <code>technical_error</code> | <code>"reconciliation independent readback changed"</code> | — | <code>diagnostics.technical_error.variant_009</code> | B4 | M |
| [401](../src/reporting.rs#L401) | <code>technical_error</code> | <code>"Operation: {stage}\nBackend result: {}\nComponent message: {}\nNative error codes: {}\nOther untrusted adapter text is omitted to protect credentials."</code> | <code>stage, arg2, arg3, arg4</code> | <code>diagnostics.technical_error.variant_010</code> | B4 | C |
| [403](../src/reporting.rs#L403) | <code>technical_error</code> | <code>"not available as a safe structured message"</code> | — | <code>diagnostics.technical_error.variant_011</code> | B4 | M |
| [405](../src/reporting.rs#L405) | <code>technical_error</code> | <code>"not available"</code> | — | <code>diagnostics.technical_error.variant_012</code> | B4 | M |
| [407](../src/reporting.rs#L407) | <code>technical_error</code> | <code>", "</code> | — | — | B4 | F |
| [419](../src/reporting.rs#L419) | <code>restored_failure_parts</code> | <code>"restoration-proof persistence failed: "</code> | — | — | B4 | P |
| [420](../src/reporting.rs#L420) | <code>restored_failure_parts</code> | <code>"; original guest failure: "</code> | — | — | B4 | P |
| [430](../src/reporting.rs#L430) | <code>public_result</code> | <code>"save_error"</code> | — | — | B4 | P |
| [430](../src/reporting.rs#L430) | <code>public_result</code> | <code>"error"</code> | — | — | B4 | P |
| [431](../src/reporting.rs#L431) | <code>public_result</code> | <code>"Operation result"</code> | — | <code>diagnostics.public_result.variant_001</code> | B4 | C |
## src/credentials.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [18](../src/credentials.rs#L18) | <code>defaults</code> | <code>"Guest credential request cancelled. No guest operation was started."</code> | — | — | B5 | P |
| [53](../src/credentials.rs#L53) | <code>key</code> | <code>"invalid credential VM identity"</code> | — | — | B5 | P |
| [55](../src/credentials.rs#L55) | <code>key</code> | <code>"HyperGpuSupport/guest/{}"</code> | <code>arg1</code> | — | B5 | P |
| [76](../src/credentials.rs#L76) | <code>read</code> | <code>"invalid stored credential"</code> | — | — | B5 | P |
| [92](../src/credentials.rs#L92) | <code>read</code> | <code>"invalid credential encoding"</code> | — | — | B5 | P |
| [114](../src/credentials.rs#L114) | <code>store</code> | <code>"invalid guest credential"</code> | — | — | B5 | P |
| [176](../src/credentials.rs#L176) | <code>explain_guest_sign_in</code> | <code>"Sign in to your virtual machine"</code> | — | <code>credentials.explain_guest_sign_in.variant_001</code> | B5 | C |
| [178](../src/credentials.rs#L178) | <code>explain_guest_sign_in</code> | <code>"{}\n\nChoose OK to enter your details. Cancel returns to the app."</code> | <code>arg1</code> | <code>credentials.explain_guest_sign_in.variant_002</code> | B5 | C |
| [193](../src/credentials.rs#L193) | <code>explain_guest_sign_in</code> | <code>"Guest sign-in explanation could not be opened."</code> | — | <code>credentials.explain_guest_sign_in.variant_003</code> | B5 | M |
| [203](../src/credentials.rs#L203) | <code>prompt_status</code> | <code>"Guest credential dialog failed."</code> | — | — | B5 | P |
| [212](../src/credentials.rs#L212) | <code>prompt_native</code> | <code>"Hyper GPU Support - Sign in"</code> | — | <code>credentials.prompt_native.variant_001</code> | B5 | C |
| [214](../src/credentials.rs#L214) | <code>prompt_native</code> | <code>"HyperGpuSupport/guest/{vm}"</code> | <code>vm</code> | — | B5 | P |
| [253](../src/credentials.rs#L253) | <code>prompt_native</code> | <code>"invalid username"</code> | — | <code>credentials.prompt_native.variant_002</code> | B5 | M |
| [254](../src/credentials.rs#L254) | <code>prompt_native</code> | <code>"invalid password"</code> | — | <code>credentials.prompt_native.variant_003</code> | B5 | M |
| [275](../src/credentials.rs#L275) | <code>guest_prompt_message</code> | <code>"Sign in to {name}\n\nEnter the Windows administrator username and password you use inside this virtual machine. The app uses them for the guest steps you approve in review.\n\nRemember is optional. Leave it unchecked to use your details just this once; selecting it saves them securely on this computer for this VM."</code> | <code>name</code> | <code>credentials.guest_prompt_message.variant_001</code> | B5 | C |
## src/model.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [54](../src/model.rs#L54) | <code>name</code> | <code>"VRAM"</code> | — | — | B2 | P |
| [55](../src/model.rs#L55) | <code>name</code> | <code>"Compute"</code> | — | — | B2 | P |
| [56](../src/model.rs#L56) | <code>name</code> | <code>"Encode"</code> | — | — | B2 | P |
| [57](../src/model.rs#L57) | <code>name</code> | <code>"Decode"</code> | — | — | B2 | P |
| [132](../src/model.rs#L132) | <code>validate</code> | <code>"resource request requires minimum &lt;= optimal &lt;= maximum"</code> | — | — | B2 | P |
| [135](../src/model.rs#L135) | <code>validate</code> | <code>"resource request is outside the provider's reported range"</code> | — | — | B2 | P |
| [224](../src/model.rs#L224) | <code>validate</code> | <code>"configuration requires schema 2 or 3 and 1..128 targets"</code> | — | — | B2 | P |
| [230](../src/model.rs#L230) | <code>validate</code> | <code>"Compute/Encode/Decode intent requires explicit schema 3"</code> | — | — | B2 | P |
| [233](../src/model.rs#L233) | <code>validate</code> | <code>"each VM may occur only once"</code> | — | — | B2 | P |
| [280](../src/model.rs#L280) | <code>validate_allocations</code> | <code>"{} capability is unavailable or unknown"</code> | <code>arg1</code> | — | B2 | P |
| [284](../src/model.rs#L284) | <code>validate_allocations</code> | <code>"{}: {error}"</code> | <code>arg1, error</code> | — | B2 | P |
| [300](../src/model.rs#L300) | <code>validate</code> | <code>r"\\?\PCI#"</code> | — | — | B2 | P |
| [301](../src/model.rs#L301) | <code>validate</code> | <code>r"\GPUPARAV"</code> | — | — | B2 | P |
| [308](../src/model.rs#L308) | <code>validate</code> | <code>"invalid VM GUID or GPU partition interface"</code> | — | — | B2 | P |
| [315](../src/model.rs#L315) | <code>validate</code> | <code>"invalid {} allocation ordering"</code> | <code>arg1</code> | — | B2 | P |
| [441](../src/model.rs#L441) | <code>resource_text</code> | <code>"{} raw provider range: {} .. {} (optimal {}); no physical GB or enforcement claim"</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.resource_text.variant_001</code> | B2 | C |
| [456](../src/model.rs#L456) | <code>resource_text</code> | <code>"Capability unknown; new nonempty requests unavailable; clear existing values to omit"</code> | — | <code>operations.resource_text.variant_002</code> | B2 | C |
| [459](../src/model.rs#L459) | <code>resource_text</code> | <code>"Invalid provider range; editing unavailable"</code> | — | <code>operations.resource_text.variant_003</code> | B2 | C |
| [462](../src/model.rs#L462) | <code>resource_text</code> | <code>"{}: {reason}. Clear an existing request to omit this category."</code> | <code>arg1, reason</code> | <code>operations.resource_text.variant_004</code> | B2 | C |
## src/workflow/planning.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [361](../src/workflow/planning.rs#L361) | <code>plan_from_decision</code> | <code>"Repair / Reinstall explicitly runs complete signed guest driver preparation, even when historical host receipts match. Files independently confirmed to match may be retained. Complete preparation and selected-hardware graphics verification are required; successful completion leaves the VM off. This is a one-operation request, not an automatic repair policy."</code> | — | <code>operations.plan_from_decision.variant_001</code> | B2 | C |
| [364](../src/workflow/planning.rs#L364) | <code>plan_from_decision</code> | <code>"Initial GPU setup includes preparation as needed and checked hardware rendering. Successful completion requires confirmed graceful shutdown; leave the VM off for the user to start in Hyper-V."</code> | — | <code>operations.plan_from_decision.variant_002</code> | B2 | C |
| [367](../src/workflow/planning.rs#L367) | <code>plan_from_decision</code> | <code>"GPU setup/update required: completed preparation/graphics evidence is missing or invalid, or differs from the current signed driver. Review the preparation, graphics check and final shutdown before confirming."</code> | — | <code>operations.plan_from_decision.variant_003</code> | B2 | C |
| [369](../src/workflow/planning.rs#L369) | <code>plan_from_decision</code> | <code>"Ordinary GPU change: no guest access, preparation or graphics check. Successful completion leaves the VM off; start it through Hyper-V when ready."</code> | — | <code>operations.plan_from_decision.variant_004</code> | B2 | C |
| [372](../src/workflow/planning.rs#L372) | <code>plan_from_decision</code> | <code>"Reconcile the pending operation before completing this request."</code> | — | <code>operations.plan_from_decision.variant_005</code> | B2 | C |
| [375](../src/workflow/planning.rs#L375) | <code>plan_from_decision</code> | <code>"Gracefully stop the running guest; shutdown refusal stops changes. Downtime is required."</code> | — | <code>operations.plan_from_decision.variant_006</code> | B2 | C |
| [377](../src/workflow/planning.rs#L377) | <code>plan_from_decision</code> | <code>"Keep the guest running; no restart is required."</code> | — | <code>operations.plan_from_decision.variant_007</code> | B2 | C |
| [379](../src/workflow/planning.rs#L379) | <code>plan_from_decision</code> | <code>"Temporarily start the guest for preparation or verification."</code> | — | <code>operations.plan_from_decision.variant_008</code> | B2 | C |
| [382](../src/workflow/planning.rs#L382) | <code>plan_from_decision</code> | <code>"Reinstall the authenticated payload named in this review; historical receipts cannot skip driver transfer."</code> | — | <code>operations.plan_from_decision.variant_009</code> | B2 | C |
| [385](../src/workflow/planning.rs#L385) | <code>plan_from_decision</code> | <code>"Refresh guest preparation: the recorded payload differs from the current signed driver."</code> | — | <code>operations.plan_from_decision.variant_010</code> | B2 | C |
| [386](../src/workflow/planning.rs#L386) | <code>plan_from_decision</code> | <code>"Prepare the current signed driver: no matching completed preparation is recorded."</code> | — | <code>operations.plan_from_decision.variant_011</code> | B2 | C |
| [389](../src/workflow/planning.rs#L389) | <code>plan_from_decision</code> | <code>"Reuse the recorded current preparation; verify guest graphics afresh."</code> | — | <code>operations.plan_from_decision.variant_012</code> | B2 | C |
| [392](../src/workflow/planning.rs#L392) | <code>plan_from_decision</code> | <code>"Detach the selected GPU."</code> | — | <code>operations.plan_from_decision.variant_013</code> | B2 | C |
| [395](../src/workflow/planning.rs#L395) | <code>plan_from_decision</code> | <code>"Attach the selected GPU."</code> | — | <code>operations.plan_from_decision.variant_014</code> | B2 | C |
| [398](../src/workflow/planning.rs#L398) | <code>plan_from_decision</code> | <code>"Compatibility settings: low MMIO {} -&gt; {} MiB; high MMIO {} -&gt; {} MiB; cache types {} -&gt; {}; automatic checkpoints {} -&gt; {}."</code> | <code>arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8</code> | <code>operations.plan_from_decision.variant_015</code> | B2 | C |
| [401](../src/workflow/planning.rs#L401) | <code>plan_from_decision</code> | <code>"Preserve unowned compatibility settings and values changed by another manager; restore only values still matching the product's last write."</code> | — | <code>operations.plan_from_decision.variant_016</code> | B2 | C |
| [404](../src/workflow/planning.rs#L404) | <code>plan_from_decision</code> | <code>"Remove host GPU support for this exact managed pair, even if guest drivers or the adapter are absent. Keep guest driver files, product authorization and historical records. No guest credentials, preparation, boot or graphics check is required. Publish disabled intent only after independent host and final-off readback."</code> | — | <code>operations.plan_from_decision.variant_017</code> | B2 | C |
| [410](../src/workflow/planning.rs#L410) | <code>plan_from_decision</code> | <code>"{} write (Minimum / Optimal / Maximum): {} / {} / {}; raw provider units, no enforcement claim."</code> | <code>arg1, arg2, arg3, arg4</code> | <code>operations.plan_from_decision.variant_018</code> | B2 | C |
| [421](../src/workflow/planning.rs#L421) | <code>plan_from_decision</code> | <code>"Omitted categories request no allocation write. A new or recreated attachment uses provider defaults; an attachment retained in place keeps its existing values."</code> | — | <code>operations.plan_from_decision.variant_019</code> | B2 | C |
| [424](../src/workflow/planning.rs#L424) | <code>plan_from_decision</code> | <code>"Guest administrator credentials are required (stored vault entry or local prompt). Check PnP health and hardware D3D11 rendering."</code> | — | <code>operations.plan_from_decision.variant_020</code> | B2 | C |
| [427](../src/workflow/planning.rs#L427) | <code>plan_from_decision</code> | <code>"Keep prepared driver files; no guest credentials or graphics check required."</code> | — | <code>operations.plan_from_decision.variant_021</code> | B2 | C |
| [430](../src/workflow/planning.rs#L430) | <code>plan_from_decision</code> | <code>"Final guest power: {:?}. Apply rechecks identity and state; a preview does not reserve them."</code> | <code>:?</code> | <code>operations.plan_from_decision.variant_022</code> | B2 | C |
| [459](../src/workflow/planning.rs#L459) | <code>plan_from_decision</code> | <code>"VRAM bytes; Compute/Encode/Decode provider-defined"</code> | — | <code>operations.plan_from_decision.variant_023</code> | B2 | C |
## src/worker.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [370](../src/worker/client.rs) | <code>apply</code> | <code>"separate graceful guest shutdown approval is required"</code> | — | <code>operations.apply.variant_001</code> | B3 | M |
| [770](../src/worker/pairing.rs), [775](../src/worker/pairing.rs) | <code>execute</code> | <code>"Remove inactive pairing and profile records"</code> | — | <code>operations.execute.variant_001</code> | B3 | M |
| [791](../src/worker/pairing.rs) | <code>execute</code> | <code>"Validate and enroll exact VM/GPU pair"</code> | — | <code>operations.execute.variant_002</code> | B3 | M |
| [796](../src/worker/pairing.rs) | <code>execute</code> | <code>"Exact pair authority read back; GPU support has not been applied"</code> | — | <code>operations.execute.variant_003</code> | B3 | M |
| [929](../src/worker/operations.rs) | <code>execute</code> | <code>"Checking reviewed VM and driver files"</code> | — | <code>operations.execute.variant_004</code> | B3 | M |
| [1006](../src/worker/operations.rs), [1015](../src/worker/operations.rs), [1367](../src/worker/operations.rs), [1376](../src/worker/operations.rs) | <code>execute</code> | <code>"Publish checked VM configuration"</code> | — | <code>operations.execute.variant_005</code> | B3 | M |
| [1437](../src/worker/recovery.rs), [1444](../src/worker/recovery.rs), [1458](../src/worker/recovery.rs) | <code>execute</code> | <code>"Save verified configuration only"</code> | — | <code>operations.execute.variant_006</code> | B3 | M |
| [1521](../src/worker/configuration_import.rs), [1539](../src/worker/configuration_import.rs) | <code>publish_import_record</code> | <code>"Import {filename}"</code> | <code>filename</code> | <code>operations.publish_import_record.variant_001</code> | B3 | M |
| [1571](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"File copying complete; waiting for guest checks"</code> | — | <code>operations.guest_progress.variant_011</code> | B3 | M |
| [1572](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"Checking copied driver files"</code> | — | <code>operations.guest_progress.variant_012</code> | B3 | M |
| [1573](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"Installing GPU driver files"</code> | — | <code>operations.guest_progress.variant_013</code> | B3 | M |
| [1574](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"Checking installed driver files"</code> | — | <code>operations.guest_progress.variant_014</code> | B3 | M |
| [1575](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"Checking current guest files and setup record"</code> | — | <code>operations.guest_progress.variant_015</code> | B3 | M |
| [1576](../src/worker/progress.rs) | <code>guest_progress</code> | <code>"Checking device health and hardware rendering"</code> | — | <code>operations.guest_progress.variant_016</code> | B3 | M |
| [1616](../src/worker/progress.rs) | <code>finish_progress</code> | <code>"Progress delivery failed: {delivery}; {primary}"</code> | <code>delivery, primary</code> | <code>operations.finish_progress.variant_001</code> | B3 | M |
| [1620](../src/worker/progress.rs) | <code>finish_progress</code> | <code>"Stage completed but progress delivery failed; outcome uncertain: {delivery}"</code> | <code>delivery</code> | <code>operations.finish_progress.variant_002</code> | B3 | M |
| [1645](../src/worker/progress.rs) | <code>stage</code> | <code>"approved-worker"</code> | — | — | B3 | P |
| [1674](../src/worker/progress.rs) | <code>save</code> | <code>"Persist operation recovery"</code> | — | <code>operations.save.variant_001</code> | B3 | M |
| [1679](../src/worker/progress.rs) | <code>power</code> | <code>"Graceful guest shutdown"</code> | — | <code>operations.power.variant_001</code> | B3 | M |
| [1681](../src/worker/progress.rs) | <code>power</code> | <code>"Start guest"</code> | — | <code>operations.power.variant_002</code> | B3 | M |
| [1689](../src/worker/progress.rs) | <code>assign</code> | <code>"Attach selected GPU"</code> | — | <code>operations.assign.variant_001</code> | B3 | M |
| [1691](../src/worker/progress.rs) | <code>assign</code> | <code>"Detach selected GPU"</code> | — | <code>operations.assign.variant_002</code> | B3 | M |
| [1697](../src/worker/progress.rs) | <code>settings</code> | <code>"Compatibility settings"</code> | — | <code>operations.settings.variant_001</code> | B3 | M |
| [1700](../src/worker/progress.rs) | <code>allocation</code> | <code>"Provider allocation"</code> | — | <code>operations.allocation.variant_001</code> | B3 | M |
| [1707](../src/worker/progress.rs), [1712](../src/worker/progress.rs), [1719](../src/worker/progress.rs), [1728](../src/worker/progress.rs) | <code>prepare</code> | <code>"Prepare current signed payload"</code> | — | <code>operations.prepare.variant_001</code> | B3 | M |
| [1712](../src/worker/progress.rs) | <code>prepare</code> | <code>"approved-worker"</code> | — | — | B3 | P |
| [1737](../src/worker/progress.rs) | <code>verify</code> | <code>"Check guest health and hardware rendering"</code> | — | <code>operations.verify.variant_001</code> | B3 | M |
| [1743](../src/worker/progress.rs) | <code>verify</code> | <code>"approved-worker"</code> | — | — | B3 | P |
## src/windows_hyperv.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [129](../src/windows_hyperv.rs#L129) | <code>resource_capability</code> | <code>"Provider did not report a complete range"</code> | — | <code>operations.resource_capability.variant_001</code> | B2 | C |
| [132](../src/windows_hyperv.rs#L132) | <code>resource_capability</code> | <code>"Provider range could not be read or validated"</code> | — | <code>operations.resource_capability.variant_002</code> | B2 | C |
| [140](../src/windows_hyperv.rs#L140) | <code>discover</code> | <code>"SELECT * FROM Msvm_ComputerSystem WHERE Caption='Virtual Machine'"</code> | — | — | B2 | P |
| [143](../src/windows_hyperv.rs#L143) | <code>discover</code> | <code>"Name"</code> | — | — | B2 | P |
| [148](../src/windows_hyperv.rs#L148) | <code>discover</code> | <code>"VM inspection"</code> | — | <code>operations.discover.variant_008</code> | B2 | C |
| [149](../src/windows_hyperv.rs#L149) | <code>discover</code> | <code>"SELECT * FROM Msvm_PartitionableGpu"</code> | — | — | B2 | P |
| [153](../src/windows_hyperv.rs#L153) | <code>discover</code> | <code>"GPU provider"</code> | — | <code>operations.discover.variant_010</code> | B2 | C |
| [179](../src/windows_hyperv.rs#L179) | <code>discover_gpus</code> | <code>r"ROOT\cimv2"</code> | — | — | B2 | P |
| [185](../src/windows_hyperv.rs#L185) | <code>discover_gpus</code> | <code>"Name"</code> | — | — | B2 | P |
| [190](../src/windows_hyperv.rs#L190) | <code>discover_gpus</code> | <code>"SELECT DeviceName,DriverVersion FROM Win32_PNPSignedDriver WHERE DeviceID='{}'"</code> | <code>arg1</code> | — | B2 | P |
| [193](../src/windows_hyperv.rs#L193), [222](../src/windows_hyperv.rs#L222) | <code>discover_gpus</code> | <code>"DeviceName"</code> | — | — | B2 | P |
| [193](../src/windows_hyperv.rs#L193), [224](../src/windows_hyperv.rs#L224) | <code>discover_gpus</code> | <code>"DriverVersion"</code> | — | — | B2 | P |
| [198](../src/windows_hyperv.rs#L198) | <code>discover_gpus</code> | <code>"partitionable GPU signed-driver identity is ambiguous"</code> | — | <code>operations.discover_gpus.variant_006</code> | B2 | M |
| [202](../src/windows_hyperv.rs#L202) | <code>discover_gpus</code> | <code>"PCI identity missing"</code> | — | <code>operations.discover_gpus.variant_007</code> | B2 | M |
| [203](../src/windows_hyperv.rs#L203) | <code>discover_gpus</code> | <code>"PCI identity truncated"</code> | — | <code>operations.discover_gpus.variant_008</code> | B2 | M |
| [206](../src/windows_hyperv.rs#L206) | <code>discover_gpus</code> | <code>"VEN_"</code> | — | — | B2 | P |
| [207](../src/windows_hyperv.rs#L207) | <code>discover_gpus</code> | <code>"DEV_"</code> | — | — | B2 | P |
| [213](../src/windows_hyperv.rs#L213) | <code>discover_gpus</code> | <code>"SUBSYS_"</code> | — | — | B2 | P |
| [214](../src/windows_hyperv.rs#L214) | <code>discover_gpus</code> | <code>"REV_"</code> | — | — | B2 | P |
| [222](../src/windows_hyperv.rs#L222) | <code>discover_gpus</code> | <code>"missing GPU name"</code> | — | <code>operations.discover_gpus.variant_013</code> | B2 | M |
| [225](../src/windows_hyperv.rs#L225) | <code>discover_gpus</code> | <code>"missing driver version"</code> | — | <code>operations.discover_gpus.variant_014</code> | B2 | M |
| [230](../src/windows_hyperv.rs#L230) | <code>discover_gpus</code> | <code>"TotalVRAM"</code> | — | <code>operations.discover_gpus.variant_015</code> | B2 | C |
## src/configuration_store.rs

| Source lines | Producer / component | Original source template | Parameters | Proposed key | Batch | Class |
|---|---|---|---|---|---|---|
| [93](../src/configuration_store.rs#L93) | <code>revision</code> | <code>"store"</code> | — | — | B2 | P |
| [94](../src/configuration_store.rs#L94) | <code>revision</code> | <code>"{id}.toml"</code> | <code>id</code> | — | B2 | P |
| [97](../src/configuration_store.rs#L97) | <code>revision</code> | <code>"Committed configuration unavailable"</code> | — | <code>operations.revision.variant_003</code> | B2 | C |
| [126](../src/configuration_store.rs#L126), [129](../src/configuration_store.rs#L129), [153](../src/configuration_store.rs#L153), [156](../src/configuration_store.rs#L156) | <code>revision</code> | <code>"Option::is_none"</code> | — | — | B2 | P |

## Snapshot fingerprints

SHA-256 of the exact UTF-8 source bytes read for this audit; line anchors describe this snapshot, not future revisions.

| Source | SHA-256 |
|---|---|
| src/gui/actions.rs | d63ce7ec1e99eea9182d84a524db34aa10146350811e92fe27f01a361be53cae |
| src/gui/diagnostics.rs | b7d8f73bac5f724592cb6d16e98659442d1a6f6b47d7f6e5fc32d4e56c9179f5 |
| src/gui/host_configuration.rs | 1c1826aa86d78e314f44cb80f388cbce25f48cd02c26f392b5d5eb753ad43942 |
| src/gui/live/polling.rs | 775d3800eb31afbc63cafe1e55a3f79ff883b07620769821c484b2efc79502d9 |
| src/gui/live/presentation.rs | 3779ff8f11f70728f4c93c5dee6ece4dfd45482cafbd1aaee409ed402204ef83 |
| src/gui/live/profile_removal.rs | fd2663c98d5d88ee92030a3961b85799f10ace5cc1f330e793f74305517bb2cc |
| src/gui/live/progress.rs | ed92215aa8e0160e5567e183cf93cdd622e135d0cbdb064dc117497507eb3d93 |
| src/gui/live/source.rs | 04c21cc9c31127b83d526a3d6cdd2c04c0f192418d0a6cffc3f2ec563f6486b8 |
| src/gui/live.rs | 3f2581716947235bc6d63df9c46dac6d68a9812b824476484d951990f45052e3 |
| src/gui/live_state.rs | 5ab15049d067687e411f5fc2c8e28d84b6cefbcab4fed33dc30d150594ec642d |
| src/gui/memory.rs | 6bf095c11f7faa21f321599c56c4cc9a1e8ff548dd1d10775d77cdcc39461bd0 |
| src/gui/mod.rs | 30a8fed3ec874bb720b36326b1f68d00e98eec0863a1b7cfc86fe5381d98daab |
| src/gui/result_notice.rs | 278e3dc5164e0baf175b0781fd854f516c1be3be0da757a5a197f2a11c63ceff |
| src/gui/wording.rs | 29c9b2838782d29f8d3706ce325e4f69d66ca32a834b4247f7e77013375ecf59 |
| src/gui/ui/app.slint | 7c52c4c124d16462870fc3e8d4de95aef99ce42890ef4ef17d799cca25d308fb |
| src/gui/ui/cards.slint | 0e9f326076611c2d2e011f6fa9e2f4564005f34fa72eebdd1fe7a22e72a00895 |
| src/gui/ui/details.slint | d72e58fd48b46b80934a61c7dcb39a87d7a742cff61d77ad8e00efe602cd24e2 |
| src/gui/ui/dialogs.slint | 916e55301d0ebc991394c1e83e278411db4b0769a01d1efb66cd872fdf567799 |
| src/gui/ui/pages.slint | 5fa6a3ee4ae3fcb75c56845f01080a8a60b4ed39682b7ae71f176b0c83b26d72 |
| src/gui/ui/resources.slint | 9d0587363e7b13cc6f5cd529f20a4b83840f8b2ed4b94a267eed63633d21cad3 |
| src/gui/ui/technical-output.slint | 2e658e2e035b050ce8213374e64fd7c2d93a5ce0a176460358f32cd60ea54880 |
| src/gui/ui/theme.slint | 9c1624e240b444e93c1a0189a746feaa7f80bd89ab0f8bfa53315944b77c9eb5 |
| src/gui_model.rs | 428cc174e13b2ff5595c02a459a0ccdba365a0153a527ce096f7e890c3b65376 |
| src/reporting.rs | e7212a15b0f8cc88d0d0b8b425a7fd1eabc76f29040ea96def9f5b24deba1c80 |
| src/credentials.rs | ed481147fa9f8cc2d4800f7d4d5664af66db1717e6db73af9a794da13fb1ce07 |
| src/model.rs | 099cfc71a7a4362e7248f0cf61cbf4c0910f8c3284c2313fd12464a36063b0d7 |
| src/workflow/planning.rs | 9a6b0e8b89ddf9bc268689d3ea80232990520ca287b0a084ec41c0d8333396c2 |
| src/worker.rs | 074d3d8e9590c635719ca7b3e72db205eaed6f0eeaf0b95826c1b89470820bfd |
| src/windows_hyperv.rs | d80c56a19297551397fa5275aeb6b966fc3185cfd9757762e7a2c84b5db5ae67 |
| src/configuration_store.rs | 4ad37f388462aaef6d8806e04ddda1fbfac50ecbba659be87b359d6c6b653e20 |
| src/enrollment.rs | 32709cc04d292ec37ecd317e4fce5f324513765537a0e2c78ec1c11149430344 |
