use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use rustlock::{crypto, vault, storage, error::{Result, RustlockError}};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Request {
    GetCredentials { url: String, master_password: String },
    SaveCredentials { website: String, username: String, password: String, master_password: String },
}

#[derive(Serialize)]
struct Credential { website: String, username: String, password: String }

#[derive(Serialize)]
struct Response {
    ok: bool,
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

fn open_vault(master_password: &str) -> Result<vault::Vault> {
    let encrypted = storage::load_vault(&vault_path())?;
    if encrypted.len() < 16 {
        return Err(RustlockError::InvalidInput("Vault file too short".into()));
    }
    let (salt, ciphertext) = encrypted.split_at(16);
    let key = crypto::derive_key(master_password, salt)?;
    let decrypted = crypto::decrypt(&key, ciphertext)?;
    Ok(serde_json::from_slice(&decrypted)?)
}

fn save_vault(master_password: &str, v: &vault::Vault) -> Result<()> {
    let json = serde_json::to_vec(v)?;
    let key = crypto::derive_key(master_password, &v.salt)?;
    let encrypted = crypto::encrypt(&key, &json)?;
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

fn handle(req: Request) -> Response {
    match req {
        Request::GetCredentials { url, master_password } => {
            let domain = extract_domain(&url);
            match open_vault(&master_password) {
                Err(e) => Response { ok: false, credentials: None, error: Some(e.to_string()) },
                Ok(v) => {
                    let creds = v.entries.iter()
                        .filter(|e| e.website.contains(&domain) || domain.contains(e.website.as_str()))
                        .map(|e| Credential {
                            website: e.website.clone(),
                            username: e.username.clone(),
                            password: e.password.clone(),
                        })
                        .collect();
                    Response { ok: true, credentials: Some(creds), error: None }
                }
            }
        }
        Request::SaveCredentials { website, username, password, master_password } => {
            match open_vault(&master_password) {
                Err(e) => Response { ok: false, credentials: None, error: Some(e.to_string()) },
                Ok(mut v) => {
                    v.add_entry(website, username, password);
                    match save_vault(&master_password, &v) {
                        Ok(_) => Response { ok: true, credentials: None, error: None },
                        Err(e) => Response { ok: false, credentials: None, error: Some(e.to_string()) },
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
            Err(e) => Response { ok: false, credentials: None, error: Some(format!("Parse error: {e}")) },
        };
        let json = serde_json::to_string(&response)
            .unwrap_or_else(|_| r#"{"ok":false,"error":"serialization failed"}"#.into());
        let _ = write_message(&json);
    }
}
