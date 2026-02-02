mod error;
mod password;
mod crypto;
mod vault;
mod storage;
mod cli;

use cli::{Commands, Cli};
use error::Result;
use clap::Parser;

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
            println!("hello")
        }
        Commands::Get { website } => {
            // Handle get command
        }
        Commands::List => {
            // Handle list command
        }
        Commands::Generate { length } => {
            let pw = password::generate_password(length, true, true, true)?;
            println!("Generated password: {}", pw);
        }
    }

    Ok(())
}
