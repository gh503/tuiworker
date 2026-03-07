//! Music player error types

use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

/// Music player error type
#[derive(Debug, Clone, ThisError)]
pub enum MusicError {
    #[error("Source not available: {0}")]
    SourceNotAvailable(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Playback failed: {0}")]
    PlaybackFailed(String),

    #[error("API error: {0}")]
    APIError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<io::Error> for MusicError {
    fn from(err: io::Error) -> Self {
        MusicError::Io(err.to_string())
    }
}

impl From<reqwest::Error> for MusicError {
    fn from(err: reqwest::Error) -> Self {
        MusicError::PlaybackFailed(err.to_string())
    }
}

/// Result type alias for music operations
pub type Result<T> = std::result::Result<T, MusicError>;
