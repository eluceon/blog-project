# Blog Project

A full-stack blog platform built in Rust as a Cargo workspace with four crates:

| Crate | Role |
|-------|------|
| `blog-server` | HTTP (port 8080) + gRPC (port 50051) backend, PostgreSQL via sqlx |
| `blog-client` | Async library wrapping both transports behind a single `BlogClient` API |
| `blog-cli`    | Command-line client powered by `blog-client` |
| `blog-wasm`   | WebAssembly frontend compiled from Rust, loaded by `index.html` |

## Architecture

`blog-server` follows **Clean Architecture**:

```
src/
├── domain/          # Pure Rust types, no external deps
├── application/     # Business logic (AuthService, BlogService)
├── data/            # Repository traits + PostgreSQL implementations
├── infrastructure/  # JWT, DB pool, tracing setup
└── presentation/    # actix-web HTTP handlers + tonic gRPC service
```

## Prerequisites

- Rust stable (1.75+) with `cargo`
- Docker + Docker Compose — for the PostgreSQL database
- `protoc` — Protocol Buffers compiler
- `trunk` — for the WASM frontend (`cargo install trunk`)

### Install protoc

```bash
# Ubuntu / Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf
```

## Setup

### 1. Start the database

Copy the example env file and start PostgreSQL via Docker Compose:

```bash
cp docker/postgres/.env.example docker/postgres/.env
# Edit docker/postgres/.env if you want different credentials

docker compose -f docker/docker-compose.yml up -d
```

### 2. Configure the server environment

Copy and edit the example env file inside `blog-server/`:

```bash
cp blog-server/.env.example blog-server/.env
# Edit DATABASE_URL to match docker/postgres/.env credentials, e.g.:
# DATABASE_URL=postgres://postgres_user:postgres_password@127.0.0.1:5433/blog_db
```

`JWT_SECRET` must be at least 32 characters long.

## Building

```bash
# Build all native crates
cargo build --workspace

# Build only the server
cargo build --bin blog-server

# Build only the CLI
cargo build --bin blog-cli

# Build the WASM frontend
cd blog-wasm && trunk build --release && cd ..
```

## Running

### Server

```bash
cd blog-server
cargo run --bin blog-server
# HTTP: http://localhost:8080
# gRPC: localhost:50051
```

### WASM frontend

```bash
cd blog-wasm
trunk serve --port 8000      # dev server with hot-reload → http://localhost:8000
trunk build --release         # production build → blog-wasm/dist/
```

### CLI

```bash
# Register
cargo run --bin blog-cli -- register --username alice --email alice@example.com --password secret123

# Login
cargo run --bin blog-cli -- login --username alice --password secret123

# Create a post
cargo run --bin blog-cli -- create --title "Hello World" --content "My first post"

# List posts
cargo run --bin blog-cli -- list --limit 10 --offset 0

# Get a specific post
cargo run --bin blog-cli -- get --id 1

# Update a post
cargo run --bin blog-cli -- update --id 1 --title "Updated title"

# Delete a post
cargo run --bin blog-cli -- delete --id 1

# Use gRPC transport instead of HTTP
cargo run --bin blog-cli -- --grpc create --title "gRPC post" --content "via gRPC"

# Point at a custom server
cargo run --bin blog-cli -- --server http://myserver:8080 list
```

### HTTP API via curl

```bash
# Register
curl -X POST http://localhost:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}'

# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secret123"}' | jq -r .token)

# Create post
curl -X POST http://localhost:8080/api/posts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Hello","content":"World"}'

# List posts
curl http://localhost:8080/api/posts?limit=10&offset=0

# Get post
curl http://localhost:8080/api/posts/1

# Update post
curl -X PUT http://localhost:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Updated","content":"New content"}'

# Delete post
curl -X DELETE http://localhost:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN"
```

## Security

- Passwords are hashed with **Argon2** before storage — never stored in plain text.
- All SQL queries use parameterised bindings — no SQL injection.
- Routes that mutate data require a valid **JWT Bearer token**.
- Tokens expire after **24 hours**.
