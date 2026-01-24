FROM rust:1.84 AS builder

# Create a dummy project to cache dependencies
RUN USER=root cargo new --bin app
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to allow cargo build to work
RUN echo 'fn main() { println!("Dummy build"); }' > src/main.rs

# Download and compile dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy the actual source code
COPY src ./src

# Build the actual application
RUN touch src/main.rs  # Ensure the timestamp is newer than the lock file
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install SSL certificates and any runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Copy the executable from the builder stage
COPY --from=builder /app/target/release/app /usr/local/bin/app

# Expose the port your Axum application will run on
EXPOSE 8000

# Run the application
CMD ["app"]