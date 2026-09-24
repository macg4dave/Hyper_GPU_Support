---
agent: agent
description: Refactor code while preserving behavior and component boundaries
---

Perform the requested refactor.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[module design](../../docs/ENGINEERING.md#code-and-module-design),
[testing](../../docs/ENGINEERING.md#testing), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Establish current observable behavior from code, tests, and documentation.
- Establish the relevant test baseline. Keep the patch narrow and avoid unrelated
  formatting churn; split a necessary larger refactor into tracked steps.
- Preserve public APIs, errors, data formats, and host/guest behavior; change
  component boundaries only when required by the task.
- Justify any necessary dependency or boundary change; avoid new global state,
  unsafe lifetime/ownership assumptions, and unrelated modernization.
- Keep core logic separate from privileged Windows/Hyper-V operations and
  hardware side effects without adding cross-platform abstractions.
- Preserve existing behavioral coverage; adapt tests to changed internals without
  weakening assertions. Cover behavior newly isolated by the refactor.
- Update architecture documentation only if ownership or boundaries change.
- Run required quality checks and affected tests; report exact results.
