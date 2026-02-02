use crate::error::{RustlockError, Result};
use argon2::{Argon2, Params};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

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


pub fn encrypt(key: &[u8; 32], plaintext:&[u8]) -> Result<Vec<u8>> {

    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);

    let nonce_obj = Nonce::from_slice(&nonce);

    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::Crypto(p.to_string())),
    };

    let ciphertext = match cipher.encrypt(nonce_obj, plaintext) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::Crypto(p.to_string())),
    };

    let mut result = nonce.to_vec();
    result.extend(ciphertext);

    Ok(result)

}

pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(RustlockError::Crypto("Text is shorter than nonce".to_string()));
    }

    let (nonce_bytes, actual_ciphertext) = ciphertext.split_at(12);

    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::Crypto(p.to_string())),
    };

    let plaintext = match cipher.decrypt(nonce, actual_ciphertext) {
        Ok(p) => p,
        Err(p) => return Err(RustlockError::Crypto(p.to_string())),
    };

    Ok(plaintext)
    


}