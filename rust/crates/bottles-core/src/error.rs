use std::path::{Path, PathBuf};

/// Errors shared by the CLI, GUI, and compatibility core.
#[derive(Debug, thiserror::Error)]
pub enum BottlesError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("bottle not found: {0}")]
    BottleNotFound(String),

    #[error("program not found: {0}")]
    ProgramNotFound(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("external command `{command}` failed: {message}")]
    CommandFailed { command: String, message: String },

    #[error("feature `{feature}` is present in the Python reference but is not ported yet (source: {reference})")]
    FeatureNotPorted { feature: String, reference: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BottlesError>;

/// Attach the file path to an [`std::io::Result`] without collapsing the source error.
pub trait IoResultExt<T> {
    fn with_path(self, path: impl AsRef<Path>) -> Result<T>;
    fn with_path_context(self, path: impl AsRef<Path>, context: &str) -> Result<T>;
}

impl<T> IoResultExt<T> for std::io::Result<T> {
    fn with_path(self, path: impl AsRef<Path>) -> Result<T> {
        self.map_err(|source| BottlesError::Io {
            path: path.as_ref().to_path_buf(),
            source,
        })
    }

    fn with_path_context(self, path: impl AsRef<Path>, context: &str) -> Result<T> {
        self.map_err(|source| BottlesError::Io {
            path: path.as_ref().to_path_buf(),
            source: std::io::Error::new(source.kind(), format!("{context}: {source}")),
        })
    }
}

pub fn io_error(path: impl AsRef<Path>, message: impl Into<String>) -> BottlesError {
    BottlesError::Io {
        path: path.as_ref().to_path_buf(),
        source: std::io::Error::other(message.into()),
    }
}

pub fn feature_not_ported(feature: impl Into<String>, reference: impl Into<String>) -> BottlesError {
    BottlesError::FeatureNotPorted {
        feature: feature.into(),
        reference: reference.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_keeps_path_and_source() {
        let error = io_error("/tmp/bottles", "disk full");
        assert!(error.to_string().contains("/tmp/bottles"));
        assert!(error.to_string().contains("disk full"));
    }
}
