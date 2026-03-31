# ─── Stage 1: Builder ─────────────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

# Install system dependencies required for compilation
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source files and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/lendchain-backend ./lendchain-backend

# Copy migrations so sqlx::migrate!() can find them at runtime
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080

CMD ["./lendchain-backend"]
