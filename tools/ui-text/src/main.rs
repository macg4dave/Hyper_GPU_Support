use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn write_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn run() -> Result<(), Vec<String>> {
    let mut args = env::args_os().skip(1);
    let directory = args.next().map(PathBuf::from).ok_or_else(|| {
        vec!["usage: ui-text CATALOGUE_DIRECTORY [--generate OUTPUT_DIRECTORY]".into()]
    })?;
    let output = match args.next() {
        None => None,
        Some(option) if option == "--generate" => Some(
            args.next()
                .map(PathBuf::from)
                .ok_or_else(|| vec!["--generate requires an output directory".into()])?,
        ),
        _ => {
            return Err(vec![
                "usage: ui-text CATALOGUE_DIRECTORY [--generate OUTPUT_DIRECTORY]".into(),
            ]);
        }
    };
    if args.next().is_some() {
        return Err(vec!["unexpected argument".into()]);
    }
    // Fail before any generated files are created. Build callers use the same API.
    let catalogue = ui_text::validate_directory(&directory, &ui_text::contract::active())?;
    if let Some(output) = output {
        let write = || -> std::io::Result<()> {
            let output_absolute = if output.is_absolute() {
                output.clone()
            } else {
                env::current_dir()?.join(&output)
            };
            let catalogue_absolute = fs::canonicalize(&directory)?;
            // Normalize existing ancestors before creation so aliases cannot place
            // outputs in the resource tree. Components after the ancestor are local.
            let mut ancestor = output_absolute.as_path();
            let mut tail = Vec::new();
            while !ancestor.exists() {
                tail.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| std::io::Error::other("invalid output path"))?,
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| std::io::Error::other("invalid output path"))?;
            }
            let mut resolved = fs::canonicalize(ancestor)?;
            for component in tail.into_iter().rev() {
                resolved.push(component);
            }
            if resolved.starts_with(&catalogue_absolute) {
                return Err(std::io::Error::other(
                    "generated outputs must be outside the catalogue",
                ));
            }
            fs::create_dir_all(&output)?;
            let sources = catalogue.generate();
            write_changed(&output.join("ui_text.rs"), sources.rust.as_bytes())?;
            write_changed(&output.join("ui-text.slint"), sources.slint.as_bytes())
        };
        write().map_err(|e| vec![format!("generation output: {e}")])?;
    }
    println!(
        "Validated {} messages against the active code contract.",
        catalogue.len()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}
