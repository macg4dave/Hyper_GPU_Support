//! Validate the active catalogue before generating Rust and the approved Slint UI.
use std::{fs, path::Path};

fn write_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn main() {
    let directory = Path::new("resources/i18n/en-GB");
    // Directory tracking covers unknown additions and required file deletions.
    println!("cargo:rerun-if-changed={}", directory.display());
    println!("cargo:rerun-if-changed=tools/ui-text/src");
    println!("cargo:rerun-if-changed=tools/ui-text/Cargo.toml");
    let contract = ui_text::contract::active();
    let files: std::collections::BTreeSet<_> =
        contract.iter().map(|message| message.feature()).collect();
    for feature in files {
        println!(
            "cargo:rerun-if-changed={}/{}.toml",
            directory.display(),
            feature
        );
    }
    let catalogue = ui_text::validate_directory(directory, &contract).unwrap_or_else(|errors| {
        panic!(
            "UI text catalogue validation failed:\n{}",
            errors.join("\n")
        )
    });
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let generated = catalogue.generate();
    write_changed(&output.join("ui_text.rs"), generated.rust.as_bytes())
        .expect("write generated Rust catalogue");
    write_changed(&output.join("ui-text.slint"), generated.slint.as_bytes())
        .expect("write generated Slint catalogue");
    slint_build::compile_with_config(
        "src/gui/ui/app.slint",
        slint_build::CompilerConfiguration::new()
            .with_style("fluent".into())
            .with_include_paths(vec![output]),
    )
    .expect("compile approved application UI");
}
