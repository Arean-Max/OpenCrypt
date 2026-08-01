use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format")]
    InvalidFormat,

    #[error("Invalid password or corrupted data")]
    InvalidPassword,

    #[error("Memory allocation failed")]
    Memory,

    #[error("Invalid parameter")]
    InvalidParam,

    #[error("Operation not supported on this platform")]
    Unsupported,

    #[error("Output file already exists")]
    OutputExists,

    #[error("Invalid input file or path")]
    InputInvalid,
}

pub type Result<T> = std::result::Result<T, CryptError>;
