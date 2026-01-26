# Koko Pic API

A Rust web API built with Axum, async-graphql, and sqlx.

## Features

- [Axum](https://github.com/tokio-rs/axum) - A web framework for building APIs
- [SQLx](https://github.com/launchbadge/sqlx) - Pure Rust SQL toolkit
- REST API with JSON payload
- JWT-based authentication
- Docker support for easy deployment

## API Documentation

The API follows REST principles and uses JSON for request/response payloads. API endpoints are versioned under `/api/v1`.

### OpenAPI Specification

An OpenAPI 3.0 specification is available in `openapi.yaml` that documents all available endpoints, request/response schemas, and authentication methods.

To view the API documentation:
1. Install an OpenAPI viewer/editor like Swagger Editor or Stoplight Studio
2. Or use online tools like [Swagger Editor](https://editor.swagger.io/) to load the `openapi.yaml` file

### Available Endpoints

- `GET /` - Health check endpoint
- `POST /api/v1/users` - Create a new user
- `POST /api/v1/login` - User login

### Authentication

Most endpoints require authentication using JWT tokens. After successful login, include the received token in the Authorization header as `Bearer {token}`.

## Building and Running

### Using Docker

```bash
make up
```

If you need to run Cargo commands inside the container, use:

```bash
docker exec koko-pic-api-app-1 cargo <subcommand>
```