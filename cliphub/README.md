# ClipHub

Clipboard history manager with multi-platform sync.

## Features

- 📋 Automatic clipboard monitoring
- 💾 Local SQLite storage
- 🔍 Search and filter
- 🔄 Sync to Notion, self-hosted server, or Obsidian

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -p cliphub
```

## Configuration

Create `cliphub.toml`:

```toml
[server]
url = "https://your-server.com"
api_key = "your-api-key"

[notion]
api_key = "your-notion-token"
database_id = "your-database-id"

[obsidian]
vault_path = "/path/to/vault"
```

## Development

```bash
# Run tests
cargo test -p cliphub

# Run with debug output
RUST_LOG=debug cargo run -p cliphub
```
