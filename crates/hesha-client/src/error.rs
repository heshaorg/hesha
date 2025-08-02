//! Client error types.

use thiserror::Error;

/// Errors that can occur in the HTTP client.
#[derive(Debug, Error)]
pub enum ClientError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// Server returned an error status.
    #[error("Server error {status}: {message}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// Error message from server.
        message: String,
    },
    
    /// Invalid response format.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Invalid URL or domain.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    /// Timeout error.
    #[error("Request timed out")]
    Timeout,
}

/// Result type for client operations.
pub type ClientResult<T> = Result<T, ClientError>;