use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using the library's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for the llm-utl library.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// IO error with context about the file path.
    #[error("IO error accessing '{path}': {message}")]
    Io {
        /// Path where the error occurred
        path: PathBuf,
        /// Error message
        message: String,
    },

    /// Template rendering error.
    #[error("Failed to render template '{template}': {message}")]
    Template {
        /// Template name
        template: String,
        /// Error message
        message: String,
    },

    /// Configuration validation error.
    #[error("Invalid configuration: {message}")]
    Config {
        /// Detailed error message
        message: String,
    },

    /// File exceeds maximum token limit.
    #[error("File '{path}' is too large: {size} tokens exceeds limit of {limit} tokens")]
    FileTooLarge {
        /// Path to the oversized file
        path: PathBuf,
        /// Actual token count
        size: usize,
        /// Maximum allowed tokens
        limit: usize,
    },

    /// No processable files found in directory.
    #[error("No processable files found in '{path}'. Check .gitignore rules or file permissions.")]
    NoFiles {
        /// Directory that was scanned
        path: PathBuf,
    },

    /// JSON serialization error.
    #[error("Serialization error: {message}")]
    Serialization {
        /// Error message
        message: String,
    },

    /// Invalid UTF-8 encountered in file.
    #[error("Invalid UTF-8 encoding in file '{path}'. File may be binary or use unsupported encoding.")]
    InvalidUtf8 {
        /// Path to file with encoding issues
        path: PathBuf,
    },

    /// System time error.
    #[error("System time error: {message}")]
    SystemTime {
        /// Error message
        message: String,
    },

    /// Multiple errors occurred during processing.
    #[error("Multiple errors occurred during processing ({count} errors)")]
    Multiple {
        /// Number of errors
        count: usize,
        /// Collection of errors
        errors: Vec<Error>,
    },

    /// Invalid output pattern.
    #[error("Invalid output pattern '{pattern}': {reason}")]
    InvalidPattern {
        /// The invalid pattern
        pattern: String,
        /// Reason why it's invalid
        reason: String,
    },
}

impl Error {
    /// Creates an IO error with path context.
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            message: source.to_string(),
        }
    }

    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates a template error.
    #[must_use]
    pub fn template(template: impl Into<String>, source: tera::Error) -> Self {
        Self::Template {
            template: template.into(),
            message: source.to_string(),
        }
    }

    /// Creates an invalid UTF-8 error.
    #[must_use]
    pub fn invalid_utf8(path: impl Into<PathBuf>) -> Self {
        Self::InvalidUtf8 { path: path.into() }
    }

    /// Creates a no files error.
    #[must_use]
    pub fn no_files(path: impl Into<PathBuf>) -> Self {
        Self::NoFiles { path: path.into() }
    }

    /// Creates an invalid pattern error.
    #[must_use]
    pub fn invalid_pattern(pattern: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPattern {
            pattern: pattern.into(),
            reason: reason.into(),
        }
    }

    /// Combines multiple errors into a single error.
    #[must_use]
    pub fn multiple(errors: Vec<Self>) -> Self {
        let count = errors.len();
        Self::Multiple { count, errors }
    }

    /// Returns true if this is an IO error.
    #[must_use]
    pub const fn is_io(&self) -> bool {
        matches!(self, Self::Io { .. })
    }

    /// Returns true if this is a configuration error.
    #[must_use]
    pub const fn is_config(&self) -> bool {
        matches!(self, Self::Config { .. })
    }
}

// Conversion implementations for convenient error handling
impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Self {
        Self::SystemTime {
            message: e.to_string(),
        }
    }
}

impl From<tera::Error> for Error {
    fn from(e: tera::Error) -> Self {
        Self::Template {
            template: "unknown".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization {
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test message");
        assert!(err.is_config());
        assert!(err.to_string().contains("test message"));
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::io("/tmp/test.txt", io_err);
        assert!(err.is_io());
        assert!(err.to_string().contains("/tmp/test.txt"));
    }

    #[test]
    fn test_multiple_errors() {
        let errors = vec![
            Error::config("error 1"),
            Error::config("error 2"),
        ];
        let combined = Error::multiple(errors);
        assert!(combined.to_string().contains("2 errors"));
    }

    #[test]
    fn test_error_clone() {
        let err = Error::config("test");
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_serialization_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: Error = json_err.into();
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_system_time_error() {
        use std::time::{Duration, SystemTime};

        // Create a time error by using invalid arithmetic
        let past = SystemTime::UNIX_EPOCH;
        let future = past + Duration::from_secs(1);
        let result = past.duration_since(future);

        if let Err(e) = result {
            let err: Error = e.into();
            assert!(err.to_string().contains("System time error"));
        }
    }
}