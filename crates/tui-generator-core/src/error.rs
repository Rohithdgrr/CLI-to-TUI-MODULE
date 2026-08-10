use std::fmt;

#[derive(Debug)]
pub enum TuiError {
    TerminalError(String),
    ValidationError(Vec<crate::validation::ValidationError>),
    ConversionError(String),
    UnsupportedType(String),
    Cancelled,
    IoError(std::io::Error),
}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TuiError::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
            TuiError::ValidationError(errors) => {
                write!(f, "Validation failed:")?;
                for e in errors {
                    write!(f, "\n  - {}", e)?;
                }
                Ok(())
            }
            TuiError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            TuiError::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
            TuiError::Cancelled => write!(f, "Cancelled by user"),
            TuiError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TuiError {}

impl From<std::io::Error> for TuiError {
    fn from(e: std::io::Error) -> Self {
        TuiError::IoError(e)
    }
}
