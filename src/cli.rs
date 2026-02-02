use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustlock")]
#[command(about = "A local password manager")]

pub struct Cli {

    #[arg(short, long)]
    pub password: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        website: String,
        username: String
    },
    Get {
        website: String
    },
    List,
    Generate {
        length: usize,
    },

    }