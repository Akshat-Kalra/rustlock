mod error;
mod password;
mod crypto;
mod vault;
mod storage;
mod cli;

use cli::{Commands, Cli};
use error::Result;
use clap::Parser;
use std::path::PathBuf;


fn main() {
    if let Err(e) = run() {
          eprintln!("Error: {}", e);
          std::process::exit(1);
      }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { website, username } => {
            let path = vault_path();
            let master_password = prompt_master_password(!path.exists());

            let mut vault = if path.exists() {
                // Load existing vault
                let encrypted = storage::load_vault(&path)?;
                let key = crypto::derive_key(&master_password, &encrypted[..16])?;
                let decrypted = crypto::decrypt(&key, &encrypted[16..])?;
                serde_json::from_slice(&decrypted)?
            } else {
                // Create new vault
                vault::Vault::new()
            };

            let password = password::generate_password(20,true,true,true)?;
            println!("Generated password for {}: {}", website, password);

            vault.add_entry(website, username, password);

            let json = serde_json::to_vec(&vault)?;
            let key = crypto::derive_key(&master_password, &vault.salt)?;
            let encrypted = crypto::encrypt(&key, &json)?;

            let mut data = vault.salt.clone();
            data.extend(encrypted);
            storage::save_vault(&path, &data)?;

            println!("Entry saved!");

        }
        Commands::Get { website } => {
            let path = vault_path();
            let master_password = prompt_master_password(false);


            let vault: vault::Vault = if path.exists() {
                let encrypted = storage::load_vault(&path)?;
                let key = crypto::derive_key(&master_password, &encrypted[..16])?;
                let decrypted = crypto::decrypt(&key, &encrypted[16..])?;
                serde_json::from_slice(&decrypted)?
            } else {
                eprintln!("No vault found. Add an entry first.");
                std::process::exit(1);
            };



            let entry = match vault.find_entry(&website) {
                Some(e) => e,
                None => {
                    eprintln!("No entry found for {}", website);
                    std::process::exit(1);
                }       
            };

            println!("Website: {}", entry.website);
            println!("Username: {}", entry.username);
            println!("Password: {}", entry.password);


        }
        Commands::List => {
            // Handle list command
            let path = vault_path();
            let master_password = prompt_master_password(false);

            let vault: vault::Vault = if path.exists() {
                let encrypted = storage::load_vault(&path)?;
                let key = crypto::derive_key(&master_password, &encrypted[..16])?;
                let decrypted = crypto::decrypt(&key, &encrypted[16..])?;
                serde_json::from_slice(&decrypted)?
            } else {
                eprintln!("No vault found. Add an entry first.");
                std::process::exit(1);
            };

            for entry in &vault.entries {
                println!("Website: {}, Username: {}", entry.website, entry.username);
            }

        }
        Commands::Generate { length } => {
            let pw = password::generate_password(length, true, true, true)?;
            println!("Generated password: {}", pw);
        }
    }

    Ok(())
}

fn vault_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".rustlock.vault");
    path
}

fn prompt_master_password(confirm: bool) -> String {
    let password = rpassword::prompt_password("Master password: ").unwrap();

    if confirm {
        let confirm = rpassword::prompt_password("Confirm password: ").unwrap();
        if password != confirm {
            eprintln!("Passwords don't match!");
            std::process::exit(1);
        }
    }

    password
}
