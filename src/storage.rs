use std::fs;
use std::path::Path;
use crate::error::Result;

pub fn save_vault(path: &Path, encrypted_data: &[u8]) -> Result<()> {
    match fs::write(path, encrypted_data) {
        Ok(p) => Ok(p),
        Err(p)  => return Err(crate::error::RustlockError::Io(p)),
    }
}

pub fn load_vault(path: &Path) -> Result<Vec<u8>> {
    match fs::read(path) {
        Ok(p) => Ok(p),
        Err(p) => return Err(crate::error::RustlockError::Io(p))
    }
}