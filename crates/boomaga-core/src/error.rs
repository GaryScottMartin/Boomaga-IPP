//! Error types for boomaga

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for boomaga
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in boomaga
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Document error: {0}")]
    Document(String),

    #[error("Print job error: {0}")]
    Job(String),

    #[error("IPP protocol error: {0}")]
    Ipp(String),

    #[error("Failed to parse document: {0}")]
    Parse(String),

    #[error("Failed to render document: {0}")]
    Render(String),

    #[error("D-Bus error: {0}")]
    Bus(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("System error: {0}")]
    System(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Graphics backend error: {0}")]
    Graphics(String),

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Error {
    /// Check if this is a transient error that should be retried
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Io(_) | Self::Timeout(_) | Self::Bus(_))
    }

    /// Check if this is a user-facing error
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Self::Document(_)
                | Self::Job(_)
                | Self::Config(_)
                | Self::Permission(_)
                | Self::Validation(_)
        )
    }

    /// Get the error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Validation(_) | Self::Permission(_) => ErrorSeverity::High,
            Self::Timeout(_) => ErrorSeverity::Medium,
            Self::Job(_) => ErrorSeverity::Medium,
            Self::Document(_) | Self::Parse(_) | Self::Render(_) => ErrorSeverity::Low,
            Self::Config(_) | Self::Plugin(_) | Self::Ipc(_) => ErrorSeverity::Medium,
            Self::Ipp(_) | Self::Bus(_) => ErrorSeverity::Medium,
            Self::System(_) | Self::Graphics(_) | Self::Pdf(_) => ErrorSeverity::Low,
            Self::Unknown(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Low,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Error is expected and indicates a bug
    High,
    /// Error is expected in normal operation
    Medium,
    /// Error is informational and non-critical
    Low,
}
