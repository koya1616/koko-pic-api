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

# Stage 3: ビルド（sqlx offline mode）
FROM chef AS builder

# 👉 sqlx offline mode: DB接続不要
ENV SQLX_OFFLINE=true

# 必要な開発パッケージ
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json

# 依存関係のみをビルド（キャッシュ）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# ソースコード
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# 本体ビルド（SQLX_OFFLINE=true により DB 接続不要）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/koko-pic-api /app/koko-pic-api

# Stage 4: 実行専用（DB情報なし）
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder /app/koko-pic-api /usr/local/bin/app

ENV SMTP_HOST=smtp.resend.com
ENV SMTP_PORT=587
ENV SMTP_USERNAME=resend
ENV SMTP_FROM_EMAIL=onboarding@resend.dev

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/app"]
