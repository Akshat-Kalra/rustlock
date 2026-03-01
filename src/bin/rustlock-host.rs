use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use rustlock::{crypto, vault, storage, error::{Result, RustlockError}};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Request {
    DeriveKey { master_password: String },
    GetCredentials {
        url: String,
        master_password: Option<String>,
        derived_key: Option<String>,
    },
    SaveCredentials {
        website: String,
        username: String,
        password: String,
        master_password: Option<String>,
        derived_key: Option<String>,
    },
}

#[derive(Serialize)]
struct Credential { website: String, username: String, password: String }

#[derive(Serialize)]
struct Response {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credentials: Option<Vec<Credential>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn vault_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().unwrap_or_default();
    p.push(".rustlock.vault");
    p
}

/// Resolve whichever credential form is present to a 32-byte key.
fn resolve_key(master_password: Option<&str>, derived_key: Option<&str>, salt: &[u8]) -> Result<[u8; 32]> {
    if let Some(dk_hex) = derived_key {
        let bytes = hex::decode(dk_hex)
            .map_err(|e| RustlockError::InvalidInput(format!("Invalid derived_key hex: {e}")))?;
        if bytes.len() != 32 {
            return Err(RustlockError::InvalidInput("derived_key must be 32 bytes (64 hex chars)".into()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else if let Some(mp) = master_password {
        crypto::derive_key(mp, salt)
    } else {
        Err(RustlockError::InvalidInput("Either master_password or derived_key is required".into()))
    }
}

fn open_vault_with_key(key: &[u8; 32], encrypted: &[u8]) -> Result<vault::Vault> {
    let ciphertext = &encrypted[16..];
    let decrypted = crypto::decrypt(key, ciphertext)?;
    Ok(serde_json::from_slice(&decrypted)?)
}

fn save_vault_with_key(key: &[u8; 32], v: &vault::Vault) -> Result<()> {
    let json = serde_json::to_vec(v)?;
    let encrypted = crypto::encrypt(key, &json)?;
    let mut data = v.salt.clone();
    data.extend(encrypted);
    storage::save_vault(&vault_path(), &data)
}

/// Extracts hostname from a URL, stripping scheme and "www." prefix.
fn extract_domain(url: &str) -> String {
    url.split("://").nth(1).unwrap_or(url)
        .split('/').next().unwrap_or(url)
        .trim_start_matches("www.")
        .to_string()
}

fn err_resp(msg: impl Into<String>) -> Response {
    Response { ok: false, key: None, credentials: None, error: Some(msg.into()) }
}

fn handle(req: Request) -> Response {
    match req {
        Request::DeriveKey { master_password } => {
            let encrypted = match storage::load_vault(&vault_path()) {
                Err(e) => return err_resp(e.to_string()),
                Ok(e) => e,
            };
            if encrypted.len() < 16 {
                return err_resp("Vault too short");
            }
            let salt = &encrypted[..16];
            match crypto::derive_key(&master_password, salt) {
                Err(e) => err_resp(e.to_string()),
                Ok(k) => Response { ok: true, key: Some(hex::encode(k)), credentials: None, error: None },
            }
        }

        Request::GetCredentials { url, master_password, derived_key } => {
            let domain = extract_domain(&url);
            let encrypted = match storage::load_vault(&vault_path()) {
                Err(e) => return err_resp(e.to_string()),
                Ok(e) => e,
            };
            if encrypted.len() < 16 {
                return err_resp("Vault too short");
            }
            let salt = &encrypted[..16];
            let key = match resolve_key(master_password.as_deref(), derived_key.as_deref(), salt) {
                Err(e) => return err_resp(e.to_string()),
                Ok(k) => k,
            };
            match open_vault_with_key(&key, &encrypted) {
                Err(e) => err_resp(e.to_string()),
                Ok(v) => {
                    let creds = v.entries.iter()
                        .filter(|e| e.website.contains(&domain) || domain.contains(e.website.as_str()))
                        .map(|e| Credential {
                            website: e.website.clone(),
                            username: e.username.clone(),
                            password: e.password.clone(),
                        })
                        .collect();
                    Response { ok: true, key: None, credentials: Some(creds), error: None }
                }
            }
        }

        Request::SaveCredentials { website, username, password, master_password, derived_key } => {
            let encrypted = match storage::load_vault(&vault_path()) {
                Err(e) => return err_resp(e.to_string()),
                Ok(e) => e,
            };
            if encrypted.len() < 16 {
                return err_resp("Vault too short");
            }
            let salt = &encrypted[..16];
            let key = match resolve_key(master_password.as_deref(), derived_key.as_deref(), salt) {
                Err(e) => return err_resp(e.to_string()),
                Ok(k) => k,
            };
            match open_vault_with_key(&key, &encrypted) {
                Err(e) => err_resp(e.to_string()),
                Ok(mut v) => {
                    v.upsert_entry(website, username, password);
                    match save_vault_with_key(&key, &v) {
                        Ok(_) => Response { ok: true, key: None, credentials: None, error: None },
                        Err(e) => err_resp(e.to_string()),
                    }
                }
            }
        }
    }
}

fn read_message() -> io::Result<String> {
    let mut len_buf = [0u8; 4];
    io::stdin().read_exact(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn write_message(msg: &str) -> io::Result<()> {
    let bytes = msg.as_bytes();
    io::stdout().write_all(&(bytes.len() as u32).to_ne_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()
}

fn main() {
    if let Ok(msg) = read_message() {
        let response = match serde_json::from_str::<Request>(&msg) {
            Ok(req) => handle(req),
            Err(e) => Response { ok: false, key: None, credentials: None, error: Some(format!("Parse error: {e}")) },
        };
        let json = serde_json::to_string(&response)
            .unwrap_or_else(|_| r#"{"ok":false,"error":"serialization failed"}"#.into());
        let _ = write_message(&json);
    }
}
