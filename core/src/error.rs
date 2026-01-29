use thiserror::Error;

/// Main error type for Dealve
#[derive(Debug, Error)]
pub enum DealveError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias using DealveError
pub type Result<T> = std::result::Result<T, DealveError>;
