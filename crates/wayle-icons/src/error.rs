use std::path::PathBuf;

use thiserror::Error;

/// Reason why SVG validation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgValidationError {
    /// SVG parsing failed.
    ParseError(String),
    /// SVG parsed but contained no extractable path geometry.
    NoExtractablePaths,
}

impl std::fmt::Display for SvgValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "{msg}"),
            Self::NoExtractablePaths => write!(f, "no extractable paths"),
        }
    }
}

/// Errors that can occur during icon operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Cannot fetch icon from CDN due to HTTP error status.
    #[error("cannot fetch icon '{slug}' from {icon_source}: HTTP {status}")]
    FetchError {
        /// The icon slug that failed to fetch.
        slug: String,
        /// The source name (e.g., "tabler", "simple-icons").
        icon_source: String,
        /// HTTP status code.
        status: reqwest::StatusCode,
    },

    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Cannot read file.
    #[error("cannot read file '{path}'")]
    ReadError {
        /// Path that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Cannot write icon to disk.
    #[error("cannot write icon to '{path}'")]
    WriteError {
        /// Path where the write failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Cannot delete icon.
    #[error("cannot delete icon '{name}'")]
    DeleteError {
        /// Icon name that failed to delete.
        name: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Icon not found.
    #[error("icon '{name}' not found")]
    NotFound {
        /// Icon name that was not found.
        name: String,
    },

    /// Icon name contains characters that would escape the icons directory.
    #[error("icon name '{name}' must not contain '/', '\\', or '..'")]
    InvalidIconName {
        /// The rejected icon name.
        name: String,
    },

    /// Invalid icon source.
    #[error("unknown icon source '{name}', expected one of: {available}")]
    InvalidSource {
        /// The invalid source name provided.
        name: String,
        /// Comma-separated list of valid source names.
        available: String,
    },

    /// Invalid SVG content.
    #[error("invalid SVG content for '{slug}': {reason}")]
    InvalidSvg {
        /// The icon slug with invalid SVG.
        slug: String,
        /// Validation error reason.
        reason: SvgValidationError,
    },

    /// Cannot create icon directory.
    #[error("cannot create icon directory '{path}'")]
    DirectoryError {
        /// Path where directory creation failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Cannot initialize icon registry with GTK.
    #[error("cannot initialize icon registry: {reason}")]
    RegistryError {
        /// Description of what went wrong.
        reason: &'static str,
    },

    /// HOME environment variable not set.
    #[error("$HOME environment variable not set")]
    HomeNotSet,

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for icon operations.
pub type Result<T> = std::result::Result<T, Error>;
