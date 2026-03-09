# kurz [kʊrts]

A minimal, self-hosted URL shortener built with Rust + Axum.

## Features

- Password-protected single-page UI
- Three slug generation modes: **random string**, **random words**, **custom**
- Persistent storage via SQLite (bundled, no install needed)
- Hit counter per link

## Quick Start

```bash
# 1. Edit the config (as needed)
cp config.toml my-config.toml
$EDITOR my-config.toml

# 2. Build & run
cargo build --release
./target/release/urlshort my-config.toml

# 3. Open http://localhost:3000 in your browser
```

## Configuration (`config.toml`)

| Key | Description |
|-----|-------------|
| `server.host` | Bind address (default `127.0.0.1`) |
| `server.port` | Port (default `3000`) |
| `server.base_url` | Public URL used in short-link output |
| `auth.password` | UI login password |
| `auth.session_secret` | Random secret for session tokens |
| `database.path` | SQLite file path |
| `shortener.random_string_length` | Length for random string slugs (default `10`) |
| `shortener.random_word_count` | Words in random-word slugs (default `3`) |
| `shortener.word_separator` | Separator between words (default `-`) |
| `ui.accent_color` | Main accent color of the application (default `#377216`) |

## Running Behind a Reverse Proxy

Set `server.host = "127.0.0.1"` and configure nginx/caddy to proxy to it.
Make sure `base_url` reflects your public domain:

```toml
[server]
base_url = "https://kurz.example.com"
```

## Binary Features
- livereload: adds `tower-livereload` middleware for faster development
- disable_auth: disables the login screen and authentication checks
