#[path = "../../build/windows_resources.rs"]
mod windows_resources;

fn main() {
    println!("cargo:rerun-if-changed=../../build/windows_resources.rs");
    windows_resources::compile(
        &["app-install"],
        std::path::Path::new("../../icons/Appicon.ico"),
    )
    .expect("compile Windows version resources");
}
