# rustlock

A local-first command-line password manager written in Rust.

## Why

This is a personal tool that stores passwords locally in an encrypted vault.

## Security

- AES-256-GCM authenticated encryption
- Argon2id key derivation (memory-hard, resistant to GPU attacks)
- Secrets are zeroized from memory after use

## Install

Requires [Rust](https://rustup.rs/).

```bash
git clone https://github.com/yourusername/rustlock.git
cd rustlock
cargo install --path .
```

## Usage

```bash
# Add a new entry (generates password automatically)
rustlock add github.com myusername

# List all entries
rustlock list

# Get a specific entry
rustlock get github.com

# Generate a standalone password
rustlock generate 24
```

## License

MIT
