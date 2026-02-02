use thiserror::Error;

#[derive(Debug, Error)]
pub enum RustlockError {

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Crypto(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, RustlockError>;
