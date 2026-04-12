use std::path::PathBuf;

use thiserror::Error;

/// Error types for the Wayle configuration infrastructure.
#[derive(Error, Debug)]
pub enum Error {
    /// Circular import detected in configuration files.
    #[error("circular import detected: {chain}")]
    CircularImport {
        /// Human-readable import chain showing the cycle.
        chain: String,
    },

    /// Configuration deserialization failed.
    #[error("invalid config: {source}")]
    ConfigDeserialization {
        /// The underlying TOML deserialization error.
        #[source]
        source: toml::de::Error,
    },

    /// Configuration field is invalid or missing.
    #[error("invalid config field '{field}' in {component}: {reason}")]
    InvalidConfigField {
        /// The field that is invalid.
        field: String,
        /// Component containing the field.
        component: String,
        /// Reason why the field is invalid.
        reason: InvalidFieldReason,
    },

    /// I/O operation failed.
    #[error("cannot {operation}")]
    Io {
        /// What operation was being attempted.
        operation: IoOperation,
        /// Path where I/O error occurred.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// TOML parsing failed.
    #[error("{source}\n  --> {}", path.display())]
    TomlParse {
        /// Location of TOML being parsed.
        path: PathBuf,
        /// The underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },

    /// TOML parsing failed for inline content.
    #[error("{source}")]
    TomlParseInline {
        /// The underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },

    /// Import operation failed.
    #[error("cannot import '{}'", path.display())]
    Import {
        /// Path of file being imported.
        path: PathBuf,
        /// The underlying import error.
        #[source]
        source: Box<Error>,
    },

    /// Import path has no parent directory.
    #[error("cannot resolve import path '{}': no parent directory", path.display())]
    ImportNoParent {
        /// Path with no parent.
        path: PathBuf,
    },

    /// TOML serialization failed.
    #[error("cannot serialize {content_type}")]
    Serialization {
        /// Type of content being serialized.
        content_type: &'static str,
        /// The underlying TOML serialization error.
        #[source]
        source: toml::ser::Error,
    },

    /// Persistence operation failed.
    #[error("cannot persist config to '{}'", path.display())]
    Persistence {
        /// Path where persistence failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Theme file read failed.
    #[error("cannot read theme file '{}'", path.display())]
    ThemeRead {
        /// Path of the theme file.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Theme TOML parsing failed.
    #[error("cannot parse theme file '{}': {source}", path.display())]
    ThemeParse {
        /// Path of the theme file.
        path: PathBuf,
        /// The underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },

    /// File watcher initialization failed.
    #[error("cannot initialize file watcher")]
    WatcherInit {
        /// The underlying notify error.
        #[source]
        source: notify::Error,
    },

    /// Watch operation failed.
    #[error("cannot watch '{}'", path.display())]
    Watch {
        /// Path that could not be watched.
        path: PathBuf,
        /// The underlying notify error.
        #[source]
        source: notify::Error,
    },

    /// Blocking task failed to complete.
    #[error("config task failed")]
    TaskJoin {
        /// The underlying join error.
        #[source]
        source: tokio::task::JoinError,
    },

    /// Watcher state is poisoned.
    #[error("watcher state is poisoned")]
    WatcherPoisoned,

    /// Config value is invalid for the target field type.
    #[error("{0}")]
    InvalidValue(String),
}

/// Reasons why a config field is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidFieldReason {
    /// The field was not found.
    NotFound,
    /// The path was empty.
    EmptyPath,
    /// The parent is not a table.
    ParentNotTable,
}

impl std::fmt::Display for InvalidFieldReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "field not found"),
            Self::EmptyPath => write!(f, "empty path"),
            Self::ParentNotTable => write!(f, "parent is not a table"),
        }
    }
}

/// I/O operations for error context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoOperation {
    /// Reading a file.
    ReadFile,
    /// Writing a file.
    WriteFile,
    /// Creating a directory.
    CreateDir,
    /// Resolving a path.
    ResolvePath,
    /// Accessing config directory.
    AccessConfigDir,
}

impl std::fmt::Display for IoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile => write!(f, "read file"),
            Self::WriteFile => write!(f, "write file"),
            Self::CreateDir => write!(f, "create directory"),
            Self::ResolvePath => write!(f, "resolve path"),
            Self::AccessConfigDir => write!(f, "access config directory"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Self::Io {
            operation: IoOperation::ReadFile,
            path: PathBuf::new(),
            source,
        }
    }
}
