# Contributor laboratory (not the product)

This independent package preserves the previous fixed-slot CLI, runner, driver pins,
golden-parent/disposable reset and extended probes as reference/contributor tooling.
The root package never depends on it or compiles it through a production feature.

Build separately to prevent executable-name collisions with product artifacts:

```powershell
cargo build --manifest-path tools/lab/Cargo.toml --locked --features dev-harness --target-dir local/lab-target
```

The main executable is now `hyper-gpu-lab`; helper names stay compatible with existing
scripts/tasks. Never install root product binaries using laboratory pins. Explicitly
select this separate build directory when refreshing laboratory artifacts. Moving
source does not migrate or replace any installed runner. Contributor configuration
remains at `config/project.toml`; normal product operation uses schema 2 instead.
Retained code is research material, not a template for new product behavior.
