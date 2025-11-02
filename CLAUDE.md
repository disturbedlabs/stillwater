# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stillwater is a Rust web service built with Axum, PostgreSQL, and Redis. The project uses Rust 2024 edition and follows an async-first architecture with Tokio runtime.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application (requires Docker services)
cargo run

# Run in release mode
cargo build --release
cargo run --release

# Check code without building
cargo check

# Format code (follows rustfmt.toml: 100 char width, 4 spaces, Unix line endings)
cargo fmt

# Run linter
cargo clippy
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

### Database Management
```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations (also runs automatically on app startup)
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Docker Services
```bash
# Start PostgreSQL and Redis containers
cd docker && just dstart

# Stop containers
cd docker && just dkill

# Or using docker-compose directly
docker-compose --env-file .env -f docker/docker-compose.yml up -d
```

## Architecture

### Application Structure

The application follows a layered architecture:

1. **Entry Point** (`src/main.rs`): Initializes tracing, database pool with migrations, Redis client, and Axum server
2. **Configuration** (`src/config.rs`): Initialization functions for logging, database, and Redis
3. **State Management** (`src/state.rs`): AppState struct that holds shared db_pool and redis_client, cloned across handlers
4. **Services Layer** (`src/services/`): Database and cache operations (currently stubbed)
5. **Handlers** (`src/handlers/`): Axum route handlers (currently stubbed)

### State Sharing Pattern

The application uses Axum's state extraction pattern:
- `AppState` is created once at startup with db_pool and redis_client
- Passed to Router via `.with_state(app_state)`
- Handlers extract state using `State(state): State<AppState>`
- AppState derives Clone for efficient sharing across handlers

### Database & Caching

- **PostgreSQL**: Connection pool managed by SQLx (max 5 connections)
- **Redis**: Client initialized at startup, shared via AppState
- **Migrations**: SQLx migrations in `migrations/` directory, run automatically on startup
- **Environment**: Requires DATABASE_URL and REDIS_URL in .env file

### Dependencies

Key dependencies and their use cases:
- `axum 0.8`: Web framework
- `tokio`: Async runtime with "full" features
- `sqlx 0.8`: Postgres database with async support and migrations
- `redis 0.27`: Redis client with tokio compatibility
- `serde/serde_json`: JSON serialization
- `tracing/tracing-subscriber`: Logging (default level: info)
- `dotenv`: Environment variable loading
- `anyhow`: Error handling

## Environment Setup

Create a `.env` file in the project root:
```
POSTGRES_USER=admin
POSTGRES_PASSWORD=stillwater
POSTGRES_DB=stillwater
DATABASE_URL=postgres://admin:stillwater@localhost:5432/stillwater
REDIS_URL=redis://localhost:6379
```

## Development Workflow

1. Start Docker services: `cd docker && just dstart`
2. Migrations run automatically on `cargo run`
3. Server starts on `http://127.0.0.1:3000`
4. Add new routes by extending Router in `src/main.rs`
5. Implement service functions in `src/services/database.rs` or `src/services/cache.rs`
6. Add handler functions and wire them to routes
