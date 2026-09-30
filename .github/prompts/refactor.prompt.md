---
agent: agent
description: Refactor code while preserving behavior and component boundaries
---

Perform the requested refactor.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[module design](../../docs/ENGINEERING.md#code-and-module-design),
[testing](../../docs/ENGINEERING.md#testing), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Establish current observable behavior from code and relevant tests. Use a fresh
  pre-edit baseline only when it helps distinguish a regression. Keep the patch
  narrow and avoid unrelated formatting churn; track a larger refactor only when
  it cannot be completed coherently in the current change.
- Preserve public APIs, errors, data formats, and host/guest behavior; change
  component boundaries only when required by the task.
- Remove duplicated mutable environment values in favor of `config/project.toml`;
  do not move protocol/API constants or fixed safety bounds into configuration.
- Justify any necessary dependency or boundary change; avoid new global state,
  unsafe lifetime/ownership assumptions, and unrelated modernization.
- Keep core logic separate from privileged Windows/Hyper-V operations and
  hardware side effects without adding cross-platform abstractions.
- Preserve existing behavioral coverage; adapt tests to changed internals without
  weakening assertions. Cover behavior newly isolated by the refactor.
- Update architecture documentation only if ownership or boundaries change.
- Run required quality checks and affected tests; report exact results.
