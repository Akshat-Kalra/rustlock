use thiserror::Error;

#[derive(Debug, Error)]
pub enum RustlockError {

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cryptography error")]
    Crypto(#[from] aes_gcm::Error),

    #[error("Key derivation error: {0}")]
    KeyDerivation(#[from] argon2::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Vault error: {0}")]
    Vault(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, RustlockError>;
