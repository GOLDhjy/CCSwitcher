use std::path::{Path, PathBuf};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to determine home directory (set HOME, CCSWITCHER_HOME, or CLAUDE_HOME).")]
    MissingHomeDirectory,
    #[error("Preset '{0}' already exists.")]
    PresetAlreadyExists(String),
    #[error("Preset '{0}' was not found.")]
    PresetNotFound(String),
    #[error("Cannot remove active preset '{0}'. Switch presets first.")]
    CannotRemoveActivePreset(String),
    #[error("Unsupported config version {0}.")]
    UnsupportedConfigVersion(u32),
    #[error("Preset '{preset}' is missing required field '{field}'.")]
    PresetIncomplete { preset: String, field: &'static str },
    #[error("JSON root in '{path}' must be an object.")]
    InvalidJsonRoot { path: PathBuf },
    #[error("Failed to write command output: {source}")]
    Output { source: std::io::Error },
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Invalid JSON at '{path}': {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl AppError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.into(),
            source,
        }
    }

    pub fn output(source: std::io::Error) -> Self {
        Self::Output { source }
    }

    pub fn invalid_json_root(path: impl AsRef<Path>) -> Self {
        Self::InvalidJsonRoot {
            path: path.as_ref().to_path_buf(),
        }
    }
}
