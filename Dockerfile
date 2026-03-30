# ─── Stage 1: Builder ─────────────────────────────────────────────────────────
FROM rust:1.77-slim AS builder

# Install system dependencies required for compilation
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files first (Docker layer cache optimisation)
COPY Cargo.toml Cargo.lock* ./

# Create a dummy src/main.rs to pre-compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Now copy the real source code
COPY src ./src
COPY migrations ./migrations

# Touch main.rs so cargo detects the change and rebuilds
RUN touch src/main.rs && cargo build --release

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
