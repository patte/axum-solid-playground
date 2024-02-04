# rust axum backend

## Usage

See the combined [README](../README.md) for usage.

## Development

Install cargo-run-bin which allows to specify [bin dependencies in the Cargo.toml](ttps://github.com/rust-lang/cargo/issues/2267) and enable a devx like `npx run` for cargo:
```bash
cargo install cargo-run-bin
```

### Migrations

Create a new migration:
```bash
cargo bin sqlx migrate add create_users
```