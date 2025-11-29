# Multi-stage Dockerfile for TinyStore
# Optimized for minimal image size and fast builds

# Stage 1: Build the Rust application
FROM rust:1.75-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo-Leptos.toml ./
COPY crates ./crates

# Build dependencies first (cached layer)
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release --bin tinystore

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash tinystore

# Create necessary directories
RUN mkdir -p /data /config && \
    chown -R tinystore:tinystore /data /config

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/tinystore /usr/local/bin/tinystore

# Copy config example
COPY config/config.example.yaml /config/config.example.yaml

# Switch to non-root user
USER tinystore

# Expose port
EXPOSE 9000

# Set default environment variables
ENV RUST_LOG=info
ENV TINYSTORE_DATA_DIR=/data
ENV TINYSTORE_CONFIG=/config/config.yaml

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:9000/health || exit 1

# Default command
CMD ["tinystore", "serve"]
