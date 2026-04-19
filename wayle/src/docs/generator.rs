//! Top-level driver for the docs generator.
//!
//! [`DocsGenerator`] reads [`super::registry::ModuleRegistry`] and writes one
//! markdown page per schema via [`super::module_page`], plus the shared
//! [`super::types_page`] and [`super::index_page`].

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;
use thiserror::Error;
use tracing::{info, instrument};

use super::{
    index_page::render_config_index,
    module_page::generate_module_page,
    registry::{ModuleEntry, ModuleRegistry},
    types_page::{collect_type_defs, known_type_names, render_types_page},
};

/// Writes one markdown reference page per registered config schema.
pub struct DocsGenerator {
    output_dir: PathBuf,
}

impl Default for DocsGenerator {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs/config"),
        }
    }
}

impl DocsGenerator {
    /// Creates a generator that writes to `docs/config`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the output directory. The generator creates it (and the
    /// `modules/` subdirectory for bar modules) on demand.
    pub fn with_output_dir(mut self, output_dir: impl Into<PathBuf>) -> Self {
        self.output_dir = output_dir.into();
        self
    }

    /// Generates one page per registered module plus the shared types and
    /// index pages.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Write`] if the output directory can't be created or
    /// any page can't be written. Returns [`Error::SerializeSchema`] if a
    /// schema can't be converted into a JSON value.
    #[instrument(skip(self), fields(output_dir = %self.output_dir.display()))]
    pub fn generate_all(&self) -> Result<(), Error> {
        self.ensure_dir(&self.output_dir)?;

        let modules = ModuleRegistry::entries();
        let type_defs = collect_type_defs(&modules);
        let known_types = known_type_names(&type_defs);

        for entry in &modules {
            self.generate_one(entry, &known_types)?;
        }

        self.write_types_page(&type_defs)?;
        self.write_index_page(&modules)?;

        info!(count = modules.len(), "Generated module pages");
        Ok(())
    }

    /// Generates the page for a single named module.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ModuleNotFound`] if nothing is registered under that
    /// name; otherwise the same errors as [`Self::generate_all`].
    #[instrument(skip(self))]
    pub fn generate_module_by_name(&self, module_name: &str) -> Result<(), Error> {
        let entry = ModuleRegistry::find(module_name).ok_or_else(|| Error::ModuleNotFound {
            name: module_name.to_string(),
        })?;

        let all_modules = ModuleRegistry::entries();
        let type_defs = collect_type_defs(&all_modules);
        let known_types = known_type_names(&type_defs);

        self.generate_one(&entry, &known_types)
    }

    /// Every registered module name, sorted alphabetically.
    pub fn list_modules(&self) -> Vec<String> {
        ModuleRegistry::names()
    }

    fn generate_one(
        &self,
        entry: &ModuleEntry,
        known_types: &BTreeSet<String>,
    ) -> Result<(), Error> {
        let content = generate_module_page(entry, known_types)?;
        let target_dir = self.dir_for(entry);
        self.ensure_dir(&target_dir)?;

        let filepath = target_dir.join(format!("{}.md", entry.info.name));
        self.write_page(&filepath, &content)
    }

    #[instrument(skip(self, type_defs))]
    fn write_types_page(&self, type_defs: &BTreeMap<String, Value>) -> Result<(), Error> {
        let filepath = self.output_dir.join("types.md");
        self.write_page(&filepath, &render_types_page(type_defs))
    }

    #[instrument(skip(self, modules))]
    fn write_index_page(&self, modules: &[ModuleEntry]) -> Result<(), Error> {
        let filepath = self.output_dir.join("index.md");
        self.write_page(&filepath, &render_config_index(modules))
    }

    /// Resolves the output directory for `entry`: bar modules live in
    /// `modules/`, top-level schemas live at the root.
    fn dir_for(&self, entry: &ModuleEntry) -> PathBuf {
        if entry.info.layout_id.is_some() {
            self.output_dir.join("modules")
        } else {
            self.output_dir.clone()
        }
    }

    fn ensure_dir(&self, dir: &Path) -> Result<(), Error> {
        fs::create_dir_all(dir).map_err(|source| Error::Write {
            path: dir.to_path_buf(),
            source,
        })
    }

    fn write_page(&self, filepath: &Path, content: &str) -> Result<(), Error> {
        fs::write(filepath, content).map_err(|source| Error::Write {
            path: filepath.to_path_buf(),
            source,
        })?;
        info!(path = %filepath.display(), "Wrote page");
        Ok(())
    }
}

/// Errors produced during page generation.
#[derive(Error, Debug)]
pub enum Error {
    /// Writing a generated file or creating its directory failed.
    #[error("cannot write `{}`", path.display())]
    Write {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: io::Error,
    },

    /// The schema could not be serialised to JSON.
    #[error("cannot serialise schema for `{module}`")]
    SerializeSchema {
        /// Module whose schema serialisation failed.
        module: String,
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// The requested module name is not registered.
    #[error("module `{name}` not registered")]
    ModuleNotFound {
        /// The name that was requested.
        name: String,
    },
}
