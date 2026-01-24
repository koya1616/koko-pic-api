# Koko Pic API

A Rust web API built with Axum, async-graphql, and sqlx.

## Features

- [Axum](https://github.com/tokio-rs/axum) - A web framework for building APIs
- [async-graphql](https://github.com/async-graphql/async-graphql) - GraphQL implementation for Rust
- [SQLx](https://github.com/launchbadge/sqlx) - Pure Rust SQL toolkit
- Docker support for easy deployment

## Building and Running

### Using Docker

Start your local Supabase instance first:

```bash
supabase start
```

Then run the application with Docker Compose:

```bash
make up
```

### Local Development

Make sure you have Rust installed, then:

```bash
cargo run
```

The API will be available at `http://localhost:8000`.

GraphQL Playground will be available at `http://localhost:8000/playground`.

## Environment Variables

- `DATABASE_URL` - Connection string for your database (defaults to connect to local Supabase)
- `PORT` - Port to run the server on (defaults to 8000)

## API Endpoints

- `GET /health` - Health check endpoint
- `GET /playground` - GraphQL Playground interface
- `POST /graphql` - GraphQL endpoint