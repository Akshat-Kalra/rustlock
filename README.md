# rustlock

A local-first command-line password manager written in Rust.

## Why

This is a personal tool that stores passwords locally in an encrypted vault.

## Security

- AES-256-GCM authenticated encryption
- Argon2id key derivation (memory-hard, resistant to GPU attacks)
- Secrets are zeroized from memory after use

## Status

Work in progress.

## Build

```bash
cargo build --release
```

## License

MIT
