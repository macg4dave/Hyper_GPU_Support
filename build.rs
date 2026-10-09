//! Compile the approved Fluent Slint components into the host application.
fn main() {
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".into()),
    )
    .expect("compile approved application UI");
}
