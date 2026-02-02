# rustlock

A local-first command-line password manager written in Rust.

## Why

Rustlock stores everything locally in an encrypted vault that never leaves my machine.

## Features

- **Generate** cryptographically secure passwords (92-character alphabet, OS-level entropy)
- **Store** credentials with AES-256-GCM authenticated encryption
- **Retrieve** passwords by website name
- **List** all stored entries

## Security

| Component | Choice | Why |
|-----------|--------|-----|
| Encryption | AES-256-GCM | Authenticated encryption (confidentiality + integrity) |
| Key Derivation | Argon2id | Memory-hard (64MB), resistant to GPU brute-force |
| Random Generation | OS entropy | Cryptographically secure via `rand` crate |

### Vault File Format

```
[salt: 16 bytes][nonce: 12 bytes][ciphertext + auth_tag]
```

The salt is stored with the encrypted data so the same master password derives the same key. Each encryption uses a unique random nonce.

## Install

Requires [Rust](https://rustup.rs/).

```bash
git clone https://github.com/akshat-kalra/rustlock.git
cd rustlock
cargo install --path .
```

## Usage

```bash
# Add a new entry (generates 20-char password automatically)
rustlock add github.com myusername

# List all stored entries
rustlock list

# Retrieve a specific entry
rustlock get github.com

# Generate a standalone password
rustlock generate 24
```

## License

MIT
