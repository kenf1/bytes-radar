use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to read file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Unsupported file extension: {extension}")]
    UnsupportedExtension { extension: String },

    #[error("Language not found: {language}")]
    LanguageNotFound { language: String },

    #[error("Invalid file statistics: {reason}")]
    InvalidStatistics { reason: String },

    #[error("Directory traversal failed: {path}")]
    DirectoryTraversalError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Language detection failed for file: {file_path}")]
    LanguageDetectionError { file_path: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Serialization error")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Aggregation error: {operation}")]
    AggregationError { operation: String },
}

pub type Result<T> = std::result::Result<T, AnalysisError>;

impl AnalysisError {
    pub fn file_read<P: AsRef<str>>(path: P, source: io::Error) -> Self {
        Self::FileReadError {
            path: path.as_ref().to_string(),
            source,
        }
    }

    pub fn unsupported_extension<E: AsRef<str>>(extension: E) -> Self {
        Self::UnsupportedExtension {
            extension: extension.as_ref().to_string(),
        }
    }

    pub fn language_not_found<L: AsRef<str>>(language: L) -> Self {
        Self::LanguageNotFound {
            language: language.as_ref().to_string(),
        }
    }

    pub fn invalid_statistics<R: AsRef<str>>(reason: R) -> Self {
        Self::InvalidStatistics {
            reason: reason.as_ref().to_string(),
        }
    }

    pub fn directory_traversal<P: AsRef<str>>(path: P, source: io::Error) -> Self {
        Self::DirectoryTraversalError {
            path: path.as_ref().to_string(),
            source,
        }
    }

    pub fn language_detection<P: AsRef<str>>(file_path: P) -> Self {
        Self::LanguageDetectionError {
            file_path: file_path.as_ref().to_string(),
        }
    }

    pub fn configuration<M: AsRef<str>>(message: M) -> Self {
        Self::ConfigurationError {
            message: message.as_ref().to_string(),
        }
    }

    pub fn aggregation<O: AsRef<str>>(operation: O) -> Self {
        Self::AggregationError {
            operation: operation.as_ref().to_string(),
        }
    }
}
