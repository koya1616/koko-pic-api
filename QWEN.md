# Repository Guidelines

## Project Structure & Module Organization
This is a small Rust API built on Axum with a single entrypoint. Source lives in `src/` with the main server in
`src/main.rs`. Container and orchestration assets live at the repo root: `Dockerfile`, `Dockerfile.dev`,
`docker-compose.yml`, and `docker-compose.prod.yml`. Operational shortcuts are provided in `Makefile`.

## Build, Test, and Development Commands
- `make build`: build the dev container images via Docker Compose.
- `make up`: start the app and Postgres containers in the foreground.
- `make down`: stop containers and remove the compose stack.
- `make logs`: follow container logs.
- `make shell`: open a shell inside the `app` container.
- `make build-prod` / `make up-prod` / `make down-prod`: production compose variants.
- `docker exec koko-pic-api-app-1 cargo run`: run cargo commands inside the container (adjust the container name if it
  differs).

## Coding Style & Naming Conventions
Use `rustfmt` with the repo config in `rustfmt.toml` (2-space indentation, 120 max width). Prefer idiomatic Rust naming:
`snake_case` for functions and modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants. Keep handlers and
routes small and focused; extract modules as the API grows.

## Testing Guidelines
No tests are present yet. Add unit tests in `mod tests` blocks in the relevant module and run `cargo test` inside the
container. When adding new endpoints, include at least one happy-path test and one failure case.

## Commit & Pull Request Guidelines
Recent commits follow Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`). Keep messages short and specific.
For PRs, include a clear description, link any issues, and note how you validated changes (commands run, screenshots for
API responses if relevant).

## Security & Configuration Tips
Configuration is provided via environment variables in `docker-compose.yml` (e.g., `DATABASE_URL`). Do not commit
secrets or local `.env` files. If you need local overrides, use a compose override file or environment injection.
