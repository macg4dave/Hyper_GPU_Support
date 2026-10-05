# Copilot repository instructions

Follow [AGENTS.md](../AGENTS.md) and relevant sections of
[ENGINEERING.md](../docs/ENGINEERING.md). Read the selected
[backlog](../docs/BACKLOG.md) card and affected source, implement, test and update
the result. Use a task prompt only when relevant.

The normal Gen 2 Hyper-V RTX 5060 baseline already passes sustained Code 0,
`nvidia-smi`, D3D11, D3D12 and CUDA computation. Implement our validated recipe;
do not reopen feasibility or reference research. Keep mutable values in shared
configuration. Approved-runner testing and disposable-guest lifecycle are
autonomous; physical-host restart, shutdown, logout or session termination require
explicit permission immediately beforehand.
