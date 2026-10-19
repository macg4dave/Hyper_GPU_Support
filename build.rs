//! Validate every UI catalogue language, generate its Rust/Slint sources, then compile the UI.
use std::{fs, path::Path};

#[path = "build/windows_resources.rs"]
mod windows_resources;

fn write_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build/windows_resources.rs");
    windows_resources::compile(&[
        "hyper-gpu-support",
        "hyper-gpu-runner",
        "hyper-gpu-guest",
        "d3d11-probe",
    ])
    .expect("compile Windows version resources");
    println!("cargo:rerun-if-changed=tools/ui-text/src");
    let loaded = ui_text::load(Path::new("resources/i18n")).unwrap_or_else(|errors| {
        panic!(
            "UI text catalogue validation failed:\n{}",
            errors.join("\n")
        )
    });
    // Directory tracking covers added and removed locale/feature files.
    for input in &loaded.inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
    for warning in &loaded.warnings {
        println!("cargo:warning={warning}");
    }
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let sources = loaded.catalogue.generate("hyper_gpu_support::ui_text");
    write_changed(&output.join("ui_text.rs"), sources.rust.as_bytes())
        .expect("write generated Rust catalogue");
    write_changed(&output.join("ui-text.slint"), sources.slint.as_bytes())
        .expect("write generated Slint catalogue");
    write_changed(
        &output.join("ui_text_apply.rs"),
        sources.slint_apply.as_bytes(),
    )
    .expect("write generated Slint catalogue binding");
    slint_build::compile_with_config(
        "src/gui/ui/app.slint",
        slint_build::CompilerConfiguration::new()
            .with_style("fluent".into())
            .with_include_paths(vec![output]),
    )
    .expect("compile approved application UI");
}
