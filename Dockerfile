# syntax=docker/dockerfile:1

# ============================================
# 本番用 Dockerfile (cargo-chef + distroless)
# ============================================

# Stage 1: cargo-chef でレシピ作成
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: 依存関係の情報を抽出
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: 依存関係のビルド（キャッシュ効率化）
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# 依存関係のみをビルド（ここがキャッシュされる）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# ソースコードをコピーして本体をビルド
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/koko-pic-api /app/koko-pic-api

# Stage 4: 最小ランタイム (distroless)
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder /app/koko-pic-api /usr/local/bin/app

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/app"]
