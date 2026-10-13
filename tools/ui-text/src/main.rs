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
        vec![
            "usage: ui-text DIRECTORY [--locale TAG (locale root)] [--generate OUTPUT_DIRECTORY]"
                .into(),
        ]
    })?;
    let mut output = None;
    let mut locale = None;
    while let Some(option) = args.next() {
        if option == "--generate" && output.is_none() {
            output = Some(
                args.next()
                    .map(PathBuf::from)
                    .ok_or_else(|| vec!["--generate requires an output directory".into()])?,
            );
        } else if option == "--locale" && locale.is_none() {
            locale = Some(
                args.next()
                    .and_then(|value| value.into_string().ok())
                    .ok_or_else(|| vec!["--locale requires a UTF-8 language tag".into()])?,
            );
        } else {
            return Err(vec!["unexpected or duplicate argument".into()]);
        }
    }
    // Fail before any generated files are created. Build callers use the same API.
    let contract = ui_text::contract::active();
    let (count, sources) = if let Some(locale) = locale {
        let loaded = ui_text::locales::load(&directory, Some(&locale), &contract)?;
        println!(
            "Resolved locale: {}; available: {}",
            loaded.locale,
            loaded.available.join(", ")
        );
        (loaded.catalogue.len(), loaded.generate())
    } else {
        let catalogue = ui_text::validate_directory(&directory, &contract)?;
        (catalogue.len(), catalogue.generate())
    };
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
            write_changed(&output.join("ui_text.rs"), sources.rust.as_bytes())?;
            write_changed(&output.join("ui-text.slint"), sources.slint.as_bytes())
        };
        write().map_err(|e| vec![format!("generation output: {e}")])?;
    }
    println!(
        "Validated {} messages against the active code contract.",
        count
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
