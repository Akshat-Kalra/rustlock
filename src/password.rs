use rand::seq::IndexedRandom;
use rand::rng;
use crate::error::{Result, RustlockError};


pub fn generate_password(
      length: usize,
      use_uppercase: bool,
      use_digits: bool,
      use_symbols: bool
  ) -> Result<String> {

    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const DIGITS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:',.<>?/`~";

    if length == 0 {
        return Err(RustlockError::InvalidInput("Password length must be greater than 0".to_string()));
    }

    let mut character_pool = Vec::new();
    character_pool.extend_from_slice(LOWERCASE);

    if use_uppercase {
        character_pool.extend_from_slice(UPPERCASE);
    }
    if use_digits {
        character_pool.extend_from_slice(DIGITS);
    }
    if use_symbols {
        character_pool.extend_from_slice(SYMBOLS);
    }

    let mut rng = rng();
    let password: Vec<u8> = (0..length)
      .map(|_| *character_pool.choose(&mut rng).unwrap())
      .collect();
    
    Ok(String::from_utf8(password).unwrap())

  }