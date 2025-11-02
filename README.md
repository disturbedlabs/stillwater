# Stillwater

A modern web service built with Rust, Axum, PostgreSQL, and Redis.

## Tech Stack

- **Rust 2024 Edition** - Modern, safe systems programming
- **Axum 0.8** - Ergonomic web framework built on Tokio
- **PostgreSQL** - Primary database with SQLx for type-safe queries
- **Redis** - In-memory caching layer
- **Tokio** - Async runtime
- **Docker** - Containerized development environment

## Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- Just (command runner) - optional but recommended

## Getting Started

### 1. Clone the repository

```bash
git clone <repository-url>
cd stillwater
```

### 2. Set up environment variables

Create a `.env` file in the project root:

```env
POSTGRES_USER=your_postgres_user
POSTGRES_PASSWORD=your_secure_password
POSTGRES_DB=stillwater
DATABASE_URL=postgres://your_postgres_user:your_secure_password@localhost:5432/stillwater
REDIS_URL=redis://localhost:6379
```

**Note**: Replace the placeholder values with your own credentials. Do not commit the `.env` file to version control.

### 3. Start Docker services

Using just:
```bash
cd docker
just dstart
```

Or using docker-compose directly:
```bash
docker-compose --env-file .env -f docker/docker-compose.yml up -d
```

### 4. Run the application

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000`

## Project Structure

```
stillwater/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library root and module declarations
│   ├── config.rs         # Configuration and initialization
│   ├── state.rs          # Shared application state
│   ├── handlers/         # HTTP route handlers (to be implemented)
│   └── services/
│       ├── database.rs   # Database operations
│       └── cache.rs      # Redis cache operations
├── migrations/           # SQLx database migrations
├── docker/
│   ├── docker-compose.yml
│   └── justfile          # Just commands for Docker
├── Cargo.toml
└── rustfmt.toml
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run with output
cargo test -- --nocapture
```

### Code Formatting

The project uses rustfmt with custom configuration (100 char width, 4 spaces):

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations (also runs automatically on startup)
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Docker Commands

Using just (recommended):
```bash
cd docker
just dstart  # Start services
just dkill   # Stop services
```

Using docker-compose:
```bash
# Start services
docker-compose --env-file .env -f docker/docker-compose.yml up -d

# Stop services
docker-compose -f docker/docker-compose.yml down
```

## Architecture

Stillwater follows a layered architecture:

- **Entry Point**: Initializes services, runs migrations, and starts the Axum server
- **State Management**: Shared `AppState` containing database pool and Redis client
- **Services Layer**: Database and cache operations
- **Handlers**: HTTP request handlers using Axum extractors

The application uses Axum's state extraction pattern for dependency injection, allowing handlers to access shared resources like the database pool and Redis client.

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `POSTGRES_USER` | PostgreSQL username | `your_username` |
| `POSTGRES_PASSWORD` | PostgreSQL password | `your_password` |
| `POSTGRES_DB` | PostgreSQL database name | `stillwater` |
| `DATABASE_URL` | Full PostgreSQL connection string | `postgres://user:pass@localhost:5432/stillwater` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |

## API Endpoints

Currently available:

- `GET /` - Health check endpoint

More endpoints coming soon as the project develops.

## License

[Add your license here]
