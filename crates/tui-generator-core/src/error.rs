use thiserror::Error;

#[derive(Debug, Error)]
pub enum TuiError {
    #[error("Terminal error: {0}")]
    TerminalError(String),
    #[error("Validation failed: {0:?}")]
    ValidationError(Vec<crate::validation::ValidationError>),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    #[error("Cancelled by user")]
    Cancelled,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
