# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the single Axum entrypoint and owns API wiring.
- Container/runtime files live at the repo root: `Dockerfile`, `Dockerfile.dev`, `docker-compose.yml`, `docker-compose.prod.yml`.
- Operational shortcuts are defined in `Makefile`.
- Tests (when added) should live alongside code in `mod tests` blocks under `src/`.

## Build, Test, and Development Commands
- `make build`: build development container images via Docker Compose.
- `make up`: start the API and Postgres containers in the foreground.
- `make down`: stop containers and remove the compose stack.
- `make logs`: follow container logs for the stack.
- `make shell`: open a shell inside the `app` container.
- `make build-prod` / `make up-prod` / `make down-prod`: production compose variants.
- `docker exec koko-pic-api-app-1 cargo run`: run `cargo` commands inside the container (adjust name if needed).

## Coding Style & Naming Conventions
- Format with `rustfmt` using the repo `rustfmt.toml` (2-space indentation, 120 max width).
- Use idiomatic Rust naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep handlers and routes small and focused; extract modules as the API grows.

## Testing Guidelines
- No tests are present yet; add unit tests in `mod tests` blocks near the code in `src/`.
- When adding endpoints, include at least one happy-path test and one failure case.
- Run tests inside the container: `docker exec koko-pic-api-app-1 cargo test`.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (e.g., `feat:`, `fix:`, `refactor:`, `chore:`) and keep messages short.
- PRs should include a clear description, link related issues, and note validation (commands run, screenshots for API responses if relevant).

## Security & Configuration Tips
- Configuration is provided via environment variables in `docker-compose.yml` (e.g., `DATABASE_URL`).
- Do not commit secrets or local `.env` files; use compose overrides or environment injection for local changes.
