# Stage 1: Chef
FROM lukemathwalker/cargo-chef:latest-rust-1.88-bookworm AS chef
WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
# Copy all workspace files needed for server
COPY Cargo.toml Cargo.lock ./
COPY server/ server/
COPY migration/ migration/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching point
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY server/ server/
COPY migration/ migration/
# Build the application - specifically the redy crate
RUN cargo build --release --bin server

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    chromium \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/server /app/server

# Expose the application port
EXPOSE 8080
EXPOSE 9091

# Set the chromium path for chromiumoxide
ENV CHROME=/usr/bin/chromium

# Run the application
ENTRYPOINT ["/app/server", "run-server"]
