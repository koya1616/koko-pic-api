# Koko Pic API

A Rust web API built with Axum, async-graphql, and sqlx.

## Features

- [Axum](https://github.com/tokio-rs/axum) - A web framework for building APIs
- [async-graphql](https://github.com/async-graphql/async-graphql) - GraphQL implementation for Rust
- [SQLx](https://github.com/launchbadge/sqlx) - Pure Rust SQL toolkit
- Docker support for easy deployment

## Building and Running

### Using Docker

```bash
make up
```

If you need to run Cargo commands inside the container, use:

```bash
docker exec koko-pic-api-app-1 cargo <subcommand>
```