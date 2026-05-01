use std::path::PathBuf;

use crate::{cli::CliAction, docs::generator::DocsGenerator};

/// Generates markdown reference pages for every registered config schema.
///
/// # Errors
///
/// Returns an error if the output directory can't be created or any page
/// fails to write.
pub fn execute(out: PathBuf, only: Option<String>) -> CliAction {
    let generator = DocsGenerator::new().with_output_dir(out);
    match only {
        Some(name) => generator
            .generate_module_by_name(&name)
            .map_err(|err| format!("cannot generate `{name}`: {err}")),
        None => generator
            .generate_all()
            .map_err(|err| format!("cannot generate docs: {err}")),
    }
}
