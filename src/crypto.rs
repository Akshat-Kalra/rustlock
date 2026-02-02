use crate::error::{RustlockError, Result};
use argon2::{Argon2, Params};
    
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let params = match Params::new(64 * 1024, 3, 1, Some(32)) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::KeyDerivation(p.to_string())),
    };

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];

    match argon2.hash_password_into(password.as_bytes(), salt, &mut key) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::KeyDerivation(p.to_string())),
    };

    Ok(key)
}