use serde::{Serialize, Deserialize};
use rand::RngCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub website: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub salt: Vec<u8>,
    pub entries: Vec<Entry>,
}

impl Vault {
    pub fn new() -> Self {
        let mut salt = [0u8; 16];
        rand::rng().fill_bytes(&mut salt);

        Vault {
            salt: salt.to_vec(),
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, website: String, username: String, password: String) {
        let entry = Entry {
            website,
            username,
            password,
        };
        self.entries.push(entry);
    }

    pub fn find_entry(&self, website: &str) -> Option<&Entry> {
        for entry in &self.entries {
            if entry.website == website {
                return Some(entry);
            }
        }
        None
    }
}